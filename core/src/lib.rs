mod pmtiles_reader;
mod pmtiles_writer;

pub use pmtiles_reader::PmTilesReader;
pub use pmtiles_writer::{Compression, PmTilesWriter};

uniffi::setup_scaffolding!();

/// Errors returned when opening an archive or reading a tile.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PmTilesError {
    /// The file could not be read.
    #[error("I/O error: {msg}")]
    Io { msg: String },
    /// The file is not a PMTiles archive (bad magic).
    #[error("not a valid PMTiles archive")]
    InvalidArchive,
    /// The archive uses an unsupported PMTiles version (only v3 is supported).
    #[error("unsupported PMTiles version: {version}")]
    UnsupportedVersion { version: u8 },
    /// A directory could not be parsed.
    #[error("corrupt directory data")]
    CorruptDirectory,
}

impl From<std::io::Error> for PmTilesError {
    fn from(e: std::io::Error) -> Self {
        PmTilesError::Io { msg: e.to_string() }
    }
}
