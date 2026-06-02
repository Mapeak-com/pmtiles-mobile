use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::pmtiles_reader::zxy_to_tile_id;

const HEADER_LEN: usize = 127;

/// Compression applied to tile data and directories in a `.pmtiles` archive.
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum Compression {
    /// Stored uncompressed.
    None,
    /// gzip-compressed.
    Gzip,
}

impl Compression {
    fn id(self) -> u8 {
        match self {
            Compression::None => 1,
            Compression::Gzip => 2,
        }
    }
}

/// Builds a `.pmtiles` archive in memory from tiles added one at a time.
///
/// This is a basic writer: it emits a single root directory (no leaf
/// directories), so it suits small archives, not large general-purpose tilesets.
#[derive(uniffi::Object)]
pub struct PmTilesWriter {
    state: Mutex<State>,
}

struct State {
    internal_compression: Compression,
    tile_compression: Compression,
    tiles: Vec<(u64, Vec<u8>)>,
}

#[uniffi::export]
impl PmTilesWriter {
    /// Creates a writer that gzip-compresses both tiles and directories.
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Self::with_compression(Compression::Gzip, Compression::Gzip)
    }

    /// Creates a writer with explicit compression for directories
    /// (`internal_compression`) and tile data (`tile_compression`).
    #[uniffi::constructor]
    pub fn with_compression(internal_compression: Compression, tile_compression: Compression) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                internal_compression,
                tile_compression,
                tiles: Vec::new(),
            }),
        })
    }

    /// Adds a tile at zoom `z`, column `x`, row `y` with the given raw bytes.
    /// The bytes are compressed per the writer's tile compression on `build`.
    pub fn add_tile(&self, z: u8, x: u32, y: u32, data: Vec<u8>) {
        self.state
            .lock()
            .unwrap()
            .tiles
            .push((zxy_to_tile_id(z, x, y), data));
    }

    /// Serializes the added tiles into a complete `.pmtiles` archive and
    /// returns its bytes.
    pub fn build(&self) -> Vec<u8> {
        let state = self.state.lock().unwrap();

        let mut tiles = state.tiles.clone();
        tiles.sort_by_key(|(id, _)| *id);

        let mut tile_data = Vec::new();
        let mut entries: Vec<(u64, u64, u32)> = Vec::new();
        for (id, data) in &tiles {
            let body = compress(data, state.tile_compression);
            entries.push((*id, tile_data.len() as u64, body.len() as u32));
            tile_data.extend_from_slice(&body);
        }

        let dir = compress(&serialize_directory(&entries), state.internal_compression);

        let mut out = vec![0u8; HEADER_LEN];
        out[0..7].copy_from_slice(b"PMTiles");
        out[7] = 3;
        put_u64(&mut out, 8, HEADER_LEN as u64);
        put_u64(&mut out, 16, dir.len() as u64);
        put_u64(&mut out, 56, HEADER_LEN as u64 + dir.len() as u64);
        put_u64(&mut out, 64, tile_data.len() as u64);
        out[97] = state.internal_compression.id();
        out[98] = state.tile_compression.id();

        out.extend_from_slice(&dir);
        out.extend_from_slice(&tile_data);
        out
    }
}

fn serialize_directory(entries: &[(u64, u64, u32)]) -> Vec<u8> {
    let mut buf = Vec::new();
    write_varint(&mut buf, entries.len() as u64);
    let mut last = 0u64;
    for (id, _, _) in entries {
        write_varint(&mut buf, id - last);
        last = *id;
    }
    for _ in entries {
        write_varint(&mut buf, 1);
    }
    for (_, _, length) in entries {
        write_varint(&mut buf, *length as u64);
    }
    let mut expected = 0u64;
    for (i, (_, offset, length)) in entries.iter().enumerate() {
        if i > 0 && *offset == expected {
            write_varint(&mut buf, 0);
        } else {
            write_varint(&mut buf, offset + 1);
        }
        expected = offset + *length as u64;
    }
    buf
}

fn compress(data: &[u8], compression: Compression) -> Vec<u8> {
    match compression {
        Compression::Gzip => {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(data).unwrap();
            enc.finish().unwrap()
        }
        Compression::None => data.to_vec(),
    }
}

fn write_varint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if v == 0 {
            break;
        }
    }
}

fn put_u64(buf: &mut [u8], at: usize, v: u64) {
    buf[at..at + 8].copy_from_slice(&v.to_le_bytes());
}
