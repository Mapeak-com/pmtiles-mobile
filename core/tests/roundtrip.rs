use std::sync::atomic::{AtomicU32, Ordering};

use pmtiles_core::{Compression, PmTilesError, PmTilesReader, PmTilesWriter};

fn temp_archive(bytes: &[u8]) -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut path = std::env::temp_dir();
    path.push(format!("pmtiles_it_{}_{}.pmtiles", std::process::id(), n));
    std::fs::write(&path, bytes).unwrap();
    path.to_string_lossy().into_owned()
}

fn roundtrip(internal: Compression, tile: Compression) {
    let t000 = b"tile-payload-0-0-0".to_vec();
    let t100 = b"tile-payload-1-0-0".to_vec();
    let t111 = b"tile-payload-1-1-1-with-a-longer-body".to_vec();

    let w = PmTilesWriter::with_compression(internal, tile);
    w.add_tile(0, 0, 0, t000.clone());
    w.add_tile(1, 0, 0, t100.clone());
    w.add_tile(1, 1, 1, t111.clone());
    let archive = w.build();

    let r = PmTilesReader::open(temp_archive(&archive)).unwrap();
    assert_eq!(r.get_tile(0, 0, 0).unwrap().as_deref(), Some(&t000[..]));
    assert_eq!(r.get_tile(1, 0, 0).unwrap().as_deref(), Some(&t100[..]));
    assert_eq!(r.get_tile(1, 1, 1).unwrap().as_deref(), Some(&t111[..]));
    assert!(r.get_tile(1, 0, 1).unwrap().is_none());
}

#[test]
fn roundtrip_gzip_dirs_and_tiles() {
    roundtrip(Compression::Gzip, Compression::Gzip);
}

#[test]
fn roundtrip_uncompressed() {
    roundtrip(Compression::None, Compression::None);
}

#[test]
fn roundtrip_uncompressed_dirs_gzip_tiles() {
    roundtrip(Compression::None, Compression::Gzip);
}

#[test]
fn rejects_non_pmtiles() {
    let path = temp_archive(&[0u8; 200]);
    assert!(matches!(PmTilesReader::open(path), Err(PmTilesError::InvalidArchive)));
}

#[test]
fn writer_new_defaults_to_gzip() {
    let payload = b"default-writer".to_vec();
    let w = PmTilesWriter::new();
    w.add_tile(0, 0, 0, payload.clone());
    let r = PmTilesReader::open(temp_archive(&w.build())).unwrap();
    assert_eq!(r.get_tile(0, 0, 0).unwrap().as_deref(), Some(&payload[..]));
}

#[test]
fn large_tile_uses_multibyte_varints() {
    // A >=128-byte payload makes the length varint multibyte on write and read.
    let payload = vec![0xABu8; 500];
    let w = PmTilesWriter::with_compression(Compression::None, Compression::None);
    w.add_tile(0, 0, 0, payload.clone());
    let r = PmTilesReader::open(temp_archive(&w.build())).unwrap();
    assert_eq!(r.get_tile(0, 0, 0).unwrap().as_deref(), Some(&payload[..]));
}

#[test]
fn open_missing_file_is_io_error() {
    let res = PmTilesReader::open("/no/such/dir/missing.pmtiles".to_string());
    assert!(matches!(res, Err(PmTilesError::Io { .. })));
}

#[test]
fn unsupported_version_is_error() {
    let mut bytes = vec![0u8; 127];
    bytes[0..7].copy_from_slice(b"PMTiles");
    bytes[7] = 2;
    let path = temp_archive(&bytes);
    assert!(matches!(
        PmTilesReader::open(path),
        Err(PmTilesError::UnsupportedVersion { version: 2 })
    ));
}

#[test]
fn empty_directory_returns_none() {
    let root = vec![0u8]; // num_entries = 0
    let bytes = with_root_dir(&root);
    let r = PmTilesReader::open(temp_archive(&bytes)).unwrap();
    assert!(r.get_tile(0, 0, 0).unwrap().is_none());
}

#[test]
fn corrupt_directory_truncated_varint() {
    let root = vec![0x80u8]; // continuation bit set, but no following byte
    let bytes = with_root_dir(&root);
    let r = PmTilesReader::open(temp_archive(&bytes)).unwrap();
    assert!(matches!(r.get_tile(0, 0, 0), Err(PmTilesError::CorruptDirectory)));
}

#[test]
fn corrupt_directory_varint_overflow() {
    let root = vec![0x80u8; 10]; // never terminates -> shift overflow
    let bytes = with_root_dir(&root);
    let r = PmTilesReader::open(temp_archive(&bytes)).unwrap();
    assert!(matches!(r.get_tile(0, 0, 0), Err(PmTilesError::CorruptDirectory)));
}

#[test]
fn reads_through_leaf_directory() {
    // Root dir holds one leaf pointer (run_length 0); the leaf holds a tile entry
    // whose run_length covers id 5 (= z2/0/0). Requesting z2/0/0 is not an exact
    // entry id, so it exercises the leaf-directory traversal and run-length match.
    let tile = b"leaf-tile".to_vec();

    let mut leaf = Vec::new();
    write_varint(&mut leaf, 1); // entries
    write_varint(&mut leaf, 0); // tile_id delta -> id 0
    write_varint(&mut leaf, 1000); // run_length covers id 5
    write_varint(&mut leaf, tile.len() as u64); // length
    write_varint(&mut leaf, 1); // offset 0 (encoded +1)

    let mut root = Vec::new();
    write_varint(&mut root, 1); // entries
    write_varint(&mut root, 0); // tile_id delta -> id 0
    write_varint(&mut root, 0); // run_length 0 -> leaf pointer
    write_varint(&mut root, leaf.len() as u64); // length of leaf dir
    write_varint(&mut root, 1); // offset 0 into leaf_dirs (encoded +1)

    let leaf_off = 127 + root.len() as u64;
    let tile_off = leaf_off + leaf.len() as u64;
    let mut bytes = header(root.len() as u64, leaf_off, tile_off, tile.len() as u64);
    bytes.extend_from_slice(&root);
    bytes.extend_from_slice(&leaf);
    bytes.extend_from_slice(&tile);

    let r = PmTilesReader::open(temp_archive(&bytes)).unwrap();
    assert_eq!(r.get_tile(2, 0, 0).unwrap().as_deref(), Some(&tile[..]));
}

// --- helpers for crafting malformed / special-layout archives ---

fn put_u64(buf: &mut [u8], at: usize, v: u64) {
    buf[at..at + 8].copy_from_slice(&v.to_le_bytes());
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

fn header(root_len: u64, leaf_off: u64, tile_off: u64, tile_len: u64) -> Vec<u8> {
    let mut h = vec![0u8; 127];
    h[0..7].copy_from_slice(b"PMTiles");
    h[7] = 3;
    put_u64(&mut h, 8, 127); // root_dir_offset
    put_u64(&mut h, 16, root_len); // root_dir_length
    put_u64(&mut h, 40, leaf_off); // leaf_dirs_offset
    put_u64(&mut h, 56, tile_off); // tile_data_offset
    put_u64(&mut h, 64, tile_len);
    h[97] = 1; // internal compression: none
    h[98] = 1; // tile compression: none
    h
}

fn with_root_dir(root: &[u8]) -> Vec<u8> {
    let mut bytes = header(root.len() as u64, 0, 127 + root.len() as u64, 0);
    bytes.extend_from_slice(root);
    bytes
}
