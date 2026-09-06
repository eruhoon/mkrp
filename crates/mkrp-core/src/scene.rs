use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceItem {
    pub text: String,
    pub target_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLine {
    pub speaker: Option<String>,
    pub text: String,
    pub background: Option<String>,
    pub music: Option<String>,
    pub sound: Option<String>,
    pub choices: Vec<ChoiceItem>,
}

impl DialogueLine {
    pub fn new(speaker: Option<String>, text: String) -> Self {
        Self {
            speaker,
            text,
            background: None,
            music: None,
            sound: None,
            choices: Vec::new(),
        }
    }

    pub fn with_background(mut self, bg: String) -> Self {
        self.background = Some(bg);
        self
    }

    pub fn with_choices(mut self, choices: Vec<ChoiceItem>) -> Self {
        self.choices = choices;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMode {
    TitleMenu,
    Dialogue,
    ChoiceMenu,
    Paused,
}

pub struct SceneState {
    pub mode: SceneMode,
    pub lines: Vec<DialogueLine>,
    pub current_index: usize,
    pub current_background: Option<String>,
    pub ui_visible: bool,
    pub auto_mode: bool,
    pub skip_mode: bool,
    pub selected_choice: usize,
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            mode: SceneMode::Dialogue,
            lines: Vec::new(),
            current_index: 0,
            current_background: None,
            ui_visible: true,
            auto_mode: false,
            skip_mode: false,
            selected_choice: 0,
        }
    }

    pub fn load_lines(&mut self, lines: Vec<DialogueLine>) {
        self.lines = lines;
        self.current_index = 0;
        self.update_current_attributes();
    }

    pub fn current_line(&self) -> Option<&DialogueLine> {
        self.lines.get(self.current_index)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.lines.len() {
            self.current_index += 1;
            self.update_current_attributes();
            true
        } else {
            false
        }
    }

    pub fn rollback(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.update_current_attributes();
            true
        } else {
            false
        }
    }

    pub fn toggle_ui(&mut self) {
        self.ui_visible = !self.ui_visible;
    }

    pub fn toggle_auto(&mut self) {
        self.auto_mode = !self.auto_mode;
        if self.auto_mode {
            self.skip_mode = false;
        }
    }

    pub fn toggle_skip(&mut self) {
        self.skip_mode = !self.skip_mode;
        if self.skip_mode {
            self.auto_mode = false;
        }
    }

    pub fn choice_up(&mut self) {
        if let Some(line) = self.current_line() {
            if !line.choices.is_empty() && self.selected_choice > 0 {
                self.selected_choice -= 1;
            }
        }
    }

    pub fn choice_down(&mut self) {
        if let Some(line) = self.current_line() {
            if !line.choices.is_empty() && self.selected_choice + 1 < line.choices.len() {
                self.selected_choice += 1;
            }
        }
    }

    fn update_current_attributes(&mut self) {
        self.selected_choice = 0;
        if let Some(line) = self.lines.get(self.current_index) {
            if !line.choices.is_empty() {
                self.mode = SceneMode::ChoiceMenu;
            } else {
                self.mode = SceneMode::Dialogue;
            }
            if let Some(bg) = &line.background {
                self.current_background = Some(bg.clone());
            }
        }
    }
}

impl Default for SceneState {
    fn default() -> Self {
        Self::new()
    }
}
