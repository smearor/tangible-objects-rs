//! `tangible-objects-tui` - live surface dashboard in the terminal
//! (TUI.md). Connects to a running daemon via `crates/client`; never
//! touches the evdev device itself.

mod app;
mod view;

use std::io;

use crate::app::App;
use clap::Parser;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tangible_objects_client::Endpoint;
use tangible_objects_client::GraphQLClient;

/// Terminal dashboard for a tangible-objects daemon.
#[derive(Debug, Parser)]
#[command(name = "tangible-objects-tui", version, about)]
struct Cli {
    /// Daemon GraphQL endpoint (flag > `TANGIBLE_ENDPOINT` >
    /// client.toml > localhost default).
    #[arg(long, env = "TANGIBLE_ENDPOINT")]
    endpoint: Option<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();
    let endpoint = Endpoint::resolve(args.endpoint);
    let client = match GraphQLClient::new(endpoint) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("endpoint invalid: {e}");
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new(client).run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    if let Err(e) = &result {
        eprintln!("{e}");
    }
    result.map_err(|e| io::Error::other(e.to_string()))
}
