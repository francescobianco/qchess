use std::{fs::File, path::PathBuf};

use anyhow::{Context, Result};
use pgn_reader::{BufferedReader, RawTag, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Position};

// ─── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct GameMetadata {
    pub white: String,
    pub black: String,
    pub result: String,
    pub event: Option<String>,
    pub date: Option<String>,
    pub eco: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameRef {
    pub file: PathBuf,
    pub game_number: usize,
    pub meta: GameMetadata,
}

pub struct LoadedGame {
    pub game_ref: GameRef,
    /// Positions: initial position + position after each move.
    /// `positions[i]` is the board state BEFORE move `i`.
    pub positions: Vec<Chess>,
    pub sans: Vec<SanPlus>,
}

// ─── MetaVisitor ─────────────────────────────────────────────────────────────

/// Reads only PGN headers to produce `GameMetadata`.
#[derive(Default)]
pub struct MetaVisitor {
    meta: GameMetadata,
}

impl Visitor for MetaVisitor {
    type Result = GameMetadata;

    fn begin_game(&mut self) {
        self.meta = GameMetadata::default();
    }

    fn tag(&mut self, name: &[u8], value: RawTag<'_>) {
        let val = String::from_utf8_lossy(value.as_bytes()).into_owned();
        match name {
            b"White" => self.meta.white = val,
            b"Black" => self.meta.black = val,
            b"Result" => self.meta.result = val,
            b"Event" => self.meta.event = Some(val),
            b"Date" => self.meta.date = Some(val),
            b"ECO" => self.meta.eco = Some(val),
            _ => {}
        }
    }

    fn end_tags(&mut self) -> Skip {
        // Skip move text — we only need headers.
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        self.meta.clone()
    }
}

// ─── FullGameLoader ───────────────────────────────────────────────────────────

/// Reads headers + SAN moves and replays them through shakmaty.
pub struct FullGameLoader {
    meta: GameMetadata,
    positions: Vec<Chess>,
    sans: Vec<SanPlus>,
    current_pos: Chess,
    error: bool,
}

impl Default for FullGameLoader {
    fn default() -> Self {
        Self {
            meta: GameMetadata::default(),
            positions: Vec::new(),
            sans: Vec::new(),
            current_pos: Chess::default(),
            error: false,
        }
    }
}

impl Visitor for FullGameLoader {
    type Result = Option<(GameMetadata, Vec<Chess>, Vec<SanPlus>)>;

    fn begin_game(&mut self) {
        self.meta = GameMetadata::default();
        self.positions = vec![Chess::default()];
        self.sans = Vec::new();
        self.current_pos = Chess::default();
        self.error = false;
    }

    fn tag(&mut self, name: &[u8], value: RawTag<'_>) {
        let val = String::from_utf8_lossy(value.as_bytes()).into_owned();
        match name {
            b"White" => self.meta.white = val,
            b"Black" => self.meta.black = val,
            b"Result" => self.meta.result = val,
            b"Event" => self.meta.event = Some(val),
            b"Date" => self.meta.date = Some(val),
            b"ECO" => self.meta.eco = Some(val),
            _ => {}
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.error {
            return;
        }
        match san_plus.san.to_move(&self.current_pos) {
            Ok(m) => match self.current_pos.clone().play(m) {
                Ok(new_pos) => {
                    self.sans.push(san_plus);
                    self.current_pos = new_pos.clone();
                    self.positions.push(new_pos);
                }
                Err(_) => {
                    self.error = true;
                }
            },
            Err(_) => {
                self.error = true;
            }
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // Skip variations
    }

    fn end_game(&mut self) -> Self::Result {
        if self.error {
            None
        } else {
            Some((
                self.meta.clone(),
                self.positions.clone(),
                self.sans.clone(),
            ))
        }
    }
}

// ─── Public helpers ───────────────────────────────────────────────────────────

/// Scan a PGN file and return one `GameRef` per game found.
pub fn scan_games_in_file(path: &std::path::Path) -> Result<Vec<GameRef>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut reader = BufferedReader::new(file);
    let mut visitor = MetaVisitor::default();
    let mut refs = Vec::new();
    let mut game_number = 0usize;

    while let Some(meta) = reader
        .read_game(&mut visitor)
        .with_context(|| format!("Error reading game #{} in {}", game_number, path.display()))?
    {
        refs.push(GameRef {
            file: path.to_path_buf(),
            game_number,
            meta,
        });
        game_number += 1;
    }

    Ok(refs)
}

/// Load a specific game (by its 0-based index within the file) fully.
pub fn load_game(game_ref: &GameRef) -> Result<LoadedGame> {
    let file = File::open(&game_ref.file)
        .with_context(|| format!("Failed to open {}", game_ref.file.display()))?;
    let mut reader = BufferedReader::new(file);
    let mut visitor = FullGameLoader::default();

    // Skip games before the target index.
    for i in 0..game_ref.game_number {
        let skipped = reader
            .skip_game::<MetaVisitor>()
            .with_context(|| format!("Error skipping game #{}", i))?;
        if !skipped {
            anyhow::bail!(
                "Unexpected EOF: game #{} not found in {}",
                game_ref.game_number,
                game_ref.file.display()
            );
        }
    }

    match reader.read_game(&mut visitor)? {
        Some(Some((meta, positions, sans))) => Ok(LoadedGame {
            game_ref: GameRef {
                file: game_ref.file.clone(),
                game_number: game_ref.game_number,
                meta,
            },
            positions,
            sans,
        }),
        Some(None) => anyhow::bail!(
            "Failed to parse moves in game #{} of {}",
            game_ref.game_number,
            game_ref.file.display()
        ),
        None => anyhow::bail!(
            "Game #{} not found in {}",
            game_ref.game_number,
            game_ref.file.display()
        ),
    }
}