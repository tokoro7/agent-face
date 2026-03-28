use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use crate::state::FaceState;

pub enum WatchEvent {
    StateChanged(FaceState),
    FileDeleted,
}

pub struct StateWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<WatchEvent>,
    state_file: PathBuf,
}

impl StateWatcher {
    pub fn new(state_file: &Path) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let state_file_buf = state_file.to_path_buf();

        let file_tx = tx.clone();
        let watched_path = state_file_buf.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let Ok(event) = res else { return };
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        if let Some(state) = read_state_file(&watched_path) {
                            let _ = file_tx.send(WatchEvent::StateChanged(state));
                        }
                    }
                    EventKind::Remove(_) => {
                        let _ = file_tx.send(WatchEvent::FileDeleted);
                    }
                    _ => {}
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(100)),
        )
        .map_err(|e| format!("failed to create watcher: {e}"))?;

        // Watch the parent directory (the file itself may not exist yet).
        let parent = state_file
            .parent()
            .ok_or_else(|| "state file has no parent directory".to_string())?;

        if parent.exists() {
            watcher
                .watch(parent, RecursiveMode::NonRecursive)
                .map_err(|e| format!("failed to watch {}: {e}", parent.display()))?;
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            state_file: state_file_buf,
        })
    }

    /// Non-blocking poll for the next event.
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.rx.try_recv().ok()
    }

    /// Read the current state from the file (for initial load).
    pub fn read_current(&self) -> Option<FaceState> {
        read_state_file(&self.state_file)
    }
}

fn read_state_file(path: &Path) -> Option<FaceState> {
    let content = std::fs::read_to_string(path).ok()?;
    content.trim().parse().ok()
}
