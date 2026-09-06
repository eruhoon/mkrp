use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid RPA header: {0}")]
    InvalidHeader(String),

    #[error("Zlib decompression error: {0}")]
    Decompress(String),

    #[error("Pickle deserialization error: {0}")]
    Pickle(String),

    #[error("File not found in archive: {0}")]
    FileNotFound(String),

    #[error("Corrupted archive data: {0}")]
    CorruptedData(String),
}
