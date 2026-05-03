//! SIXEL display for terminals that support bitmap graphics through VTE/xterm.
//!
//! GNOME Terminal on Ubuntu exposes VTE environment variables, so this backend
//! gives it a PNG board path without depending on the Kitty graphics protocol.

use std::collections::HashMap;
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::ExecutableCommand;

pub fn display(png_data: &[u8], col: u16, row: u16) -> io::Result<()> {
    let img = image::load_from_memory_with_format(png_data, image::ImageFormat::Png)
        .map_err(io::Error::other)?
        .into_rgba8();
    let sixel = encode_sixel(&img);

    let mut out = io::stdout().lock();
    out.execute(MoveTo(col, row))?;
    out.write_all(sixel.as_bytes())?;
    out.flush()
}

fn encode_sixel(img: &image::RgbaImage) -> String {
    let (width, height) = img.dimensions();
    let (palette, indexed) = indexed_pixels(img);
    let mut out = String::new();

    out.push_str("\x1bPq");
    for (idx, &(r, g, b)) in palette.iter().enumerate() {
        out.push_str(&format!(
            "#{idx};2;{};{};{}",
            to_sixel_rgb(r),
            to_sixel_rgb(g),
            to_sixel_rgb(b)
        ));
    }

    for band_y in (0..height).step_by(6) {
        for color_idx in 0..palette.len() {
            out.push_str(&format!("#{color_idx}"));
            let mut last = None;
            let mut run = 0usize;

            for x in 0..width {
                let mut bits = 0u8;
                for bit in 0..6 {
                    let y = band_y + bit;
                    if y >= height {
                        continue;
                    }

                    let pos = (y * width + x) as usize;
                    if indexed[pos] == color_idx as u8 {
                        bits |= 1 << bit;
                    }
                }

                let ch = (63 + bits) as char;
                if Some(ch) == last {
                    run += 1;
                } else {
                    flush_run(&mut out, last, run);
                    last = Some(ch);
                    run = 1;
                }
            }

            flush_run(&mut out, last, run);
            out.push('$');
        }
        out.push('-');
    }

    out.push_str("\x1b\\");
    out
}

fn indexed_pixels(img: &image::RgbaImage) -> (Vec<(u8, u8, u8)>, Vec<u8>) {
    let mut palette = Vec::new();
    let mut lookup = HashMap::new();
    let mut indexed = Vec::with_capacity((img.width() * img.height()) as usize);

    for pixel in img.pixels() {
        let rgb = (pixel[0], pixel[1], pixel[2]);
        let idx = match lookup.get(&rgb) {
            Some(&idx) => idx,
            None => {
                let idx = palette.len().min(255) as u8;
                if palette.len() < 256 {
                    palette.push(rgb);
                    lookup.insert(rgb, idx);
                    idx
                } else {
                    nearest_palette_color(rgb, &palette) as u8
                }
            }
        };
        indexed.push(idx);
    }

    (palette, indexed)
}

fn nearest_palette_color(rgb: (u8, u8, u8), palette: &[(u8, u8, u8)]) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, &(r, g, b))| {
            let dr = rgb.0 as i32 - r as i32;
            let dg = rgb.1 as i32 - g as i32;
            let db = rgb.2 as i32 - b as i32;
            dr * dr + dg * dg + db * db
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn flush_run(out: &mut String, ch: Option<char>, run: usize) {
    let Some(ch) = ch else {
        return;
    };

    if run > 3 {
        out.push_str(&format!("!{run}{ch}"));
    } else {
        for _ in 0..run {
            out.push(ch);
        }
    }
}

fn to_sixel_rgb(value: u8) -> u8 {
    ((u16::from(value) * 100 + 127) / 255) as u8
}
