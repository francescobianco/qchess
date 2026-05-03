use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use shakmaty::{Chess, Color as ChessColor, File, Position, Rank, Role, Square};

// Each square: 4 chars wide × 2 rows tall (~1:1 aspect at typical font metrics)
pub const SQ_W: u16 = 4;
pub const SQ_H: u16 = 2;
pub const LABEL_W: u16 = 2; // rank label column ("8 ")
pub const BOARD_COLS: u16 = LABEL_W + SQ_W * 8; // 34
pub const BOARD_ROWS: u16 = SQ_H * 8 + 1; // 17

// Fritz-inspired board palette
const LIGHT_BG: Color = Color::White;
const LIGHT_FG: Color = Color::Black;
const DARK_BG: Color = Color::White;
const DARK_HATCH: Color = Color::DarkGray;
const DARK_PIECE_FG: Color = Color::Black;
const HL_BG: Color = Color::Yellow;
const HL_HATCH: Color = Color::DarkGray;
const HL_PIECE_FG: Color = Color::Black;
const SURROUND: Color = Color::Blue;

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

pub struct BoardWidget<'a> {
    pub pos: &'a Chess,
    pub options: RenderOptions,
}

/// Chess symbol + U+FE0E (text-presentation selector) prevents emoji rendering.
fn piece_str(color: ChessColor, role: Role) -> &'static str {
    match (color, role) {
        (ChessColor::White, Role::King) => "\u{2654}\u{FE0E}",
        (ChessColor::White, Role::Queen) => "\u{2655}\u{FE0E}",
        (ChessColor::White, Role::Rook) => "\u{2656}\u{FE0E}",
        (ChessColor::White, Role::Bishop) => "\u{2657}\u{FE0E}",
        (ChessColor::White, Role::Knight) => "\u{2658}\u{FE0E}",
        (ChessColor::White, Role::Pawn) => "\u{2659}\u{FE0E}",
        (ChessColor::Black, Role::King) => "\u{265A}\u{FE0E}",
        (ChessColor::Black, Role::Queen) => "\u{265B}\u{FE0E}",
        (ChessColor::Black, Role::Rook) => "\u{265C}\u{FE0E}",
        (ChessColor::Black, Role::Bishop) => "\u{265D}\u{FE0E}",
        (ChessColor::Black, Role::Knight) => "\u{265E}\u{FE0E}",
        (ChessColor::Black, Role::Pawn) => "\u{265F}\u{FE0E}",
    }
}

fn sq_is_dark(file_idx: u32, rank_idx: u32) -> bool {
    (file_idx + rank_idx) % 2 == 0 // a1 = dark
}

/// Write a symbol (≤ 4 bytes UTF-8) into one terminal cell.
#[inline]
fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, fg: Color, bg: Color) {
    let cell = &mut buf[(x, y)];
    cell.set_symbol(symbol);
    cell.set_fg(fg);
    cell.set_bg(bg);
}

impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let board = self.pos.board();
        let flipped = self.options.flipped;

        let hl_from = self.options.last_move.as_ref().and_then(|m| m.from());
        let hl_to = self.options.last_move.as_ref().map(|m| m.to());

        let rank_order: Vec<u32> = if flipped {
            (0..8).collect()
        } else {
            (0..8).rev().collect()
        };
        let file_order: Vec<u32> = if flipped {
            (0..8).rev().collect()
        } else {
            (0..8).collect()
        };

        for (ri, &rank_idx) in rank_order.iter().enumerate() {
            let rank = Rank::new(rank_idx);
            let sq_top = area.y + ri as u16 * SQ_H;
            let sq_bot = sq_top + 1;

            // ── Rank label ─────────────────────────────────────────────────────
            // Use encode_utf8 for zero-alloc char→&str conversion.
            let digit_char = char::from_digit(rank_idx + 1, 10).unwrap_or(' ');
            let mut digit_buf = [0u8; 4];
            let digit_str = digit_char.encode_utf8(&mut digit_buf);

            for dy in 0..SQ_H {
                let y = sq_top + dy;
                if y >= area.bottom() {
                    continue;
                }
                for lx in 0..LABEL_W {
                    let x = area.x + lx;
                    if x >= area.right() {
                        continue;
                    }
                    let sym = if dy == 1 && lx == 0 {
                        digit_str as &str
                    } else {
                        " "
                    };
                    set_cell(buf, x, y, sym, Color::White, SURROUND);
                }
            }

            // ── Squares ─────────────────────────────────────────────────────────
            for (fi, &file_idx) in file_order.iter().enumerate() {
                let file = File::new(file_idx);
                let sq = Square::from_coords(file, rank);
                let piece = board.piece_at(sq);

                let is_hl = Some(sq) == hl_from || Some(sq) == hl_to;
                let is_dark = sq_is_dark(file_idx, rank_idx);

                let x0 = area.x + LABEL_W + fi as u16 * SQ_W;

                // Top row: hatch or blank
                if sq_top < area.bottom() {
                    for dx in 0..SQ_W {
                        let x = x0 + dx;
                        if x >= area.right() {
                            continue;
                        }
                        let (sym, fg, bg): (&str, _, _) = if is_hl {
                            ("\u{2591}", HL_HATCH, HL_BG)
                        } else if is_dark {
                            ("\u{2591}", DARK_HATCH, DARK_BG)
                        } else {
                            (" ", LIGHT_FG, LIGHT_BG)
                        };
                        set_cell(buf, x, sq_top, sym, fg, bg);
                    }
                }

                // Bottom row: piece at dx=1, fill elsewhere
                if sq_bot < area.bottom() {
                    for dx in 0..SQ_W {
                        let x = x0 + dx;
                        if x >= area.right() {
                            continue;
                        }

                        if dx == 1 {
                            let (sym, fg, bg): (&str, _, _) = match piece {
                                Some(p) => {
                                    let (fg, bg) = if is_hl {
                                        (HL_PIECE_FG, HL_BG)
                                    } else if is_dark {
                                        (DARK_PIECE_FG, DARK_BG)
                                    } else {
                                        (LIGHT_FG, LIGHT_BG)
                                    };
                                    (piece_str(p.color, p.role), fg, bg)
                                }
                                None => {
                                    if is_hl {
                                        ("\u{2591}", HL_HATCH, HL_BG)
                                    } else if is_dark {
                                        ("\u{2591}", DARK_HATCH, DARK_BG)
                                    } else {
                                        (" ", LIGHT_FG, LIGHT_BG)
                                    }
                                }
                            };
                            set_cell(buf, x, sq_bot, sym, fg, bg);
                        } else {
                            let (sym, fg, bg): (&str, _, _) = if is_hl {
                                ("\u{2591}", HL_HATCH, HL_BG)
                            } else if is_dark {
                                ("\u{2591}", DARK_HATCH, DARK_BG)
                            } else {
                                (" ", LIGHT_FG, LIGHT_BG)
                            };
                            set_cell(buf, x, sq_bot, sym, fg, bg);
                        }
                    }
                }
            }
        }

        // ── File labels ────────────────────────────────────────────────────────
        let label_y = area.y + SQ_H * 8;
        if label_y < area.bottom() {
            for lx in 0..LABEL_W {
                let x = area.x + lx;
                if x < area.right() {
                    set_cell(buf, x, label_y, " ", Color::White, SURROUND);
                }
            }

            let file_chars: &[char] = if flipped {
                &['h', 'g', 'f', 'e', 'd', 'c', 'b', 'a']
            } else {
                &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']
            };

            for (fi, &ch) in file_chars.iter().enumerate() {
                let mut fc_buf = [0u8; 4];
                let fc_str = ch.encode_utf8(&mut fc_buf);
                let x0 = area.x + LABEL_W + fi as u16 * SQ_W;
                for dx in 0..SQ_W {
                    let x = x0 + dx;
                    if x >= area.right() {
                        continue;
                    }
                    if dx == 1 {
                        set_cell(buf, x, label_y, fc_str, Color::White, SURROUND);
                    } else {
                        set_cell(buf, x, label_y, " ", SURROUND, SURROUND);
                    }
                }
            }
        }
    }
}
