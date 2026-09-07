use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use rayon::prelude::*;

use mkrp_rpa::RpaArchive;

enum ConvertedImage {
    Jpeg(Vec<u8>),
    Png(Vec<u8>),
}

fn decode_avif_and_convert(avif_bytes: &[u8]) -> Result<(ConvertedImage, &'static str), String> {
    let buffer = zenavif::decode(avif_bytes)
        .map_err(|e| format!("AVIF decode error: {:?}", e))?;
    let width = buffer.width();
    let height = buffer.height();
    let raw_bytes = buffer.copy_to_contiguous_bytes();
    let bpp = buffer.descriptor().bytes_per_pixel();

    let has_transparency = if bpp == 4 {
        // Check if any pixel has alpha < 250
        raw_bytes.chunks_exact(4).any(|p| p[3] < 250)
    } else {
        false
    };

    if has_transparency {
        // Encode to PNG to preserve alpha channel
        let mut png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        image::ImageEncoder::write_image(
            encoder,
            &raw_bytes,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        ).map_err(|e| format!("PNG encode error: {:?}", e))?;

        Ok((ConvertedImage::Png(png_bytes), ".png"))
    } else {
        // Encode to high-quality JPEG (Quality 82)
        let rgb_bytes = if bpp == 4 {
            let mut rgb = Vec::with_capacity((width * height * 3) as usize);
            for chunk in raw_bytes.chunks_exact(4) {
                rgb.extend_from_slice(&[chunk[0], chunk[1], chunk[2]]);
            }
            rgb
        } else if bpp == 3 {
            raw_bytes
        } else {
            return Err(format!("Unsupported bpp: {}", bpp));
        };

        let mut jpeg_bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 82);
        encoder.encode(
            &rgb_bytes,
            width,
            height,
            image::ExtendedColorType::Rgb8,
        ).map_err(|e| format!("JPEG encode error: {:?}", e))?;

        Ok((ConvertedImage::Jpeg(jpeg_bytes), ".jpg"))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let game_dir = Path::new("Y:/ports/VIRTUES/game");
    let local_output_path = Path::new("d:/workspace/mkrp/patch_virtues_assets.rpa");
    let target_rpa_on_device = Path::new("Y:/ports/VIRTUES/game/patch_virtues_assets.rpa");

    println!("============================================================");
    println!("🚀 Starting Optimized AVIF -> JPEG/PNG Conversion & RPA Packaging");
    println!("   Source Directory: {:?}", game_dir);
    println!("   Output File: {:?}", local_output_path);
    println!("============================================================");

    // 1. Scan all RPA archives for AVIF files
    let mut rpa_files = Vec::new();
    for entry in std::fs::read_dir(game_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rpa") {
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if !fname.starts_with("patch_virtues") {
                rpa_files.push(path);
            }
        }
    }
    rpa_files.sort();

    // 2. Initialize output RPA file
    let key: u64 = 0x42424242;
    let mut out_file = File::create(&local_output_path)?;

    // Write placeholder header (RPA-3.0 <offset:16 hex> <key:16 hex>\n)
    let placeholder_header = format!("RPA-3.0 {:016x} {:016x}\n", 0u64, key);
    out_file.write_all(placeholder_header.as_bytes())?;
    let mut current_offset = placeholder_header.len() as u64;

    let mut index_map: BTreeMap<String, Vec<(u64, u64, Vec<u8>)>> = BTreeMap::new();
    let total_converted = AtomicUsize::new(0);
    let total_jpeg = AtomicUsize::new(0);
    let total_png = AtomicUsize::new(0);
    let total_failed = AtomicUsize::new(0);

    // 3. Process each RPA archive
    for (rpa_idx, rpa_path) in rpa_files.iter().enumerate() {
        let rpa_name = rpa_path.file_name().unwrap().to_string_lossy();
        println!(
            "\n[{}/{}] Processing archive: {}",
            rpa_idx + 1,
            rpa_files.len(),
            rpa_name
        );

        let mut archive = match RpaArchive::open(rpa_path) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("   Warning: Failed to open {:?}: {}", rpa_name, e);
                continue;
            }
        };

        let avif_names: Vec<String> = archive
            .list_files()
            .filter(|n| n.to_lowercase().ends_with(".avif"))
            .map(|s| s.to_string())
            .collect();

        if avif_names.is_empty() {
            println!("   No AVIF files found, skipping.");
            continue;
        }

        println!("   Found {} AVIF files. Extracting raw data...", avif_names.len());

        let mut raw_items = Vec::with_capacity(avif_names.len());
        for name in avif_names {
            match archive.read(&name) {
                Ok(data) => raw_items.push((name, data)),
                Err(e) => {
                    eprintln!("   Failed to read {}: {}", name, e);
                    total_failed.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        println!("   Converting {} images in parallel (JPEG q=82 / PNG)...", raw_items.len());

        // Parallel decode AVIF -> encode JPEG/PNG
        let converted_items: Vec<(String, Vec<u8>, bool)> = raw_items
            .into_par_iter()
            .filter_map(|(name, avif_data)| {
                match decode_avif_and_convert(&avif_data) {
                    Ok((ConvertedImage::Jpeg(bytes), ext)) => {
                        let out_name = format!("{}{}", &name[..name.len() - 5], ext);
                        Some((out_name, bytes, false))
                    }
                    Ok((ConvertedImage::Png(bytes), ext)) => {
                        let out_name = format!("{}{}", &name[..name.len() - 5], ext);
                        Some((out_name, bytes, true))
                    }
                    Err(e) => {
                        eprintln!("   Convert failed for {}: {}", name, e);
                        total_failed.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
            })
            .collect();

        // Write converted files into output RPA
        for (out_name, out_bytes, is_png) in converted_items {
            let len = out_bytes.len() as u64;
            out_file.write_all(&out_bytes)?;

            index_map.insert(
                out_name,
                vec![(current_offset ^ key, len ^ key, Vec::new())],
            );

            current_offset += len;
            total_converted.fetch_add(1, Ordering::Relaxed);
            if is_png {
                total_png.fetch_add(1, Ordering::Relaxed);
            } else {
                total_jpeg.fetch_add(1, Ordering::Relaxed);
            }
        }

        println!(
            "   Archive done. Total converted: {} (JPEG: {}, PNG: {})",
            total_converted.load(Ordering::Relaxed),
            total_jpeg.load(Ordering::Relaxed),
            total_png.load(Ordering::Relaxed)
        );
    }

    // 4. Write RPA Index
    println!("\n============================================================");
    let index_offset = current_offset;
    println!("📦 Packaging RPA Index ({} files)...", index_map.len());

    let pickled_index = serde_pickle::to_vec(&index_map, serde_pickle::SerOptions::default())?;
    println!("   Pickled index size: {} bytes", pickled_index.len());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&pickled_index)?;
    let compressed_index = encoder.finish()?;
    println!("   Compressed index size: {} bytes", compressed_index.len());

    out_file.write_all(&compressed_index)?;

    // 5. Update RPA Header with final index offset
    out_file.seek(SeekFrom::Start(0))?;
    let final_header = format!("RPA-3.0 {:016x} {:016x}\n", index_offset, key);
    out_file.write_all(final_header.as_bytes())?;
    out_file.flush()?;

    let total_elapsed = start_time.elapsed();
    let out_metadata = std::fs::metadata(&local_output_path)?;
    let file_size_mb = out_metadata.len() as f64 / (1024.0 * 1024.0);

    println!("\n🎉 RPA Build Successful!");
    println!("   Total Converted: {}", total_converted.load(Ordering::Relaxed));
    println!("   JPEG (q=82):     {}", total_jpeg.load(Ordering::Relaxed));
    println!("   PNG (alpha):     {}", total_png.load(Ordering::Relaxed));
    println!("   Total Failed:    {}", total_failed.load(Ordering::Relaxed));
    println!("   Package Size:    {:.2} MB ({:.2} GB)", file_size_mb, file_size_mb / 1024.0);
    println!("   Elapsed Time:    {:.2} seconds", total_elapsed.as_secs_f64());
    println!("============================================================");

    // 6. Copy to device
    println!("\n🚚 Copying patch_virtues_assets.rpa to device ({:?})...", target_rpa_on_device);
    let copy_start = Instant::now();
    std::fs::copy(&local_output_path, &target_rpa_on_device)?;
    println!(
        "✅ Device copy completed in {:.2} seconds!",
        copy_start.elapsed().as_secs_f64()
    );

    Ok(())
}
