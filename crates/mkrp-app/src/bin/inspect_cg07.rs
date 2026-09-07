use mkrp_rpa::RpaArchive;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpa_path = Path::new("Y:/ports/VIRTUES/game/cg07.rpa");
    let mut archive = RpaArchive::open(rpa_path)?;
    let sample = archive.list_files().find(|s| s.contains("d1_1_33")).unwrap().to_string();
    println!("Found in cg07.rpa: {}", sample);
    let data = archive.read(&sample)?;
    println!("Read {} bytes", data.len());
    println!("Header bytes: {:02X?}", &data[..std::cmp::min(16, data.len())]);
    Ok(())
}
