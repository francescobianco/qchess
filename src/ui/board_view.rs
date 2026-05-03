//! Board area widget.
//!
//! The Unicode board is always rendered first. Bitmap backends then overlay
//! the PNG after `terminal.draw`, so unsupported image protocols still leave a
//! visible board instead of an empty panel.

use ratatui::{layout::Rect, Frame};
use shakmaty::Chess;

use crate::app::App;
use crate::renderer::{BoardWidget, RenderOptions as UniRO};

/// Render the board panel. `use_bitmap_overlay` is accepted so callers can keep
/// one code path, but the fallback board is intentionally always visible.
pub fn render_board(f: &mut Frame, area: Rect, app: &App, _use_bitmap_overlay: bool) {
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
