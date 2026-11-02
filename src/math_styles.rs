//! Wallpaper styles drawn from formulas: fractals, wave interference,
//! tilings, curves, cellular automata and strange attractors.
//!
//! Most are a scalar field `f(u, v) → 0..1` that `field` maps through one
//! color ramp (Crust → Surface1 → the accent → a light accent), so they all
//! follow the flavor and accent like the other styles. `u`, `v` are centered
//! and scaled by the image height: `v` runs from -0.5 to 0.5.

use std::f64::consts::PI;

use crate::noise::{clamp01, fbm, hash01, mix};
use crate::palette::{Palette, Rgb};
use crate::styles::{PixelFn, companions};

pub fn build(style: &str, p: Palette, accent: &str, w: usize, h: usize) -> Option<PixelFn> {
    Some(match style {
        "mandelbrot" => field(p, accent, w, h, 1, |u, v| {
            band(escape(
                (0.0, 0.0),
                (-0.743_644_8 + u * 0.02, 0.131_825_3 + v * 0.02),
                900,
                false,
            ))
        }),
        "burningship" => field(p, accent, w, h, 1, |u, v| {
            band(escape(
                (0.0, 0.0),
                (-1.762 + u * 0.1, -0.028 + v * 0.1),
                300,
                true,
            ))
        }),
        "newton" => newton(p, accent, w, h),
        "quasicrystal" => {
            let px = 1.0 / h as f64;
            field(p, accent, w, h, 2, move |u, v| {
                crisp(quasicrystal, u, v, px, 6.0)
            })
        }
        "plasma" => {
            let px = 1.0 / h as f64;
            field(p, accent, w, h, 2, move |u, v| crisp(plasma, u, v, px, 8.0))
        }
        "spiral" => field(p, accent, w, h, 2, spiral),
        "hexgrid" => field(p, accent, w, h, 2, hexgrid),
        "truchet" => field(p, accent, w, h, 2, truchet),
        "sierpinski" => field(p, accent, w, h, 1, sierpinski),
        "xorfractal" => field(p, accent, w, h, 1, |u, v| {
            let (i, j) = (((u + 1.0) * 200.0) as i64, ((v + 0.5) * 200.0) as i64);
            (((i ^ j) & 63) as f64 / 63.0).powf(1.6)
        }),
        "opart" => field(p, accent, w, h, 2, opart),
        "gyroid" => field(p, accent, w, h, 2, gyroid),
        "marble" => field(p, accent, w, h, 1, marble),
        "ripples" => {
            let px = 1.0 / h as f64;
            field(p, accent, w, h, 2, move |u, v| {
                crisp(ripples, u, v, px, 8.0)
            })
        }
        "grating" => field(p, accent, w, h, 2, grating),
        "inversion" => field(p, accent, w, h, 2, inversion),
        "domain" => field(p, accent, w, h, 1, domain),
        "automaton" => automaton(p, accent, w, h),
        "spirograph" => spirograph(p, accent, w, h),
        "lissajous" => lissajous(p, accent, w, h),
        "clifford" => clifford(p, accent, w, h),
        _ => return None,
    })
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

fn fract(x: f64) -> f64 {
    x - x.floor()
}

/// Posterizes a smooth field into `n` flat levels with edges about one pixel
/// wide (`px` is one pixel in `u`, `v` units). Smooth sine fields otherwise
/// look blurred.
pub(crate) fn crisp(s: impl Fn(f64, f64) -> f64, u: f64, v: f64, px: f64, n: f64) -> f64 {
    let s0 = s(u, v);
    let grad = ((s(u + px, v) - s0).abs() + (s(u, v + px) - s0).abs()) * n;
    let x = s0 * n;
    let w = (grad * 0.75).clamp(0.02, 0.5);
    (x.floor() + smooth(1.0 - w, 1.0, fract(x))) / n
}

/// Crust → Surface1 → accent → a lighter accent, for t in 0..1.
pub(crate) fn ramp(p: Palette, accent: &str) -> impl Fn(f64) -> Rgb + Sync + Send + use<> {
    let acc = p.get(accent);
    let stops = [
        p.get("crust"),
        p.get("surface1"),
        acc,
        mix(acc, p.get("text"), 0.4),
    ];
    let at = [0.0, 0.5, 0.85, 1.0];
    move |t| {
        let t = clamp01(t);
        for i in 0..3 {
            if t <= at[i + 1] {
                return mix(stops[i], stops[i + 1], (t - at[i]) / (at[i + 1] - at[i]));
            }
        }
        stops[3]
    }
}

/// Maps a scalar field to colors. `ss` × `ss` samples per pixel smooth the
/// edges of line patterns.
fn field(
    p: Palette,
    accent: &str,
    w: usize,
    h: usize,
    ss: usize,
    f: impl Fn(f64, f64) -> f64 + Sync + Send + 'static,
) -> PixelFn {
    let ramp = ramp(p, accent);
    let (wf, hf) = (w as f64, h as f64);
    Box::new(move |x, y| {
        let mut t = 0.0;
        for sy in 0..ss {
            for sx in 0..ss {
                let px = x as f64 + (sx as f64 + 0.5) / ss as f64;
                let py = y as f64 + (sy as f64 + 0.5) / ss as f64;
                t += f((px - wf / 2.0) / hf, (py - hf / 2.0) / hf);
            }
        }
        ramp(t / (ss * ss) as f64)
    })
}

/// Smooth escape time of z → z² + c (or the burning ship), None inside.
fn escape(z0: (f64, f64), c: (f64, f64), max: u32, ship: bool) -> Option<f64> {
    let (mut zr, mut zi) = z0;
    for n in 0..max {
        let (r2, i2) = (zr * zr, zi * zi);
        if r2 + i2 > 256.0 {
            return Some(f64::from(n) + 1.0 - ((r2 + i2).ln() / 2.0).log2());
        }
        let (ar, ai) = if ship { (zr.abs(), zi.abs()) } else { (zr, zi) };
        zi = 2.0 * ar * ai + c.1;
        zr = ar * ar - ai * ai + c.0;
    }
    None
}

/// Repeating bands of the escape time; the inside stays dark.
fn band(mu: Option<f64>) -> f64 {
    mu.map_or(0.0, |m| 0.15 + 0.85 * (0.5 + 0.5 * (m * 0.25).sin()))
}

/// Newton's method on z³ = 1: one color per root, brighter the faster it converges.
fn newton(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let (second, third) = companions(accent);
    let cols = [p.get(accent), p.get(second), p.get(third)];
    let bg = p.get("crust");
    let (wf, hf) = (w as f64, h as f64);
    Box::new(move |x, y| {
        let mut z = (
            (x as f64 - wf / 2.0) / hf * 3.2,
            (y as f64 - hf / 2.0) / hf * 3.2,
        );
        for n in 0..40 {
            let z2 = (z.0 * z.0 - z.1 * z.1, 2.0 * z.0 * z.1);
            let z3 = (z2.0 * z.0 - z2.1 * z.1, z2.0 * z.1 + z2.1 * z.0);
            let num = (z3.0 - 1.0, z3.1);
            let den = (3.0 * z2.0, 3.0 * z2.1);
            let d = den.0 * den.0 + den.1 * den.1;
            if d < 1e-12 {
                break;
            }
            let q = (
                (num.0 * den.0 + num.1 * den.1) / d,
                (num.1 * den.0 - num.0 * den.1) / d,
            );
            z = (z.0 - q.0, z.1 - q.1);
            if q.0 * q.0 + q.1 * q.1 < 1e-10 {
                // The roots sit at 0°, 120° and 240°.
                let k = ((z.1.atan2(z.0) / (2.0 * PI / 3.0)).round() as i64).rem_euclid(3);
                return mix(
                    bg,
                    cols[k as usize],
                    0.2 + 0.8 * (-f64::from(n) * 0.18).exp(),
                );
            }
        }
        bg
    })
}

/// Seven plane waves at equal angles add up to a pattern without a period.
fn quasicrystal(u: f64, v: f64) -> f64 {
    let k = 2.0 * PI * 14.0;
    let s: f64 = (0..7)
        .map(|i| {
            let a = f64::from(i) * PI / 7.0;
            (k * (u * a.cos() + v * a.sin())).cos()
        })
        .sum();
    clamp01(0.5 + s * 0.16)
}

fn plasma(u: f64, v: f64) -> f64 {
    let s = (u * 10.0).sin()
        + (v * 8.0 + u * 3.0).sin()
        + ((u * 0.5_f64.sin() + v * 0.3_f64.cos()) * 8.0 + 2.0).sin()
        + (u.hypot(v) * 12.0).sin();
    0.5 + 0.5 * (s * 1.4).sin()
}

/// A logarithmic spiral with five arms.
fn spiral(u: f64, v: f64) -> f64 {
    let r = u.hypot(v);
    let phase = 5.0 * v.atan2(u) + 9.0 * (r + 1e-3).ln();
    let arm = smooth(-0.2, 0.3, phase.sin());
    arm * (0.25 + 0.75 * (-r * 1.4).exp()) + 0.05
}

fn modv(a: f64, b: f64) -> f64 {
    a - b * (a / b).floor()
}

/// A honeycomb: hexagon borders over cells of slightly different tone.
fn hexgrid(u: f64, v: f64) -> f64 {
    let (px, py) = (u * 9.0, v * 9.0);
    let a = (modv(px, 1.0) - 0.5, modv(py, 1.732_050_8) - 0.866_025_4);
    let b = (
        modv(px - 0.5, 1.0) - 0.5,
        modv(py - 0.866_025_4, 1.732_050_8) - 0.866_025_4,
    );
    let g = if a.0 * a.0 + a.1 * a.1 < b.0 * b.0 + b.1 * b.1 {
        a
    } else {
        b
    };
    let (ax, ay) = (g.0.abs(), g.1.abs());
    let edge = 0.5 - (ax * 0.5 + ay * 0.866_025_4).max(ax);
    let line = 1.0 - smooth(0.015, 0.045, edge);
    let id = ((px - g.0) * 2.0).round() as i64;
    let idy = ((py - g.1) * 2.0).round() as i64;
    0.1 + 0.3 * hash01(id, idy) * (0.6 + 0.8 * (v + 0.5)).min(1.0) + 0.6 * line
}

/// Truchet tiles: each cell holds two quarter circles, flipped at random.
fn truchet(u: f64, v: f64) -> f64 {
    let (gx, gy) = (u * 10.0, v * 10.0);
    let (i, j) = (gx.floor(), gy.floor());
    let (mut fx, fy) = (gx - i, gy - j);
    let h = hash01(i as i64, j as i64);
    if h > 0.5 {
        fx = 1.0 - fx;
    }
    let d = ((fx.hypot(fy) - 0.5).abs()).min(((fx - 1.0).hypot(fy - 1.0) - 0.5).abs());
    0.08 + (0.5 + 0.4 * hash01(i as i64 + 91, j as i64)) * (1.0 - smooth(0.05, 0.1, d))
}

/// Sierpinski's triangle: the cells where i AND j is zero.
fn sierpinski(u: f64, v: f64) -> f64 {
    let (i, j) = (((u + 1.0) * 150.0) as i64, ((v + 0.5) * 150.0) as i64);
    if i & j == 0 {
        0.55 + 0.4 * (v + 0.5)
    } else {
        0.05 + 0.1 * (1.0 - (v + 0.5))
    }
}

/// A checkerboard bent by two sine waves.
fn opart(u: f64, v: f64) -> f64 {
    let a = u * 7.0 + 0.3 * (v * 10.0).sin();
    let b = v * 7.0 + 0.3 * (u * 10.0).sin();
    let dark = 1.0 - 0.6 * clamp01(u.hypot(v) * 1.1);
    if (a.floor() as i64 + b.floor() as i64).rem_euclid(2) == 0 {
        0.9 * dark
    } else {
        0.1
    }
}

/// A slice through the gyroid surface, sin x cos y + sin y cos z + sin z cos x.
fn gyroid(u: f64, v: f64) -> f64 {
    let (x, y, z) = (u * 16.0, v * 16.0, 1.1_f64);
    let f = x.sin() * y.cos() + y.sin() * z.cos() + z.sin() * x.cos();
    clamp01(0.4 + 0.2 * f) + 0.5 * (1.0 - smooth(0.03, 0.1, f.abs()))
}

/// Veins of marble: a sine warped by fractal noise.
fn marble(u: f64, v: f64) -> f64 {
    let q = fbm(u * 2.5 + 5.0, v * 2.5 + 5.0, 5, 3);
    let s = (u * 3.0 + v * 1.5 + q * 4.0) * PI;
    0.1 + 0.7 * (1.0 - s.sin().abs()).powf(8.0) + 0.2 * q
}

/// Waves from five point sources, fading with distance.
fn ripples(u: f64, v: f64) -> f64 {
    const SOURCES: [(f64, f64); 5] = [
        (-0.5, -0.1),
        (0.4, 0.2),
        (0.0, -0.4),
        (0.7, -0.2),
        (-0.2, 0.35),
    ];
    let s: f64 = SOURCES
        .iter()
        .map(|&(sx, sy)| {
            let d = (u - sx).hypot(v - sy);
            (2.0 * PI * 16.0 * d).sin() / (1.0 + 5.0 * d)
        })
        .sum();
    clamp01(0.5 + s * 0.45)
}

/// Two line gratings a few degrees apart.
fn grating(u: f64, v: f64) -> f64 {
    let k = 2.0 * PI * 60.0;
    let line = |angle: f64| smooth(0.55, 0.75, (k * (u * angle.cos() + v * angle.sin())).cos());
    0.08 + 0.4 * line(0.35) + 0.5 * line(0.40)
}

/// A square grid seen through circle inversion.
fn inversion(u: f64, v: f64) -> f64 {
    let r2 = u * u + v * v + 0.015;
    let (wx, wy) = (u / r2 * 0.15 * 5.0, v / r2 * 0.15 * 5.0);
    let d = (fract(wx + 0.5) - 0.5)
        .abs()
        .min((fract(wy + 0.5) - 0.5).abs());
    // The inversion shrinks lengths by about 0.75 / r²; undo it so lines keep their width.
    0.08 + 0.85 * (1.0 - smooth(0.004, 0.009, d * r2 / 0.75))
}

fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

/// Domain coloring of (z² - 1)(z - 2 - i)² / (z² + 2 + 2i): the argument as
/// bands, the modulus as contours.
fn domain(u: f64, v: f64) -> f64 {
    let z = (u * 3.0, v * 3.0);
    let z2 = cmul(z, z);
    let a = (z2.0 - 1.0, z2.1);
    let b = cmul((z.0 - 2.0, z.1 - 1.0), (z.0 - 2.0, z.1 - 1.0));
    let num = cmul(a, b);
    let den = (z2.0 + 2.0, z2.1 + 2.0);
    let d = den.0 * den.0 + den.1 * den.1;
    let f = (
        (num.0 * den.0 + num.1 * den.1) / d,
        (num.1 * den.0 - num.0 * den.1) / d,
    );
    let arg = f.1.atan2(f.0);
    let modulus = (f.0.hypot(f.1) + 1e-9).ln();
    (0.5 + 0.5 * (3.0 * arg).sin()) * (0.55 + 0.45 * fract(modulus * 1.2))
}

/// Rule 30, one cell at the top, row after row downward.
fn automaton(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let ramp = ramp(p, accent);
    let cell = (h / 200).max(2);
    let (cols, rows) = (w / cell + 1, h / cell + 1);
    let mut grid = vec![vec![false; cols]; rows];
    grid[0][cols / 2] = true;
    for j in 1..rows {
        for i in 0..cols {
            let at = |k: isize| {
                let k = i as isize + k;
                k >= 0 && (k as usize) < cols && grid[j - 1][k as usize]
            };
            grid[j][i] = at(-1) ^ (at(0) | at(1));
        }
    }
    Box::new(move |x, y| {
        let (i, j) = (x / cell, y / cell);
        let fade = 1.0 - j as f64 / rows as f64;
        ramp(if grid[j][i] {
            0.5 + 0.45 * fade
        } else {
            0.04 + 0.1 * fade
        })
    })
}

/// Adds a soft dot at every point (a gaussian about `radius` px wide) into a
/// w × h buffer.
pub(crate) fn splat(
    w: usize,
    h: usize,
    points: impl Iterator<Item = (f64, f64)>,
    radius: i64,
) -> Vec<f32> {
    let mut buf = vec![0f32; w * h];
    let s2 = (radius as f64 * 0.6).powi(2).max(0.25);
    for (x, y) in points {
        let (ix, iy) = (x.round() as i64, y.round() as i64);
        for py in iy - radius..=iy + radius {
            for px in ix - radius..=ix + radius {
                if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                    let d2 = (px as f64 - x).powi(2) + (py as f64 - y).powi(2);
                    buf[py as usize * w + px as usize] += (-d2 / (2.0 * s2)).exp() as f32;
                }
            }
        }
    }
    buf
}

/// A density buffer as a picture: `shade` turns a density into 0..1.
pub(crate) fn density(
    p: Palette,
    accent: &str,
    w: usize,
    buf: Vec<f32>,
    shade: impl Fn(f64) -> f64 + Sync + Send + 'static,
) -> PixelFn {
    let ramp = ramp(p, accent);
    Box::new(move |x, y| ramp(shade(f64::from(buf[y * w + x]))))
}

/// Two hypotrochoids, the curves a spirograph draws.
fn spirograph(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let (wf, hf) = (w as f64, h as f64);
    let curve = |r: f64, d: f64, turns: f64, size: f64| {
        let scale = hf * size / (1.0 - r + d);
        let n = h * 100;
        (0..n).map(move |i| {
            let t = i as f64 / n as f64 * turns * 2.0 * PI;
            let k = (1.0 - r) / r;
            (
                wf / 2.0 + scale * ((1.0 - r) * t.cos() + d * (k * t).cos()),
                hf / 2.0 + scale * ((1.0 - r) * t.sin() - d * (k * t).sin()),
            )
        })
    };
    let points = curve(5.0 / 16.0, 0.85, 5.0, 0.46).chain(curve(0.4, 0.5, 2.0, 0.3));
    let buf = splat(w, h, points, 2);
    density(p, accent, w, buf, |d| {
        0.04 + 0.96 * (1.0 - (-d * 0.5).exp())
    })
}

/// Nine Lissajous curves, x = sin(3t + δ), y = sin(2t), the phase δ stepping.
fn lissajous(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let (wf, hf) = (w as f64, h as f64);
    let n = h * 10;
    let points = (0..9).flat_map(move |k| {
        let delta = f64::from(k) * 0.35;
        (0..n).map(move |i| {
            let t = i as f64 / n as f64 * 2.0 * PI;
            (
                wf / 2.0 + wf * 0.4 * (3.0 * t + delta).sin(),
                hf / 2.0 + hf * 0.44 * (2.0 * t).sin(),
            )
        })
    });
    let buf = splat(w, h, points, 2);
    density(p, accent, w, buf, |d| {
        0.04 + 0.96 * (1.0 - (-d * 0.6).exp())
    })
}

/// The Clifford attractor, x' = sin(a y) + c cos(a x), y' = sin(b x) + d cos(b y),
/// scaled to fit the picture.
fn clifford(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let (a, b, c, d) = (-1.4, 1.6, 1.0, 0.7);
    let mut z = (0.1_f64, 0.1_f64);
    let n = (w * h * 2 / 5).clamp(20_000, 1_500_000);
    let orbit: Vec<(f64, f64)> = (0..n)
        .map(|_| {
            z = (
                (a * z.1).sin() + c * (a * z.0).cos(),
                (b * z.0).sin() + d * (b * z.1).cos(),
            );
            z
        })
        .collect();
    let range = |f: fn(&(f64, f64)) -> f64| {
        orbit
            .iter()
            .map(f)
            .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)))
    };
    let (x, y) = (range(|q| q.0), range(|q| q.1));
    let scale = (h as f64 * 0.85 / (y.1 - y.0)).min(w as f64 * 0.85 / (x.1 - x.0));
    let (cx, cy) = ((x.0 + x.1) / 2.0, (y.0 + y.1) / 2.0);
    let points = orbit.iter().map(|q| {
        (
            w as f64 / 2.0 + (q.0 - cx) * scale,
            h as f64 / 2.0 + (q.1 - cy) * scale,
        )
    });
    let buf = splat(w, h, points, 1);
    let max = f64::from(buf.iter().copied().fold(1.0, f32::max));
    density(p, accent, w, buf, move |d| {
        0.03 + 0.97 * ((1.0 + d).ln() / (1.0 + max * 0.5).ln()).min(1.0).powf(0.8)
    })
}
