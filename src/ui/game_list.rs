use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget},
    Frame,
};

use crate::app::App;
use crate::ui::{Q_BG, Q_DIM, Q_SEL_BG, Q_SEL_FG, Q_TEXT};

pub fn render_game_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .db
        .games
        .iter()
        .enumerate()
        .map(|(i, gr)| {
            let m = &gr.meta;
            let num = format!("{:>4}. ", i + 1);
            let white = if m.white.is_empty() { "?" } else { &m.white };
            let black = if m.black.is_empty() { "?" } else { &m.black };
            let vs = format!("{} vs {}", white, black);
            let event = m.event.as_deref().unwrap_or("?");
            let date  = m.date.as_deref().unwrap_or("????");
            let eco   = m.eco.as_deref().unwrap_or("---");
            let result = &m.result;

            Line::from(vec![
                Span::styled(num, Style::default().fg(Q_DIM).bg(Q_BG)),
                Span::styled(format!("{:<30}", vs), Style::default().fg(Q_TEXT).bg(Q_BG)),
                Span::styled(format!(" {:>4} ", result), Style::default().fg(Q_DIM).bg(Q_BG)),
                Span::styled(format!("{:<6} ", eco), Style::default().fg(Q_DIM).bg(Q_BG)),
                Span::styled(format!("{:.10} {}", event, date), Style::default().fg(Q_DIM).bg(Q_BG)),
            ])
        })
        .map(ListItem::new)
        .collect();

    let list = List::new(items)
        .style(Style::default().bg(Q_BG))
        .highlight_style(
            Style::default()
                .bg(Q_SEL_BG)
                .fg(Q_SEL_FG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    let mut state = app.picker_state.clone();
    StatefulWidget::render(list, area, f.buffer_mut(), &mut state);
}