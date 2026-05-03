mod app;
mod assets;
mod db;
mod event;
mod kitty;
mod pgn;
mod png_renderer;
mod renderer;
mod tui;
mod ui;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use event::{Event, EventLoop};
use ratatui::layout::Rect;
use shakmaty::Chess;

use crate::{
    app::App,
    assets::{FritzBoardStyle, FritzPieceSet},
    png_renderer::{PngBoardRenderer, RenderOptions as PngRO},
    tui::init_panic_hook,
    ui::board_view::last_move_for,
};

#[derive(Parser, Debug)]
#[command(
    name = "qchess",
    about = "TUI chess database browser — any folder with PGN files is a valid database"
)]
struct Cli {
    /// Path to the folder containing PGN files (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Force Unicode board rendering even if Kitty graphics are available
    #[arg(long)]
    unicode: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_panic_hook();

    let mut app = App::new(&cli.path)?;

    // Detect Kitty support: use env TERM / TERM_PROGRAM heuristic.
    let use_kitty = !cli.unicode && supports_kitty();

    // Load Fritz assets (embedded in the binary).
    let board_style = FritzBoardStyle::new();
    let piece_set   = FritzPieceSet::new();
    let renderer    = PngBoardRenderer { style: &board_style, pieces: &piece_set };

    let start_pos = Chess::default();

    let mut terminal = tui::enter_alternate_screen()?;
    let events = EventLoop::new();

    let mut board_area = Rect::default();

    while app.running {
        // ── Draw TUI ─────────────────────────────────────────────────────────
        terminal.draw(|f| {
            board_area = ui::draw(f, &app, use_kitty);
        })?;

        // ── Overlay Kitty board PNG ───────────────────────────────────────────
        if use_kitty && !board_area.is_empty() {
            let pos: &Chess = app
                .current_game
                .as_ref()
                .map(|g| &g.positions[app.current_ply])
                .unwrap_or(&start_pos);

            let last = last_move_for(&app);
            let (hl_from, hl_to) = match &last {
                Some(m) => (m.from(), Some(m.to())),
                None    => (None, None),
            };

            let png = renderer.render(
                pos,
                &PngRO { flipped: false, hl_from, hl_to },
            );

            let _ = kitty::display(
                &png,
                board_area.x,
                board_area.y,
                board_area.width,
                board_area.height,
            );
        }

        // ── Handle events ─────────────────────────────────────────────────────
        match events.next()? {
            Event::Key(key) => app.handle_key(key),
            Event::Tick => {}
        }
    }

    tui::leave_alternate_screen(&mut terminal)?;
    Ok(())
}

fn supports_kitty() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    term.contains("kitty")
        || term_prog.to_lowercase().contains("kitty")
        || term_prog.to_lowercase().contains("wezterm")
        || std::env::var("KITTY_WINDOW_ID").is_ok()
}
