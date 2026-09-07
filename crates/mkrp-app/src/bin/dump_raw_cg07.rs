use mkrp_rpa::RpaArchive;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use flate2::read::ZlibDecoder;
use serde_pickle::{DeOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpa_path = Path::new("Y:/ports/VIRTUES/game/patch_virtues_assets.rpa");
    let mut f = File::open(rpa_path)?;
    let mut header_line = String::new();
    let mut r = std::io::BufReader::new(f);
    use std::io::BufRead;
    r.read_line(&mut header_line)?;
    let parts: Vec<&str> = header_line.trim().split_whitespace().collect();
    let offset = u64::from_str_radix(parts[1], 16)?;
    let key = u64::from_str_radix(parts[2], 16)?;
    r.seek(SeekFrom::Start(offset))?;
    let mut comp = Vec::new();
    r.read_to_end(&mut comp)?;
    let mut decomp = Vec::new();
    ZlibDecoder::new(&comp[..]).read_to_end(&mut decomp)?;
    let val: Value = serde_pickle::value_from_reader(&decomp[..], DeOptions::new())?;
    if let Value::Dict(d) = val {
        for (k, v) in d.iter().take(3) {
            println!("Key: {:?}, Value: {:?}", k, v);
        }
    }
    Ok(())
}
