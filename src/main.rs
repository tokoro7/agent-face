mod character;
mod cli;
mod renderer;
mod state;
mod watcher;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use character::Character;
use cli::{Cli, Command};
use renderer::Renderer;
use state::FaceState;
use watcher::StateWatcher;

fn default_state_file() -> PathBuf {
    dirs_state_file().unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("agent-face")
            .join("state")
    })
}

fn dirs_state_file() -> Option<PathBuf> {
    // For now, use a simple HOME-based path.
    // TODO: use `dirs` crate for proper XDG/macOS paths (DD-06)
    None
}

fn default_characters_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home)
        .join(".config")
        .join("agent-face")
        .join("characters")
}

/// Built-in cat character (fallback when no external files exist).
const BUILTIN_CAT: &str = include_str!("../characters/cat.toml");

fn load_characters(characters_dir: &PathBuf) -> Vec<Character> {
    let mut characters = Vec::new();

    // Load from external directory.
    if characters_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(characters_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    match Character::load(&path) {
                        Ok(c) => characters.push(c),
                        Err(e) => eprintln!("warning: {}: {e}", path.display()),
                    }
                }
            }
        }
    }

    // Fallback to built-in.
    if characters.is_empty() {
        characters.push(
            Character::from_toml(BUILTIN_CAT).expect("built-in cat character is invalid"),
        );
    }

    characters
}

fn cmd_set(state_name: &str, state_file: &PathBuf) -> Result<(), String> {
    // Validate state name.
    let _: FaceState = state_name
        .parse()
        .map_err(|_| format!("invalid state: {state_name}\nvalid states: idle, thinking, writing, error, success, listening"))?;

    // Ensure parent directory exists.
    if let Some(parent) = state_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }

    // Atomic write: write to temp file, then rename.
    let tmp = state_file.with_extension("tmp");
    fs::write(&tmp, format!("{state_name}\n"))
        .map_err(|e| format!("failed to write state: {e}"))?;
    fs::rename(&tmp, state_file)
        .map_err(|e| format!("failed to rename state file: {e}"))?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let state_file = cli.state_file.unwrap_or_else(default_state_file);

    match cli.command {
        Some(Command::Set { state }) => {
            if let Err(e) = cmd_set(&state, &state_file) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        None => {
            // Set up panic hook to restore terminal.
            let original_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(
                    io::stdout(),
                    crossterm::cursor::Show,
                    crossterm::terminal::LeaveAlternateScreen
                );
                original_hook(info);
            }));

            let characters_dir = cli.characters_dir.unwrap_or_else(default_characters_dir);
            let characters = load_characters(&characters_dir);

            // Select initial character.
            let initial_idx = characters
                .iter()
                .position(|c| c.name == cli.character)
                .unwrap_or(0);

            let watcher = StateWatcher::new(&state_file).ok();
            let mut renderer = Renderer::new(characters, watcher);
            renderer.set_character_index(initial_idx);

            if let Err(e) = renderer.run() {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
