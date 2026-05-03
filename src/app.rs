use std::path::Path;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::{
    db::FolderDatabase,
    pgn::LoadedGame,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    GameList,
    BoardView,
}

pub struct App {
    pub db: FolderDatabase,
    pub screen: AppScreen,
    pub list_state: ListState,
    pub current_game: Option<LoadedGame>,
    pub current_ply: usize,
    pub running: bool,
}

impl App {
    pub fn new(path: &Path) -> Result<Self> {
        let db = FolderDatabase::load(path)?;
        let mut list_state = ListState::default();
        if !db.is_empty() {
            list_state.select(Some(0));
        }
        Ok(App {
            db,
            screen: AppScreen::GameList,
            list_state,
            current_game: None,
            current_ply: 0,
            running: true,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global quit bindings.
        if key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.running = false;
            return;
        }

        match self.screen {
            AppScreen::GameList => self.handle_key_game_list(key),
            AppScreen::BoardView => self.handle_key_board_view(key),
        }
    }

    fn handle_key_game_list(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.next_game(),
            KeyCode::Up | KeyCode::Char('k') => self.prev_game(),
            KeyCode::Enter => self.open_game(),
            _ => {}
        }
    }

    fn handle_key_board_view(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Right | KeyCode::Char('l') => self.next_move(),
            KeyCode::Left | KeyCode::Char('h') => self.prev_move(),
            KeyCode::Home => self.go_start(),
            KeyCode::End => self.go_end(),
            KeyCode::Esc => {
                self.screen = AppScreen::GameList;
                self.current_game = None;
                self.current_ply = 0;
            }
            _ => {}
        }
    }

    pub fn next_game(&mut self) {
        let len = self.db.len();
        if len == 0 {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let next = (selected + 1).min(len - 1);
        self.list_state.select(Some(next));
    }

    pub fn prev_game(&mut self) {
        let len = self.db.len();
        if len == 0 {
            return;
        }
        let selected = self.list_state.selected().unwrap_or(0);
        let prev = selected.saturating_sub(1);
        self.list_state.select(Some(prev));
    }

    pub fn open_game(&mut self) {
        let Some(idx) = self.list_state.selected() else {
            return;
        };
        match self.db.load_game(idx) {
            Ok(game) => {
                self.current_ply = 0;
                self.current_game = Some(game);
                self.screen = AppScreen::BoardView;
            }
            Err(e) => {
                // In a TUI app we can't easily print to stderr while running,
                // but we stay on the list screen so the user can try again.
                let _ = e;
            }
        }
    }

    pub fn next_move(&mut self) {
        let Some(game) = &self.current_game else { return };
        // positions has len = moves + 1; last valid ply = positions.len() - 1
        let max_ply = game.positions.len().saturating_sub(1);
        if self.current_ply < max_ply {
            self.current_ply += 1;
        }
    }

    pub fn prev_move(&mut self) {
        if self.current_ply > 0 {
            self.current_ply -= 1;
        }
    }

    pub fn go_start(&mut self) {
        self.current_ply = 0;
    }

    pub fn go_end(&mut self) {
        if let Some(game) = &self.current_game {
            let max_ply = game.positions.len().saturating_sub(1);
            self.current_ply = max_ply;
        }
    }
}