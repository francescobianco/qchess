use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn render_move_list(f: &mut Frame, area: Rect, app: &App) {
    let Some(game) = &app.current_game else {
        let placeholder = Paragraph::new("No game loaded")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, area);
        return;
    };

    // Build pairs of moves: (move_number, white_san, Option<black_san>)
    // current_ply: 0 = initial position, 1 = after first move, etc.
    let sans = &game.sans;

    let mut lines: Vec<Line> = Vec::new();

    let mut i = 0usize;
    while i < sans.len() {
        let move_num = i / 2 + 1;

        // White's move (even index 0, 2, 4, ...)
        let white_san = format!("{}", sans[i]);
        let white_ply = i + 1; // ply that results from this move
        let white_style = if app.current_ply == white_ply {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Black's move (odd index 1, 3, 5, ...) — may not exist
        let black_part = if i + 1 < sans.len() {
            let black_san = format!("{}", sans[i + 1]);
            let black_ply = i + 2;
            let black_style = if app.current_ply == black_ply {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Some((black_san, black_style))
        } else {
            None
        };

        let move_num_span = Span::styled(
            format!("{:>3}. ", move_num),
            Style::default().fg(Color::DarkGray),
        );
        let white_span = Span::styled(format!("{:<8}", white_san), white_style);

        let mut spans = vec![move_num_span, white_span];

        if let Some((black_san, black_style)) = black_part {
            spans.push(Span::styled(format!("{:<8}", black_san), black_style));
        }

        lines.push(Line::from(spans));
        i += 2;
    }

    // If at the initial position, show a hint.
    if sans.is_empty() {
        lines.push(Line::from(Span::styled(
            "No moves",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}