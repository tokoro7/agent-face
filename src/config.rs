use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub claude_code_setup: bool,
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home)
        .join(".config")
        .join("agent-face")
        .join("config.toml")
}

pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("failed to serialize config: {e}"))?;
    fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    Ok(())
}
