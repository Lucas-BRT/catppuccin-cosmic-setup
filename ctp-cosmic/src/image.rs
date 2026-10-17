//! Rendering a style to RGB bytes (rows in parallel) and writing PNGs.

use std::fs::File;
use std::io::BufWriter;
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

pub fn write_png(path: &Path, w: usize, h: usize, rgb: &[u8]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Balanced);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgb).map_err(|e| e.to_string())
}
