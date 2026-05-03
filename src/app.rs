use std::path::Path;

use crate::{
    config::{EngineConfig, UserConfig},
    db::FolderDatabase,
    pgn::LoadedGame,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use shakmaty::Square;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    Main,       // board left, moves right, engine bottom
    GamePicker, // floating overlay with game list
    EngineMenu,
    EngineEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEditorButton {
    Delete,
    Cancel,
    Save,
    SaveAndUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEditorFocus {
    Name,
    Command,
    Args,
    Button(EngineEditorButton),
}

#[derive(Debug, Clone)]
pub struct EngineEditorState {
    pub index: Option<usize>,
    pub draft: EngineConfig,
    pub focus: EngineEditorFocus,
}

pub struct App {
    pub db: FolderDatabase,
    pub screen: AppScreen,
    pub picker_state: ListState,
    pub engine_menu_state: ListState,
    pub config: UserConfig,
    pub engine_editor: Option<EngineEditorState>,
    pub current_game: Option<LoadedGame>,
    pub current_ply: usize,
    pub selected_square: Square,
    pub running: bool,
    pub engine_lines: Vec<String>, // placeholder for engine output
}

impl App {
    pub fn new(path: &Path) -> Result<Self> {
        let db = FolderDatabase::load(path)?;
        let config = UserConfig::load().unwrap_or_default();
        let mut picker_state = ListState::default();
        if !db.is_empty() {
            picker_state.select(Some(0));
        }
        let mut engine_menu_state = ListState::default();
        engine_menu_state.select(Some(0));
        let engine_lines = engine_status_lines(&config);
        Ok(App {
            db,
            screen: AppScreen::Main,
            picker_state,
            engine_menu_state,
            config,
            engine_editor: None,
            current_game: None,
            current_ply: 0,
            selected_square: Square::E2,
            running: true,
            engine_lines,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.running = false;
            return;
        }

        match self.screen {
            AppScreen::Main => self.handle_key_main(key),
            AppScreen::GamePicker => self.handle_key_picker(key),
            AppScreen::EngineMenu => self.handle_key_engine_menu(key),
            AppScreen::EngineEditor => self.handle_key_engine_editor(key),
        }
    }

    fn handle_key_main(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('g') | KeyCode::Char('G') => {
                self.screen = AppScreen::GamePicker;
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.screen = AppScreen::EngineMenu;
                self.engine_menu_state.select(Some(0));
            }
            KeyCode::Right => self.move_square_cursor(1, 0),
            KeyCode::Left => self.move_square_cursor(-1, 0),
            KeyCode::Up => self.move_square_cursor(0, 1),
            KeyCode::Down => self.move_square_cursor(0, -1),
            KeyCode::Char('l') => self.next_move(),
            KeyCode::Char('h') => self.prev_move(),
            KeyCode::Home | KeyCode::Char('s') => self.go_start(),
            KeyCode::End => self.go_end(),
            _ => {}
        }
    }

    fn handle_key_picker(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('g') | KeyCode::Char('G') => {
                self.screen = AppScreen::Main;
            }
            KeyCode::Down | KeyCode::Char('j') => self.picker_next(),
            KeyCode::Up | KeyCode::Char('k') => self.picker_prev(),
            KeyCode::Enter => self.open_selected_game(),
            _ => {}
        }
    }

    fn handle_key_engine_menu(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('e') | KeyCode::Char('E') => {
                self.screen = AppScreen::Main;
            }
            KeyCode::Down | KeyCode::Char('j') => self.engine_menu_next(),
            KeyCode::Up | KeyCode::Char('k') => self.engine_menu_prev(),
            KeyCode::Enter => self.open_selected_engine_menu_item(),
            _ => {}
        }
    }

    fn handle_key_engine_editor(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.screen = AppScreen::EngineMenu,
            KeyCode::Tab | KeyCode::Down => self.engine_editor_next_focus(),
            KeyCode::BackTab | KeyCode::Up => self.engine_editor_prev_focus(),
            KeyCode::Left => self.engine_editor_prev_button(),
            KeyCode::Right => self.engine_editor_next_button(),
            KeyCode::Enter => self.engine_editor_activate(),
            KeyCode::Backspace => self.engine_editor_backspace(),
            KeyCode::Char(ch) => self.engine_editor_char(ch),
            _ => {}
        }
    }

    fn picker_next(&mut self) {
        let len = self.db.len();
        if len == 0 {
            return;
        }
        let sel = self.picker_state.selected().unwrap_or(0);
        self.picker_state.select(Some((sel + 1).min(len - 1)));
    }

    fn picker_prev(&mut self) {
        let len = self.db.len();
        if len == 0 {
            return;
        }
        let sel = self.picker_state.selected().unwrap_or(0);
        self.picker_state.select(Some(sel.saturating_sub(1)));
    }

    fn open_selected_game(&mut self) {
        let Some(idx) = self.picker_state.selected() else {
            return;
        };
        if let Ok(game) = self.db.load_game(idx) {
            self.current_game = Some(game);
            self.current_ply = 0;
        }
        self.screen = AppScreen::Main;
    }

    fn next_move(&mut self) {
        let Some(game) = &self.current_game else {
            return;
        };
        let max = game.positions.len().saturating_sub(1);
        if self.current_ply < max {
            self.current_ply += 1;
        }
    }

    fn prev_move(&mut self) {
        if self.current_ply > 0 {
            self.current_ply -= 1;
        }
    }

    fn go_start(&mut self) {
        self.current_ply = 0;
    }

    fn go_end(&mut self) {
        if let Some(game) = &self.current_game {
            self.current_ply = game.positions.len().saturating_sub(1);
        }
    }

    fn move_square_cursor(&mut self, df: i32, dr: i32) {
        let file = self.selected_square.file().offset(df);
        let rank = self.selected_square.rank().offset(dr);

        if let (Some(file), Some(rank)) = (file, rank) {
            self.selected_square = Square::from_coords(file, rank);
        }
    }

    pub fn engine_menu_len(&self) -> usize {
        2 + self.config.engines.len()
    }

    fn engine_menu_next(&mut self) {
        if self.config.engines.is_empty() {
            self.engine_menu_state.select(Some(0));
            return;
        }
        let len = self.engine_menu_len();
        let sel = self.engine_menu_state.selected().unwrap_or(0);
        let next = match sel {
            0 | 1 => 2,
            _ => (sel + 1).min(len - 1),
        };
        self.engine_menu_state.select(Some(next));
    }

    fn engine_menu_prev(&mut self) {
        if self.config.engines.is_empty() {
            self.engine_menu_state.select(Some(0));
            return;
        }
        let sel = self.engine_menu_state.selected().unwrap_or(0);
        let prev = match sel {
            0 | 1 | 2 => 0,
            _ => sel - 1,
        };
        self.engine_menu_state.select(Some(prev));
    }

    fn open_selected_engine_menu_item(&mut self) {
        let sel = self.engine_menu_state.selected().unwrap_or(0);
        if sel == 0 {
            self.engine_editor = Some(EngineEditorState {
                index: None,
                draft: EngineConfig {
                    name: "New Engine".to_string(),
                    command: String::new(),
                    args: String::new(),
                },
                focus: EngineEditorFocus::Name,
            });
        } else if sel == 1 {
            return;
        } else if let Some(engine) = self.config.engines.get(sel - 2).cloned() {
            self.engine_editor = Some(EngineEditorState {
                index: Some(sel - 2),
                draft: engine,
                focus: EngineEditorFocus::Name,
            });
        }
        self.screen = AppScreen::EngineEditor;
    }

    fn engine_editor_next_focus(&mut self) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        editor.focus = match editor.focus {
            EngineEditorFocus::Name => EngineEditorFocus::Command,
            EngineEditorFocus::Command => EngineEditorFocus::Args,
            EngineEditorFocus::Args => EngineEditorFocus::Button(EngineEditorButton::Delete),
            EngineEditorFocus::Button(EngineEditorButton::Delete) => {
                EngineEditorFocus::Button(EngineEditorButton::Cancel)
            }
            EngineEditorFocus::Button(EngineEditorButton::Cancel) => {
                EngineEditorFocus::Button(EngineEditorButton::Save)
            }
            EngineEditorFocus::Button(EngineEditorButton::Save) => {
                EngineEditorFocus::Button(EngineEditorButton::SaveAndUse)
            }
            EngineEditorFocus::Button(EngineEditorButton::SaveAndUse) => EngineEditorFocus::Name,
        };
    }

    fn engine_editor_prev_focus(&mut self) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        editor.focus = match editor.focus {
            EngineEditorFocus::Name => EngineEditorFocus::Button(EngineEditorButton::SaveAndUse),
            EngineEditorFocus::Command => EngineEditorFocus::Name,
            EngineEditorFocus::Args => EngineEditorFocus::Command,
            EngineEditorFocus::Button(EngineEditorButton::Delete) => EngineEditorFocus::Args,
            EngineEditorFocus::Button(EngineEditorButton::Cancel) => {
                EngineEditorFocus::Button(EngineEditorButton::Delete)
            }
            EngineEditorFocus::Button(EngineEditorButton::Save) => {
                EngineEditorFocus::Button(EngineEditorButton::Cancel)
            }
            EngineEditorFocus::Button(EngineEditorButton::SaveAndUse) => {
                EngineEditorFocus::Button(EngineEditorButton::Save)
            }
        };
    }

    fn engine_editor_prev_button(&mut self) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        if matches!(editor.focus, EngineEditorFocus::Button(_)) {
            self.engine_editor_prev_focus();
        }
    }

    fn engine_editor_next_button(&mut self) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        if matches!(editor.focus, EngineEditorFocus::Button(_)) {
            self.engine_editor_next_focus();
        }
    }

    fn engine_editor_char(&mut self, ch: char) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        match editor.focus {
            EngineEditorFocus::Name => editor.draft.name.push(ch),
            EngineEditorFocus::Command => editor.draft.command.push(ch),
            EngineEditorFocus::Args => editor.draft.args.push(ch),
            EngineEditorFocus::Button(_) => {}
        }
    }

    fn engine_editor_backspace(&mut self) {
        let Some(editor) = self.engine_editor.as_mut() else {
            return;
        };
        match editor.focus {
            EngineEditorFocus::Name => {
                editor.draft.name.pop();
            }
            EngineEditorFocus::Command => {
                editor.draft.command.pop();
            }
            EngineEditorFocus::Args => {
                editor.draft.args.pop();
            }
            EngineEditorFocus::Button(_) => {}
        }
    }

    fn engine_editor_activate(&mut self) {
        let Some(editor) = self.engine_editor.clone() else {
            return;
        };
        match editor.focus {
            EngineEditorFocus::Button(EngineEditorButton::Delete) => {
                if let Some(index) = editor.index {
                    self.config.engines.remove(index);
                    if self.config.active_engine == Some(index) {
                        self.config.active_engine = None;
                    } else if let Some(active) = self.config.active_engine {
                        if active > index {
                            self.config.active_engine = Some(active - 1);
                        }
                    }
                    self.save_config();
                }
                self.engine_editor = None;
                self.screen = AppScreen::EngineMenu;
            }
            EngineEditorFocus::Button(EngineEditorButton::Cancel) => {
                self.engine_editor = None;
                self.screen = AppScreen::EngineMenu;
            }
            EngineEditorFocus::Button(EngineEditorButton::Save) => {
                self.save_engine(editor, false);
            }
            EngineEditorFocus::Button(EngineEditorButton::SaveAndUse) => {
                self.save_engine(editor, true);
            }
            _ => self.engine_editor_next_focus(),
        }
    }

    fn save_engine(&mut self, editor: EngineEditorState, use_engine: bool) {
        let index = match editor.index {
            Some(index) => {
                self.config.engines[index] = editor.draft;
                index
            }
            None => {
                self.config.engines.push(editor.draft);
                self.config.engines.len() - 1
            }
        };
        if use_engine {
            self.config.active_engine = Some(index);
        }
        self.save_config();
        self.engine_menu_state.select(Some(index + 2));
        self.engine_editor = None;
        self.screen = AppScreen::EngineMenu;
    }

    fn save_config(&mut self) {
        match self.config.save() {
            Ok(()) => self.engine_lines = engine_status_lines(&self.config),
            Err(err) => self.engine_lines = vec![format!("Config save failed: {err}")],
        }
    }
}

fn engine_status_lines(config: &UserConfig) -> Vec<String> {
    match config.active_engine.and_then(|idx| config.engines.get(idx)) {
        Some(engine) => vec![format!(
            "Active engine: {} ({})",
            engine.name, engine.command
        )],
        None => vec!["No active engine. Press E to manage engines.".into()],
    }
}
