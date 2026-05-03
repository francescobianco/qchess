use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, StatefulWidget},
    Frame,
};

use crate::app::App;

pub fn render_game_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .db
        .games
        .iter()
        .map(|game_ref| {
            let meta = &game_ref.meta;
            let event = meta
                .event
                .as_deref()
                .unwrap_or("?");
            let result = &meta.result;

            let white = if meta.white.is_empty() { "?" } else { &meta.white };
            let black = if meta.black.is_empty() { "?" } else { &meta.black };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} vs {}", white, black),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{}", event),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(
                    result.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    let mut list_state = app.list_state.clone();
    StatefulWidget::render(list, area, f.buffer_mut(), &mut list_state);
}