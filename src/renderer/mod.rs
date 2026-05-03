use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
};
use shakmaty::{Chess, Color as ChessColor, File, Position, Rank, Role, Square};

// Each square: 4 chars wide × 2 rows tall (~1:1 aspect at typical font metrics)
pub const SQ_W: u16 = 4;
pub const SQ_H: u16 = 2;
pub const LABEL_W: u16 = 2; // rank label column ("8 ")
pub const BOARD_COLS: u16 = LABEL_W + SQ_W * 8; // 34
pub const BOARD_ROWS: u16 = SQ_H * 8 + 1;       // 17 (16 for squares + 1 for file labels)

// Fritz-inspired board palette
const LIGHT_BG: Color    = Color::White;
const LIGHT_FG: Color    = Color::Black;
const DARK_BG: Color     = Color::White;      // dark squares: white bg with ░ overlay
const DARK_HATCH: Color  = Color::DarkGray;   // ░ foreground colour on dark squares
const DARK_PIECE_FG: Color = Color::Black;
const HL_BG: Color       = Color::Yellow;     // last-move highlight
const HL_HATCH: Color    = Color::DarkGray;
const HL_PIECE_FG: Color = Color::Black;
const SURROUND: Color    = Color::Blue;       // QBasic blue around the board

pub struct RenderOptions {
    pub flipped: bool,
    pub last_move: Option<shakmaty::Move>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions { flipped: false, last_move: None }
    }
}

pub struct BoardWidget<'a> {
    pub pos: &'a Chess,
    pub options: RenderOptions,
}

fn piece_char(color: ChessColor, role: Role) -> char {
    match (color, role) {
        (ChessColor::White, Role::King)   => '\u{2654}',
        (ChessColor::White, Role::Queen)  => '\u{2655}',
        (ChessColor::White, Role::Rook)   => '\u{2656}',
        (ChessColor::White, Role::Bishop) => '\u{2657}',
        (ChessColor::White, Role::Knight) => '\u{2658}',
        (ChessColor::White, Role::Pawn)   => '\u{2659}',
        (ChessColor::Black, Role::King)   => '\u{265A}',
        (ChessColor::Black, Role::Queen)  => '\u{265B}',
        (ChessColor::Black, Role::Rook)   => '\u{265C}',
        (ChessColor::Black, Role::Bishop) => '\u{265D}',
        (ChessColor::Black, Role::Knight) => '\u{265E}',
        (ChessColor::Black, Role::Pawn)   => '\u{265F}',
    }
}

// a1 is a dark square: (file=0) + (rank=0) = 0 → even → dark
fn sq_is_dark(file_idx: u32, rank_idx: u32) -> bool {
    (file_idx + rank_idx) % 2 == 0
}

/// Set one terminal cell using the ratatui Buffer index API.
#[inline]
fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: char, fg: Color, bg: Color) {
    let cell = &mut buf[(x, y)];
    cell.set_char(ch);
    cell.set_fg(fg);
    cell.set_bg(bg);
}

impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let board = self.pos.board();
        let flipped = self.options.flipped;

        let hl_from: Option<Square> = self.options.last_move.as_ref().and_then(|m| m.from());
        let hl_to:   Option<Square> = self.options.last_move.as_ref().map(|m| m.to());

        let rank_order: Vec<u32> = if flipped { (0..8).collect() } else { (0..8).rev().collect() };
        let file_order: Vec<u32> = if flipped { (0..8).rev().collect() } else { (0..8).collect() };

        for (ri, &rank_idx) in rank_order.iter().enumerate() {
            let rank  = Rank::new(rank_idx);
            let sq_top = area.y + ri as u16 * SQ_H;
            let sq_bot = sq_top + 1;

            // ── Rank label (left 2 cells of each rank) ────────────────────────
            for dy in 0..SQ_H {
                let y = sq_top + dy;
                if y >= area.bottom() { continue; }
                for lx in 0..LABEL_W {
                    let x = area.x + lx;
                    if x >= area.right() { continue; }
                    let ch = if dy == 1 && lx == 0 {
                        char::from_digit(rank_idx + 1, 10).unwrap_or(' ')
                    } else {
                        ' '
                    };
                    set_cell(buf, x, y, ch, Color::White, SURROUND);
                }
            }

            // ── Squares ────────────────────────────────────────────────────────
            for (fi, &file_idx) in file_order.iter().enumerate() {
                let file  = File::new(file_idx);
                let sq    = Square::from_coords(file, rank);
                let piece = board.piece_at(sq);

                let is_hl   = Some(sq) == hl_from || Some(sq) == hl_to;
                let is_dark = sq_is_dark(file_idx, rank_idx);

                let x0 = area.x + LABEL_W + fi as u16 * SQ_W;

                // Top row: hatch fill or blank
                if sq_top < area.bottom() {
                    for dx in 0..SQ_W {
                        let x = x0 + dx;
                        if x >= area.right() { continue; }
                        let (ch, fg, bg) = if is_hl {
                            ('\u{2591}', HL_HATCH, HL_BG)
                        } else if is_dark {
                            ('\u{2591}', DARK_HATCH, DARK_BG)
                        } else {
                            (' ', LIGHT_FG, LIGHT_BG)
                        };
                        set_cell(buf, x, sq_top, ch, fg, bg);
                    }
                }

                // Bottom row: piece at dx=1, hatch/blank elsewhere
                if sq_bot < area.bottom() {
                    for dx in 0..SQ_W {
                        let x = x0 + dx;
                        if x >= area.right() { continue; }

                        if dx == 1 {
                            match piece {
                                Some(p) => {
                                    let (fg, bg) = if is_hl {
                                        (HL_PIECE_FG, HL_BG)
                                    } else if is_dark {
                                        (DARK_PIECE_FG, DARK_BG)
                                    } else {
                                        (LIGHT_FG, LIGHT_BG)
                                    };
                                    set_cell(buf, x, sq_bot, piece_char(p.color, p.role), fg, bg);
                                }
                                None => {
                                    let (ch, fg, bg) = if is_hl {
                                        ('\u{2591}', HL_HATCH, HL_BG)
                                    } else if is_dark {
                                        ('\u{2591}', DARK_HATCH, DARK_BG)
                                    } else {
                                        (' ', LIGHT_FG, LIGHT_BG)
                                    };
                                    set_cell(buf, x, sq_bot, ch, fg, bg);
                                }
                            }
                        } else {
                            let (ch, fg, bg) = if is_hl {
                                ('\u{2591}', HL_HATCH, HL_BG)
                            } else if is_dark {
                                ('\u{2591}', DARK_HATCH, DARK_BG)
                            } else {
                                (' ', LIGHT_FG, LIGHT_BG)
                            };
                            set_cell(buf, x, sq_bot, ch, fg, bg);
                        }
                    }
                }
            }
        }

        // ── File labels row ────────────────────────────────────────────────────
        let label_y = area.y + SQ_H * 8;
        if label_y < area.bottom() {
            for lx in 0..LABEL_W {
                let x = area.x + lx;
                if x < area.right() {
                    set_cell(buf, x, label_y, ' ', Color::White, SURROUND);
                }
            }

            let file_chars: &[char] = if flipped {
                &['h', 'g', 'f', 'e', 'd', 'c', 'b', 'a']
            } else {
                &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']
            };

            for (fi, &ch) in file_chars.iter().enumerate() {
                let x0 = area.x + LABEL_W + fi as u16 * SQ_W;
                for dx in 0..SQ_W {
                    let x = x0 + dx;
                    if x >= area.right() { continue; }
                    let (label_ch, fg) = if dx == 1 { (ch, Color::White) } else { (' ', SURROUND) };
                    set_cell(buf, x, label_y, label_ch, fg, SURROUND);
                }
            }
        }
    }
}
