//! Board area widget.
//!
//! The board itself is bitmap-only. This widget only clears the board area in
//! the TUI; the PNG is overlaid by the caller after `terminal.draw`.

use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

use crate::app::App;
use crate::ui::Q_BG;

/// Clear the board panel. The bitmap renderer is responsible for drawing the
/// actual board; there is no Unicode fallback.
pub fn render_board(f: &mut Frame, area: Rect, app: &App, _use_bitmap_overlay: bool) {
    let _ = app;
    f.render_widget(Paragraph::new("").style(Style::default().bg(Q_BG)), area);
}

pub fn last_move_for(app: &App) -> Option<shakmaty::Move> {
    app.current_game.as_ref().and_then(|g| {
        if app.current_ply > 0 {
            g.sans[app.current_ply - 1]
                .san
                .to_move(&g.positions[app.current_ply - 1])
                .ok()
        } else {
            None
        }
    })
}
