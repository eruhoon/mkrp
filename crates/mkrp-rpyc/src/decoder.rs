use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use byteorder::{ByteOrder, LittleEndian};
use flate2::read::ZlibDecoder;
use tracing::debug;

use crate::error::RpycError;

pub const RPYC2_MAGIC: &[u8] = b"RENPY RPC2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenpyVersion {
    Renpy8, // Python 3 based
    Renpy7, // Python 2 based
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub id: u32,
    pub offset: usize,
    pub length: usize,
}

pub struct RpycFile {
    version: RenpyVersion,
    is_v2: bool,
    slots: HashMap<u32, Slot>,
    #[allow(dead_code)]
    raw_payload: Vec<u8>,
    decompressed_payload: Vec<u8>,
}

impl RpycFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, RpycError> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Self, RpycError> {
        if raw.len() < 10 {
            return Err(RpycError::InvalidMagic("File is too short".into()));
        }

        let is_v2 = raw.starts_with(RPYC2_MAGIC);
        let mut slots = HashMap::new();
        let compressed_blob: &[u8];

        if is_v2 {
            let mut position = 10;
            loop {
                if position + 12 > raw.len() {
                    return Err(RpycError::CorruptedSlotTable("Unexpected end of slot table".into()));
                }

                let slot_id = LittleEndian::read_u32(&raw[position..position + 4]);
                let start = LittleEndian::read_u32(&raw[position + 4..position + 8]) as usize;
                let length = LittleEndian::read_u32(&raw[position + 8..position + 12]) as usize;
                position += 12;

                if slot_id == 0 {
                    break;
                }

                if start + length > raw.len() {
                    return Err(RpycError::CorruptedSlotTable(format!(
                        "Slot {} data bounds [{}..{}] exceed file size {}",
                        slot_id,
                        start,
                        start + length,
                        raw.len()
                    )));
                }

                slots.insert(
                    slot_id,
                    Slot {
                        id: slot_id,
                        offset: start,
                        length,
                    },
                );
            }

            let slot1 = slots
                .get(&1)
                .ok_or(RpycError::SlotNotFound(1))?;
            compressed_blob = &raw[slot1.offset..slot1.offset + slot1.length];
        } else {
            // RPYC v1 is raw compressed blob
            compressed_blob = raw;
        }

        // Decompress blob
        let mut decoder = ZlibDecoder::new(compressed_blob);
        let mut decompressed_payload = Vec::new();
        decoder
            .read_to_end(&mut decompressed_payload)
            .map_err(|e| RpycError::Decompress(e.to_string()))?;

        // Detect Ren'Py version (Python 2 vs Python 3 AST format)
        let version = Self::detect_version(&decompressed_payload);

        debug!(
            "Loaded RPYC: is_v2={}, slots={}, decompressed_size={} bytes, detected_version={:?}",
            is_v2,
            slots.len(),
            decompressed_payload.len(),
            version
        );

        Ok(Self {
            version,
            is_v2,
            slots,
            raw_payload: compressed_blob.to_vec(),
            decompressed_payload,
        })
    }

    /// Inspect decompressed pickle bytecode for Python 2 vs Python 3 / Ren'Py 7 vs 8 signatures.
    fn detect_version(decompressed: &[u8]) -> RenpyVersion {
        // Ren'Py 8 uses 'renpy.astsupport' for PyExpr and Python 3 builtins
        let has_astsupport = decompressed
            .windows(b"renpy.astsupport".len())
            .any(|w| w == b"renpy.astsupport");

        let has_py3_builtins = decompressed
            .windows(b"cbuiltins".len())
            .any(|w| w == b"cbuiltins");

        let has_py2_builtins = decompressed
            .windows(b"c__builtin__".len())
            .any(|w| w == b"c__builtin__");

        if has_astsupport || has_py3_builtins {
            RenpyVersion::Renpy8
        } else if has_py2_builtins {
            RenpyVersion::Renpy7
        } else {
            // Default to Ren'Py 8 if ambiguous
            RenpyVersion::Renpy8
        }
    }

    pub fn version(&self) -> RenpyVersion {
        self.version
    }

    pub fn is_v2(&self) -> bool {
        self.is_v2
    }

    pub fn slots(&self) -> &HashMap<u32, Slot> {
        &self.slots
    }

    pub fn decompressed_payload(&self) -> &[u8] {
        &self.decompressed_payload
    }
}
