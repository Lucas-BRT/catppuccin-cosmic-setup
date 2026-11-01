//! `ctp-cosmic wallpaper`: procedural Catppuccin wallpapers for any flavor
//! and accent.
//!
//! Every image is drawn from the official palette only, and the output is
//! deterministic: the same options always produce the same pixels.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::palette::{self, Palette};
use crate::{image, styles};

const USAGE: &str = "\
ctp-cosmic wallpaper: procedural Catppuccin wallpapers

Usage:
  ctp-cosmic wallpaper [OPTIONS] OUTPUT.png   render one wallpaper
  ctp-cosmic wallpaper [OPTIONS] --all DIR    render every style into DIR as
                                              catppuccin-<flavor>-<accent>-<style>.png
  ctp-cosmic wallpaper --preview OUTPUT.png [--accent A] [--thumb WxH]
                                              grid of every style (rows) in every
                                              flavor (columns), for the README
  ctp-cosmic wallpaper recolor IN.png OUT.png [--flavor NAME] [--sigma N]
                                              your own PNG, pulled toward the flavor's
                                              colors (sigma 40; lower snaps harder)
  ctp-cosmic wallpaper --list-styles          style names, one per line
  ctp-cosmic wallpaper --describe-styles      style names with a description

Options:
  --style NAME     glow (default), waves, stripes, dots, triangles, mesh,
                   aurora, sunset, topography, mountains
  --flavor NAME    latte, frappe, macchiato, mocha (default: mocha)
  --accent NAME    any of the 14 accents (default: mauve)
  --size WxH       image size (default: 2560x1440)
  --thumb WxH      thumbnail size for --preview (default: 256x144)
  -h, --help       show this help
";

const DEFAULT_SIZE: (usize, usize) = (2560, 1440);

struct Args {
    output: Option<PathBuf>,
    all: Option<PathBuf>,
    preview: Option<PathBuf>,
    style: String,
    flavor: String,
    accent: String,
    size: (usize, usize),
    thumb: (usize, usize),
}

fn parse_size(s: &str) -> Result<(usize, usize), String> {
    let (w, h) = s
        .to_lowercase()
        .split_once('x')
        .map(|(w, h)| (w.to_owned(), h.to_owned()))
        .ok_or("")?;
    match (w.parse(), h.parse()) {
        (Ok(w), Ok(h)) if w >= 16 && h >= 16 => Ok((w, h)),
        _ => Err(format!(
            "invalid size '{s}' (expected WIDTHxHEIGHT, at least 16x16)"
        )),
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<Args>, String> {
    let mut a = Args {
        output: None,
        all: None,
        preview: None,
        style: "glow".into(),
        flavor: "mocha".into(),
        accent: "mauve".into(),
        size: DEFAULT_SIZE,
        thumb: (256, 144),
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(None);
            }
            "--list-styles" => {
                styles::list().iter().for_each(|(n, _)| println!("{n}"));
                return Ok(None);
            }
            "--describe-styles" => {
                styles::list()
                    .iter()
                    .for_each(|(n, d)| println!("{n:<11} {d}"));
                return Ok(None);
            }
            "--style" => a.style = value("--style")?,
            "--flavor" => a.flavor = value("--flavor")?,
            "--accent" => a.accent = value("--accent")?,
            "--size" => a.size = parse_size(&value("--size")?)?,
            "--thumb" => a.thumb = parse_size(&value("--thumb")?)?,
            "--all" => a.all = Some(value("--all")?.into()),
            "--preview" => a.preview = Some(value("--preview")?.into()),
            s if s.starts_with('-') => return Err(format!("unknown option '{s}' (see --help)")),
            s if a.output.is_none() => a.output = Some(s.into()),
            s => return Err(format!("unexpected argument '{s}' (see --help)")),
        }
    }
    if !palette::flavors().contains(&a.flavor.as_str()) {
        return Err(format!(
            "unknown flavor '{}' (choose: {})",
            a.flavor,
            palette::flavors().join(", ")
        ));
    }
    if !palette::accents().contains(&a.accent.as_str()) {
        return Err(format!(
            "unknown accent '{}' (choose: {})",
            a.accent,
            palette::accents().join(", ")
        ));
    }
    if !styles::list().iter().any(|(n, _)| *n == a.style) {
        let names: Vec<_> = styles::list().iter().map(|(n, _)| *n).collect();
        return Err(format!(
            "unknown style '{}' (choose: {})",
            a.style,
            names.join(", ")
        ));
    }
    let modes = [a.output.is_some(), a.all.is_some(), a.preview.is_some()];
    if modes.iter().filter(|m| **m).count() != 1 {
        return Err(
            "give exactly one of OUTPUT, --all DIR or --preview OUTPUT (see --help)".into(),
        );
    }
    Ok(Some(a))
}

pub(crate) fn render_one(
    style: &str,
    flavor: &str,
    accent: &str,
    (w, h): (usize, usize),
) -> Vec<u8> {
    let p = Palette::new(flavor).expect("validated flavor");
    let pixel = styles::build(style, p, accent, w, h).expect("validated style");
    image::render(w, h, &pixel)
}

fn save(path: &Path, style: &str, a: &Args) -> Result<(), String> {
    let rgb = render_one(style, &a.flavor, &a.accent, a.size);
    image::write_png(path, a.size.0, a.size.1, &rgb)
}

/// Grid of every style (rows) in every flavor (columns) on a Crust border.
fn preview(path: &Path, accent: &str, (tw, th): (usize, usize)) -> Result<(), String> {
    const GAP: usize = 8;
    let names: Vec<_> = styles::list().iter().map(|(n, _)| *n).collect();
    let flavors = palette::flavors();
    let (w, h) = (
        flavors.len() * (tw + GAP) + GAP,
        names.len() * (th + GAP) + GAP,
    );
    let crust = Palette::new("mocha").unwrap().get("crust").map(|v| v as u8);
    let mut out: Vec<u8> = crust.iter().copied().cycle().take(w * h * 3).collect();
    let jobs: Vec<(usize, usize)> = (0..names.len())
        .flat_map(|r| (0..flavors.len()).map(move |c| (r, c)))
        .collect();
    let thumbs: Vec<_> = jobs
        .par_iter()
        .map(|&(r, c)| ((r, c), render_one(names[r], flavors[c], accent, (tw, th))))
        .collect();
    for ((r, c), rgb) in thumbs {
        let (x0, y0) = (GAP + c * (tw + GAP), GAP + r * (th + GAP));
        for y in 0..th {
            let dst = ((y0 + y) * w + x0) * 3;
            out[dst..dst + tw * 3].copy_from_slice(&rgb[y * tw * 3..(y + 1) * tw * 3]);
        }
    }
    image::write_png(path, w, h, &out)
}

fn run(a: &Args) -> Result<(), String> {
    if let Some(out) = &a.output {
        return save(out, &a.style, a);
    }
    if let Some(out) = &a.preview {
        return preview(out, &a.accent, a.thumb);
    }
    let dir = a.all.as_ref().expect("one mode is set");
    render_all(dir, &a.flavor, &a.accent, a.size)
}

/// One wallpaper at the default size.
pub fn render_to(path: &Path, style: &str, flavor: &str, accent: &str) -> Result<(), String> {
    let (w, h) = DEFAULT_SIZE;
    image::write_png(path, w, h, &render_one(style, flavor, accent, (w, h)))
}

/// Every style into `dir`, named catppuccin-<flavor>-<accent>-<style>.png.
pub fn render_all(
    dir: &Path,
    flavor: &str,
    accent: &str,
    (w, h): (usize, usize),
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    // Rows are rendered in parallel inside each image, and the images run in
    // parallel too so their (single-threaded) PNG compression overlaps.
    styles::list().par_iter().try_for_each(|(style, _)| {
        let path = dir.join(format!("catppuccin-{flavor}-{accent}-{style}.png"));
        image::write_png(&path, w, h, &render_one(style, flavor, accent, (w, h)))
    })
}

/// Pulls every pixel toward the flavor's colors: each output is the average of
/// the palette weighted by how close it is to the input (Gaussian, width
/// `sigma` in 0..255 RGB units). A small sigma snaps to the nearest color
/// (posterized); a large one washes everything toward the average.
pub fn recolor_pixels(rgb: &mut [u8], palette: &[[f64; 3]], sigma: f64) {
    let k = 1.0 / (2.0 * sigma * sigma);
    let pal: Vec<[f32; 3]> = palette
        .iter()
        .map(|c| [c[0] as f32, c[1] as f32, c[2] as f32])
        .collect();
    rgb.par_chunks_mut(3 * 4096).for_each(|chunk| {
        let mut d2 = vec![0f32; pal.len()];
        for px in chunk.as_chunks_mut::<3>().0.iter_mut() {
            let p = [f32::from(px[0]), f32::from(px[1]), f32::from(px[2])];
            let mut min = f32::MAX;
            for (d, c) in d2.iter_mut().zip(&pal) {
                *d = (0..3).map(|i| (p[i] - c[i]).powi(2)).sum();
                min = min.min(*d);
            }
            let (mut sum, mut acc) = (0f32, [0f32; 3]);
            for (d, c) in d2.iter().zip(&pal) {
                // Relative to the nearest color, so far-off pixels still get a weight.
                let w = (-(d - min) * k as f32).exp();
                sum += w;
                (0..3).for_each(|i| acc[i] += w * c[i]);
            }
            (0..3).for_each(|i| px[i] = (acc[i] / sum).round().clamp(0.0, 255.0) as u8);
        }
    });
}

/// Recolors a PNG with a flavor's palette.
pub fn recolor(input: &Path, output: &Path, flavor: &str, sigma: f64) -> Result<(), String> {
    let p = Palette::new(flavor).ok_or_else(|| format!("unknown flavor '{flavor}'"))?;
    let (w, h, mut rgb) = image::read_png(input)?;
    recolor_pixels(&mut rgb, &p.all(), sigma);
    image::write_png(output, w, h, &rgb)
}

fn recolor_main(args: &[String]) -> Result<(), String> {
    let (mut flavor, mut sigma) = ("mocha".to_string(), 40.0);
    let mut paths = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--flavor" => flavor = it.next().ok_or("--flavor needs a value")?.clone(),
            "--sigma" => {
                let v = it.next().ok_or("--sigma needs a value")?;
                sigma = v.parse().map_err(|_| format!("bad --sigma '{v}'"))?;
                if sigma <= 0.0 {
                    return Err("--sigma must be above 0".into());
                }
            }
            p if !p.starts_with("--") => paths.push(PathBuf::from(p)),
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
    }
    match paths.as_slice() {
        [input, output] => recolor(input, output, &flavor, sigma),
        _ => Err(format!("recolor needs INPUT.png and OUTPUT.png\n\n{USAGE}")),
    }
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) == Some("recolor") {
        return recolor_main(&args[1..]);
    }
    match parse_args(args)? {
        Some(a) => run(&a),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_style_renders_every_flavor_deterministically() {
        for (style, _) in styles::list() {
            for flavor in palette::flavors() {
                let a = render_one(style, flavor, "mauve", (64, 36));
                assert_eq!(a.len(), 64 * 36 * 3);
                assert_eq!(
                    a,
                    render_one(style, flavor, "mauve", (64, 36)),
                    "{style}/{flavor}"
                );
            }
        }
    }

    #[test]
    fn recolor_lands_on_the_palette_and_keeps_order() {
        let p = Palette::new("mocha").unwrap().all();
        // A pixel that is exactly a palette color stays (almost) itself.
        let target = p[5];
        let mut px = vec![target[0] as u8, target[1] as u8, target[2] as u8];
        recolor_pixels(&mut px, &p, 12.0);
        for i in 0..3 {
            assert!(
                (f64::from(px[i]) - target[i]).abs() <= 6.0,
                "{px:?} vs {target:?}"
            );
        }
        // Dark stays darker than light, whatever the sigma.
        let mut two = vec![10, 10, 10, 245, 245, 245];
        recolor_pixels(&mut two, &p, 40.0);
        assert!(two[0] < two[3]);
        // Far from every color still gives a valid pixel (no NaN).
        let mut far = vec![255, 0, 255];
        recolor_pixels(&mut far, &p, 1.0);
        assert!(far.iter().any(|&v| v > 0));
    }

    #[test]
    fn recolor_round_trips_a_png() {
        let dir = std::env::temp_dir().join(format!("ctp-rc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (input, output) = (dir.join("in.png"), dir.join("out.png"));
        let rgb: Vec<u8> = (0..16 * 9 * 3).map(|i| (i * 7 % 256) as u8).collect();
        image::write_png(&input, 16, 9, &rgb).unwrap();
        recolor(&input, &output, "latte", 40.0).unwrap();
        let (w, h, out) = image::read_png(&output).unwrap();
        assert_eq!((w, h, out.len()), (16, 9, 16 * 9 * 3));
        assert!(recolor(&dir.join("missing.png"), &output, "latte", 40.0).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sizes_parse() {
        assert_eq!(parse_size("2560x1440"), Ok((2560, 1440)));
        assert_eq!(parse_size("3840X2160"), Ok((3840, 2160)));
        assert!(parse_size("2560").is_err());
        assert!(parse_size("8x8").is_err());
    }
}
