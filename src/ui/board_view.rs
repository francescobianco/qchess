//! Board area widget.
//!
//! When the Kitty protocol is active, this widget only paints the background
//! (the PNG is overlaid by the caller after `terminal.draw`).
//! When Kitty is not available it falls back to the Unicode `BoardWidget`.

use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};
use shakmaty::Chess;

use crate::app::App;
use crate::renderer::{BoardWidget, RenderOptions as UniRO};
use crate::ui::Q_BG;

/// Render the board panel.  `use_kitty` = true fills the area with the
/// background colour so the caller can place the PNG on top; false renders
/// the Unicode fallback directly.
pub fn render_board(f: &mut Frame, area: Rect, app: &App, use_kitty: bool) {
    if use_kitty {
        // Just paint the background so ratatui doesn't leave stale cells.
        let blank = Paragraph::new("").style(Style::default().bg(Q_BG));
        f.render_widget(blank, area);
        return;
    }

    // ── Unicode fallback ──────────────────────────────────────────────────────
    let last_move = last_move_for(app);
    match &app.current_game {
        Some(game) => {
            let pos = &game.positions[app.current_ply];
            render_unicode(f, area, pos, last_move);
        }
        None => {
            let start = Chess::default();
            render_unicode(f, area, &start, None);
        }
    }
}

fn render_unicode(f: &mut Frame, area: Rect, pos: &Chess, last_move: Option<shakmaty::Move>) {
    f.render_widget(
        BoardWidget {
            pos,
            options: UniRO {
                flipped: false,
                last_move,
            },
        },
        area,
    );
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
