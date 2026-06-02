use std::fs::File;
use std::io::Read;
use std::os::unix::fs::FileExt;
use std::sync::Arc;

use crate::PmTilesError;

const HEADER_LEN: usize = 127;
const MAGIC: &[u8; 7] = b"PMTiles";
const COMPRESSION_GZIP: u8 = 2;

struct Header {
    root_dir_offset: u64,
    root_dir_length: u64,
    leaf_dirs_offset: u64,
    tile_data_offset: u64,
    internal_compression: u8,
    tile_compression: u8,
}

#[derive(Clone, Copy, Default)]
struct Entry {
    tile_id: u64,
    offset: u64,
    length: u32,
    run_length: u32,
}

/// Reads map tiles from a local `.pmtiles` archive (PMTiles v3).
#[derive(uniffi::Object)]
pub struct PmTilesReader {
    file: File,
    header: Header,
}

#[uniffi::export]
impl PmTilesReader {
    /// Opens a `.pmtiles` file at `path`, reading and validating its header.
    ///
    /// Throws if the file cannot be opened, is not a PMTiles archive, or is not
    /// version 3.
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, PmTilesError> {
        let file = File::open(&path)?;

        let mut buf = [0u8; HEADER_LEN];
        file.read_exact_at(&mut buf, 0)?;
        if &buf[0..7] != MAGIC {
            return Err(PmTilesError::InvalidArchive);
        }
        let version = buf[7];
        if version != 3 {
            return Err(PmTilesError::UnsupportedVersion { version });
        }

        let header = Header {
            root_dir_offset: read_u64(&buf, 8),
            root_dir_length: read_u64(&buf, 16),
            leaf_dirs_offset: read_u64(&buf, 40),
            tile_data_offset: read_u64(&buf, 56),
            internal_compression: buf[97],
            tile_compression: buf[98],
        };
        Ok(Arc::new(PmTilesReader { file, header }))
    }

    /// Returns the decompressed bytes of the tile at zoom `z`, column `x`, row
    /// `y`, or `null` if that tile is not present in the archive.
    pub fn get_tile(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>, PmTilesError> {
        let tile_id = zxy_to_tile_id(z, x, y);

        let mut dir_offset = self.header.root_dir_offset;
        let mut dir_length = self.header.root_dir_length;

        for _ in 0..4 {
            let raw = self.read_range(dir_offset, dir_length)?;
            let dir_bytes = decompress(&raw, self.header.internal_compression)?;
            let entries = deserialize_directory(&dir_bytes)?;

            match find_tile(&entries, tile_id) {
                None => return Ok(None),
                Some(entry) => {
                    if entry.run_length == 0 {
                        dir_offset = self.header.leaf_dirs_offset + entry.offset;
                        dir_length = entry.length as u64;
                    } else {
                        let raw_tile = self
                            .read_range(self.header.tile_data_offset + entry.offset, entry.length as u64)?;
                        return Ok(Some(decompress(&raw_tile, self.header.tile_compression)?));
                    }
                }
            }
        }
        Ok(None)
    }
}

impl PmTilesReader {
    fn read_range(&self, offset: u64, length: u64) -> Result<Vec<u8>, PmTilesError> {
        let mut buf = vec![0u8; length as usize];
        self.file.read_exact_at(&mut buf, offset)?;
        Ok(buf)
    }
}

fn read_u64(buf: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(buf[at..at + 8].try_into().unwrap())
}

fn decompress(data: &[u8], compression: u8) -> Result<Vec<u8>, PmTilesError> {
    match compression {
        COMPRESSION_GZIP => {
            let mut out = Vec::new();
            flate2::read::MultiGzDecoder::new(data).read_to_end(&mut out)?;
            Ok(out)
        }
        _ => Ok(data.to_vec()),
    }
}

fn read_varint(data: &[u8], p: &mut usize) -> Result<u64, PmTilesError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *data.get(*p).ok_or(PmTilesError::CorruptDirectory)?;
        *p += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err(PmTilesError::CorruptDirectory);
        }
    }
}

fn deserialize_directory(data: &[u8]) -> Result<Vec<Entry>, PmTilesError> {
    let mut p = 0usize;
    let num = read_varint(data, &mut p)? as usize;
    let mut entries = vec![Entry::default(); num];

    let mut last_id: u64 = 0;
    for e in entries.iter_mut() {
        last_id += read_varint(data, &mut p)?;
        e.tile_id = last_id;
    }
    for e in entries.iter_mut() {
        e.run_length = read_varint(data, &mut p)? as u32;
    }
    for e in entries.iter_mut() {
        e.length = read_varint(data, &mut p)? as u32;
    }
    for i in 0..num {
        let v = read_varint(data, &mut p)?;
        entries[i].offset = if v == 0 && i > 0 {
            entries[i - 1].offset + entries[i - 1].length as u64
        } else {
            v - 1
        };
    }
    Ok(entries)
}

fn find_tile(entries: &[Entry], tile_id: u64) -> Option<Entry> {
    let mut m: i64 = 0;
    let mut n: i64 = entries.len() as i64 - 1;
    while m <= n {
        let k = ((m + n) >> 1) as usize;
        let entry_id = entries[k].tile_id;
        if tile_id > entry_id {
            m = k as i64 + 1;
        } else if tile_id < entry_id {
            n = k as i64 - 1;
        } else {
            return Some(entries[k]);
        }
    }
    if n >= 0 {
        let e = entries[n as usize];
        if e.run_length == 0 {
            return Some(e);
        }
        if tile_id - e.tile_id < e.run_length as u64 {
            return Some(e);
        }
    }
    None
}

pub(crate) fn zxy_to_tile_id(z: u8, x: u32, y: u32) -> u64 {
    let mut acc: u64 = 0;
    for t_z in 0..z {
        acc += (1u64 << t_z) * (1u64 << t_z);
    }
    let n: i64 = 1i64 << z;
    let mut tx = x as i64;
    let mut ty = y as i64;
    let mut d: i64 = 0;
    let mut s = n / 2;
    while s > 0 {
        let rx: i64 = if (tx & s) > 0 { 1 } else { 0 };
        let ry: i64 = if (ty & s) > 0 { 1 } else { 0 };
        d += s * s * ((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                tx = s - 1 - tx;
                ty = s - 1 - ty;
            }
            std::mem::swap(&mut tx, &mut ty);
        }
        s /= 2;
    }
    acc + d as u64
}

#[cfg(test)]
mod tests {
    use super::zxy_to_tile_id;

    #[test]
    fn tile_id_matches_known_values() {
        assert_eq!(zxy_to_tile_id(0, 0, 0), 0);
        assert_eq!(zxy_to_tile_id(1, 0, 0), 1);
        assert_eq!(zxy_to_tile_id(1, 0, 1), 2);
        assert_eq!(zxy_to_tile_id(1, 1, 1), 3);
        assert_eq!(zxy_to_tile_id(1, 1, 0), 4);
    }
}
