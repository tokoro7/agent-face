use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "agent-face", about = "Give your AI coding agent a face")]
pub struct Cli {
    /// Character name to use
    #[arg(long, default_value = "cat")]
    pub character: String,

    /// Path to the state file
    #[arg(long)]
    pub state_file: Option<PathBuf>,

    /// Directory to search for character TOML files
    #[arg(long)]
    pub characters_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Write a state to the state file
    Set {
        /// State name (idle, thinking, writing, error, success, listening)
        state: String,
    },
}
