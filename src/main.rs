mod app;
mod db;
mod event;
mod pgn;
mod renderer;
mod tui;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use event::{Event, EventLoop};

use crate::{app::App, tui::init_panic_hook};

#[derive(Parser, Debug)]
#[command(
    name = "qchess",
    about = "TUI chess database browser — any folder with PGN files is a valid database"
)]
struct Cli {
    /// Path to the folder containing PGN files (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Install panic hook before entering the TUI so the terminal is always
    // restored if something panics.
    init_panic_hook();

    // Build application state (scans PGN files).
    let mut app = App::new(&cli.path)?;

    // Enter alternate screen / raw mode.
    let mut terminal = tui::enter_alternate_screen()?;

    // Event loop.
    let events = EventLoop::new();

    while app.running {
        terminal.draw(|f| ui::draw(f, &app))?;

        match events.next()? {
            Event::Key(key) => app.handle_key(key),
            Event::Tick => {} // just redraw
        }
    }

    // Restore terminal.
    tui::leave_alternate_screen(&mut terminal)?;

    Ok(())
}