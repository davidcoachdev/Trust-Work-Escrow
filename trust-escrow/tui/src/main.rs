mod app;
mod config;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing::info;

fn main() -> anyhow::Result<()> {
    // Iniciar logger en archivo ANTES de entrar en raw mode
    let _log_guard = init_logger();
    info!("Trust Work Escrow TUI iniciado");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let ev = event::read()?;
        app.handle_event(ev);

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

/// Inicializa el logger solo en archivo (no stderr, para no romper el TUI).
/// Devuelve un guard que debe vivir mientras dure el programa.
fn init_logger() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("trust-escrow-tui");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "trust-escrow.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .init();

    guard
}
