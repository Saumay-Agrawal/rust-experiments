use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const RESET: &str = "\x1b[0m";
const MUTED: &str = "\x1b[2;90m";
const COMBINATION: &str = "\x1b[1;34m";
const CODE: &str = "\x1b[37m";
const METADATA: &str = "\x1b[2;37m";
const HEADER: &str = "\x1b[1;97m";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputStyle {
    pub color: bool,
}

impl OutputStyle {
    pub fn for_stdout(is_terminal: bool) -> Self {
        Self {
            color: is_terminal && std::env::var_os("NO_COLOR").is_none(),
        }
    }

    fn paint(self, color: &str, value: &str) -> String {
        if self.color {
            format!("{color}{value}{RESET}")
        } else {
            value.to_owned()
        }
    }
}

pub fn combination(event: &KeyEvent) -> String {
    let mut parts = Vec::new();
    for (flag, name) in [
        (KeyModifiers::CONTROL, "Ctrl"),
        (KeyModifiers::ALT, "Alt"),
        (KeyModifiers::SHIFT, "Shift"),
        (KeyModifiers::SUPER, "Super"),
        (KeyModifiers::HYPER, "Hyper"),
        (KeyModifiers::META, "Meta"),
    ] {
        if event.modifiers.contains(flag) {
            parts.push(name.to_owned());
        }
    }
    parts.push(match event.code {
        KeyCode::Char(' ') => "Space".into(),
        KeyCode::Char('+') => "Plus".into(),
        KeyCode::Char(c) => c.escape_debug().to_string(),
        KeyCode::F(n) => format!("F{n}"),
        code => format!("{code:?}"),
    });
    parts.join("+")
}

pub fn header(style: OutputStyle) -> String {
    let row = format!(
        "{}│ {} │ {} │ {} │ {} │ {}",
        style.paint(HEADER, &fit("#", 6)),
        style.paint(HEADER, &fit("COMBINATION", 18)),
        style.paint(HEADER, &fit("CODE", 24)),
        style.paint(HEADER, &fit("MODIFIERS", 22)),
        style.paint(HEADER, &fit("KIND", 10)),
        style.paint(HEADER, "STATE"),
    );
    format!(
        "{}\r\n{}",
        style.paint(
            MUTED,
            &"─".repeat(6 + 2 + 18 + 2 + 3 + 24 + 2 + 3 + 22 + 2 + 3 + 10 + 2 + 5)
        ),
        row
    )
}

pub fn banner(capability: &str, style: OutputStyle) -> String {
    const WIDTH: usize = 110;
    let border = style.paint(MUTED, &format!("┌{}┐", "─".repeat(WIDTH + 2)));
    let bottom = style.paint(MUTED, &format!("└{}┘", "─".repeat(WIDTH + 2)));
    let mut lines = vec![
        boxed(style, WIDTH, &[(HEADER, "KEYBOARD EVENT INSPECTOR")]),
        boxed(style, WIDTH, &[(COMBINATION, "● Ready")]),
        boxed(style, WIDTH, &[]),
        boxed(
            style,
            WIDTH,
            &[
                (HEADER, "Description: "),
                (
                    MUTED,
                    "Inspect exactly what your terminal reports for each key event.",
                ),
            ],
        ),
        boxed(
            style,
            WIDTH,
            &[(HEADER, "Reporting: "), (MUTED, capability)],
        ),
        boxed(
            style,
            WIDTH,
            &[
                (HEADER, "Events: "),
                (
                    MUTED,
                    "Every received key event is logged with its code, modifiers, kind, and state.",
                ),
            ],
        ),
        boxed(
            style,
            WIDTH,
            &[
                (HEADER, "Exit: "),
                (MUTED, "Press Escape three times within one second."),
            ],
        ),
        boxed(
            style,
            WIDTH,
            &[
                (HEADER, "Reset: "),
                (MUTED, "Any other key press resets the exit sequence."),
            ],
        ),
        boxed(
            style,
            WIDTH,
            &[
                (HEADER, "Note: "),
                (
                    MUTED,
                    "Repeat and release details depend on terminal support.",
                ),
            ],
        ),
    ];
    lines.insert(0, border);
    lines.push(bottom);
    lines.join("\r\n")
}

fn boxed(style: OutputStyle, width: usize, parts: &[(&str, &str)]) -> String {
    let visible = parts
        .iter()
        .map(|(_, text)| text.chars().count())
        .sum::<usize>();
    let content = parts
        .iter()
        .map(|(color, text)| style.paint(color, text))
        .collect::<String>();
    format!(
        "│ {}{} │",
        content,
        " ".repeat(width.saturating_sub(visible))
    )
}

pub fn event_line_styled(number: u64, event: &KeyEvent, style: OutputStyle) -> String {
    let combination = combination(event);
    let code = format!("{:?}", event.code);
    let modifiers = format!("{:?}", event.modifiers);
    let kind = format!("{:?}", event.kind);
    let state = format!("{:?}", event.state);
    format!(
        "{}│ {} │ {} │ {} │ {} │ {}",
        style.paint(MUTED, &fit(&format!("#{number}"), 6)),
        style.paint(COMBINATION, &fit(&combination, 18)),
        style.paint(CODE, &fit(&code, 24)),
        style.paint(METADATA, &fit(&modifiers, 22)),
        style.paint(METADATA, &fit(&kind, 10)),
        style.paint(MUTED, &state),
    )
}

fn fit(value: &str, width: usize) -> String {
    let mut result: String = value.chars().take(width.saturating_sub(1)).collect();
    if value.chars().count() > width {
        result.push('…');
    }
    format!("{result:<width$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    #[test]
    fn preserves_case_and_orders_modifiers() {
        let event = KeyEvent::new(
            KeyCode::Char('A'),
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        );
        assert_eq!(combination(&event), "Ctrl+Shift+A");
        assert_eq!(
            combination(&KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE)),
            "A"
        );
    }

    #[test]
    fn names_special_keys_and_escapes_controls() {
        for (code, expected) in [
            (KeyCode::Char(' '), "Space"),
            (KeyCode::Char('+'), "Plus"),
            (KeyCode::Char('\n'), "\\n"),
            (KeyCode::Char('\u{1b}'), "\\u{1b}"),
            (KeyCode::Char('é'), "é"),
            (KeyCode::F(12), "F12"),
            (KeyCode::BackTab, "BackTab"),
        ] {
            assert_eq!(
                combination(&KeyEvent::new(code, KeyModifiers::NONE)),
                expected
            );
        }
    }

    #[test]
    fn includes_all_reported_fields_without_terminal_controls() {
        let event = KeyEvent::new_with_kind_and_state(
            KeyCode::Char('\u{1b}'),
            KeyModifiers::ALT,
            KeyEventKind::Release,
            KeyEventState::CAPS_LOCK,
        );
        let line = event_line_styled(7, &event, OutputStyle { color: false });
        for expected in [
            "#7",
            "Char('\\u{1b}')",
            "KeyModifiers(ALT)",
            "Release",
            "CAPS_LOCK",
        ] {
            assert!(line.contains(expected), "{line}");
        }
        assert!(!line.chars().any(char::is_control));
    }

    #[test]
    fn plain_rows_are_aligned_and_color_rows_are_reset() {
        let event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::CONTROL);
        let plain = event_line_styled(1, &event, OutputStyle { color: false });
        assert!(plain.contains("#1  "));
        assert!(plain.contains("│ Char('A')"));
        assert!(!plain.contains('\x1b'));
        let colored = event_line_styled(1, &event, OutputStyle { color: true });
        assert!(colored.contains(COMBINATION));
        assert!(colored.contains(CODE));
        assert!(colored.contains(MUTED));
        assert!(colored.ends_with(RESET));
    }

    #[test]
    fn long_values_are_truncated() {
        let event = KeyEvent::new(
            KeyCode::Char('界'),
            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
        );
        let line = event_line_styled(1, &event, OutputStyle { color: false });
        assert!(line.contains('…'));
        assert!(!line.chars().any(char::is_control));
    }

    #[test]
    fn banner_contains_the_complete_startup_context() {
        let banner = banner("enhanced reporting", OutputStyle { color: false });
        for text in [
            "KEYBOARD EVENT INSPECTOR",
            "Ready",
            "Description:",
            "Reporting:",
            "Events:",
            "Exit:",
            "Reset:",
            "Note:",
            "enhanced reporting",
        ] {
            assert!(banner.contains(text), "{text} missing from {banner}");
        }
        assert!(!banner.contains('\x1b'));
    }
}
