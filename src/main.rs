mod app;
mod assets;
mod db;
mod event;
mod kitty;
mod pgn;
mod png_renderer;
mod renderer;
mod sixel;
mod tui;
mod ui;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use clap::{Parser, ValueEnum};
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

    /// Board graphics backend: auto, kitty, sixel, unicode
    #[arg(long, value_enum, default_value_t = GraphicsMode::Auto)]
    graphics: GraphicsMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum GraphicsMode {
    Auto,
    Kitty,
    Sixel,
    Unicode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GraphicsBackend {
    Kitty,
    Sixel,
    Unicode,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_panic_hook();

    let graphics = select_graphics_backend(&cli);
    if graphics == GraphicsBackend::Unicode && reopen_in_graphics_terminal(&cli)? {
        return Ok(());
    }

    let mut app = App::new(&cli.path)?;

    let use_png_board = graphics != GraphicsBackend::Unicode;

    // Load Fritz assets (embedded in the binary).
    let board_style = FritzBoardStyle::new();
    let piece_set = FritzPieceSet::new();
    let renderer = PngBoardRenderer {
        style: &board_style,
        pieces: &piece_set,
    };

    let start_pos = Chess::default();

    let mut terminal = tui::enter_alternate_screen()?;
    let events = EventLoop::new();

    let mut board_area = Rect::default();

    while app.running {
        // ── Draw TUI ─────────────────────────────────────────────────────────
        terminal.draw(|f| {
            board_area = ui::draw(f, &app, use_png_board);
        })?;

        // ── Overlay bitmap board PNG ─────────────────────────────────────────
        if use_png_board && !board_area.is_empty() {
            let pos: &Chess = app
                .current_game
                .as_ref()
                .map(|g| &g.positions[app.current_ply])
                .unwrap_or(&start_pos);

            let last = last_move_for(&app);
            let (hl_from, hl_to) = match &last {
                Some(m) => (m.from(), Some(m.to())),
                None => (None, None),
            };

            let png = renderer.render(
                pos,
                &PngRO {
                    flipped: false,
                    hl_from,
                    hl_to,
                },
            );

            let _ = match graphics {
                GraphicsBackend::Kitty => kitty::display(
                    &png,
                    board_area.x,
                    board_area.y,
                    board_area.width,
                    board_area.height,
                ),
                GraphicsBackend::Sixel => sixel::display(&png, board_area.x, board_area.y),
                GraphicsBackend::Unicode => Ok(()),
            };
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

fn select_graphics_backend(cli: &Cli) -> GraphicsBackend {
    if cli.unicode {
        return GraphicsBackend::Unicode;
    }

    match cli.graphics {
        GraphicsMode::Auto => {
            if supports_kitty() {
                GraphicsBackend::Kitty
            } else if supports_sixel() {
                GraphicsBackend::Sixel
            } else {
                GraphicsBackend::Unicode
            }
        }
        GraphicsMode::Kitty => GraphicsBackend::Kitty,
        GraphicsMode::Sixel => GraphicsBackend::Sixel,
        GraphicsMode::Unicode => GraphicsBackend::Unicode,
    }
}

fn reopen_in_graphics_terminal(cli: &Cli) -> Result<bool> {
    if cli.unicode
        || cli.graphics != GraphicsMode::Auto
        || std::env::var_os("QCHESS_GRAPHICS_CHILD").is_some()
    {
        return Ok(false);
    }

    let Some(launcher) = find_graphics_terminal() else {
        eprintln!(
            "qchess: bitmap board unavailable in this terminal; install kitty or wezterm, or run with --unicode"
        );
        return Ok(false);
    };

    let exe = std::env::current_exe()?;
    let cwd = std::env::current_dir()?;
    let path = cli.path.as_os_str().to_os_string();

    let mut command = launcher.command(&exe, &cwd, &path);
    command.env("QCHESS_GRAPHICS_CHILD", "1");
    command.spawn()?;
    Ok(true)
}

#[derive(Clone, Copy, Debug)]
enum GraphicsTerminal {
    Kitty,
    WezTerm,
}

const GRAPHICS_TERMINAL_FONT_SIZE: &str = "14.0";

impl GraphicsTerminal {
    fn command(self, exe: &Path, cwd: &Path, path: &OsString) -> Command {
        match self {
            GraphicsTerminal::Kitty => {
                let mut command = Command::new("kitty");
                command
                    .arg("--title")
                    .arg("qchess")
                    .arg("--working-directory")
                    .arg(cwd)
                    .arg("--override")
                    .arg(format!("font_size={GRAPHICS_TERMINAL_FONT_SIZE}"))
                    .arg(exe)
                    .arg("--graphics")
                    .arg("kitty")
                    .arg(path);
                command
            }
            GraphicsTerminal::WezTerm => {
                let mut command = Command::new("wezterm");
                command
                    .arg("--config")
                    .arg(format!("font_size={GRAPHICS_TERMINAL_FONT_SIZE}"))
                    .arg("start")
                    .arg("--cwd")
                    .arg(cwd)
                    .arg("--")
                    .arg(exe)
                    .arg("--graphics")
                    .arg("kitty")
                    .arg(path);
                command
            }
        }
    }
}

fn find_graphics_terminal() -> Option<GraphicsTerminal> {
    if command_exists("kitty") {
        Some(GraphicsTerminal::Kitty)
    } else if command_exists("wezterm") {
        Some(GraphicsTerminal::WezTerm)
    } else {
        None
    }
}

fn command_exists(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|path| path.join(name).is_file())
}

fn supports_kitty() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    term.contains("kitty")
        || term_prog.to_lowercase().contains("kitty")
        || term_prog.to_lowercase().contains("wezterm")
        || std::env::var("KITTY_WINDOW_ID").is_ok()
}

fn supports_sixel() -> bool {
    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
    let term_prog = std::env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_lowercase();

    std::env::var("VTE_VERSION").is_ok()
        || std::env::var("GNOME_TERMINAL_SCREEN").is_ok()
        || term_prog.contains("gnome")
        || term.contains("sixel")
        || term.contains("mlterm")
        || term.contains("foot")
}
