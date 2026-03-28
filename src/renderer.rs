use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{self, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::character::Character;
use crate::state::{FaceState, StateMachine};
use crate::watcher::{StateWatcher, WatchEvent};

const TICK_DURATION: Duration = Duration::from_millis(50);

pub struct Renderer {
    state_machine: StateMachine,
    watcher: Option<StateWatcher>,
    characters: Vec<Character>,
    current_char_idx: usize,
    frame_index: usize,
    last_frame_time: Instant,
    needs_redraw: bool,
}

pub enum Action {
    Continue,
    Quit,
}

impl Renderer {
    pub fn new(
        characters: Vec<Character>,
        watcher: Option<StateWatcher>,
    ) -> Self {
        Self {
            state_machine: StateMachine::new(),
            watcher,
            characters,
            current_char_idx: 0,
            frame_index: 0,
            last_frame_time: Instant::now(),
            needs_redraw: true,
        }
    }

    pub fn set_character_index(&mut self, idx: usize) {
        if idx < self.characters.len() {
            self.current_char_idx = idx;
            self.frame_index = 0;
            self.needs_redraw = true;
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

        // Always start from idle, ignoring any stale state file content.

        loop {
            match self.tick()? {
                Action::Quit => break,
                Action::Continue => {}
            }
        }

        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn tick(&mut self) -> io::Result<Action> {
        // Handle keyboard input.
        if event::poll(TICK_DURATION)? {
            if let Event::Key(key) = event::read()? {
                match self.handle_key(key) {
                    Action::Quit => return Ok(Action::Quit),
                    Action::Continue => {}
                }
            }
        }

        // Handle state file changes.
        if let Some(watcher) = &self.watcher {
            while let Some(ev) = watcher.try_recv() {
                match ev {
                    WatchEvent::StateChanged(state) => {
                        if self.state_machine.set(state) {
                            self.frame_index = 0;
                            self.needs_redraw = true;
                        }
                    }
                    WatchEvent::FileDeleted => {
                        self.state_machine.set(FaceState::Idle);
                        self.frame_index = 0;
                        self.needs_redraw = true;
                    }
                }
            }
        }

        // Handle timed transitions.
        if self.state_machine.tick() {
            self.frame_index = 0;
            self.needs_redraw = true;
        }

        // Advance frame animation.
        let character = &self.characters[self.current_char_idx];
        let state_frames = character.state(self.state_machine.current());
        let speed = Duration::from_millis(state_frames.speed_ms);
        if self.last_frame_time.elapsed() >= speed {
            self.frame_index = (self.frame_index + 1) % state_frames.frames.len();
            self.last_frame_time = Instant::now();
            self.needs_redraw = true;
        }

        if self.needs_redraw {
            self.draw()?;
            self.needs_redraw = false;
        }

        Ok(Action::Continue)
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Action::Quit
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.characters.len() > 1 {
                    self.current_char_idx =
                        (self.current_char_idx + 1) % self.characters.len();
                    self.frame_index = 0;
                    self.needs_redraw = true;
                }
            }
            KeyCode::Char(ch @ '1'..='6') => {
                let states = [
                    FaceState::Idle,
                    FaceState::Thinking,
                    FaceState::Writing,
                    FaceState::Error,
                    FaceState::Success,
                    FaceState::Listening,
                ];
                let idx = (ch as usize) - ('1' as usize);
                if self.state_machine.set(states[idx]) {
                    self.frame_index = 0;
                    self.needs_redraw = true;
                }
            }
            _ => {}
        }
        Action::Continue
    }

    fn draw(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let (cols, rows) = terminal::size()?;
        let character = &self.characters[self.current_char_idx];
        let state = self.state_machine.current();
        let state_frames = character.state(state);
        let frame = &state_frames.frames[self.frame_index];
        let color = state_frames.color;

        execute!(stdout, terminal::Clear(ClearType::All))?;

        // Header
        let header = format!("┤ agent-face ─ {} ├", character.display_name);
        let header_col = cols.saturating_sub(header.len() as u16) / 2;
        execute!(stdout, cursor::MoveTo(header_col, 1))?;
        write!(
            stdout,
            "{}",
            style::style(&header).with(style::Color::AnsiValue(color.ansi256))
        )?;

        // Character frame (centered)
        let frame_height = frame.len() as u16;
        let frame_start_row = (rows.saturating_sub(frame_height + 6)) / 2 + 3;
        for (i, line) in frame.iter().enumerate() {
            let line_width = line.chars().count() as u16;
            let col = cols.saturating_sub(line_width) / 2;
            execute!(stdout, cursor::MoveTo(col, frame_start_row + i as u16))?;
            write!(
                stdout,
                "{}",
                style::style(line).with(style::Color::AnsiValue(color.ansi256))
            )?;
        }

        // State badge
        let badge = format!("▸▸▸ {} ◂◂◂", state.as_str().to_uppercase());
        let badge_col = cols.saturating_sub(badge.len() as u16) / 2;
        let badge_row = frame_start_row + frame_height + 2;
        execute!(stdout, cursor::MoveTo(badge_col, badge_row))?;
        write!(
            stdout,
            "{}",
            style::style(&badge)
                .with(style::Color::AnsiValue(color.ansi256))
                .bold()
        )?;

        // Footer
        let mode = if self.watcher.is_some() { "AUTO" } else { "MANUAL" };
        let footer = format!(
            "[1]idle [2]think [3]write [4]err [5]ok [6]listen [c]char [q]quit  {mode}"
        );
        let footer_col = cols.saturating_sub(footer.len() as u16) / 2;
        execute!(stdout, cursor::MoveTo(footer_col, rows - 2))?;
        write!(
            stdout,
            "{}",
            style::style(&footer).with(style::Color::DarkGrey)
        )?;

        stdout.flush()?;
        Ok(())
    }
}
