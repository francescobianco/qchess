//! PNG board compositor: composites square tiles + piece images into a full
//! board PNG ready for Kitty graphics protocol display.

use image::{Rgba, RgbaImage};
use shakmaty::{Chess, File, Position, Rank, Square};

use crate::assets::{BoardStyle, PieceSet};

pub struct RenderOptions {
    pub flipped: bool,
    /// Square to highlight as "from" (last move source).
    pub hl_from: Option<Square>,
    /// Square to highlight as "to" (last move destination).
    pub hl_to: Option<Square>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions { flipped: false, hl_from: None, hl_to: None }
    }
}

/// Highlight overlay colour (semi-transparent yellow).
const HL: Rgba<u8> = Rgba([255, 220, 0, 100]);

pub struct PngBoardRenderer<'a> {
    pub style: &'a dyn BoardStyle,
    pub pieces: &'a dyn PieceSet,
}

impl PngBoardRenderer<'_> {
    pub fn render(&self, pos: &Chess, opts: &RenderOptions) -> Vec<u8> {
        let sq_px = self.style.square_size();
        let board_px = sq_px * 8;
        let mut canvas = RgbaImage::new(board_px, board_px);

        for rank_from_bottom in 0..8u32 {
            let rank = Rank::new(rank_from_bottom);
            // Top of image = rank 8 (unless flipped)
            let pixel_row = if opts.flipped {
                rank_from_bottom * sq_px
            } else {
                (7 - rank_from_bottom) * sq_px
            };

            for file_from_left in 0..8u32 {
                let file = File::new(file_from_left);
                let pixel_col = if opts.flipped {
                    (7 - file_from_left) * sq_px
                } else {
                    file_from_left * sq_px
                };

                let sq = Square::from_coords(file, rank);
                let is_dark = (file_from_left + rank_from_bottom) % 2 == 0; // a1 = dark

                // ── 1. Draw square background ──────────────────────────────────
                let tile = if is_dark { self.style.dark_square() } else { self.style.light_square() };
                blit(&mut canvas, tile, pixel_col, pixel_row);

                // ── 2. Highlight overlay ───────────────────────────────────────
                if opts.hl_from == Some(sq) || opts.hl_to == Some(sq) {
                    alpha_fill(&mut canvas, pixel_col, pixel_row, sq_px, HL);
                }

                // ── 3. Draw piece ──────────────────────────────────────────────
                if let Some(piece) = pos.board().piece_at(sq) {
                    let piece_img = self.pieces.piece(piece.color, piece.role);
                    blit_alpha(&mut canvas, piece_img, pixel_col, pixel_row);
                }
            }
        }

        encode_png(canvas)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Copy `src` onto `dst` starting at (`dx`, `dy`) — no alpha blending.
fn blit(dst: &mut RgbaImage, src: &RgbaImage, dx: u32, dy: u32) {
    let (sw, sh) = src.dimensions();
    let (dw, dh) = dst.dimensions();
    for y in 0..sh {
        if dy + y >= dh { break; }
        for x in 0..sw {
            if dx + x >= dw { break; }
            *dst.get_pixel_mut(dx + x, dy + y) = *src.get_pixel(x, y);
        }
    }
}

/// Alpha-blend `src` onto `dst` starting at (`dx`, `dy`).
fn blit_alpha(dst: &mut RgbaImage, src: &RgbaImage, dx: u32, dy: u32) {
    let (sw, sh) = src.dimensions();
    let (dw, dh) = dst.dimensions();
    for y in 0..sh {
        if dy + y >= dh { break; }
        for x in 0..sw {
            if dx + x >= dw { break; }
            let sp = src.get_pixel(x, y);
            let a = sp[3] as f32 / 255.0;
            if a < 0.004 { continue; }
            let dp = dst.get_pixel_mut(dx + x, dy + y);
            let inv = 1.0 - a;
            dp[0] = (sp[0] as f32 * a + dp[0] as f32 * inv) as u8;
            dp[1] = (sp[1] as f32 * a + dp[1] as f32 * inv) as u8;
            dp[2] = (sp[2] as f32 * a + dp[2] as f32 * inv) as u8;
            dp[3] = 255;
        }
    }
}

/// Fill a `sq_px × sq_px` region at (`dx`, `dy`) with colour `col` using alpha.
fn alpha_fill(dst: &mut RgbaImage, dx: u32, dy: u32, sq_px: u32, col: Rgba<u8>) {
    let a = col[3] as f32 / 255.0;
    let inv = 1.0 - a;
    let (dw, dh) = dst.dimensions();
    for y in 0..sq_px {
        if dy + y >= dh { break; }
        for x in 0..sq_px {
            if dx + x >= dw { break; }
            let dp = dst.get_pixel_mut(dx + x, dy + y);
            dp[0] = (col[0] as f32 * a + dp[0] as f32 * inv) as u8;
            dp[1] = (col[1] as f32 * a + dp[1] as f32 * inv) as u8;
            dp[2] = (col[2] as f32 * a + dp[2] as f32 * inv) as u8;
            dp[3] = 255;
        }
    }
}

fn encode_png(img: RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("PNG encoding error");
    buf
}
