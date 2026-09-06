use tracing::info;
use crate::scene::DialogueLine;

pub struct ScriptExtractor;

impl ScriptExtractor {
    /// Extracts readable dialogue lines with associated background scene tags from decompressed Python pickle bytecode.
    pub fn extract_dialogue_from_payload(payload: &[u8], known_images: &std::collections::HashSet<String>) -> Vec<DialogueLine> {
        let mut lines = Vec::new();
        let mut current_speaker: Option<String> = None;
        let mut current_bg: Option<String> = None;
        let mut i = 0;

        while i < payload.len() {
            // Opcode 'X': UTF-8 string: X\xLL\xLL\xLL\xLL<bytes>
            if payload[i] == b'X' && i + 5 <= payload.len() {
                let len = u32::from_le_bytes([
                    payload[i + 1],
                    payload[i + 2],
                    payload[i + 3],
                    payload[i + 4],
                ]) as usize;
                i += 5;

                if i + len <= payload.len() {
                    if let Ok(s) = std::str::from_utf8(&payload[i..i + len]) {
                        let trimmed = s.trim();

                        // Check if string is a scene/background tag or known CG image
                        let is_known_cg = known_images.contains(&trimmed.to_lowercase());
                        let is_bg_tag = (trimmed.contains("background") || trimmed.contains("_bg") || trimmed.starts_with("d1_") || trimmed.starts_with("d2_") || trimmed.starts_with("d3_") || trimmed.starts_with("d4_") || trimmed.starts_with("d5_") || trimmed.starts_with("d6_") || trimmed.starts_with("d7_") || trimmed.starts_with("a_") || trimmed.starts_with("b_") || trimmed.starts_with("c_") || trimmed.starts_with("d_") || trimmed.starts_with("e_") || trimmed.starts_with("f_") || trimmed.starts_with("g_"))
                            && trimmed.len() <= 40
                            && !trimmed.contains('/')
                            && !trimmed.contains(' ')
                            && !trimmed.contains('.')
                            && !trimmed.starts_with("crenpy")
                            && trimmed != "version"
                            && trimmed != "unlocked";

                        if is_known_cg || is_bg_tag {
                            current_bg = Some(trimmed.to_string());
                        }

                        // Ignore technical identifiers, file paths, python code, dicts, json
                        let is_technical = trimmed.is_empty()
                            || trimmed.starts_with('/')
                            || trimmed.starts_with('{')
                            || trimmed.ends_with('}')
                            || trimmed.ends_with(".png")
                            || trimmed.ends_with(".jpg")
                            || trimmed.ends_with(".rpy")
                            || trimmed.ends_with(".rpyc")
                            || trimmed.contains("renpy.")
                            || trimmed.contains("def ")
                            || trimmed.contains("class ")
                            || trimmed.contains("import ")
                            || trimmed.contains("\":\"")
                            || trimmed.contains("\": \"")
                            || trimmed.contains("#")
                            || trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

                        if !is_technical {
                            // Check if this string is a character name in quotes: e.g. "밥" or "베라"
                            if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() <= 20 {
                                let name = trimmed[1..trimmed.len() - 1].trim();
                                if !name.is_empty() {
                                    current_speaker = Some(name.to_string());
                                }
                            } else if trimmed.len() >= 2 {
                                // Clean dialogue text
                                let dialogue = trimmed.to_string();
                                let mut line = DialogueLine::new(current_speaker.take(), dialogue);
                                if let Some(bg) = &current_bg {
                                    line.background = Some(bg.clone());
                                }
                                lines.push(line);
                            }
                        }
                    }
                    i += len;
                    continue;
                }
            }
            i += 1;
        }

        info!("Extracted {} clean dialogue lines from script payload.", lines.len());
        lines
    }
}
