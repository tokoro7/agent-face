use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::state::FaceState;

#[derive(Debug, Deserialize)]
pub struct CharacterFile {
    pub name: String,
    pub display_name: String,
    pub states: HashMap<String, StateFrames>,
}

#[derive(Debug, Deserialize)]
pub struct StateFrames {
    pub color: String,
    pub speed_ms: u64,
    pub frames: Vec<String>,
}

/// Validated character ready for rendering.
pub struct Character {
    pub name: String,
    pub display_name: String,
    states: HashMap<FaceState, ValidatedStateFrames>,
}

pub struct ValidatedStateFrames {
    pub color: Color,
    pub speed_ms: u64,
    /// Each frame is a vec of lines.
    pub frames: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub ansi256: u8,
}

impl Color {
    fn from_name(name: &str) -> Result<Self, String> {
        let ansi256 = match name {
            "peach" => 216,
            "amber" => 214,
            "cyan" => 116,
            "red" => 174,
            "green" => 114,
            "lavender" => 183,
            other => return Err(format!("unknown color: {other}")),
        };
        Ok(Self { ansi256 })
    }
}

impl Character {
    pub fn state(&self, state: FaceState) -> &ValidatedStateFrames {
        &self.states[&state]
    }

    /// Load and validate a character from a TOML file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_toml(&content)
    }

    /// Parse and validate a character from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, String> {
        let file: CharacterFile =
            toml::from_str(content).map_err(|e| format!("TOML parse error: {e}"))?;

        let required = [
            FaceState::Idle,
            FaceState::Thinking,
            FaceState::Writing,
            FaceState::Error,
            FaceState::Success,
            FaceState::Listening,
        ];

        let mut states = HashMap::new();
        for face_state in required {
            let key = face_state.as_str();
            let raw = file
                .states
                .get(key)
                .ok_or_else(|| format!("missing state: {key}"))?;

            if raw.frames.is_empty() {
                return Err(format!("state {key}: must have at least one frame"));
            }
            if raw.speed_ms == 0 {
                return Err(format!("state {key}: speed_ms must be > 0"));
            }

            let color = Color::from_name(&raw.color)?;
            let frames: Vec<Vec<String>> = raw
                .frames
                .iter()
                .map(|f| f.lines().map(String::from).collect())
                .collect();

            states.insert(
                face_state,
                ValidatedStateFrames {
                    color,
                    speed_ms: raw.speed_ms,
                    frames,
                },
            );
        }

        Ok(Self {
            name: file.name,
            display_name: file.display_name,
            states,
        })
    }
}
