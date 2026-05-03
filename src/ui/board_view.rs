//! Board area widget.
//!
//! The board itself is bitmap-only. This widget only clears the board area in
//! the TUI; the PNG is overlaid by the caller after `terminal.draw`.

use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

use crate::app::App;
use crate::renderer::{LABEL_W, SQ_H, SQ_W};
use crate::ui::{Q_BG, Q_TEXT};

/// Clear the board panel and draw rank/file labels around it.
/// The bitmap renderer draws the actual board over the inner area.
pub fn render_board(f: &mut Frame, area: Rect, app: &App, _use_bitmap_overlay: bool) {
    let _ = app;
    f.render_widget(Paragraph::new("").style(Style::default().bg(Q_BG)), area);

    // Rank labels (1-8) down the left edge.
    for rank in 0..8_u16 {
        let rank_num = 8 - rank;
        let y = area.y + rank * SQ_H;
        if y < area.y + area.height {
            let mut cell = ratatui::buffer::Cell::default();
            cell.set_symbol(&rank_num.to_string());
            cell.fg = Q_TEXT;
            cell.bg = Q_BG;
            if let Some(buf_cell) = f.buffer_mut().cell_mut((area.x, y)) {
                *buf_cell = cell;
            }
        }
    }

    // File labels (a-h) along the bottom edge.
    let file_label_y = area.y + SQ_H * 8;
    if file_label_y < area.y + area.height {
        for file in 0..8_u16 {
            let file_char = (b'a' + file as u8) as char;
            let x = area.x + LABEL_W + file * SQ_W + 1;
            if x < area.x + area.width {
                let mut cell = ratatui::buffer::Cell::default();
                cell.set_symbol(&file_char.to_string());
                cell.fg = Q_TEXT;
                cell.bg = Q_BG;
                if let Some(buf_cell) = f.buffer_mut().cell_mut((x, file_label_y)) {
                    *buf_cell = cell;
                }
            }
        }
    }
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
