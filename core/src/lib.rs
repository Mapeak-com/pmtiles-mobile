use std::fs::File;
use std::sync::{Arc, Mutex};

use pmtiles2::util::decompress_all;
use pmtiles2::PMTiles;

uniffi::setup_scaffolding!();

/// Errors returned when opening an archive or reading a tile.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PmTilesError {
    /// The file could not be read.
    #[error("I/O error: {msg}")]
    Io { msg: String },
    /// The archive could not be parsed or a tile could not be decompressed.
    #[error("PMTiles error: {msg}")]
    Pmtiles { msg: String },
}

impl PmTilesError {
    /// Wraps an error from the `pmtiles2` crate.
    fn from_pmtiles(e: impl std::fmt::Display) -> Self {
        PmTilesError::Pmtiles { msg: e.to_string() }
    }
}

/// Reads map tiles from a local `.pmtiles` archive (PMTiles v3).
///
/// A thin wrapper over [`pmtiles2`]: tiles are read lazily on demand and
/// decompressed (gzip/brotli/zstd/none) using the archive's own header.
#[derive(uniffi::Object)]
pub struct PmTilesReader {
    inner: Mutex<PMTiles<File>>,
}

#[uniffi::export]
impl PmTilesReader {
    /// Opens a `.pmtiles` file at `path`, reading and validating its header and
    /// directories. Throws if the file cannot be read or is not a valid archive.
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, PmTilesError> {
        let file = File::open(&path).map_err(|e| PmTilesError::Io { msg: e.to_string() })?;
        let pmtiles = PMTiles::from_reader(file).map_err(PmTilesError::from_pmtiles)?;
        Ok(Arc::new(Self {
            inner: Mutex::new(pmtiles),
        }))
    }

    /// Returns the decompressed bytes of the tile at zoom `z`, column `x`, row
    /// `y`, or `null` if that tile is not present in the archive.
    pub fn get_tile(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>, PmTilesError> {
        let mut pmtiles = self.inner.lock().unwrap();
        let raw = pmtiles
            .get_tile(x as u64, y as u64, z)
            .map_err(PmTilesError::from_pmtiles)?;
        match raw {
            None => Ok(None),
            Some(bytes) => {
                let decompressed = decompress_all(pmtiles.tile_compression, &bytes)
                    .map_err(PmTilesError::from_pmtiles)?;
                Ok(Some(decompressed))
            }
        }
    }
}
