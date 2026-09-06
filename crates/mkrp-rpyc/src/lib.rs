pub mod decoder;
pub mod error;

pub use decoder::{RenpyVersion, RpycFile, Slot, RPYC2_MAGIC};
pub use error::RpycError;

pub fn info() -> &'static str {
    "mkrp-rpyc v0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{ByteOrder, LittleEndian};
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_rpyc2_parsing_and_version_detection() {
        // Create mock decompressed payload with Ren'Py 8 signature
        let payload_r8 = b"(dp1\nS'renpy.astsupport'\np2\n";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload_r8).unwrap();
        let compressed_r8 = encoder.finish().unwrap();

        // Build RPYC2 file structure
        let mut rpyc_bytes = Vec::new();
        rpyc_bytes.extend_from_slice(RPYC2_MAGIC); // 10 bytes

        // Slot table:
        // Slot 1: offset 34, length of compressed blob
        let slot_table_size = 12 + 12; // Slot 1 entry + Slot 0 terminator = 24 bytes
        let start_offset = (10 + slot_table_size) as u32;

        let mut slot1_entry = [0u8; 12];
        LittleEndian::write_u32(&mut slot1_entry[0..4], 1); // slot id = 1
        LittleEndian::write_u32(&mut slot1_entry[4..8], start_offset);
        LittleEndian::write_u32(&mut slot1_entry[8..12], compressed_r8.len() as u32);
        rpyc_bytes.extend_from_slice(&slot1_entry);

        let mut slot0_term = [0u8; 12];
        LittleEndian::write_u32(&mut slot0_term[0..4], 0); // slot id = 0 (terminator)
        rpyc_bytes.extend_from_slice(&slot0_term);

        // Append compressed data
        rpyc_bytes.extend_from_slice(&compressed_r8);

        // Parse with RpycFile
        let rpyc = RpycFile::from_bytes(&rpyc_bytes).expect("Failed to parse RPYC2 file");
        assert!(rpyc.is_v2());
        assert_eq!(rpyc.version(), RenpyVersion::Renpy8);
        assert_eq!(rpyc.decompressed_payload(), payload_r8);
        assert_eq!(rpyc.slots().len(), 1);
        assert_eq!(rpyc.slots().get(&1).unwrap().length, compressed_r8.len());
    }

    #[test]
    fn test_rpyc2_renpy7_detection() {
        // Create mock decompressed payload with Ren'Py 7 signature
        let payload_r7 = b"(dp1\nS'c__builtin__'\np2\n";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload_r7).unwrap();
        let compressed_r7 = encoder.finish().unwrap();

        let mut rpyc_bytes = Vec::new();
        rpyc_bytes.extend_from_slice(RPYC2_MAGIC);

        let slot_table_size = 12 + 12;
        let start_offset = (10 + slot_table_size) as u32;

        let mut slot1_entry = [0u8; 12];
        LittleEndian::write_u32(&mut slot1_entry[0..4], 1);
        LittleEndian::write_u32(&mut slot1_entry[4..8], start_offset);
        LittleEndian::write_u32(&mut slot1_entry[8..12], compressed_r7.len() as u32);
        rpyc_bytes.extend_from_slice(&slot1_entry);

        let mut slot0_term = [0u8; 12];
        LittleEndian::write_u32(&mut slot0_term[0..4], 0);
        rpyc_bytes.extend_from_slice(&slot0_term);

        rpyc_bytes.extend_from_slice(&compressed_r7);

        let rpyc = RpycFile::from_bytes(&rpyc_bytes).expect("Failed to parse RPYC2 file");
        assert_eq!(rpyc.version(), RenpyVersion::Renpy7);
        assert_eq!(rpyc.decompressed_payload(), payload_r7);
    }
}
