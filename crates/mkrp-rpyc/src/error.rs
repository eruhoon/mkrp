use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpycError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid RPYC magic: expected 'RENPY RPC2', found {0:?}")]
    InvalidMagic(String),

    #[error("Slot table is corrupted or truncated: {0}")]
    CorruptedSlotTable(String),

    #[error("Required slot {0} not found in RPYC file")]
    SlotNotFound(u32),

    #[error("Zlib decompression error: {0}")]
    Decompress(String),

    #[error("Pickle parsing error: {0}")]
    Pickle(String),
}
