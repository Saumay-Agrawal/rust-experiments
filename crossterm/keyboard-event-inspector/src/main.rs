mod exit_sequence;
mod format;

use crossterm::{
    event::{
        self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement},
};
use exit_sequence::ExitSequence;
use std::{
    io::{self, IsTerminal, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

static RAW: AtomicBool = AtomicBool::new(false);
static ENHANCED: AtomicBool = AtomicBool::new(false);

struct TerminalSession;

impl TerminalSession {
    fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        RAW.store(true, Ordering::SeqCst);
        Ok(Self)
    }
}

fn restore_terminal() -> io::Result<()> {
    let enhancements = if ENHANCED.swap(false, Ordering::SeqCst) {
        execute!(io::stdout(), PopKeyboardEnhancementFlags)
    } else {
        Ok(())
    };
    let raw = if RAW.swap(false, Ordering::SeqCst) {
        disable_raw_mode()
    } else {
        Ok(())
    };
    enhancements.and(raw)
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

fn run() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "stdin and stdout must both be interactive terminals",
        ));
    }
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        previous_hook(info);
    }));
    let _session = TerminalSession::start()?;
    let mut output = io::stdout();
    let capability = match supports_keyboard_enhancement() {
        Ok(true) => {
            // Mark before writing so partial setup also attempts to pop on failure.
            ENHANCED.store(true, Ordering::SeqCst);
            execute!(
                output,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
            "enhanced (disambiguation and press/repeat/release reporting requested)".to_owned()
        }
        Ok(false) if cfg!(windows) => {
            "standard Windows console (native event reporting)".to_owned()
        }
        Ok(false) => {
            "basic (press events only; this terminal does not provide enhanced repeat/release reporting)"
                .to_owned()
        }
        Err(error) => format!("basic (capability detection failed: {error:?})"),
    };
    let style = format::OutputStyle::for_stdout(io::stdout().is_terminal());
    write!(
        output,
        "\r\n{}\r\n\r\n{}\r\n",
        format::banner(&capability, style),
        format::header(style)
    )?;
    output.flush()?;
    let mut sequence = ExitSequence::default();
    let mut number = 0;
    loop {
        if let Event::Key(key) = event::read()? {
            let received = Instant::now();
            number += 1;
            write!(
                output,
                "{}\r\n",
                format::event_line_styled(number, &key, style)
            )?;
            output.flush()?;
            if sequence.observe(&key, received) {
                break;
            }
        }
    }
    restore_terminal()
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("keyboard-event-inspector: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
