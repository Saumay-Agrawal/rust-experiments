use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct ExitSequence {
    first: Option<Instant>,
    count: u8,
}

impl ExitSequence {
    pub fn observe(&mut self, event: &KeyEvent, now: Instant) -> bool {
        if event.kind != KeyEventKind::Press {
            return false;
        }
        if event.code != KeyCode::Esc || event.modifiers != KeyModifiers::NONE {
            self.first = None;
            self.count = 0;
            return false;
        }
        match self.first {
            Some(first) if now.duration_since(first) <= Duration::from_secs(1) => self.count += 1,
            _ => {
                self.first = Some(now);
                self.count = 1;
            }
        }
        self.count == 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn esc() -> KeyEvent {
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
    }

    #[test]
    fn accepts_three_presses_at_inclusive_boundary() {
        let mut sequence = ExitSequence::default();
        let start = Instant::now();
        assert!(!sequence.observe(&esc(), start));
        assert!(!sequence.observe(&esc(), start + Duration::from_millis(500)));
        assert!(sequence.observe(&esc(), start + Duration::from_secs(1)));
    }

    #[test]
    fn expired_window_restarts_at_current_press() {
        let mut sequence = ExitSequence::default();
        let start = Instant::now();
        for ms in [0, 900, 1001, 1500] {
            assert!(!sequence.observe(&esc(), start + Duration::from_millis(ms)));
        }
        assert!(sequence.observe(&esc(), start + Duration::from_millis(2001)));
    }

    #[test]
    fn other_and_modified_presses_reset() {
        for interrupt in [
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT),
        ] {
            let mut sequence = ExitSequence::default();
            let now = Instant::now();
            assert!(!sequence.observe(&esc(), now));
            assert!(!sequence.observe(&esc(), now));
            assert!(!sequence.observe(&interrupt, now));
            assert!(!sequence.observe(&esc(), now));
            assert!(!sequence.observe(&esc(), now));
            assert!(sequence.observe(&esc(), now));
        }
    }

    #[test]
    fn repeats_and_releases_do_not_advance_or_reset() {
        let mut sequence = ExitSequence::default();
        let now = Instant::now();
        assert!(!sequence.observe(&esc(), now));
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert!(!sequence.observe(
                &KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, kind),
                now
            ));
        }
        assert!(!sequence.observe(&esc(), now));
        assert!(sequence.observe(&esc(), now));
    }
}
