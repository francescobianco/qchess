pub mod board_view;
pub mod game_list;
pub mod move_list;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, StatefulWidget},
    Frame,
};

use crate::app::{App, AppScreen, EngineEditorButton, EngineEditorFocus};
use crate::renderer::{BOARD_COLS, BOARD_ROWS};

// ── Current application palette ───────────────────────────────────────────────
pub const Q_BG: Color = Color::Black;
pub const Q_TEXT: Color = Color::White;
pub const Q_MENU_BG: Color = Color::Black;
pub const Q_MENU_FG: Color = Color::White;
pub const Q_BORDER: Color = Color::Black;
pub const Q_SEL_BG: Color = Color::Blue;
pub const Q_SEL_FG: Color = Color::White;
pub const Q_STATUS_BG: Color = Color::Gray;
pub const Q_STATUS_FG: Color = Color::DarkGray;
pub const Q_DIM: Color = Color::DarkGray;

fn menu_bar() -> Paragraph<'static> {
    let spans = vec![
        Span::styled(" Database", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Games", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Moves", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Engine", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  Options", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
        Span::styled("  ?", Style::default().fg(Q_MENU_FG).bg(Q_MENU_BG)),
    ];
    Paragraph::new(Line::from(spans)).style(Style::default().bg(Q_MENU_BG))
}

fn status_bar(app: &App) -> Paragraph<'_> {
    let game_info = match &app.current_game {
        Some(g) => format!(
            " {} vs {}  Ply {}/{}",
            g.game_ref.meta.white,
            g.game_ref.meta.black,
            app.current_ply,
            g.positions.len().saturating_sub(1),
        ),
        None => format!(" {} game(s) in database", app.db.len()),
    };

    let hint = match app.screen {
        AppScreen::Main => " │ ↑↓←→ square  G=games  D/M/E=menu  Q=quit",
        AppScreen::GamePicker => " │ ↑↓ navigate  Enter=open  Esc/G=close",
        AppScreen::EngineMenu => " │ ↑↓ navigate  Enter=open  Esc/E=close",
        AppScreen::EngineEditor => " │ Tab=next field  Enter=activate  Esc=cancel",
    };

    let spans = vec![
        Span::styled(game_info, Style::default().fg(Q_STATUS_FG).bg(Q_STATUS_BG)),
        Span::styled(hint, Style::default().fg(Q_DIM).bg(Q_STATUS_BG)),
    ];
    Paragraph::new(Line::from(spans)).style(Style::default().bg(Q_STATUS_BG))
}

/// Returns a centered rect of fixed `width × height` within `parent`.
fn centered_fixed(width: u16, height: u16, parent: Rect) -> Rect {
    let x = parent.x + parent.width.saturating_sub(width) / 2;
    let y = parent.y + parent.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(parent.width),
        height: height.min(parent.height),
    }
}

/// Renders the full TUI and returns the inner Rect of the board panel
/// (used by the caller to overlay the Kitty PNG image).
pub fn draw(f: &mut Frame, app: &App, use_kitty: bool) -> ratatui::layout::Rect {
    let full = f.area();

    // ── Outer split: menu | main | status ─────────────────────────────────────
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // menu bar
            Constraint::Min(0),    // main content
            Constraint::Length(1), // status bar
        ])
        .split(full);

    f.render_widget(menu_bar(), outer[0]);
    f.render_widget(status_bar(app), outer[2]);

    let main = outer[1];

    // ── Main area: board (fixed width) | moves; engine below both ─────────────
    // Vertical: top row (board + moves) | engine panel
    let main_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(BOARD_ROWS), Constraint::Min(0)])
        .split(main);

    let top_row = main_rows[0];
    let engine_area = main_rows[1];

    // Horizontal split: board | moves
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(BOARD_COLS), Constraint::Min(20)])
        .split(top_row);

    let board_area = top_cols[0];
    let moves_area = top_cols[1];

    // ── Board ─────────────────────────────────────────────────────────────────
    board_view::render_board(f, board_area, app, use_kitty);

    // ── Moves ─────────────────────────────────────────────────────────────────
    move_list::render_move_list(f, moves_area, app);

    // ── Engine panel ──────────────────────────────────────────────────────────
    let engine_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(engine_area);
    f.render_widget(
        Paragraph::new(engine_separator(engine_rows[0].width))
            .style(Style::default().fg(Q_TEXT).bg(Q_BG)),
        engine_rows[0],
    );

    let engine_text: Vec<Line> = app
        .engine_lines
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                l.as_str(),
                Style::default().fg(Q_DIM).bg(Q_BG),
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(engine_text).style(Style::default().bg(Q_BG)),
        engine_rows[1],
    );

    // ── Game picker overlay ───────────────────────────────────────────────────
    if app.screen == AppScreen::GamePicker {
        let popup_w = full.width.min(70);
        let popup_h = full.height.min(24);
        let popup_area = centered_fixed(popup_w, popup_h, full);

        f.render_widget(Clear, popup_area);

        let db_path = app.db.root.display().to_string();
        let popup_block = Block::default()
            .title(format!(" Games — {} ({} games) ", db_path, app.db.len()))
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Q_SEL_BG).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Q_BG));

        let popup_inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        game_list::render_game_list(f, popup_inner, app);
    }

    if app.screen == AppScreen::EngineMenu {
        render_engine_menu(f, app, full);
    }

    if app.screen == AppScreen::EngineEditor {
        render_engine_editor(f, app, full);
    }

    board_area
}

fn engine_separator(width: u16) -> String {
    let title = " Engine Analysis ";
    let width = width as usize;
    if width <= title.len() {
        return "-".repeat(width);
    }

    let left = (width - title.len()) / 2;
    let right = width - title.len() - left;
    format!("{}{}{}", "-".repeat(left), title, "-".repeat(right))
}

fn render_engine_menu(f: &mut Frame, app: &App, full: Rect) {
    let menu_x = 23;
    let menu_y = 1;
    let menu_w = 44.min(full.width.saturating_sub(menu_x));
    let item_count = 2 + app.config.engines.len() as u16 + u16::from(app.config.engines.is_empty());
    let menu_h = (item_count + 2).min(full.height.saturating_sub(menu_y));
    let popup_area = Rect {
        x: menu_x,
        y: menu_y,
        width: menu_w,
        height: menu_h,
    };
    if popup_area.is_empty() {
        return;
    }
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Q_BORDER))
        .style(Style::default().bg(Q_BG));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut items = vec![
        ListItem::new(Line::from(Span::styled(
            "Add New Engine",
            Style::default().fg(Q_TEXT).bg(Q_BG),
        ))),
        ListItem::new(Line::from(Span::styled(
            "----------------",
            Style::default().fg(Q_DIM).bg(Q_BG),
        ))),
    ];

    for (idx, engine) in app.config.engines.iter().enumerate() {
        let active = if app.config.active_engine == Some(idx) {
            "[active]"
        } else {
            "        "
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{active} "), Style::default().fg(Q_DIM).bg(Q_BG)),
            Span::styled(engine.name.as_str(), Style::default().fg(Q_TEXT).bg(Q_BG)),
            Span::styled(
                format!("  {}", engine.command),
                Style::default().fg(Q_DIM).bg(Q_BG),
            ),
        ])));
    }

    if app.config.engines.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No registered engines",
            Style::default().fg(Q_DIM).bg(Q_BG),
        ))));
    }

    let list = List::new(items)
        .style(Style::default().bg(Q_BG))
        .highlight_style(Style::default().fg(Q_SEL_FG).bg(Q_SEL_BG));
    let mut state = app.engine_menu_state.clone();
    StatefulWidget::render(list, inner, f.buffer_mut(), &mut state);
}

fn render_engine_editor(f: &mut Frame, app: &App, full: Rect) {
    let Some(editor) = app.engine_editor.as_ref() else {
        return;
    };

    let popup_area = centered_fixed(full.width.min(72), full.height.min(16), full);
    f.render_widget(Clear, popup_area);

    let title = if editor.index.is_some() {
        " Edit Engine "
    } else {
        " Add Engine "
    };
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Q_BORDER))
        .style(Style::default().bg(Q_BG));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_editor_field(
        f,
        rows[0],
        "Name",
        &editor.draft.name,
        editor.focus == EngineEditorFocus::Name,
    );
    render_editor_field(
        f,
        rows[1],
        "Command",
        &editor.draft.command,
        editor.focus == EngineEditorFocus::Command,
    );
    render_editor_field(
        f,
        rows[2],
        "Args",
        &editor.draft.args,
        editor.focus == EngineEditorFocus::Args,
    );

    let buttons = [
        (EngineEditorButton::Delete, " Delete "),
        (EngineEditorButton::Cancel, " Cancel "),
        (EngineEditorButton::Save, " Save "),
        (EngineEditorButton::SaveAndUse, " Save and Use "),
    ];
    let spans = buttons
        .iter()
        .map(|(button, label)| {
            let selected = editor.focus == EngineEditorFocus::Button(*button);
            Span::styled(
                *label,
                if selected {
                    Style::default().fg(Q_SEL_FG).bg(Q_SEL_BG)
                } else {
                    Style::default().fg(Q_TEXT).bg(Q_BG)
                },
            )
        })
        .collect::<Vec<_>>();
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Q_BG)),
        rows[4],
    );
}

fn render_editor_field(f: &mut Frame, area: Rect, label: &str, value: &str, selected: bool) {
    let style = if selected {
        Style::default().fg(Q_SEL_FG).bg(Q_SEL_BG)
    } else {
        Style::default().fg(Q_TEXT).bg(Q_BG)
    };
    let text = format!("{label:<8} {value}");
    f.render_widget(Paragraph::new(text).style(style), area);
}
