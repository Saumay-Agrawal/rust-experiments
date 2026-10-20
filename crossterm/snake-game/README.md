# Snake

A small terminal Snake game built with Rust and Crossterm.

Run it from this directory with:

```sh
cargo run
```

Press Enter or `1` to start a solo game, or `2` to start two-player mode. In
solo mode, move with the arrow keys or WASD. In two-player mode, Player 1 uses
WASD and Player 2 uses the arrow keys. Eat `*` to grow and score; the pace
increases as food is eaten. In two-player mode, both players compete for food;
a collision awards the round to the other player, while simultaneous crashes
are a draw. Press `P` to pause, `R` to restart the selected mode, and `Q` or
Escape to quit. Press `M` during a game to return to the mode menu. The game uses
a compact ASCII arcade theme with distinct player colors and framed status
panels. The play area follows the terminal size and asks you to enlarge the
window if it becomes too small.

Solo mode shows the current score and persistent high score. Two-player mode
shows a separate score for each player; multiplayer scores do not change the
solo high score. The solo record is saved per user in the operating system's
application data directory and persists across runs.

Development checks:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```
