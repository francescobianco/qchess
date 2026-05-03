use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::renderer::{BoardRenderer, RenderOptions, RenderedBoard, UnicodeRenderer};

pub fn render_board(f: &mut Frame, area: Rect, app: &App) {
    let Some(game) = &app.current_game else {
        let placeholder = Paragraph::new("Select a game and press Enter")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, area);
        return;
    };

    let pos = &game.positions[app.current_ply];

    let last_move = if app.current_ply > 0 {
        // Reconstruct the move from the SAN at the previous ply index.
        // We stored positions[i] = position before move i, so:
        // the move that led to positions[current_ply] is sans[current_ply - 1].
        let prev_pos = &game.positions[app.current_ply - 1];
        game.sans[app.current_ply - 1]
            .san
            .to_move(prev_pos)
            .ok()
    } else {
        None
    };

    let renderer = UnicodeRenderer;
    let options = RenderOptions {
        flipped: false,
        last_move,
    };

    let RenderedBoard::Unicode(lines) = renderer.render(pos, &options);

    let paragraph_lines: Vec<Line> = lines
        .into_iter()
        .map(|l| Line::from(Span::raw(l)))
        .collect();

    let paragraph = Paragraph::new(paragraph_lines);
    f.render_widget(paragraph, area);
}