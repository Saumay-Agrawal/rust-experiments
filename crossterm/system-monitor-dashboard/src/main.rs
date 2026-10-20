use std::{
    io::{self, IsTerminal, Write},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use sysinfo::System;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const BAR_WIDTH: usize = 32;

struct TerminalSession;

impl TerminalSession {
    fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        let session = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(session)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen, ResetColor);
        let _ = disable_raw_mode();
    }
}

fn main() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "system-monitor-dashboard requires an interactive terminal",
        ));
    }

    run()
}

fn run() -> io::Result<()> {
    let _terminal = TerminalSession::start()?;
    let mut system = System::new();
    system.refresh_cpu_usage();
    system.refresh_memory();

    // CPU usage is calculated from the difference between two samples.
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_usage();

    let mut stdout = io::stdout();
    let mut next_refresh = Instant::now();
    loop {
        if Instant::now() >= next_refresh {
            system.refresh_cpu_usage();
            system.refresh_memory();
            draw(&mut stdout, &system)?;
            next_refresh = Instant::now() + REFRESH_INTERVAL;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(
                key.code,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
            )
        {
            break;
        }
    }

    Ok(())
}

fn draw(stdout: &mut impl Write, system: &System) -> io::Result<()> {
    let cpu = system.global_cpu_usage().clamp(0.0, 100.0);
    let used_memory = system.used_memory();
    let total_memory = system.total_memory();
    let memory_percent = if total_memory == 0 {
        0.0
    } else {
        (used_memory as f64 / total_memory as f64 * 100.0).clamp(0.0, 100.0) as f32
    };
    let (width, height) = terminal::size()?;

    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    if width < 52 || height < 12 {
        queue!(stdout, MoveTo(0, 0))?;
        write!(
            stdout,
            "Resize terminal to at least 52x12. Press q to quit."
        )?;
        return stdout.flush();
    }

    queue!(stdout, MoveTo(2, 1), SetForegroundColor(Color::Cyan))?;
    write!(stdout, "SYSTEM MONITOR")?;
    queue!(stdout, ResetColor, MoveTo(2, 2))?;
    write!(stdout, "Live system metrics  |  refreshes every second")?;

    draw_metric(stdout, 4, "CPU", cpu, "", Color::Green)?;
    draw_metric(
        stdout,
        7,
        "MEMORY",
        memory_percent,
        &format!(
            "{} / {}",
            format_bytes(used_memory),
            format_bytes(total_memory)
        ),
        Color::Yellow,
    )?;

    queue!(stdout, MoveTo(2, 10), SetForegroundColor(Color::Magenta))?;
    write!(stdout, "UPTIME")?;
    queue!(stdout, ResetColor)?;
    write!(stdout, "  {}", format_uptime(System::uptime()))?;
    queue!(stdout, MoveTo(2, height - 1))?;
    write!(stdout, "Press q or Esc to quit")?;
    stdout.flush()
}

fn draw_metric(
    stdout: &mut impl Write,
    row: u16,
    label: &str,
    percent: f32,
    detail: &str,
    color: Color,
) -> io::Result<()> {
    let filled = ((percent / 100.0) * BAR_WIDTH as f32).round() as usize;
    queue!(stdout, MoveTo(2, row), SetForegroundColor(color))?;
    write!(stdout, "{label:<8}")?;
    queue!(stdout, ResetColor)?;
    write!(stdout, " {:>5.1}%  [", percent)?;
    queue!(stdout, SetForegroundColor(color))?;
    write!(stdout, "{}", "#".repeat(filled))?;
    queue!(stdout, ResetColor)?;
    write!(stdout, "{}]", "-".repeat(BAR_WIDTH - filled))?;
    if !detail.is_empty() {
        write!(stdout, "  {detail}")?;
    }
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else {
        format!("{:.0} MiB", bytes as f64 / MIB as f64)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if days > 0 {
        format!("{days}d {hours:02}h {minutes:02}m {seconds:02}s")
    } else {
        format!("{hours:02}h {minutes:02}m {seconds:02}s")
    }
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, format_uptime};

    #[test]
    fn formats_uptime_with_and_without_days() {
        assert_eq!(format_uptime(3_661), "01h 01m 01s");
        assert_eq!(format_uptime(90_061), "1d 01h 01m 01s");
    }

    #[test]
    fn formats_memory_in_binary_units() {
        assert_eq!(format_bytes(1024 * 1024), "1 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }
}
