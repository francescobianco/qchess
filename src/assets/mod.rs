//! Asset system: abstract traits + Fritz built-in implementation.
//!
//! To create a custom theme, implement `BoardStyle` and `PieceSet`
//! and pass them to `PngBoardRenderer`.

use image::RgbaImage;
use shakmaty::{Color, Role};

// ── Traits ────────────────────────────────────────────────────────────────────

/// Background tiles for the chessboard.
pub trait BoardStyle: Send + Sync {
    /// Width (= height) of a single square tile in pixels.
    fn square_size(&self) -> u32;
    /// Tile image for a light square (e.g. a1 in the standard orientation is dark,
    /// b1 is light).
    fn light_square(&self) -> &RgbaImage;
    /// Tile image for a dark square.
    fn dark_square(&self) -> &RgbaImage;
}

/// Piece images with alpha transparency.
pub trait PieceSet: Send + Sync {
    /// Returns the RGBA image for the given piece.  The image must be exactly
    /// `square_size × square_size` pixels and use alpha to indicate transparency
    /// so the board tile shows through.
    fn piece(&self, color: Color, role: Role) -> &RgbaImage;
}

// ── Fritz built-in ────────────────────────────────────────────────────────────

fn decode(bytes: &[u8]) -> RgbaImage {
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("embedded PNG corrupt")
        .into_rgba8()
}

pub struct FritzBoardStyle {
    light: RgbaImage,
    dark: RgbaImage,
    sq: u32,
}

impl FritzBoardStyle {
    pub fn new() -> Self {
        let light = decode(include_bytes!("../../assets/fritz/board/light.png"));
        let dark = decode(include_bytes!("../../assets/fritz/board/dark.png"));
        let sq = light.width();
        FritzBoardStyle { light, dark, sq }
    }
}

impl Default for FritzBoardStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardStyle for FritzBoardStyle {
    fn square_size(&self) -> u32 {
        self.sq
    }
    fn light_square(&self) -> &RgbaImage {
        &self.light
    }
    fn dark_square(&self) -> &RgbaImage {
        &self.dark
    }
}

// Role → index (King=0, Queen=1, Rook=2, Bishop=3, Knight=4, Pawn=5)
fn role_idx(role: Role) -> usize {
    match role {
        Role::King => 0,
        Role::Queen => 1,
        Role::Rook => 2,
        Role::Bishop => 3,
        Role::Knight => 4,
        Role::Pawn => 5,
    }
}

pub struct FritzPieceSet {
    white: [RgbaImage; 6],
    black: [RgbaImage; 6],
}

impl FritzPieceSet {
    pub fn new() -> Self {
        FritzPieceSet {
            white: [
                decode(include_bytes!("../../assets/fritz/pieces/wK.png")),
                decode(include_bytes!("../../assets/fritz/pieces/wQ.png")),
                decode(include_bytes!("../../assets/fritz/pieces/wR.png")),
                decode(include_bytes!("../../assets/fritz/pieces/wB.png")),
                decode(include_bytes!("../../assets/fritz/pieces/wN.png")),
                decode(include_bytes!("../../assets/fritz/pieces/wP.png")),
            ],
            black: [
                decode(include_bytes!("../../assets/fritz/pieces/bK.png")),
                decode(include_bytes!("../../assets/fritz/pieces/bQ.png")),
                decode(include_bytes!("../../assets/fritz/pieces/bR.png")),
                decode(include_bytes!("../../assets/fritz/pieces/bB.png")),
                decode(include_bytes!("../../assets/fritz/pieces/bN.png")),
                decode(include_bytes!("../../assets/fritz/pieces/bP.png")),
            ],
        }
    }
}

impl Default for FritzPieceSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceSet for FritzPieceSet {
    fn piece(&self, color: Color, role: Role) -> &RgbaImage {
        let idx = role_idx(role);
        match color {
            Color::White => &self.white[idx],
            Color::Black => &self.black[idx],
        }
    }
}
