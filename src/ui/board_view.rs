use ratatui::{layout::Rect, Frame};
use shakmaty::Chess;

use crate::app::App;
use crate::renderer::{BoardWidget, RenderOptions};

pub fn render_board(f: &mut Frame, area: Rect, app: &App) {
    let last_move = app.current_game.as_ref().and_then(|g| {
        if app.current_ply > 0 {
            g.sans[app.current_ply - 1]
                .san
                .to_move(&g.positions[app.current_ply - 1])
                .ok()
        } else {
            None
        }
    });

    // If a game is loaded, show the game position; otherwise show the start position.
    match &app.current_game {
        Some(game) => {
            let pos = &game.positions[app.current_ply];
            render_inner(f, area, pos, last_move);
        }
        None => {
            let start = Chess::default();
            render_inner(f, area, &start, None);
        }
    }
}

fn render_inner(
    f: &mut Frame,
    area: Rect,
    pos: &Chess,
    last_move: Option<shakmaty::Move>,
) {
    f.render_widget(
        BoardWidget {
            pos,
            options: RenderOptions { flipped: false, last_move },
        },
        area,
    );
}
