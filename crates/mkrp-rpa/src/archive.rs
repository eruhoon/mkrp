use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use flate2::read::ZlibDecoder;
use serde_pickle::{DeOptions, HashableValue, Value};
use tracing::{debug, info};

use crate::error::RpaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpaVersion {
    Rpa3,
    Rpa2,
    Rpa1,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub offset: u64,
    pub length: u64,
    pub prefix: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub segments: Vec<Segment>,
}

#[allow(dead_code)]
pub struct RpaArchive {
    path: PathBuf,
    file: BufReader<File>,
    version: RpaVersion,
    key: u64,
    index: HashMap<String, FileEntry>,
}

impl RpaArchive {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, RpaError> {
        let path_buf = path.as_ref().to_path_buf();
        let file = File::open(&path_buf)?;
        let mut reader = BufReader::new(file);

        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;

        let (version, offset, key) = if header_line.starts_with("RPA-3.2 ") {
            let parts: Vec<&str> = header_line.trim().split_whitespace().collect();
            if parts.len() < 4 {
                return Err(RpaError::InvalidHeader(format!(
                    "RPA-3.2 header has insufficient fields: {}",
                    header_line.trim()
                )));
            }
            let raw_offset = u64::from_str_radix(parts[1], 16)
                .map_err(|e| RpaError::InvalidHeader(format!("Bad offset: {}", e)))?;
            let mut key: u64 = 0;
            for subkey_str in &parts[3..] {
                if let Ok(subkey) = u64::from_str_radix(subkey_str, 16) {
                    key ^= subkey;
                }
            }
            (RpaVersion::Rpa3, raw_offset, key)
        } else if header_line.starts_with("RPA-3.0 ") {
            let parts: Vec<&str> = header_line.trim().split_whitespace().collect();
            if parts.len() < 3 {
                return Err(RpaError::InvalidHeader(format!(
                    "RPA-3.0 header has insufficient fields: {}",
                    header_line.trim()
                )));
            }
            let raw_offset = u64::from_str_radix(parts[1], 16)
                .map_err(|e| RpaError::InvalidHeader(format!("Bad offset: {}", e)))?;
            let mut key: u64 = 0;
            for subkey_str in &parts[2..] {
                if let Ok(subkey) = u64::from_str_radix(subkey_str, 16) {
                    key ^= subkey;
                }
            }
            (RpaVersion::Rpa3, raw_offset, key)
        } else if header_line.starts_with("RPA-2.0 ") {
            let parts: Vec<&str> = header_line.trim().split_whitespace().collect();
            if parts.len() < 3 {
                return Err(RpaError::InvalidHeader(format!(
                    "RPA-2.0 header has insufficient fields: {}",
                    header_line.trim()
                )));
            }
            let raw_offset = u64::from_str_radix(parts[1], 16)
                .map_err(|e| RpaError::InvalidHeader(format!("Bad offset: {}", e)))?;
            let mut key: u64 = 0;
            for subkey_str in &parts[2..] {
                if let Ok(subkey) = u64::from_str_radix(subkey_str, 16) {
                    key ^= subkey;
                }
            }
            (RpaVersion::Rpa2, raw_offset, key)
        } else {
            // Check if it's RPA-1.0 (unencrypted pickle or raw offset)
            reader.seek(SeekFrom::Start(0))?;
            (RpaVersion::Rpa1, 0, 0)
        };

        debug!(
            "Opened RPA archive {:?}: version={:?}, index_offset={}, key={:x}",
            path_buf, version, offset, key
        );

        // Seek to index
        reader.seek(SeekFrom::Start(offset))?;
        let mut compressed_index = Vec::new();
        reader.read_to_end(&mut compressed_index)?;

        // Zlib decompress
        let mut decoder = ZlibDecoder::new(&compressed_index[..]);
        let mut decompressed = Vec::new();
        if let Err(e) = decoder.read_to_end(&mut decompressed) {
            // Some RPA-1.0 archives might not be zlib compressed
            if version == RpaVersion::Rpa1 {
                decompressed = compressed_index;
            } else {
                return Err(RpaError::Decompress(e.to_string()));
            }
        }

        // Deserialize pickle index
        let pickle_value: Value = serde_pickle::value_from_reader(&decompressed[..], DeOptions::new())
            .map_err(|e| RpaError::Pickle(e.to_string()))?;

        let mut index = HashMap::new();

        if let Value::Dict(dict) = pickle_value {
            for (k, v) in dict {
                let filename = match k {
                    HashableValue::String(s) => s,
                    HashableValue::Bytes(b) => String::from_utf8_lossy(&b).to_string(),
                    _ => continue,
                };

                let mut segments = Vec::new();

                match v {
                    Value::List(list) => {
                        for item in list {
                            if let Some(seg) = Self::parse_segment(item, key, version) {
                                segments.push(seg);
                            }
                        }
                    }
                    Value::Tuple(tuple) => {
                        if let Some(seg) = Self::parse_segment_items(&tuple, key, version) {
                            segments.push(seg);
                        }
                    }
                    _ => {}
                }

                if !segments.is_empty() {
                    let normalized_name = filename.replace('\\', "/");
                    index.insert(normalized_name.clone(), FileEntry {
                        name: normalized_name,
                        segments,
                    });
                }
            }
        } else {
            return Err(RpaError::CorruptedData("Pickle root is not a dictionary".into()));
        }

        info!(
            "RPA archive {:?} loaded successfully with {} files (version {:?})",
            path_buf.file_name().unwrap_or_default(),
            index.len(),
            version
        );

        Ok(Self {
            path: path_buf,
            file: reader,
            version,
            key,
            index,
        })
    }

    fn parse_segment(val: Value, key: u64, version: RpaVersion) -> Option<Segment> {
        match val {
            Value::Tuple(items) | Value::List(items) => {
                Self::parse_segment_items(&items, key, version)
            }
            _ => None,
        }
    }

    fn parse_segment_items(items: &[Value], key: u64, version: RpaVersion) -> Option<Segment> {
        if items.len() < 2 {
            return None;
        }

        let raw_offset = match &items[0] {
            Value::I64(v) => *v as u64,
            _ => return None,
        };

        let raw_len = match &items[1] {
            Value::I64(v) => *v as u64,
            _ => return None,
        };

        let (offset, length) = if version == RpaVersion::Rpa3 || version == RpaVersion::Rpa2 {
            (raw_offset ^ key, raw_len ^ key)
        } else {
            (raw_offset, raw_len)
        };

        let prefix = if items.len() >= 3 {
            match &items[2] {
                Value::Bytes(b) => b.clone(),
                Value::String(s) => s.as_bytes().to_vec(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        Some(Segment {
            offset,
            length,
            prefix,
        })
    }

    pub fn version(&self) -> RpaVersion {
        self.version
    }

    pub fn contains(&self, filename: &str) -> bool {
        let normalized = filename.replace('\\', "/");
        self.index.contains_key(&normalized)
    }

    pub fn list_files(&self) -> impl Iterator<Item = &str> {
        self.index.keys().map(|s| s.as_str())
    }

    pub fn read(&mut self, filename: &str) -> Result<Vec<u8>, RpaError> {
        let normalized = filename.replace('\\', "/");
        let entry = self
            .index
            .get(&normalized)
            .ok_or_else(|| RpaError::FileNotFound(normalized.clone()))?
            .clone();

        let mut output = Vec::new();

        for seg in entry.segments {
            if !seg.prefix.is_empty() {
                output.extend_from_slice(&seg.prefix);
            }

            self.file.seek(SeekFrom::Start(seg.offset))?;
            let mut buffer = vec![0u8; seg.length as usize];
            self.file.read_exact(&mut buffer)?;
            output.extend(buffer);
        }

        Ok(output)
    }
}
