use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::pgn::{
    parser::{load_game, scan_games_in_file},
    scanner::scan_pgn_files,
    GameRef, LoadedGame,
};

pub struct FolderDatabase {
    pub root: PathBuf,
    pub games: Vec<GameRef>,
}

impl FolderDatabase {
    /// Scan `path` recursively for PGN files and index all games found.
    pub fn load(path: &Path) -> Result<Self> {
        let pgn_files = scan_pgn_files(path);

        let mut games = Vec::new();
        for pgn_path in pgn_files {
            match scan_games_in_file(&pgn_path) {
                Ok(refs) => games.extend(refs),
                Err(e) => {
                    // Log but continue — one bad file shouldn't abort everything.
                    eprintln!("Warning: {}", e);
                }
            }
        }

        // Sort by file path then game number for stable ordering.
        games.sort_by(|a, b| a.file.cmp(&b.file).then(a.game_number.cmp(&b.game_number)));

        Ok(FolderDatabase {
            root: path.to_path_buf(),
            games,
        })
    }

    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    #[allow(dead_code)]
    pub fn game_at(&self, idx: usize) -> Option<&GameRef> {
        self.games.get(idx)
    }

    pub fn load_game(&self, idx: usize) -> Result<LoadedGame> {
        let game_ref = self
            .games
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("Game index {} out of range", idx))?;
        load_game(game_ref)
    }
}
