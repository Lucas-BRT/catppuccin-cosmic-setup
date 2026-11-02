//! Rendering a style to RGB bytes (rows in parallel) and writing PNGs.

use std::fs::File;
use std::io::BufReader;
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

/// Rows per strip. Each strip is compressed on its own core, so bigger strips
/// compress better and smaller ones use more cores.
const STRIP_ROWS: usize = 160;
const IEND: [u8; 12] = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82];

pub fn write_png(path: &Path, w: usize, h: usize, rgb: &[u8]) -> Result<(), String> {
    write_png_tagged(path, w, h, rgb, None)
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(data);
    let mut crc = crc32fast::Hasher::new();
    crc.update(kind);
    crc.update(data);
    out.extend(crc.finalize().to_be_bytes());
}

/// Raw deflate of one strip. Non-final strips end on a sync flush (byte
/// aligned), so the strips join into one valid stream.
fn deflate(data: &[u8], last: bool) -> Vec<u8> {
    use flate2::{Compress, Compression, FlushCompress, Status};
    let mut c = Compress::new(Compression::new(6), false);
    let flush = if last {
        FlushCompress::Finish
    } else {
        FlushCompress::Sync
    };
    let mut out = Vec::with_capacity(data.len() / 4 + 1024);
    loop {
        out.reserve(64 * 1024);
        let done = c.total_in() as usize;
        let status = c
            .compress_vec(&data[done..], &mut out, flush)
            .expect("deflate");
        let all_in = c.total_in() as usize == data.len();
        if status == Status::StreamEnd || (!last && all_in && out.len() < out.capacity()) {
            return out;
        }
    }
}

/// Writes an RGB PNG, compressing strips of rows in parallel. `tag` goes in a
/// text chunk that `has_tag` finds again.
pub fn write_png_tagged(
    path: &Path,
    w: usize,
    h: usize,
    rgb: &[u8],
    tag: Option<&str>,
) -> Result<(), String> {
    let stride = w * 3;
    // Filter type 1 (Sub) needs only the row itself, so strips are independent.
    let filtered: Vec<Vec<u8>> = rgb
        .par_chunks(stride * STRIP_ROWS)
        .map(|rows| {
            let mut out = Vec::with_capacity(rows.len() + STRIP_ROWS);
            for row in rows.chunks(stride) {
                out.push(1);
                out.extend(&row[..3]);
                out.extend((3..stride).map(|i| row[i].wrapping_sub(row[i - 3])));
            }
            out
        })
        .collect();
    let n = filtered.len();
    let packed: Vec<Vec<u8>> = filtered
        .par_iter()
        .enumerate()
        .map(|(i, f)| deflate(f, i + 1 == n))
        .collect();
    let mut adler = adler2::Adler32::new();
    filtered.iter().for_each(|f| adler.write_slice(f));

    let mut zlib = vec![0x78, 0x01];
    packed.iter().for_each(|p| zlib.extend(p));
    zlib.extend(adler.checksum().to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend((w as u32).to_be_bytes());
    ihdr.extend((h as u32).to_be_bytes());
    ihdr.extend([8, 2, 0, 0, 0]);
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut out, b"IHDR", &ihdr);
    if let Some(tag) = tag {
        chunk(&mut out, b"tEXt", format!("ctp-cosmic\0{tag}").as_bytes());
    }
    chunk(&mut out, b"IDAT", &zlib);
    chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out).map_err(|e| format!("{}: {e}", path.display()))
}

/// Whether `path` is a complete PNG that `write_png_tagged` wrote with `tag`.
pub fn has_tag(path: &Path, tag: &str) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let want = format!("ctp-cosmic\0{tag}");
    let mut head = vec![0u8; 33 + 12 + want.len()];
    let mut end = [0u8; 12];
    f.read_exact(&mut head).is_ok()
        && head.windows(want.len()).any(|w| w == want.as_bytes())
        && f.seek(SeekFrom::End(-12)).is_ok()
        && f.read_exact(&mut end).is_ok()
        && end == IEND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_round_trip_and_the_tag_is_found() {
        // Taller than one strip, with a width that is not a multiple of anything.
        let (w, h) = (37, STRIP_ROWS * 2 + 5);
        let rgb: Vec<u8> = (0..w * h * 3).map(|i| (i * 7 + i / 5) as u8).collect();
        let path = std::env::temp_dir().join(format!("ctp-png-{}.png", std::process::id()));
        write_png_tagged(&path, w, h, &rgb, Some("v1 x")).unwrap();
        assert_eq!(read_png(&path).unwrap(), (w, h, rgb));
        assert!(has_tag(&path, "v1 x"));
        assert!(!has_tag(&path, "v2 x"));
        // A cut-off file is not current.
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..bytes.len() - 20]).unwrap();
        assert!(!has_tag(&path, "v1 x"));
        std::fs::remove_file(&path).unwrap();
    }
}
