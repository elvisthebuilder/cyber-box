mod app;
mod event;
mod ui;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use cyberbox_core::config::Config;
use cyberbox_core::docker::DockerClient;
use event::AppEvent;
use cyberbox_core::registry::Registry;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init_logging();

    let config = Config::default();

    let docker = DockerClient::connect(&config)?;
    if let Err(e) = docker.ensure_container().await {
        eprintln!(
            "warning: could not ensure toolbox container is running: {e}\n\
             (build it first with `make image`, and make sure Docker is running)"
        );
    }
    let docker = Arc::new(docker);

    let registry = Registry::load(&config.registry_path).unwrap_or_else(|e| {
        eprintln!("warning: failed to load tool registry ({e}); starting with an empty toolset");
        Registry { tools: Vec::new() }
    });

    let mut app = App::new(config, docker, registry);
    app.refresh_status().await;
    app.check_installed_status().await;

    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, &mut app).await;
    restore_terminal(&mut terminal)?;

    result
}

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", "cyberbox.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).with_ansi(false).init();
    guard
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    let mut input_rx = event::spawn_input_reader();
    let mut tick = tokio::time::interval(Duration::from_millis(100));

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        tokio::select! {
            Some(ev) = input_rx.recv() => {
                match ev {
                    AppEvent::Key(key) => app.handle_key(key).await,
                    AppEvent::Resize => {}
                }
            }
            _ = tick.tick() => {
                app.tick();
                if app.status_due() {
                    app.refresh_status().await;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
