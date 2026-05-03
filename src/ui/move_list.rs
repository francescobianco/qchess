use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::ui::{Q_BG, Q_DIM, Q_TEXT};

pub fn render_move_list(f: &mut Frame, area: Rect, app: &App) {
    let Some(game) = &app.current_game else {
        let p = Paragraph::new("No game open. Press G to browse games.")
            .style(Style::default().fg(Q_DIM).bg(Q_BG));
        f.render_widget(p, area);
        return;
    };

    let sans = &game.sans;
    let mut lines: Vec<Line> = Vec::new();
    let mut i = 0usize;

    while i < sans.len() {
        let move_num = i / 2 + 1;
        let num_span = Span::styled(
            format!("{:>3}.", move_num),
            Style::default().fg(Q_DIM).bg(Q_BG),
        );

        // White move (i → ply i+1)
        let w_ply = i + 1;
        let w_san = format!("{}", sans[i]);
        let w_style = if app.current_ply == w_ply {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Q_TEXT).bg(Q_BG)
        };
        let w_span = Span::styled(format!(" {:<8}", w_san), w_style);

        let mut spans = vec![num_span, w_span];

        // Black move (i+1 → ply i+2)
        if i + 1 < sans.len() {
            let b_ply = i + 2;
            let b_san = format!("{}", sans[i + 1]);
            let b_style = if app.current_ply == b_ply {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Q_DIM).bg(Q_BG)
            };
            spans.push(Span::styled(format!("{:<8}", b_san), b_style));
        }

        lines.push(Line::from(spans));
        i += 2;
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Starting position",
            Style::default().fg(Q_DIM).bg(Q_BG),
        )));
    }

    // Scroll to keep current move visible
    let current_line = app.current_ply.saturating_sub(1) / 2;
    let scroll = current_line.saturating_sub((area.height as usize) / 2) as u16;

    let para = Paragraph::new(lines)
        .style(Style::default().bg(Q_BG))
        .scroll((scroll, 0));
    f.render_widget(para, area);
}
