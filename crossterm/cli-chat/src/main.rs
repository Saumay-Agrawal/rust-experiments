mod editor;
mod ollama;
mod terminal;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ollama::{GenerationEvent, Message, Model};
use reqwest::Client;
use std::{env, io, thread, time::Duration};
use tokio::sync::mpsc;

struct Options {
    base_url: String,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cli-chat: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options()?;
    let _terminal = terminal::TerminalSession::start()?;
    let (input_tx, mut input_rx) = mpsc::unbounded_channel();
    spawn_input_reader(input_tx);
    let client = Client::builder().build()?;

    let Some(models) = discover_models(&client, &options.base_url, &mut input_rx).await? else {
        return Ok(());
    };
    let Some(model) = choose_model(&models, &mut input_rx).await? else {
        return Ok(());
    };
    run_chat(client, options.base_url, model.name, &mut input_rx).await?;
    Ok(())
}

async fn discover_models(
    client: &Client,
    base_url: &str,
    input_rx: &mut mpsc::UnboundedReceiver<Event>,
) -> io::Result<Option<Vec<Model>>> {
    loop {
        ui::draw_notice(
            "Connecting to Ollama",
            &format!("Reading installed models from {base_url}…"),
            "Ctrl+C quits",
        )?;
        match ollama::list_models(client, base_url).await {
            Ok(models) if !models.is_empty() => return Ok(Some(models)),
            Ok(_) => {
                ui::draw_notice(
                    "No installed models",
                    &format!(
                        "There are no models installed on {base_url}. Pull one in another terminal, for example:\nollama pull llama3.2"
                    ),
                    "r refresh model list · q or Ctrl+C quit · F1 help",
                )?;
            }
            Err(error) => {
                ui::draw_notice(
                    "Ollama is unavailable",
                    &format!("{error}\n\nStart Ollama, then retry."),
                    "r retry connection · q or Ctrl+C quit · F1 help",
                )?;
            }
        }

        match wait_key(input_rx).await {
            Some(key) if is_quit(&key) => return Ok(None),
            Some(key) if key.code == KeyCode::Char('r') => {}
            Some(key) if key.code == KeyCode::F(1) => {
                ui::draw_help(
                    "Ollama setup",
                    &[
                        "r  Retry connection or refresh installed models",
                        "q  Quit",
                        "Ctrl+C  Quit",
                        "Install models separately with: ollama pull <model>",
                    ],
                )?;
                match wait_key(input_rx).await {
                    Some(key)
                        if key.code == KeyCode::Esc
                            || key.code == KeyCode::F(1)
                            || key.code == KeyCode::Char('?') => {}
                    Some(key) if is_quit(&key) => return Ok(None),
                    Some(key) if key.code == KeyCode::Char('r') => {}
                    _ => continue,
                }
            }
            _ => {}
        }
    }
}

async fn choose_model(
    models: &[Model],
    input_rx: &mut mpsc::UnboundedReceiver<Event>,
) -> io::Result<Option<Model>> {
    let mut filter = String::new();
    let mut selected = 0usize;
    let mut help = false;
    loop {
        if help {
            ui::draw_help(
                "Model picker",
                &[
                    "Type to filter installed model names",
                    "Up/Down  Move through matches",
                    "Enter  Select highlighted model",
                    "Backspace  Edit filter",
                    "Esc  Clear filter or close help",
                    "q  Quit when the filter is empty",
                    "Ctrl+C  Quit",
                    "F1 or ?  Toggle help",
                ],
            )?;
        } else {
            let matches = filter_models(models, &filter);
            selected = selected.min(matches.len().saturating_sub(1));
            ui::draw_menu(
                &matches,
                selected,
                &filter,
                "Installed models are read from Ollama.",
            )?;
        }

        let Some(Event::Key(key)) = input_rx.recv().await else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if help {
            if key.code == KeyCode::Esc
                || key.code == KeyCode::F(1)
                || key.code == KeyCode::Char('?')
            {
                help = false;
            } else if is_quit(&key) {
                return Ok(None);
            }
            continue;
        }
        if is_quit(&key) && (key.code != KeyCode::Char('q') || filter.is_empty()) {
            return Ok(None);
        }
        match key.code {
            KeyCode::Esc => filter.clear(),
            KeyCode::F(1) => help = true,
            KeyCode::Char('?') if filter.is_empty() => help = true,
            KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::Down => {
                let match_count = filter_models(models, &filter).len();
                if match_count > 0 {
                    selected = (selected + 1).min(match_count - 1);
                }
            }
            KeyCode::Backspace => {
                filter.pop();
                selected = 0;
            }
            KeyCode::Enter => {
                if let Some(model) = filter_models(models, &filter).get(selected) {
                    return Ok(Some((*model).clone()));
                }
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                filter.push(character);
                selected = 0;
            }
            _ => {}
        }
    }
}

async fn run_chat(
    client: Client,
    base_url: String,
    model: String,
    input_rx: &mut mpsc::UnboundedReceiver<Event>,
) -> io::Result<()> {
    let mut messages = Vec::<Message>::new();
    let mut input = Vec::<char>::new();
    let mut cursor = 0usize;
    let mut status = "Ready · type a message".to_owned();
    let mut error: Option<String> = None;
    let mut generation_rx: Option<mpsc::UnboundedReceiver<GenerationEvent>> = None;
    let mut generation_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut generating = false;
    let mut help = false;
    let mut scroll_from_bottom = 0usize;

    loop {
        if help {
            ui::draw_help(
                "Chat",
                &[
                    "Enter  Send message",
                    "Shift+Enter or Ctrl+J  Insert newline",
                    "Left/Right, Up/Down, Home/End  Move through prompt",
                    "Page Up/Down, mouse wheel  Scroll transcript",
                    "End  Return to latest when the prompt is empty",
                    "Esc  Cancel generation or close help",
                    "r  Retry a failed request when the prompt is empty",
                    "q  Quit when idle and the prompt is empty",
                    "Ctrl+C  Quit",
                    "F1 or ?  Toggle help (? when the prompt is empty)",
                ],
            )?;
        } else {
            ui::draw_chat(ui::ChatView {
                model: &model,
                messages: &messages,
                input: &input,
                cursor,
                status: &status,
                error: error.as_deref(),
                generating,
                scroll_from_bottom,
            })?;
        }

        enum Action {
            Input(Option<Event>),
            Generation(Option<GenerationEvent>),
        }
        let action = tokio::select! {
            event = input_rx.recv() => Action::Input(event),
            event = async {
                match generation_rx.as_mut() {
                    Some(receiver) => receiver.recv().await,
                    None => std::future::pending().await,
                }
            } => Action::Generation(event),
        };

        match action {
            Action::Input(Some(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                if help {
                    if key.code == KeyCode::Esc
                        || key.code == KeyCode::F(1)
                        || key.code == KeyCode::Char('?')
                    {
                        help = false;
                    }
                    continue;
                }
                if key.code == KeyCode::F(1) {
                    help = true;
                    continue;
                }
                if generating {
                    if key.code == KeyCode::Esc {
                        if let Some(task) = generation_task.take() {
                            task.abort();
                        }
                        generation_rx = None;
                        generating = false;
                        status =
                            "Generation cancelled · partial reply kept in transcript".to_owned();
                    }
                    continue;
                }

                if key.code == KeyCode::Char('q') && input.is_empty() {
                    break;
                }
                if key.code == KeyCode::Char('r') && input.is_empty() && error.is_some() {
                    error = None;
                    status = "Retrying last request…".to_owned();
                    start_generation(
                        client.clone(),
                        base_url.clone(),
                        model.clone(),
                        &mut messages,
                        &mut generation_rx,
                        &mut generation_task,
                    );
                    generating = true;
                    scroll_from_bottom = 0;
                    continue;
                }

                let empty_help_shortcut = input.is_empty() && key.code == KeyCode::Char('?');
                if empty_help_shortcut {
                    help = true;
                    continue;
                }
                match key.code {
                    KeyCode::Esc => {}
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        input.insert(cursor, '\n');
                        cursor += 1;
                    }
                    KeyCode::Enter => {
                        let prompt: String = input.iter().collect();
                        if prompt.trim().is_empty() {
                            status = "Enter a message before sending.".to_owned();
                            continue;
                        }
                        input.clear();
                        cursor = 0;
                        error = None;
                        status = "Generating…".to_owned();
                        messages.push(Message {
                            role: "user".to_owned(),
                            content: prompt,
                        });
                        start_generation(
                            client.clone(),
                            base_url.clone(),
                            model.clone(),
                            &mut messages,
                            &mut generation_rx,
                            &mut generation_task,
                        );
                        generating = true;
                        scroll_from_bottom = 0;
                    }
                    KeyCode::Char(character)
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && character.eq_ignore_ascii_case(&'j') =>
                    {
                        input.insert(cursor, '\n');
                        cursor += 1;
                    }
                    KeyCode::Char(character)
                        if !key
                            .modifiers
                            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                    {
                        input.insert(cursor, character);
                        cursor += 1;
                    }
                    KeyCode::Backspace if cursor > 0 => {
                        cursor -= 1;
                        input.remove(cursor);
                    }
                    KeyCode::Delete if cursor < input.len() => {
                        input.remove(cursor);
                    }
                    KeyCode::Left if cursor > 0 => cursor -= 1,
                    KeyCode::Right if cursor < input.len() => cursor += 1,
                    KeyCode::Home => {
                        cursor = input[..cursor]
                            .iter()
                            .rposition(|character| *character == '\n')
                            .map_or(0, |index| index + 1)
                    }
                    KeyCode::End if input.is_empty() => scroll_from_bottom = 0,
                    KeyCode::End => {
                        cursor = input[cursor..]
                            .iter()
                            .position(|character| *character == '\n')
                            .map_or(input.len(), |index| cursor + index)
                    }
                    KeyCode::Up => {
                        let width = crossterm::terminal::size()
                            .map(|(width, _)| usize::from(width.saturating_sub(2)).max(1))
                            .unwrap_or(78);
                        cursor = editor::move_vertical(&input, cursor, width, false);
                    }
                    KeyCode::Down => {
                        let width = crossterm::terminal::size()
                            .map(|(width, _)| usize::from(width.saturating_sub(2)).max(1))
                            .unwrap_or(78);
                        cursor = editor::move_vertical(&input, cursor, width, true);
                    }
                    KeyCode::PageUp => {
                        scroll_from_bottom = scroll_from_bottom.saturating_add(page_size())
                    }
                    KeyCode::PageDown => {
                        scroll_from_bottom = scroll_from_bottom.saturating_sub(page_size())
                    }
                    _ => {}
                }
            }
            Action::Input(Some(Event::Mouse(mouse))) => match mouse.kind {
                MouseEventKind::ScrollUp
                    if mouse.row
                        < crossterm::terminal::size()
                            .map_or(u16::MAX, |(_, height)| height.saturating_sub(6)) =>
                {
                    scroll_from_bottom = scroll_from_bottom.saturating_add(3)
                }
                MouseEventKind::ScrollDown
                    if mouse.row
                        < crossterm::terminal::size()
                            .map_or(u16::MAX, |(_, height)| height.saturating_sub(6)) =>
                {
                    scroll_from_bottom = scroll_from_bottom.saturating_sub(3)
                }
                _ => {}
            },
            Action::Input(Some(Event::Resize(_, _))) => {}
            Action::Input(Some(_)) => {}
            Action::Input(None) => break,
            Action::Generation(Some(GenerationEvent::Token(token))) => {
                let terminal_width = crossterm::terminal::size()
                    .map(|(width, _)| width)
                    .unwrap_or(80);
                let old_row_count = if scroll_from_bottom > 0 {
                    ui::transcript_row_count(&messages, None, terminal_width)
                } else {
                    0
                };
                if let Some(message) = messages.last_mut() {
                    message.content.push_str(&token);
                }
                if scroll_from_bottom > 0 {
                    let new_row_count = ui::transcript_row_count(&messages, None, terminal_width);
                    scroll_from_bottom = scroll_from_bottom
                        .saturating_add(new_row_count.saturating_sub(old_row_count));
                }
            }
            Action::Generation(Some(GenerationEvent::Finished(result))) => {
                generating = false;
                generation_rx = None;
                generation_task = None;
                match result {
                    Ok(()) => {
                        error = None;
                        status = "Ready · type a message".to_owned();
                    }
                    Err(message) => {
                        if messages
                            .last()
                            .is_some_and(|entry| entry.role == "assistant")
                        {
                            messages.pop();
                        }
                        error = Some(message);
                        status = "Request failed · clear prompt, then r retries".to_owned();
                    }
                }
            }
            Action::Generation(None) => {
                generating = false;
                generation_rx = None;
                generation_task = None;
                if messages
                    .last()
                    .is_some_and(|entry| entry.role == "assistant")
                {
                    messages.pop();
                }
                error = Some("Ollama response stream closed unexpectedly".to_owned());
                status = "Request failed · clear prompt, then r retries".to_owned();
            }
        }
    }
    Ok(())
}

fn start_generation(
    client: Client,
    base_url: String,
    model: String,
    messages: &mut Vec<Message>,
    receiver: &mut Option<mpsc::UnboundedReceiver<GenerationEvent>>,
    task: &mut Option<tokio::task::JoinHandle<()>>,
) {
    messages.push(Message {
        role: "assistant".to_owned(),
        content: String::new(),
    });
    let request_messages = request_history(messages);
    let (sender, stream_receiver) = mpsc::unbounded_channel();
    *task = Some(tokio::spawn(ollama::stream_chat(
        client,
        base_url,
        model,
        request_messages,
        sender,
    )));
    *receiver = Some(stream_receiver);
}

fn request_history(messages: &[Message]) -> Vec<Message> {
    if messages
        .last()
        .is_some_and(|message| message.role == "assistant")
    {
        messages[..messages.len() - 1].to_vec()
    } else {
        messages.to_vec()
    }
}

fn filter_models<'a>(models: &'a [Model], filter: &str) -> Vec<&'a Model> {
    let needle = filter.to_lowercase();
    models
        .iter()
        .filter(|model| model.name.to_lowercase().contains(&needle))
        .collect()
}

fn page_size() -> usize {
    crossterm::terminal::size()
        .map(|(_, height)| usize::from(height.saturating_sub(8)).max(1))
        .unwrap_or(10)
}

fn is_quit(key: &crossterm::event::KeyEvent) -> bool {
    key.code == KeyCode::Char('q')
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

async fn wait_key(
    input_rx: &mut mpsc::UnboundedReceiver<Event>,
) -> Option<crossterm::event::KeyEvent> {
    while let Some(event) = input_rx.recv().await {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            return Some(key);
        }
    }
    None
}

fn spawn_input_reader(sender: mpsc::UnboundedSender<Event>) {
    thread::spawn(move || {
        while !sender.is_closed() {
            match event::poll(Duration::from_millis(40)) {
                Ok(true) => match event::read() {
                    Ok(event) => {
                        if sender.send(event).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                },
                Ok(false) => {}
                Err(_) => break,
            }
        }
    });
}

fn parse_options() -> Result<Options, Box<dyn std::error::Error>> {
    let mut base_url =
        env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_owned());
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ollama-url" | "-u" => {
                base_url = args.next().ok_or("--ollama-url requires a URL")?;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: cli-chat [--ollama-url URL]\n\nEnvironment: OLLAMA_HOST sets the Ollama server URL (default: http://localhost:11434)"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}; use --help for usage").into()),
        }
    }
    let base_url = base_url.trim_end_matches('/').to_owned();
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("Ollama URL must start with http:// or https://".into());
    }
    Ok(Options { base_url })
}

#[cfg(test)]
mod tests {
    use super::{filter_models, request_history};
    use crate::ollama::{Message, Model};

    #[test]
    fn model_filter_is_case_insensitive_and_returns_all_matches() {
        let models = vec![
            Model {
                name: "Llama3.2".into(),
            },
            Model {
                name: "mistral".into(),
            },
            Model {
                name: "llama3.1".into(),
            },
        ];
        let matches = filter_models(&models, "LLAMA");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].name, "Llama3.2");
        assert_eq!(matches[1].name, "llama3.1");
    }

    #[test]
    fn retry_history_keeps_failed_prompt_but_drops_partial_reply() {
        let messages = vec![
            Message {
                role: "user".into(),
                content: "earlier".into(),
            },
            Message {
                role: "assistant".into(),
                content: "answer".into(),
            },
            Message {
                role: "user".into(),
                content: "retry this".into(),
            },
            Message {
                role: "assistant".into(),
                content: "partial".into(),
            },
        ];

        let history = request_history(&messages);
        assert_eq!(history.len(), 3);
        assert_eq!(history[2].role, "user");
        assert_eq!(history[2].content, "retry this");
    }
}
