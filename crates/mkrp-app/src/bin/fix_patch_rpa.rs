use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde_bytes::ByteBuf;
use serde_pickle::{DeOptions, HashableValue, SerOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpa_path = Path::new("Y:/ports/VIRTUES/game/patch_virtues_assets.rpa");
    println!("Opening {:?}...", rpa_path);

    let mut file = OpenOptions::new().read(true).write(true).open(rpa_path)?;

    // 1. Read header
    let mut header_buf = [0u8; 40];
    file.read_exact(&mut header_buf)?;
    let header_line = std::str::from_utf8(&header_buf)?;
    let parts: Vec<&str> = header_line.trim().split_whitespace().collect();
    if parts.len() < 3 || !parts[0].starts_with("RPA-3.0") {
        panic!("Invalid header: {}", header_line);
    }

    let index_offset = u64::from_str_radix(parts[1], 16)?;
    let key = u64::from_str_radix(parts[2], 16)?;
    println!("Header: index_offset = 0x{:x}, key = 0x{:x}", index_offset, key);

    // 2. Read compressed index
    file.seek(SeekFrom::Start(index_offset))?;
    let mut compressed_index = Vec::new();
    file.read_to_end(&mut compressed_index)?;
    println!("Read {} bytes of compressed index", compressed_index.len());

    // 3. Decompress zlib
    let mut decoder = ZlibDecoder::new(&compressed_index[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    println!("Decompressed index: {} bytes", decompressed.len());

    // 4. Parse pickle
    let pickle_val: Value = serde_pickle::value_from_reader(&decompressed[..], DeOptions::new())?;

    let mut new_index: BTreeMap<String, Vec<(u64, u64, ByteBuf)>> = BTreeMap::new();
    if let Value::Dict(dict) = pickle_val {
        for (k, v) in dict {
            let filename = match k {
                HashableValue::String(s) => s,
                HashableValue::Bytes(b) => String::from_utf8_lossy(&b).to_string(),
                _ => continue,
            };

            let mut segs = Vec::new();
            match v {
                Value::List(list) => {
                    for item in list {
                        if let Value::Tuple(t) = item {
                            if t.len() >= 2 {
                                let off = match t[0] {
                                    Value::I64(x) => x as u64,
                                    _ => 0,
                                };
                                let len = match t[1] {
                                    Value::I64(x) => x as u64,
                                    _ => 0,
                                };
                                segs.push((off, len, ByteBuf::from(vec![])));
                            }
                        }
                    }
                }
                _ => {}
            }
            if !segs.is_empty() {
                new_index.insert(filename, segs);
            }
        }
    }

    println!("Converted {} file entries in index.", new_index.len());

    // 5. Pickle new index
    let new_pickled = serde_pickle::to_vec(&new_index, SerOptions::default())?;
    println!("New pickled size: {} bytes", new_pickled.len());

    // 6. Compress with zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&new_pickled)?;
    let new_compressed = encoder.finish()?;
    println!("New compressed index size: {} bytes", new_compressed.len());

    // 7. Write new index back to file at index_offset
    file.seek(SeekFrom::Start(index_offset))?;
    file.write_all(&new_compressed)?;
    let final_pos = file.stream_position()?;
    file.set_len(final_pos)?;
    file.flush()?;

    println!("SUCCESS! Repaired RPA index with ByteBuf. Final file size: {} bytes", final_pos);
    Ok(())
}
