use std::io::{self, stdout};

use crossterm::{cursor, execute, style, terminal};

pub struct TerminalGuard {
    raw_mode: bool,
    alternate_screen: bool,
    cursor_hidden: bool,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let mut guard = Self {
            raw_mode: false,
            alternate_screen: false,
            cursor_hidden: false,
        };
        terminal::enable_raw_mode()?;
        guard.raw_mode = true;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        guard.alternate_screen = true;
        execute!(stdout(), cursor::Hide)?;
        guard.cursor_hidden = true;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(out, style::ResetColor);
        if self.cursor_hidden {
            let _ = execute!(out, cursor::Show);
        }
        if self.alternate_screen {
            let _ = execute!(out, terminal::LeaveAlternateScreen);
        }
        if self.raw_mode {
            let _ = terminal::disable_raw_mode();
        }
    }
}
