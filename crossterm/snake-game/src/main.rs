mod game;
mod high_score;
mod renderer;
mod terminal;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::SynchronizedUpdate;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal as term;
use game::{Direction, Game, GameStatus};
use renderer::GameMode;

const MIN_COLUMNS: u16 = 40;
const MIN_ROWS: u16 = 17;

fn main() -> io::Result<()> {
    let (mut high_score, mut score_storage_warning) = match high_score::load() {
        Ok(score) => (score, false),
        Err(_) => (0, true),
    };
    let _terminal = terminal::TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut game: Option<Game> = None;
    let mut mode = GameMode::Solo;
    let mut last_tick = Instant::now();
    let mut last_size = (0, 0);
    let mut dirty = true;
    let mut running = true;

    while running {
        let (columns, rows) = term::size()?;
        let size_changed = last_size != (columns, rows);
        if size_changed {
            last_size = (columns, rows);
            dirty = true;
            if columns >= MIN_COLUMNS
                && rows >= MIN_ROWS
                && let Some(current) = game.as_mut()
            {
                current.resize(usize::from(columns - 2), usize::from(rows - 10));
            }
        }
        let too_small = columns < MIN_COLUMNS || rows < MIN_ROWS;

        if dirty {
            stdout.sync_update(|out| {
                crossterm::queue!(
                    out,
                    term::Clear(term::ClearType::All),
                    crossterm::cursor::MoveTo(0, 0)
                )?;
                if too_small {
                    renderer::draw_too_small(out, MIN_COLUMNS, MIN_ROWS)?;
                } else if let Some(current) = game.as_ref() {
                    renderer::draw_game(
                        out,
                        current,
                        mode,
                        high_score,
                        score_storage_warning,
                        columns,
                        rows,
                    )?;
                } else {
                    renderer::draw_title(out, high_score, score_storage_warning, columns, rows)?;
                }
                out.flush()
            })??;
            dirty = false;
        }

        let wait = if too_small {
            Duration::from_millis(250)
        } else if let Some(current) = game.as_ref()
            && current.status() == GameStatus::Playing
        {
            current.tick_duration().saturating_sub(last_tick.elapsed())
        } else {
            Duration::from_millis(250)
        };

        if event::poll(wait)? {
            match event::read()? {
                Event::Resize(_, _) => dirty = true,
                Event::Key(key) => match key.code {
                    KeyCode::Char('q' | 'Q') | KeyCode::Esc => running = false,
                    code if too_small => {
                        if matches!(code, KeyCode::Char('q' | 'Q') | KeyCode::Esc) {
                            running = false;
                        }
                    }
                    KeyCode::Char('1') => {
                        mode = GameMode::Solo;
                        game = Some(new_game(columns, rows, mode));
                        last_tick = Instant::now();
                        dirty = true;
                    }
                    KeyCode::Char('2') => {
                        mode = GameMode::TwoPlayer;
                        game = Some(new_game(columns, rows, mode));
                        last_tick = Instant::now();
                        dirty = true;
                    }
                    KeyCode::Char('m' | 'M') => {
                        game = None;
                        dirty = true;
                    }
                    KeyCode::Char('p' | 'P') => {
                        if let Some(current) = game.as_mut() {
                            current.toggle_pause();
                            last_tick = Instant::now();
                            dirty = true;
                        }
                    }
                    KeyCode::Char('r' | 'R') => {
                        game = Some(new_game(columns, rows, mode));
                        last_tick = Instant::now();
                        dirty = true;
                    }
                    KeyCode::Enter => {
                        if game
                            .as_ref()
                            .is_none_or(|g| g.status() != GameStatus::Playing)
                        {
                            game = Some(new_game(columns, rows, mode));
                            last_tick = Instant::now();
                            dirty = true;
                        }
                    }
                    code => {
                        if let Some(current) = game.as_mut()
                            && current.status() == GameStatus::Playing
                            && let Some((player, direction)) = direction_for(code, mode)
                        {
                            current.set_player_direction(player, direction);
                            dirty = true;
                        }
                    }
                },
                _ => {}
            }
        }

        if let Some(current) = game.as_mut()
            && current.status() == GameStatus::Playing
            && last_tick.elapsed() >= current.tick_duration()
        {
            current.step();
            last_tick = Instant::now();
            dirty = true;
            if mode == GameMode::Solo && current.score() > high_score {
                high_score = current.score();
                score_storage_warning = high_score::save(high_score).is_err();
            }
        }
    }
    Ok(())
}

fn new_game(columns: u16, rows: u16, mode: GameMode) -> Game {
    let width = usize::from(columns - 2);
    let height = usize::from(rows - 10);
    match mode {
        GameMode::Solo => Game::new(width, height),
        GameMode::TwoPlayer => Game::new_two_player(width, height),
    }
}

fn direction_for(code: KeyCode, mode: GameMode) -> Option<(usize, Direction)> {
    let player = match (mode, code) {
        (GameMode::TwoPlayer, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) => 1,
        _ => 0,
    };
    let direction = match code {
        KeyCode::Up | KeyCode::Char('w' | 'W') => Some(Direction::Up),
        KeyCode::Down | KeyCode::Char('s' | 'S') => Some(Direction::Down),
        KeyCode::Left | KeyCode::Char('a' | 'A') => Some(Direction::Left),
        KeyCode::Right | KeyCode::Char('d' | 'D') => Some(Direction::Right),
        _ => None,
    }?;
    Some((player, direction))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_player_controls_route_to_their_owners() {
        assert_eq!(
            direction_for(KeyCode::Char('w'), GameMode::TwoPlayer),
            Some((0, Direction::Up))
        );
        assert_eq!(
            direction_for(KeyCode::Left, GameMode::TwoPlayer),
            Some((1, Direction::Left))
        );
        assert_eq!(
            direction_for(KeyCode::Up, GameMode::Solo),
            Some((0, Direction::Up))
        );
    }
}
