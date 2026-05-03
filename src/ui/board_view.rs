use ratatui::{layout::Rect, Frame};
use shakmaty::Chess;

use crate::app::App;
use crate::renderer::{BoardWidget, RenderOptions};

static INITIAL_POS: std::sync::OnceLock<Chess> = std::sync::OnceLock::new();

fn initial_pos() -> &'static Chess {
    INITIAL_POS.get_or_init(Chess::default)
}

pub fn render_board(f: &mut Frame, area: Rect, app: &App) {
    let last_move = app.current_game.as_ref().and_then(|g| {
        if app.current_ply > 0 {
            let prev_pos = &g.positions[app.current_ply - 1];
            g.sans[app.current_ply - 1].san.to_move(prev_pos).ok()
        } else {
            None
        }
    });

    let pos: &Chess = app
        .current_game
        .as_ref()
        .map(|g| &g.positions[app.current_ply])
        .unwrap_or_else(initial_pos);

    let widget = BoardWidget {
        pos,
        options: RenderOptions {
            flipped: false,
            last_move,
        },
    };

    f.render_widget(widget, area);
}