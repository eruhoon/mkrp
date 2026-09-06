pub mod archive;
pub mod error;
pub mod vfs;

pub use archive::{FileEntry, RpaArchive, RpaVersion, Segment};
pub use error::RpaError;
pub use vfs::Vfs;

pub fn info() -> &'static str {
    "mkrp-rpa v0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_rpa3_archive_creation_and_reading() {
        let dir = tempdir().unwrap();
        let rpa_path = dir.path().join("archive.rpa");

        let key: u64 = 0x42424242;
        let test_content_1 = b"Hello from Ren'Py RPA file 1!";
        let test_content_2 = b"Another asset with some data in it.";

        let mut file = File::create(&rpa_path).unwrap();

        // Write placeholder header (RPA-3.0 <offset:16 hex> <key:16 hex>\n)
        let header_str = format!("RPA-3.0 {:016x} {:016x}\n", 0u64, key);
        file.write_all(header_str.as_bytes()).unwrap();
        let header_len = header_str.len() as u64;

        // Write file 1
        let offset1 = header_len;
        let len1 = test_content_1.len() as u64;
        file.write_all(test_content_1).unwrap();

        // Write file 2
        let offset2 = offset1 + len1;
        let len2 = test_content_2.len() as u64;
        file.write_all(test_content_2).unwrap();

        // Index location
        let index_offset = offset2 + len2;

        // Build index dictionary
        // { "story/intro.txt": [(offset ^ key, len ^ key, b"")], ... }
        let mut index_map: BTreeMap<String, Vec<(u64, u64, Vec<u8>)>> = BTreeMap::new();
        index_map.insert(
            "story/intro.txt".into(),
            vec![(offset1 ^ key, len1 ^ key, vec![])],
        );
        index_map.insert(
            "images/bg.png".into(),
            vec![(offset2 ^ key, len2 ^ key, vec![])],
        );

        let pickle_bytes = serde_pickle::to_vec(&index_map, serde_pickle::SerOptions::new()).unwrap();

        // Zlib compress index
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&pickle_bytes).unwrap();
        let compressed_index = encoder.finish().unwrap();

        file.write_all(&compressed_index).unwrap();
        drop(file);

        // Rewrite real header with index_offset (not XORed)
        let mut file = std::fs::OpenOptions::new().write(true).open(&rpa_path).unwrap();
        let real_header = format!("RPA-3.0 {:016x} {:016x}\n", index_offset, key);
        file.write_all(real_header.as_bytes()).unwrap();
        drop(file);

        // Now test opening with RpaArchive
        let mut archive = RpaArchive::open(&rpa_path).expect("Failed to open archive");
        assert_eq!(archive.version(), RpaVersion::Rpa3);
        assert!(archive.contains("story/intro.txt"));
        assert!(archive.contains("images/bg.png"));

        let read1 = archive.read("story/intro.txt").expect("Failed to read file 1");
        assert_eq!(read1, test_content_1);

        let read2 = archive.read("images/bg.png").expect("Failed to read file 2");
        assert_eq!(read2, test_content_2);

        // Test with Vfs
        let mut vfs = Vfs::new();
        vfs.mount_archive(&rpa_path).expect("Failed to mount archive in VFS");
        assert!(vfs.exists("story/intro.txt"));
        assert_eq!(vfs.read("story/intro.txt").unwrap(), test_content_1);
    }
}
