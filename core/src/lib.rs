mod pmtiles_reader;
mod pmtiles_writer;

pub use pmtiles_reader::PmTilesReader;
pub use pmtiles_writer::{Compression, PmTilesWriter};

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PmTilesError {
    #[error("I/O error: {msg}")]
    Io { msg: String },
    #[error("not a valid PMTiles archive")]
    InvalidArchive,
    #[error("unsupported PMTiles version: {version}")]
    UnsupportedVersion { version: u8 },
    #[error("corrupt directory data")]
    CorruptDirectory,
}

impl From<std::io::Error> for PmTilesError {
    fn from(e: std::io::Error) -> Self {
        PmTilesError::Io { msg: e.to_string() }
    }
}
