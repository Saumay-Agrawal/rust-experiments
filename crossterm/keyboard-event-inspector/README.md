# Keyboard Event Inspector

A small terminal application that shows the keyboard events reported by
[crossterm](https://crates.io/crates/crossterm).

It is useful for checking key names, modifiers, terminal escape sequences, and
press/repeat/release support.

## Features

- Readable key combinations such as `Ctrl+Shift+A`
- Raw key code, modifiers, event kind, and state
- Enhanced keyboard reporting when the terminal supports it
- Aligned, color-coded output that remains readable in scrollback
- Plain output with `NO_COLOR` or redirected stdout
- Safe terminal cleanup on normal exit, errors, and panics

## Run

From the project root (the crossterm/keyboard-event-inspector directory):

```sh
cargo run
```

Both standard input and standard output must be interactive terminals.

## Controls

Press Escape three times within one second to quit. The timer starts with the
first Escape. Any other key press, including modified Escape, resets the
sequence. All three Escape events are logged before the program exits.

## Output

The log uses fixed-width columns and vertical separators:

`text
     # │ COMBINATION        │ CODE                     │ MODIFIERS              │ KIND       │ STATE
───────┼────────────────────┼──────────────────────────┼────────────────────────┼────────────┼────────
    #1 │ Ctrl+Shift+A       │ Char('A')                │ KeyModifiers(...)       │ Press      │ KeyEventState(...)
`

Interactive output uses a monochromatic palette:

- combinations: bold blue
- code: white
- modifiers and event kind: soft gray
- event number, state, borders, and separators: dim gray
- headings: bold bright white

Colors are disabled when stdout is redirected or when `NO_COLOR` is set.
Long values are shortened with an ellipsis to preserve alignment.

## Terminal support

The inspector reports the capability detected at startup. Terminals that
support the Kitty keyboard protocol can provide unambiguous combinations and
press, repeat, and release events. Other terminals generally provide
press-only events and may merge combinations such as `Tab` and `Ctrl+I`.

On macOS, Terminal.app does not provide the enhanced protocol. For release
events, use a compatible terminal such as Kitty, WezTerm, or an Alacritty
configuration with Kitty keyboard protocol support. VS Code's integrated
terminal supports this protocol when
`terminal.integrated.enableKittyKeyboardProtocol` is enabled.

“Exact” means the event delivered by the terminal; the application cannot
identify physical keys or recover shortcuts intercepted by the operating
system, terminal, editor, or multiplexer.

## Development

Run these commands from this directory:

`sh
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
`

For a manual check, try text, Unicode, modifier combinations, arrows, function
keys, held keys, and the triple-Escape exit gesture. After exit, verify that
the shell's normal echo and line editing have been restored.
