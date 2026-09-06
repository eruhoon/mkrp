pub mod scene;
pub mod script;

pub use scene::{DialogueLine, SceneMode, SceneState};
pub use script::ScriptExtractor;

pub fn info() -> &'static str {
    "mkrp-core v0.1.0"
}
