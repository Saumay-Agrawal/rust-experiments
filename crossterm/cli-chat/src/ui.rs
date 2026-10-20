use crate::{
    editor,
    ollama::{Message, Model},
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::{
    io::{self, Write},
    ops::Range,
};
use unicode_width::UnicodeWidthChar;

pub fn draw_menu(models: &[&Model], selected: usize, filter: &str, status: &str) -> io::Result<()> {
    let mut out = io::stdout();
    let (width, height) = terminal::size()?;
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(2, 1),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        Print("Ollama Chat"),
        ResetColor,
        SetAttribute(Attribute::Reset)
    )?;
    queue!(
        out,
        MoveTo(2, 3),
        SetForegroundColor(Color::Yellow),
        Print("Filter models: "),
        ResetColor,
        Print(truncate(filter, width.saturating_sub(20) as usize))
    )?;

    let visible = usize::from(height.saturating_sub(8)).max(1);
    let first = selected.saturating_sub(visible.saturating_sub(1));
    if models.is_empty() {
        queue!(
            out,
            MoveTo(3, 5),
            SetForegroundColor(Color::DarkGrey),
            Print("No matches. Clear the filter or try another name."),
            ResetColor
        )?;
    } else {
        for (offset, model) in models.iter().skip(first).take(visible).enumerate() {
            let index = first + offset;
            queue!(out, MoveTo(3, 5 + offset as u16))?;
            if index == selected {
                queue!(
                    out,
                    SetForegroundColor(Color::Yellow),
                    SetAttribute(Attribute::Bold),
                    Print("> ")
                )?;
            } else {
                queue!(out, Print("  "))?;
            }
            queue!(
                out,
                Print(truncate(&model.name, width.saturating_sub(8) as usize)),
                ResetColor,
                SetAttribute(Attribute::Reset)
            )?;
        }
    }
    queue!(
        out,
        MoveTo(2, height.saturating_sub(3)),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(status, width.saturating_sub(4) as usize)),
        MoveTo(2, height.saturating_sub(2)),
        Print("Type to filter · ↑/↓ move · Enter select · F1/? help · q quit"),
        ResetColor
    )?;
    let filter_cursor = 2usize
        + "Filter models: ".len()
        + filter
            .chars()
            .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
            .sum::<usize>();
    queue!(
        out,
        MoveTo(
            filter_cursor.min(width.saturating_sub(1) as usize) as u16,
            3
        ),
        Show
    )?;
    out.flush()
}

pub fn draw_notice(title: &str, body: &str, footer: &str) -> io::Result<()> {
    let mut out = io::stdout();
    let (width, height) = terminal::size()?;
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(2, 1),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        Print(title),
        ResetColor,
        SetAttribute(Attribute::Reset)
    )?;
    for (index, line) in wrap(body, width.saturating_sub(4) as usize)
        .into_iter()
        .take(usize::from(height.saturating_sub(5)))
        .enumerate()
    {
        queue!(out, MoveTo(2, 3 + index as u16), Print(line))?;
    }
    queue!(
        out,
        MoveTo(2, height.saturating_sub(2)),
        SetForegroundColor(Color::DarkGrey),
        Print(truncate(footer, width.saturating_sub(4) as usize)),
        ResetColor,
        Hide
    )?;
    out.flush()
}

pub fn draw_help(title: &str, lines: &[&str]) -> io::Result<()> {
    let mut out = io::stdout();
    let (width, height) = terminal::size()?;
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(2, 1),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        Print(format!("Help · {title}")),
        ResetColor,
        SetAttribute(Attribute::Reset)
    )?;
    for (index, line) in lines.iter().enumerate() {
        let y = 3 + index as u16;
        if y >= height.saturating_sub(2) {
            break;
        }
        queue!(
            out,
            MoveTo(2, y),
            Print(truncate(line, width.saturating_sub(4) as usize))
        )?;
    }
    queue!(
        out,
        MoveTo(2, height.saturating_sub(2)),
        SetForegroundColor(Color::DarkGrey),
        Print("Press Esc, F1, or ? to close help."),
        ResetColor,
        Hide
    )?;
    out.flush()
}

pub struct ChatView<'a> {
    pub model: &'a str,
    pub messages: &'a [Message],
    pub input: &'a [char],
    pub cursor: usize,
    pub status: &'a str,
    pub error: Option<&'a str>,
    pub generating: bool,
    pub scroll_from_bottom: usize,
}

pub fn draw_chat(view: ChatView<'_>) -> io::Result<()> {
    let ChatView {
        model,
        messages,
        input,
        cursor,
        status,
        error,
        generating,
        scroll_from_bottom,
    } = view;
    let mut out = io::stdout();
    let (terminal_width, height) = terminal::size()?;
    let width = usize::from(terminal_width.max(1));
    let divider_y = height.saturating_sub(6);
    let transcript_height = usize::from(divider_y.saturating_sub(1));
    let body_top = divider_y.saturating_add(2);
    let status_y = height.saturating_sub(1);
    let editor_rows = usize::from(status_y.saturating_sub(body_top)).max(1);

    let rows = transcript_rows(messages, error, width);

    let visible_rows = transcript_window(rows.len(), transcript_height, scroll_from_bottom);
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(1, 0),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        Print(format!(" Ollama Chat  ·  {model}")),
        ResetColor,
        SetAttribute(Attribute::Reset)
    )?;
    for (offset, (line, color)) in rows[visible_rows].iter().enumerate() {
        let y = 1 + offset as u16;
        queue!(
            out,
            MoveTo(1, y),
            SetForegroundColor(*color),
            Print(line),
            ResetColor
        )?;
    }
    queue!(out, MoveTo(0, divider_y), Print("─".repeat(width)))?;
    queue!(
        out,
        MoveTo(1, divider_y + 1),
        SetForegroundColor(Color::Yellow),
        Print("Message · Enter sends · Shift+Enter/Ctrl+J newline · F1/? help"),
        ResetColor
    )?;

    let editor_width = width.saturating_sub(2).max(1);
    let input_lines = editor::layout(input, editor_width);
    let (cursor_line, cursor_column) = editor::cursor_position(input, cursor, editor_width);
    let editor_start = cursor_line.saturating_sub(editor_rows.saturating_sub(1));
    for (offset, line) in input_lines
        .iter()
        .skip(editor_start)
        .take(editor_rows)
        .enumerate()
    {
        queue!(
            out,
            MoveTo(1, body_top + offset as u16),
            Print(truncate(&line.text, editor_width))
        )?;
    }
    let mut footer = if generating {
        "Generating · Esc cancels · Ctrl+C quits".to_owned()
    } else {
        status.to_owned()
    };
    if scroll_from_bottom == 0 {
        footer.push_str("  ·  PgUp/PgDn scroll · ? help · q quit");
    } else if input.is_empty() {
        footer.push_str("  ·  Scrolled back · End returns to latest");
    } else {
        footer.push_str("  ·  Scrolled back · clear prompt, then End for latest");
    }
    queue!(
        out,
        MoveTo(1, status_y),
        SetForegroundColor(if error.is_some() {
            Color::Red
        } else {
            Color::DarkGrey
        }),
        Print(truncate(&footer, width.saturating_sub(2))),
        ResetColor
    )?;
    let cursor_y = body_top + cursor_line.saturating_sub(editor_start) as u16;
    if cursor_y < status_y {
        queue!(
            out,
            MoveTo(
                (cursor_column + 1).min(width.saturating_sub(1)) as u16,
                cursor_y
            ),
            Show
        )?;
    }
    out.flush()
}

pub fn transcript_row_count(
    messages: &[Message],
    error: Option<&str>,
    terminal_width: u16,
) -> usize {
    transcript_rows(messages, error, usize::from(terminal_width.max(1))).len()
}

fn transcript_rows(
    messages: &[Message],
    error: Option<&str>,
    width: usize,
) -> Vec<(String, Color)> {
    let mut rows = Vec::<(String, Color)>::new();
    for message in messages {
        let (label, color) = match message.role.as_str() {
            "user" => ("You", Color::Green),
            _ => ("Assistant", Color::Cyan),
        };
        let text = format!("{label}: {}", message.content);
        for line in text.split('\n') {
            wrap_styled(line, width.saturating_sub(2).max(1), &mut rows, color);
        }
        rows.push((String::new(), Color::Reset));
    }
    if let Some(error) = error {
        for line in wrap_styled_owned(
            &format!("Request failed: {error}"),
            width.saturating_sub(2).max(1),
        ) {
            rows.push((line, Color::Red));
        }
        rows.push((String::new(), Color::Reset));
    }

    rows
}

fn transcript_window(
    total_rows: usize,
    visible_rows: usize,
    scroll_from_bottom: usize,
) -> Range<usize> {
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let end = total_rows.saturating_sub(scroll_from_bottom.min(max_scroll));
    end.saturating_sub(visible_rows)..end
}

fn wrap_styled(line: &str, width: usize, rows: &mut Vec<(String, Color)>, color: Color) {
    let mut current = String::new();
    let mut column = 0;
    for character in line.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if column > 0 && column + character_width > width {
            rows.push((std::mem::take(&mut current), color));
            column = 0;
        }
        current.push(character);
        column += character_width;
    }
    rows.push((current, color));
}

fn wrap_styled_owned(line: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    wrap_styled(line, width, &mut rows, Color::Red);
    rows.into_iter().map(|(line, _)| line).collect()
}

fn wrap(line: &str, width: usize) -> Vec<String> {
    wrap_styled_owned(line, width.max(1))
}

fn truncate(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    for character in text.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if width + character_width > max_width {
            break;
        }
        result.push(character);
        width += character_width;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::transcript_window;

    #[test]
    fn transcript_follows_latest_until_user_scrolls_back() {
        assert_eq!(transcript_window(10, 3, 0), 7..10);
        assert_eq!(transcript_window(10, 3, 2), 5..8);
        assert_eq!(transcript_window(10, 3, usize::MAX), 0..3);
    }
}
