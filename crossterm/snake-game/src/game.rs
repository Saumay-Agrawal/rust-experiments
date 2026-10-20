use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStatus {
    Playing,
    Paused,
    GameOver,
    Won,
    PlayerWon(usize),
    Draw,
}

struct Snake {
    body: VecDeque<Position>,
    direction: Direction,
    next_direction: Direction,
    score: usize,
}

impl Snake {
    fn new(head: Position, direction: Direction, body_offsets: &[isize]) -> Self {
        let body = body_offsets
            .iter()
            .map(|offset| match direction {
                Direction::Right | Direction::Left => Position {
                    x: head.x.saturating_add_signed(*offset),
                    y: head.y,
                },
                Direction::Up | Direction::Down => Position {
                    x: head.x,
                    y: head.y.saturating_add_signed(*offset),
                },
            })
            .collect();
        Self {
            body,
            direction,
            next_direction: direction,
            score: 0,
        }
    }
}

pub struct Game {
    width: usize,
    height: usize,
    snakes: Vec<Snake>,
    food: Option<Position>,
    status: GameStatus,
    random_state: u64,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_snakes(
            width,
            height,
            vec![Snake::new(
                Position {
                    x: width / 2,
                    y: height / 2,
                },
                Direction::Right,
                &[0, -1, -2],
            )],
        )
    }

    pub fn new_two_player(width: usize, height: usize) -> Self {
        let y = height / 2;
        let first_head = width / 4;
        let second_head = (width * 3) / 4;
        Self::with_snakes(
            width,
            height,
            vec![
                Snake::new(
                    Position { x: first_head, y },
                    Direction::Right,
                    &[0, -1, -2],
                ),
                Snake::new(Position { x: second_head, y }, Direction::Left, &[0, 1, 2]),
            ],
        )
    }

    fn with_snakes(width: usize, height: usize, snakes: Vec<Snake>) -> Self {
        let random_state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |duration| duration.as_nanos() as u64)
            .max(1);
        let food = choose_food(width, height, &snakes, random_state);
        let status = if food.is_none() {
            GameStatus::Won
        } else {
            GameStatus::Playing
        };
        Self {
            width,
            height,
            snakes,
            food,
            status,
            random_state,
        }
    }

    pub fn snake_for_player(&self, player: usize) -> &VecDeque<Position> {
        &self.snakes[player].body
    }

    pub fn player_count(&self) -> usize {
        self.snakes.len()
    }

    pub fn score(&self) -> usize {
        self.score_for_player(0)
    }

    pub fn score_for_player(&self, player: usize) -> usize {
        self.snakes[player].score
    }

    pub fn total_score(&self) -> usize {
        self.snakes.iter().map(|snake| snake.score).sum()
    }

    pub fn food(&self) -> Option<Position> {
        self.food
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    pub fn tick_duration(&self) -> Duration {
        Duration::from_millis(
            150_u64
                .saturating_sub((self.total_score() as u64) * 5)
                .max(55),
        )
    }

    pub fn set_player_direction(&mut self, player: usize, direction: Direction) {
        let snake = &mut self.snakes[player];
        if !direction.is_opposite(snake.direction) {
            snake.next_direction = direction;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.status = match self.status {
            GameStatus::Playing => GameStatus::Paused,
            GameStatus::Paused => GameStatus::Playing,
            status => status,
        };
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        if self
            .snakes
            .iter()
            .flat_map(|snake| &snake.body)
            .any(|segment| segment.x >= width || segment.y >= height)
        {
            self.status = GameStatus::GameOver;
            return;
        }
        if self
            .food
            .is_none_or(|food| food.x >= width || food.y >= height)
        {
            self.random_state = xorshift(self.random_state);
            self.food = choose_food(width, height, &self.snakes, self.random_state);
            if self.food.is_none() {
                self.finish_full_board();
            }
        }
    }

    pub fn step(&mut self) {
        if self.status != GameStatus::Playing {
            return;
        }
        for snake in &mut self.snakes {
            if !snake.next_direction.is_opposite(snake.direction) {
                snake.direction = snake.next_direction;
            }
        }

        let next_positions: Vec<Option<Position>> = self
            .snakes
            .iter()
            .map(|snake| {
                next_position(
                    *snake.body.front().expect("snake always has a head"),
                    snake.direction,
                )
            })
            .collect();
        let growing: Vec<bool> = next_positions
            .iter()
            .map(|position| *position == self.food)
            .collect();
        let mut crashed = vec![false; self.snakes.len()];

        for (player, next) in next_positions.iter().enumerate() {
            let Some(next) =
                next.filter(|position| position.x < self.width && position.y < self.height)
            else {
                crashed[player] = true;
                continue;
            };
            for (other_player, other) in self.snakes.iter().enumerate() {
                let tail_vacates = !growing[other_player];
                let body_end = other.body.len() - usize::from(tail_vacates);
                if other
                    .body
                    .iter()
                    .take(body_end)
                    .any(|segment| *segment == next)
                {
                    crashed[player] = true;
                }
            }
        }

        if self.snakes.len() == 2 {
            if next_positions[0].is_some() && next_positions[0] == next_positions[1] {
                crashed[0] = true;
                crashed[1] = true;
            }
            let heads_swapped = next_positions[0] == self.snakes[1].body.front().copied()
                && next_positions[1] == self.snakes[0].body.front().copied();
            if heads_swapped {
                crashed[0] = true;
                crashed[1] = true;
            }
        }

        if crashed.iter().any(|did_crash| *did_crash) {
            self.status = match (
                crashed.iter().filter(|did_crash| **did_crash).count(),
                self.snakes.len(),
            ) {
                (2, 2) => GameStatus::Draw,
                (_, 2) => GameStatus::PlayerWon(if crashed[0] { 2 } else { 1 }),
                _ => GameStatus::GameOver,
            };
            return;
        }

        for (snake, next) in self.snakes.iter_mut().zip(next_positions) {
            let next = next.expect("wall collisions are handled before moving");
            snake.body.push_front(next);
            if next == self.food.expect("food remains until eaten") {
                snake.score += 1;
            } else {
                snake.body.pop_back();
            }
        }

        if growing.iter().any(|ate| *ate) {
            self.random_state = xorshift(self.random_state);
            self.food = choose_food(self.width, self.height, &self.snakes, self.random_state);
            if self.food.is_none() {
                self.finish_full_board();
            }
        }
    }

    fn finish_full_board(&mut self) {
        self.status = if self.snakes.len() == 2 {
            match self.snakes[0].score.cmp(&self.snakes[1].score) {
                std::cmp::Ordering::Greater => GameStatus::PlayerWon(1),
                std::cmp::Ordering::Less => GameStatus::PlayerWon(2),
                std::cmp::Ordering::Equal => GameStatus::Draw,
            }
        } else {
            GameStatus::Won
        };
    }
}

fn next_position(head: Position, direction: Direction) -> Option<Position> {
    match direction {
        Direction::Up => head.y.checked_sub(1).map(|y| Position { x: head.x, y }),
        Direction::Down => Some(Position {
            x: head.x,
            y: head.y + 1,
        }),
        Direction::Left => head.x.checked_sub(1).map(|x| Position { x, y: head.y }),
        Direction::Right => Some(Position {
            x: head.x + 1,
            y: head.y,
        }),
    }
}

fn choose_food(width: usize, height: usize, snakes: &[Snake], seed: u64) -> Option<Position> {
    let board_cells = width.checked_mul(height)?;
    let occupied = snakes.iter().map(|snake| snake.body.len()).sum::<usize>();
    let available = board_cells.checked_sub(occupied)?;
    if available == 0 {
        return None;
    }
    let mut target = (seed as usize) % available;
    for y in 0..height {
        for x in 0..width {
            let position = Position { x, y };
            if !snakes.iter().any(|snake| snake.body.contains(&position)) {
                if target == 0 {
                    return Some(position);
                }
                target -= 1;
            }
        }
    }
    None
}

fn xorshift(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_is_inside_the_board_and_not_on_a_snake() {
        let game = Game::new(20, 12);
        let food = game.food().expect("food should be placed");
        assert!(food.x < 20 && food.y < 12);
        assert!(!game.snake_for_player(0).contains(&food));
    }

    #[test]
    fn eating_grows_the_snake_increases_score_and_speeds_up() {
        let mut game = Game::new(20, 12);
        let old_length = game.snake_for_player(0).len();
        let old_tick = game.tick_duration();
        let head = game.snake_for_player(0)[0];
        game.food = Some(Position {
            x: head.x + 1,
            y: head.y,
        });
        game.step();
        assert_eq!(game.score(), 1);
        assert_eq!(game.snake_for_player(0).len(), old_length + 1);
        assert!(game.tick_duration() < old_tick);
    }

    #[test]
    fn rejects_a_direction_that_reverses_into_the_neck() {
        let mut game = Game::new(20, 12);
        game.set_player_direction(0, Direction::Left);
        game.step();
        assert_eq!(game.status(), GameStatus::Playing);
        assert_eq!(game.snake_for_player(0)[0].x, 11);
    }

    #[test]
    fn wall_collision_ends_a_solo_game() {
        let mut game = Game::new(8, 8);
        game.snakes[0].body = VecDeque::from([Position { x: 7, y: 4 }]);
        game.food = Some(Position { x: 0, y: 0 });
        game.step();
        assert_eq!(game.status(), GameStatus::GameOver);
    }

    #[test]
    fn moving_into_the_vacating_tail_is_allowed() {
        let mut game = Game::new(8, 8);
        game.snakes[0].body = VecDeque::from([
            Position { x: 3, y: 3 },
            Position { x: 3, y: 4 },
            Position { x: 2, y: 4 },
            Position { x: 2, y: 3 },
        ]);
        game.snakes[0].direction = Direction::Left;
        game.snakes[0].next_direction = Direction::Left;
        game.food = Some(Position { x: 7, y: 7 });
        game.step();
        assert_eq!(game.status(), GameStatus::Playing);
        assert_eq!(
            game.snake_for_player(0).front(),
            Some(&Position { x: 2, y: 3 })
        );
    }

    #[test]
    fn pause_toggle_only_changes_playing_states() {
        let mut game = Game::new(20, 12);
        game.toggle_pause();
        assert_eq!(game.status(), GameStatus::Paused);
        game.toggle_pause();
        assert_eq!(game.status(), GameStatus::Playing);
        game.status = GameStatus::GameOver;
        game.toggle_pause();
        assert_eq!(game.status(), GameStatus::GameOver);
    }

    #[test]
    fn two_player_game_starts_with_separate_snakes_facing_each_other() {
        let game = Game::new_two_player(24, 12);
        assert_eq!(game.player_count(), 2);
        assert_ne!(game.snake_for_player(0)[0], game.snake_for_player(1)[0]);
        assert!(!game.snake_for_player(0).contains(&game.food().unwrap()));
        assert!(!game.snake_for_player(1).contains(&game.food().unwrap()));
    }

    #[test]
    fn two_player_food_gives_score_to_the_eating_player() {
        let mut game = Game::new_two_player(24, 12);
        let first_head = game.snake_for_player(0)[0];
        game.food = Some(Position {
            x: first_head.x + 1,
            y: first_head.y,
        });
        game.step();
        assert_eq!(game.score_for_player(0), 1);
        assert_eq!(game.score_for_player(1), 0);
        assert_eq!(game.total_score(), 1);
    }

    #[test]
    fn two_player_head_on_collision_is_a_draw() {
        let mut game = Game::new_two_player(24, 12);
        game.snakes[0].body = VecDeque::from([Position { x: 10, y: 5 }]);
        game.snakes[1].body = VecDeque::from([Position { x: 12, y: 5 }]);
        game.snakes[0].direction = Direction::Right;
        game.snakes[0].next_direction = Direction::Right;
        game.snakes[1].direction = Direction::Left;
        game.snakes[1].next_direction = Direction::Left;
        game.food = Some(Position { x: 0, y: 0 });
        game.step();
        assert_eq!(game.status(), GameStatus::Draw);
    }

    #[test]
    fn hitting_the_other_snakes_body_loses_the_round() {
        let mut game = Game::new_two_player(24, 12);
        game.snakes[0].body = VecDeque::from([Position { x: 4, y: 4 }]);
        game.snakes[1].body = VecDeque::from([
            Position { x: 10, y: 8 },
            Position { x: 5, y: 4 },
            Position { x: 6, y: 4 },
        ]);
        game.snakes[0].direction = Direction::Right;
        game.snakes[0].next_direction = Direction::Right;
        game.snakes[1].direction = Direction::Right;
        game.snakes[1].next_direction = Direction::Right;
        game.food = Some(Position { x: 0, y: 0 });
        game.step();
        assert_eq!(game.status(), GameStatus::PlayerWon(2));
    }

    #[test]
    fn a_wall_collision_awards_the_round_to_the_other_player() {
        let mut game = Game::new_two_player(24, 12);
        game.snakes[0].body = VecDeque::from([Position { x: 23, y: 5 }]);
        game.snakes[1].body = VecDeque::from([Position { x: 10, y: 8 }]);
        game.snakes[0].direction = Direction::Right;
        game.snakes[0].next_direction = Direction::Right;
        game.snakes[1].direction = Direction::Right;
        game.snakes[1].next_direction = Direction::Right;
        game.food = Some(Position { x: 0, y: 0 });
        game.step();
        assert_eq!(game.status(), GameStatus::PlayerWon(2));
    }
}
