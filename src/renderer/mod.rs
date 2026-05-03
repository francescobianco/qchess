// Geometry shared by the TUI layout and bitmap board placement.
// The board itself is bitmap-only; these dimensions describe the reserved
// terminal-cell area around it.

/// Square width in terminal cells.
pub const SQ_W: u16 = 4;
/// Square height in terminal rows.
pub const SQ_H: u16 = 2;
/// Rank label column width, directly adjacent to the board.
pub const LABEL_W: u16 = 1;
/// Black separator column at the right edge before the move list.
pub const SEP_W: u16 = 1;
/// Full board widget width: rank labels + 8 files + separator.
pub const BOARD_COLS: u16 = LABEL_W + SQ_W * 8 + SEP_W;
/// Full board widget height: 8 ranks + file label row.
pub const BOARD_ROWS: u16 = SQ_H * 8 + 1;
