use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde_bytes::ByteBuf;
use serde_pickle::{DeOptions, HashableValue, SerOptions, Value};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let header = b"RPA-3.0 0000000099169b6c 42424242        \n";
    assert_eq!(header.len(), 42);
    let l = &header[..40];
    let offset = u64::from_str_radix(std::str::from_utf8(&l[8..24])?, 16)?;
    let key = u64::from_str_radix(std::str::from_utf8(&l[25..33])?, 16)?;
    println!("Parsed offset: {}, key: 0x{:x}", offset, key);
    assert_eq!(offset, 2568395628);
    assert_eq!(key, 0x42424242);

    // Write this 42-byte header to patch_virtues_assets.rpa
    let mut file = std::fs::OpenOptions::new().read(true).write(true).open("Y:/ports/VIRTUES/game/patch_virtues_assets.rpa")?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(header)?;
    file.flush()?;
    println!("Successfully updated header of patch_virtues_assets.rpa!");

    // Now test reading first file (day5_a5.jpg) and bg1700.png
    let mut read_file = std::fs::File::open("Y:/ports/VIRTUES/game/patch_virtues_assets.rpa")?;
    let mut h = [0u8; 42];
    read_file.read_exact(&mut h)?;
    println!("Header in file: {:?}", String::from_utf8_lossy(&h));

    // Read index
    read_file.seek(SeekFrom::Start(offset))?;
    let mut dec = Vec::new();
    flate2::read::ZlibDecoder::new(&mut read_file).read_to_end(&mut dec)?;
    let val: serde_pickle::Value = serde_pickle::from_slice(&dec, serde_pickle::DeOptions::new())?;
    if let serde_pickle::Value::Dict(d) = val {
        for (k, v) in d.iter() {
            let k_str = format!("{:?}", k);
            if k_str.contains("bg1700.png") {
                if let serde_pickle::Value::List(l) = v {
                    if let serde_pickle::Value::List(t) = &l[0] {
                        if let (serde_pickle::Value::I64(raw_off), serde_pickle::Value::I64(raw_len)) = (&t[0], &t[1]) {
                            let real_off = (*raw_off as u64) ^ key;
                            let real_len = (*raw_len as u64) ^ key;
                            read_file.seek(SeekFrom::Start(real_off))?;
                            let mut buf = vec![0u8; 16];
                            read_file.read_exact(&mut buf)?;
                            println!("bg1700.png at {}: first 16 bytes: {:02x?}", real_off, buf);
                            assert_eq!(&buf[1..4], b"PNG");
                            println!("VERIFICATION PASSED! PERFECT PNG HEADER!");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}







