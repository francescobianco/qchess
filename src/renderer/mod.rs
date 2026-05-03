use shakmaty::{Chess, Color, File, Position, Rank, Role, Square};

// ─── Types ────────────────────────────────────────────────────────────────────

pub struct RenderOptions {
    pub flipped: bool,
    pub last_move: Option<shakmaty::Move>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            flipped: false,
            last_move: None,
        }
    }
}

pub enum RenderedBoard {
    Unicode(Vec<String>),
}

// ─── Trait ────────────────────────────────────────────────────────────────────

pub trait BoardRenderer {
    fn render(&self, pos: &Chess, options: &RenderOptions) -> RenderedBoard;
}

// ─── Unicode renderer ─────────────────────────────────────────────────────────

pub struct UnicodeRenderer;

fn piece_char(color: Color, role: Role) -> char {
    match (color, role) {
        (Color::White, Role::King) => '♔',
        (Color::White, Role::Queen) => '♕',
        (Color::White, Role::Rook) => '♖',
        (Color::White, Role::Bishop) => '♗',
        (Color::White, Role::Knight) => '♘',
        (Color::White, Role::Pawn) => '♙',
        (Color::Black, Role::King) => '♚',
        (Color::Black, Role::Queen) => '♛',
        (Color::Black, Role::Rook) => '♜',
        (Color::Black, Role::Bishop) => '♝',
        (Color::Black, Role::Knight) => '♞',
        (Color::Black, Role::Pawn) => '♟',
    }
}

impl BoardRenderer for UnicodeRenderer {
    fn render(&self, pos: &Chess, options: &RenderOptions) -> RenderedBoard {
        let board = pos.board();
        let mut lines = Vec::with_capacity(10);

        // Column header: "  a b c d e f g h"
        let file_header = if options.flipped {
            "  h g f e d c b a".to_string()
        } else {
            "  a b c d e f g h".to_string()
        };
        lines.push(file_header);

        // Ranks: draw from rank 8 (top) to rank 1 (bottom), or reversed if flipped.
        let rank_indices: Vec<u32> = if options.flipped {
            (0..8u32).collect()
        } else {
            (0..8u32).rev().collect()
        };

        for &rank_idx in &rank_indices {
            let rank = Rank::new(rank_idx);
            let rank_label = rank_idx + 1; // "1".."8"

            let mut row = format!("{} ", rank_label);

            let file_indices: Vec<u32> = if options.flipped {
                (0..8u32).rev().collect()
            } else {
                (0..8u32).collect()
            };

            for &file_idx in &file_indices {
                let file = File::new(file_idx);
                let sq = Square::from_coords(file, rank);
                let cell = match board.piece_at(sq) {
                    Some(piece) => piece_char(piece.color, piece.role),
                    None => '·',
                };
                row.push(cell);
                row.push(' ');
            }

            // Trim trailing space and add rank label on the right.
            let row = row.trim_end().to_string();
            lines.push(format!("{} {}", row, rank_label));
        }

        // Bottom file header.
        let file_footer = if options.flipped {
            "  h g f e d c b a".to_string()
        } else {
            "  a b c d e f g h".to_string()
        };
        lines.push(file_footer);

        RenderedBoard::Unicode(lines)
    }
}