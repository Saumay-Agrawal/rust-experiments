use unicode_width::UnicodeWidthChar;

#[derive(Debug, PartialEq, Eq)]
pub struct VisualLine {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

pub fn layout(input: &[char], width: usize) -> Vec<VisualLine> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut text = String::new();
    let mut start = 0;
    let mut end = 0;
    let mut column = 0;

    for (index, character) in input.iter().copied().enumerate() {
        if character == '\n' {
            end = index;
            lines.push(VisualLine { text, start, end });
            text = String::new();
            start = index + 1;
            end = start;
            column = 0;
            continue;
        }

        let character_width = if character == '\t' {
            4
        } else {
            UnicodeWidthChar::width(character).unwrap_or(0)
        };
        if column > 0 && column + character_width > width {
            end = index;
            lines.push(VisualLine {
                text: std::mem::take(&mut text),
                start,
                end,
            });
            start = index;
            column = 0;
        }
        text.push(character);
        column = column.saturating_add(character_width);
        end = index + 1;
    }

    lines.push(VisualLine { text, start, end });
    lines
}

pub fn cursor_position(input: &[char], cursor: usize, width: usize) -> (usize, usize) {
    let lines = layout(input, width);
    let cursor = cursor.min(input.len());
    lines
        .iter()
        .enumerate()
        .find(|(index, line)| {
            cursor >= line.start
                && cursor <= line.end
                && !(cursor == line.end
                    && lines
                        .get(index + 1)
                        .is_some_and(|next| next.start == cursor))
        })
        .map(|(index, line)| {
            let column = input[line.start..cursor]
                .iter()
                .map(|character| {
                    if *character == '\t' {
                        4
                    } else {
                        UnicodeWidthChar::width(*character).unwrap_or(0)
                    }
                })
                .sum();
            (index, column)
        })
        .unwrap_or((0, 0))
}

pub fn move_vertical(input: &[char], cursor: usize, width: usize, down: bool) -> usize {
    let (line_index, column) = cursor_position(input, cursor, width);
    let lines = layout(input, width);
    let target_line = if down {
        line_index.saturating_add(1)
    } else {
        line_index.saturating_sub(1)
    };
    if target_line == line_index || target_line >= lines.len() {
        return cursor;
    }

    let line = &lines[target_line];
    let mut display_column = 0;
    for (offset, character) in input[line.start..line.end].iter().enumerate() {
        if display_column >= column {
            return line.start + offset;
        }
        let character_width = if *character == '\t' {
            4
        } else {
            UnicodeWidthChar::width(*character).unwrap_or(0)
        };
        if display_column + character_width > column {
            return line.start + offset;
        }
        display_column += character_width;
    }
    line.end
}

#[cfg(test)]
mod tests {
    use super::{VisualLine, cursor_position, layout, move_vertical};

    fn chars(value: &str) -> Vec<char> {
        value.chars().collect()
    }

    #[test]
    fn wraps_at_terminal_width_and_tracks_boundaries() {
        let input = chars("abcdef");
        assert_eq!(
            layout(&input, 3),
            vec![
                VisualLine {
                    text: "abc".into(),
                    start: 0,
                    end: 3
                },
                VisualLine {
                    text: "def".into(),
                    start: 3,
                    end: 6
                },
            ]
        );
        assert_eq!(cursor_position(&input, 3, 3), (1, 0));
        assert_eq!(cursor_position(&input, 6, 3), (1, 3));
    }

    #[test]
    fn preserves_explicit_lines_and_empty_trailing_line() {
        let lines = layout(&chars("a\nb\n"), 8);
        assert_eq!(
            lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", ""]
        );
        assert_eq!(cursor_position(&chars("a\nb"), 2, 8), (1, 0));
    }

    #[test]
    fn wraps_wide_characters_by_display_width() {
        let lines = layout(&chars("a界b"), 3);
        assert_eq!(
            lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a界", "b"]
        );
        assert_eq!(cursor_position(&chars("a界b"), 2, 3), (1, 0));
    }

    #[test]
    fn vertical_cursor_motion_follows_wrapped_visual_lines() {
        let input = chars("abcdef");
        assert_eq!(move_vertical(&input, 1, 3, true), 4);
        assert_eq!(move_vertical(&input, 4, 3, false), 1);
    }
}
