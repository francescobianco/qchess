pub mod board_view;
pub mod game_list;
pub mod move_list;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppScreen};

pub fn draw(f: &mut Frame, app: &App) {
    // ── Outer layout ──────────────────────────────────────────────────────────
    // Title bar | main area | status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(0),    // main content
            Constraint::Length(1), // status / keybindings
        ])
        .split(f.area());

    // ── Title bar ─────────────────────────────────────────────────────────────
    let title_text = format!(
        " qchess — {}",
        app.db.root.display()
    );
    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, outer[0]);

    // ── Main area split into left (game list) and right (board + moves) ───────
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // game list
            Constraint::Percentage(65), // board + moves
        ])
        .split(outer[1]);

    // ── Game list panel ───────────────────────────────────────────────────────
    let list_block = Block::default()
        .title(" Games ")
        .borders(Borders::ALL);
    let list_inner = list_block.inner(main[0]);
    f.render_widget(list_block, main[0]);
    game_list::render_game_list(f, list_inner, app);

    // ── Right panel: board + move list ────────────────────────────────────────
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(12),        // board view
            Constraint::Percentage(40), // move list
        ])
        .split(main[1]);

    let board_block = Block::default()
        .title(" Board ")
        .borders(Borders::ALL);
    let board_inner = board_block.inner(right[0]);
    f.render_widget(board_block, right[0]);
    board_view::render_board(f, board_inner, app);

    let moves_block = Block::default()
        .title(" Moves ")
        .borders(Borders::ALL);
    let moves_inner = moves_block.inner(right[1]);
    f.render_widget(moves_block, right[1]);
    move_list::render_move_list(f, moves_inner, app);

    // ── Status bar ────────────────────────────────────────────────────────────
    let hint = match app.screen {
        AppScreen::GameList => {
            " ↑/↓ navigate  Enter open  q quit"
        }
        AppScreen::BoardView => {
            " ←/→ move  Home start  End end  Esc back  q quit"
        }
    };

    let game_count = format!("  {} game(s)", app.db.len());

    let status_spans = vec![
        Span::styled(
            &game_count,
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
    ];
    let status = Paragraph::new(Line::from(status_spans))
        .style(Style::default().bg(Color::Reset));
    f.render_widget(status, outer[2]);
}