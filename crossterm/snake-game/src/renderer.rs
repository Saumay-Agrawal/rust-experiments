use std::io::{self, Write};

use crossterm::{cursor, queue, style};

use crate::game::{Game, GameStatus};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameMode {
    Solo,
    TwoPlayer,
}

pub fn draw_too_small(
    stdout: &mut impl Write,
    minimum_columns: u16,
    minimum_rows: u16,
) -> io::Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(2, 2),
        style::SetForegroundColor(style::Color::Yellow),
        style::SetAttribute(style::Attribute::Bold),
        style::Print("RESIZE TO CONTINUE"),
        style::SetAttribute(style::Attribute::Reset),
        cursor::MoveTo(2, 4),
        style::Print(format!(
            "Minimum terminal size: {minimum_columns} columns x {minimum_rows} rows"
        )),
        cursor::MoveTo(2, 6),
        style::Print("The game will resume when the window is large enough."),
        cursor::MoveTo(2, 8),
        style::Print("Press Q or Escape to quit."),
        style::ResetColor
    )
}

pub fn draw_title(
    stdout: &mut impl Write,
    high_score: usize,
    score_storage_warning: bool,
    columns: u16,
    rows: u16,
) -> io::Result<()> {
    let title = "S N A K E";
    let title_x = columns.saturating_sub(title.len() as u16) / 2;
    queue!(
        stdout,
        cursor::MoveTo(title_x, 1),
        style::SetForegroundColor(style::Color::Green),
        style::SetAttribute(style::Attribute::Bold),
        style::Print(title),
        style::SetAttribute(style::Attribute::Reset),
        style::ResetColor
    )?;

    let panel_width = usize::from(columns.saturating_sub(4).min(40));
    let panel_height = 12_usize;
    let panel_x = usize::from(columns).saturating_sub(panel_width) / 2;
    let panel_y = usize::from(rows).saturating_sub(panel_height) / 2;
    draw_box(
        stdout,
        panel_x as u16,
        panel_y as u16,
        panel_width as u16,
        panel_height as u16,
        style::Color::DarkCyan,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 1,
        "SELECT GAME MODE",
        style::Color::Cyan,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 3,
        "[1] SOLO",
        style::Color::Green,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 4,
        "[2] TWO PLAYER",
        style::Color::Cyan,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 6,
        &format!("SOLO BEST: {high_score}"),
        style::Color::Yellow,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 8,
        "ENTER starts solo",
        style::Color::White,
    )?;
    draw_centered(
        stdout,
        panel_x,
        panel_width,
        panel_y + 9,
        "Q quits",
        style::Color::DarkGrey,
    )?;
    if score_storage_warning {
        draw_centered(
            stdout,
            panel_x,
            panel_width,
            panel_y + 10,
            "BEST SCORE NOT SAVED",
            style::Color::Yellow,
        )?;
    }
    Ok(())
}

pub fn draw_game(
    stdout: &mut impl Write,
    game: &Game,
    mode: GameMode,
    high_score: usize,
    score_storage_warning: bool,
    columns: u16,
    rows: u16,
) -> io::Result<()> {
    let board_width = usize::from(columns - 2);
    let board_height = usize::from(rows - 10);
    let board_top = 5_u16;
    let board_bottom = board_top + board_height as u16 + 1;

    queue!(
        stdout,
        cursor::MoveTo(1, 0),
        style::SetForegroundColor(style::Color::Green),
        style::SetAttribute(style::Attribute::Bold),
        style::Print("SNAKE"),
        style::SetAttribute(style::Attribute::Reset),
        style::SetForegroundColor(style::Color::DarkGrey),
        style::Print(match mode {
            GameMode::Solo => "  /  SOLO",
            GameMode::TwoPlayer => "  /  TWO PLAYER",
        }),
        style::ResetColor
    )?;
    draw_scoreboard(stdout, game, high_score)?;
    queue!(
        stdout,
        cursor::MoveTo(1, 2),
        style::SetForegroundColor(style::Color::White),
        style::Print(if mode == GameMode::Solo {
            "MOVE  WASD / ARROWS"
        } else {
            "P1  WASD     P2  ARROWS"
        }),
        cursor::MoveTo(1, 3),
        style::SetForegroundColor(style::Color::DarkGrey),
        style::Print("P PAUSE   R RESTART   M MENU   Q QUIT"),
        style::ResetColor
    )?;

    let top_border = format!("+{}+", "-".repeat(board_width));
    queue!(
        stdout,
        cursor::MoveTo(0, board_top),
        style::SetForegroundColor(style::Color::DarkCyan),
        style::Print(&top_border),
        cursor::MoveTo(0, board_bottom),
        style::Print(&top_border)
    )?;
    for y in (board_top + 1)..board_bottom {
        queue!(
            stdout,
            cursor::MoveTo(0, y),
            style::Print("|"),
            cursor::MoveTo(columns - 1, y),
            style::Print("|")
        )?;
    }
    queue!(stdout, style::ResetColor)?;

    if let Some(food) = game
        .food()
        .filter(|food| food.x < board_width && food.y < board_height)
    {
        queue!(
            stdout,
            cursor::MoveTo((food.x + 1) as u16, board_top + 1 + food.y as u16),
            style::SetForegroundColor(style::Color::Yellow),
            style::SetAttribute(style::Attribute::Bold),
            style::Print("*"),
            style::SetAttribute(style::Attribute::Reset),
            style::ResetColor
        )?;
    }
    for player in 0..game.player_count() {
        let (head, body, head_color, body_color) = if player == 0 {
            ("@", "o", style::Color::Green, style::Color::DarkGreen)
        } else {
            ("&", "x", style::Color::Cyan, style::Color::DarkCyan)
        };
        for (index, segment) in game.snake_for_player(player).iter().enumerate() {
            if segment.x >= board_width || segment.y >= board_height {
                continue;
            }
            queue!(
                stdout,
                cursor::MoveTo((segment.x + 1) as u16, board_top + 1 + segment.y as u16),
                style::SetForegroundColor(if index == 0 { head_color } else { body_color }),
                style::SetAttribute(if index == 0 {
                    style::Attribute::Bold
                } else {
                    style::Attribute::Reset
                }),
                style::Print(if index == 0 { head } else { body }),
                style::SetAttribute(style::Attribute::Reset),
                style::ResetColor
            )?;
        }
    }

    queue!(
        stdout,
        cursor::MoveTo(1, board_bottom + 2),
        style::SetForegroundColor(style::Color::DarkGrey),
        style::Print(if mode == GameMode::Solo {
            "EAT * TO GROW"
        } else {
            "SHARED FOOD  /  FIRST TO CRASH LOSES"
        }),
        style::ResetColor
    )?;
    if score_storage_warning && mode == GameMode::Solo {
        queue!(
            stdout,
            cursor::MoveTo(1, board_bottom + 3),
            style::SetForegroundColor(style::Color::Yellow),
            style::Print("BEST SCORE NOT SAVED"),
            style::ResetColor
        )?;
    }

    if let Some((title, hint)) = status_panel(game.status()) {
        draw_status_panel(stdout, title, hint, columns, board_top, board_height)?;
    }
    Ok(())
}

fn draw_scoreboard(stdout: &mut impl Write, game: &Game, high_score: usize) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(1, 1))?;
    if game.player_count() == 1 {
        queue!(
            stdout,
            style::SetForegroundColor(style::Color::White),
            style::Print(format!("SCORE {:>3}  ", game.score())),
            style::SetForegroundColor(style::Color::Yellow),
            style::Print(format!("BEST {:>3}", high_score)),
            style::ResetColor
        )?;
    } else {
        queue!(
            stdout,
            style::SetForegroundColor(style::Color::Green),
            style::Print(format!("P1 {:>3}   ", game.score_for_player(0))),
            style::SetForegroundColor(style::Color::Cyan),
            style::Print(format!("P2 {:>3}", game.score_for_player(1))),
            style::ResetColor
        )?;
    }
    Ok(())
}

fn status_panel(status: GameStatus) -> Option<(&'static str, &'static str)> {
    match status {
        GameStatus::Playing => None,
        GameStatus::Paused => Some(("PAUSED", "P RESUME   R RESTART   M MENU")),
        GameStatus::GameOver => Some(("GAME OVER", "R RESTART   M MENU   Q QUIT")),
        GameStatus::Won => Some(("YOU WIN", "R PLAY AGAIN   M MENU   Q QUIT")),
        GameStatus::PlayerWon(1) => Some(("PLAYER 1 WINS", "R PLAY AGAIN   M MENU")),
        GameStatus::PlayerWon(2) => Some(("PLAYER 2 WINS", "R PLAY AGAIN   M MENU")),
        GameStatus::PlayerWon(_) => Some(("ROUND WON", "R PLAY AGAIN   M MENU")),
        GameStatus::Draw => Some(("DRAW", "R PLAY AGAIN   M MENU")),
    }
}

fn draw_status_panel(
    stdout: &mut impl Write,
    title: &str,
    hint: &str,
    columns: u16,
    board_top: u16,
    board_height: usize,
) -> io::Result<()> {
    let panel_width = usize::from(columns.saturating_sub(4).min(38));
    let panel_height = 5_usize;
    let x = (usize::from(columns).saturating_sub(panel_width)) / 2;
    let y = usize::from(board_top) + board_height.saturating_sub(panel_height) / 2;
    draw_box(
        stdout,
        x as u16,
        y as u16,
        panel_width as u16,
        panel_height as u16,
        style::Color::Yellow,
    )?;
    queue!(
        stdout,
        style::SetBackgroundColor(style::Color::Black),
        cursor::MoveTo((x + 1) as u16, (y + 1) as u16),
        style::Print(" ".repeat(panel_width.saturating_sub(2))),
        cursor::MoveTo((x + 1) as u16, (y + 2) as u16),
        style::Print(" ".repeat(panel_width.saturating_sub(2))),
        cursor::MoveTo((x + 1) as u16, (y + 3) as u16),
        style::Print(" ".repeat(panel_width.saturating_sub(2))),
        style::ResetColor
    )?;
    draw_centered(stdout, x, panel_width, y + 1, title, style::Color::Yellow)?;
    draw_centered(stdout, x, panel_width, y + 2, hint, style::Color::White)?;
    Ok(())
}

fn draw_box(
    stdout: &mut impl Write,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    color: style::Color,
) -> io::Result<()> {
    if width < 2 || height < 2 {
        return Ok(());
    }
    let horizontal = "-".repeat(usize::from(width - 2));
    queue!(
        stdout,
        style::SetForegroundColor(color),
        cursor::MoveTo(x, y),
        style::Print(format!("+{horizontal}+")),
        cursor::MoveTo(x, y + height - 1),
        style::Print(format!("+{horizontal}+"))
    )?;
    for row in (y + 1)..(y + height - 1) {
        queue!(
            stdout,
            cursor::MoveTo(x, row),
            style::Print("|"),
            cursor::MoveTo(x + width - 1, row),
            style::Print("|")
        )?;
    }
    queue!(stdout, style::ResetColor)
}

fn draw_centered(
    stdout: &mut impl Write,
    x: usize,
    width: usize,
    y: usize,
    text: &str,
    color: style::Color,
) -> io::Result<()> {
    let text = if text.len() > width.saturating_sub(2) {
        &text[..width.saturating_sub(2)]
    } else {
        text
    };
    let left = x + (width.saturating_sub(text.len())) / 2;
    queue!(
        stdout,
        cursor::MoveTo(left as u16, y as u16),
        style::SetForegroundColor(color),
        style::Print(text),
        style::ResetColor
    )
}
