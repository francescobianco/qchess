pub mod board_view;
pub mod game_list;
pub mod move_list;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, AppScreen};
use crate::renderer::{BOARD_COLS, BOARD_ROWS};

// ── Current application palette ───────────────────────────────────────────────
pub const Q_BG: Color = Color::White;
pub const Q_TEXT: Color = Color::Black;
pub const Q_MENU_BG: Color = Color::Black;
pub const Q_MENU_FG: Color = Color::White;
pub const Q_BORDER: Color = Color::Black;
pub const Q_SEL_BG: Color = Color::Blue;
pub const Q_SEL_FG: Color = Color::White;
pub const Q_STATUS_BG: Color = Color::Black;
pub const Q_STATUS_FG: Color = Color::White;
pub const Q_DIM: Color = Color::DarkGray;

fn menu_bar() -> Paragraph<'static> {
    let spans = vec![
        Span::styled(" Database", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Partite", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Mosse", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Motore", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Opzioni", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  ?", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Esci", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
    ];
    Paragraph::new(Line::from(spans)).style(Style::default().bg(Q_MENU_BG))
}

fn status_bar(app: &App) -> Paragraph<'_> {
    let game_info = match &app.current_game {
        Some(g) => format!(
            " {} vs {}  Ply {}/{}",
            g.game_ref.meta.white,
            g.game_ref.meta.black,
            app.current_ply,
            g.positions.len().saturating_sub(1),
        ),
        None => format!(" {} game(s) in database", app.db.len()),
    };

    let hint = match app.screen {
        AppScreen::Main => " │ ↑↓←→ casella  G=partite  D/M/E=menu  Q=esci",
        AppScreen::GamePicker => " │ ↑↓ navigate  Enter=open  Esc/G=close",
    };

    let spans = vec![
        Span::styled(game_info, Style::default().fg(Q_STATUS_FG).bg(Q_STATUS_BG)),
        Span::styled(hint, Style::default().fg(Q_DIM).bg(Q_STATUS_BG)),
    ];
    Paragraph::new(Line::from(spans)).style(Style::default().bg(Q_STATUS_BG))
}

fn qblock(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Q_BORDER))
        .style(Style::default().bg(Q_BG))
}

/// Returns a centered rect of fixed `width × height` within `parent`.
fn centered_fixed(width: u16, height: u16, parent: Rect) -> Rect {
    let x = parent.x + parent.width.saturating_sub(width) / 2;
    let y = parent.y + parent.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(parent.width),
        height: height.min(parent.height),
    }
}

/// Renders the full TUI and returns the inner Rect of the board panel
/// (used by the caller to overlay the Kitty PNG image).
pub fn draw(f: &mut Frame, app: &App, use_kitty: bool) -> ratatui::layout::Rect {
    let full = f.area();

    // ── Outer split: menu | main | status ─────────────────────────────────────
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // menu bar
            Constraint::Min(0),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(full);

    f.render_widget(menu_bar(), outer[0]);
    f.render_widget(status_bar(app), outer[2]);

    let main = outer[1];

    // ── Main area: board (fixed width) | moves; engine below both ─────────────
    // Vertical: top row (board + moves) | engine panel
    let engine_height: u16 = 6;
    let main_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(BOARD_ROWS + 2), // board+moves (block border = +2)
            Constraint::Length(engine_height),
        ])
        .split(main);

    let top_row = main_rows[0];
    let engine_area = main_rows[1];

    // Horizontal split: board | moves
    let board_total = BOARD_COLS + 2; // +2 for block borders
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(board_total), Constraint::Min(20)])
        .split(top_row);

    let board_panel = top_cols[0];
    let moves_panel = top_cols[1];

    // ── Board ─────────────────────────────────────────────────────────────────
    let board_title = match &app.current_game {
        Some(g) => format!(
            "{} vs {}  {}  {}",
            g.game_ref.meta.white,
            g.game_ref.meta.black,
            g.game_ref.meta.event.as_deref().unwrap_or("?"),
            g.game_ref.meta.result,
        ),
        None => "No game — starting position".to_string(),
    };
    let board_block = qblock(&board_title);
    let board_inner = board_block.inner(board_panel);
    f.render_widget(board_block, board_panel);
    board_view::render_board(f, board_inner, app, use_kitty);

    // ── Moves ─────────────────────────────────────────────────────────────────
    let moves_block = qblock("Moves");
    let moves_inner = moves_block.inner(moves_panel);
    f.render_widget(moves_block, moves_panel);
    move_list::render_move_list(f, moves_inner, app);

    // ── Engine panel ──────────────────────────────────────────────────────────
    let engine_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(engine_area);
    f.render_widget(
        Paragraph::new(" Engine Analysis ").style(
            Style::default()
                .fg(Q_MENU_FG)
                .bg(Q_MENU_BG)
                .add_modifier(Modifier::BOLD),
        ),
        engine_rows[0],
    );

    let engine_text: Vec<Line> = app
        .engine_lines
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                l.as_str(),
                Style::default().fg(Q_DIM).bg(Q_BG),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(engine_text).style(Style::default().bg(Q_BG)),
        engine_rows[1],
    );

    // ── Game picker overlay ───────────────────────────────────────────────────
    if app.screen == AppScreen::GamePicker {
        let popup_w = full.width.min(70);
        let popup_h = full.height.min(24);
        let popup_area = centered_fixed(popup_w, popup_h, full);

        f.render_widget(Clear, popup_area);

        let db_path = app.db.root.display().to_string();
        let popup_block = Block::default()
            .title(format!(" Partite — {} ({} games) ", db_path, app.db.len()))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Q_SEL_BG).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Q_BG));

        let popup_inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        game_list::render_game_list(f, popup_inner, app);
    }

    board_inner
}
