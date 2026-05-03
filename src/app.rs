use std::path::Path;

use crate::{db::FolderDatabase, pgn::LoadedGame};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    Main,       // board left, moves right, engine bottom
    GamePicker, // floating overlay with game list
}

pub struct App {
    pub db: FolderDatabase,
    pub screen: AppScreen,
    pub picker_state: ListState,
    pub current_game: Option<LoadedGame>,
    pub current_ply: usize,
    pub running: bool,
    pub engine_lines: Vec<String>, // placeholder for engine output
}

impl App {
    pub fn new(path: &Path) -> Result<Self> {
        let db = FolderDatabase::load(path)?;
        let mut picker_state = ListState::default();
        if !db.is_empty() {
            picker_state.select(Some(0));
        }
        Ok(App {
            db,
            screen: AppScreen::Main,
            picker_state,
            current_game: None,
            current_ply: 0,
            running: true,
            engine_lines: vec!["No engine loaded. [Future: UCI integration]".into()],
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.running = false;
            return;
        }

        match self.screen {
            AppScreen::Main => self.handle_key_main(key),
            AppScreen::GamePicker => self.handle_key_picker(key),
        }
    }

    fn handle_key_main(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::F(1) | KeyCode::Char('b') => {
                self.screen = AppScreen::GamePicker;
            }
            KeyCode::Right | KeyCode::Char('l') => self.next_move(),
            KeyCode::Left | KeyCode::Char('h') => self.prev_move(),
            KeyCode::Home | KeyCode::Char('s') => self.go_start(),
            KeyCode::End | KeyCode::Char('e') => self.go_end(),
            _ => {}
        }
    }

    fn handle_key_picker(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('b') => {
                self.screen = AppScreen::Main;
            }
            KeyCode::Down | KeyCode::Char('j') => self.picker_next(),
            KeyCode::Up | KeyCode::Char('k') => self.picker_prev(),
            KeyCode::Enter => self.open_selected_game(),
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
}
