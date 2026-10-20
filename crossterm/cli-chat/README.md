# CLI Chat

A full-screen Rust terminal chat client for [Ollama](https://ollama.com/), built with Crossterm.

## Requirements

- Rust and Cargo
- Ollama running locally, with at least one model installed

Pull a model separately before launching the app, for example:

```sh
ollama pull llama3.2
```

## Run

From this directory:

```sh
cargo run
```

By default the app connects to `http://localhost:11434`. Set `OLLAMA_HOST` or pass `--ollama-url` to use another server:

```sh
OLLAMA_HOST=http://192.168.1.10:11434 cargo run
cargo run -- --ollama-url http://192.168.1.10:11434
```

The app displays models already installed on that Ollama server. It does not download models.

## Controls

- Model picker: type to filter, Up/Down to move, Enter to select, `q` or Ctrl+C to quit
- Chat: Enter sends; Shift+Enter or Ctrl+J inserts a newline
- Page Up/Down or the mouse wheel scrolls the transcript; End returns to the latest message when the prompt is empty
- Escape cancels generation or dismisses help; `q` or Ctrl+C quits
- Press F1 or `?` (with an empty prompt/filter) for screen-specific help
- After a request fails, press `r` with an empty prompt to retry it
- If Ollama is unavailable, press `r` to retry; if no models are installed, pull one separately and press `r` to refresh

Long prompts wrap to the terminal width, and the editor scrolls to keep the cursor visible. The transcript follows streamed output until you scroll back; new tokens preserve the viewed history position. Both layouts reflow when the terminal is resized.

Messages and conversation context stay in memory for the current session and are discarded on exit.

## Development

Run from this directory:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```
