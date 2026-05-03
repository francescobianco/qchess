//! Kitty graphics protocol display for the board PNG.
//!
//! Writes an APC escape sequence directly to stdout, overlaid on the ratatui
//! TUI.  The image covers exactly the terminal cells occupied by the board
//! panel.
//!
//! Reference: https://sw.kovidgoyal.net/kitty/graphics-protocol/

use std::io::{self, Write};

use base64::Engine;
use crossterm::cursor::MoveTo;
use crossterm::ExecutableCommand;

/// Maximum bytes of base64 data per APC chunk (Kitty recommends ≤ 4 096).
const CHUNK: usize = 4096;

/// Display `png_data` at terminal position (`col`, `row`) spanning
/// `cols` × `rows` terminal cells.
///
/// Returns `Ok(())` on success.  If the terminal does not support Kitty
/// graphics, the APC sequence is simply ignored.
pub fn display(
    png_data: &[u8],
    col: u16,
    row: u16,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let mut out = io::stdout().lock();

    // Position cursor at the top-left of the board area.
    out.execute(MoveTo(col, row))?;

    // Base64-encode the entire PNG.
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_data);
    let bytes = b64.as_bytes();

    let total_chunks = (bytes.len() + CHUNK - 1).max(1) / CHUNK;

    for (i, chunk) in bytes.chunks(CHUNK).enumerate() {
        let is_last = i + 1 == total_chunks;
        let m = if is_last { 0u8 } else { 1u8 };
        let data = std::str::from_utf8(chunk)
            .expect("base64 is always ASCII");

        if i == 0 {
            // First chunk: include all parameters.
            // a=T  → action: transmit + display
            // f=100 → format: PNG
            // t=d  → transmission: direct (inline)
            // c,r  → display size in terminal columns/rows
            // q=2  → suppress all OK/error responses
            // m    → more chunks? 1=yes, 0=final
            write!(
                out,
                "\x1b_Ga=T,f=100,t=d,c={cols},r={rows},q=2,m={m};{data}\x1b\\",
            )?;
        } else {
            write!(out, "\x1b_Gm={m};{data}\x1b\\")?;
        }
    }

    out.flush()
}

/// Delete all Kitty images placed at (`col`, `row`) (useful before re-placing).
#[allow(dead_code)]
pub fn clear_at(col: u16, row: u16) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.execute(MoveTo(col, row))?;
    write!(out, "\x1b_Ga=d,d=C\x1b\\")?;
    out.flush()
}
