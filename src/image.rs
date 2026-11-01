//! Rendering a style to RGB bytes (rows in parallel) and writing PNGs.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use rayon::prelude::*;

use crate::noise::dither;
use crate::styles::PixelFn;

/// Render every pixel, in parallel by row, into packed RGB bytes. A fixed
/// dither of about ±2 levels is added so gradients don't show banding.
pub fn render(w: usize, h: usize, pixel: &PixelFn) -> Vec<u8> {
    let mut buf = vec![0u8; w * h * 3];
    buf.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let c = pixel(x, y);
            let n = dither(x, y) * 0.6;
            for i in 0..3 {
                // Truncate toward zero, then clamp, as the original generator did.
                row[x * 3 + i] = ((c[i] + n) as i64).clamp(0, 255) as u8;
            }
        }
    });
    buf
}

/// Reads a PNG (any color type or depth) as packed RGB bytes: (width, height, bytes).
/// Transparency is dropped.
pub fn read_png(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut dec = png::Decoder::new(BufReader::new(file));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().map_err(|e| {
        format!(
            "{}: not a readable PNG ({e}); only PNG is supported",
            path.display()
        )
    })?;
    let mut buf = vec![0u8; reader.output_buffer_size().ok_or("image too large")?];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let (w, h) = (info.width as usize, info.height as usize);
    let px = &buf[..info.buffer_size()];
    let rgb: Vec<u8> = match info.color_type {
        png::ColorType::Rgb => px.to_vec(),
        png::ColorType::Rgba => px
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect(),
        png::ColorType::Grayscale => px.iter().flat_map(|&g| [g, g, g]).collect(),
        png::ColorType::GrayscaleAlpha => px
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0]])
            .collect(),
        png::ColorType::Indexed => return Err("indexed PNG was not expanded".into()),
    };
    Ok((w, h, rgb))
}

pub fn write_png(path: &Path, w: usize, h: usize, rgb: &[u8]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Balanced);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgb).map_err(|e| e.to_string())
}
