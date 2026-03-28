use std::fmt;
use std::str::FromStr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceState {
    Idle,
    Thinking,
    Writing,
    Error,
    Success,
    Listening,
}

impl FaceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            FaceState::Idle => "idle",
            FaceState::Thinking => "thinking",
            FaceState::Writing => "writing",
            FaceState::Error => "error",
            FaceState::Success => "success",
            FaceState::Listening => "listening",
        }
    }
}

impl fmt::Display for FaceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FaceState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "idle" => Ok(FaceState::Idle),
            "thinking" => Ok(FaceState::Thinking),
            "writing" => Ok(FaceState::Writing),
            "error" => Ok(FaceState::Error),
            "success" => Ok(FaceState::Success),
            "listening" => Ok(FaceState::Listening),
            other => Err(format!("unknown state: {other}")),
        }
    }
}

/// Timeout after which success auto-transitions to idle.
const SUCCESS_TIMEOUT: Duration = Duration::from_secs(3);

pub struct StateMachine {
    current: FaceState,
    entered_at: Instant,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: FaceState::Idle,
            entered_at: Instant::now(),
        }
    }

    pub fn current(&self) -> FaceState {
        self.current
    }

    /// Transition to a new state. Returns true if the state actually changed.
    pub fn set(&mut self, state: FaceState) -> bool {
        if self.current == state {
            return false;
        }
        self.current = state;
        self.entered_at = Instant::now();
        true
    }

    /// Call this on each tick to handle timed transitions (e.g. success → idle).
    pub fn tick(&mut self) -> bool {
        if self.current == FaceState::Success
            && self.entered_at.elapsed() >= SUCCESS_TIMEOUT
        {
            self.current = FaceState::Idle;
            self.entered_at = Instant::now();
            return true;
        }
        false
    }
}
