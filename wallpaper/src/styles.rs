//! The wallpaper styles. Each one takes the palette, the accent and the
//! image size, precomputes what it can, and returns a per-pixel function
//! (x, y) → color. Dithering and clamping happen afterwards in `image.rs`.
//!
//! Every style uses only the chosen flavor's palette and accent, so all of
//! them work for all 4 flavors × 14 accents.

use std::f64::consts::PI;

use rayon::prelude::*;

use crate::noise::{clamp01, fbm, hash01, mix, vnoise};
use crate::palette::{Palette, Rgb};

pub type PixelFn = Box<dyn Fn(usize, usize) -> Rgb + Sync + Send>;

/// Style names and descriptions in display order, from wallpaper/styles.txt
/// (shared with the installer, which validates style names before running).
pub fn list() -> Vec<(&'static str, &'static str)> {
    include_str!("../styles.txt")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            l.split_once('\t')
                .expect("styles.txt: name<TAB>description")
        })
        .collect()
}

pub fn build(style: &str, p: Palette, accent: &str, w: usize, h: usize) -> Option<PixelFn> {
    let (wf, hf) = (w as f64, h as f64);
    Some(match style {
        "glow" => glow(p, accent, wf, hf),
        "waves" => waves(p, accent, w, wf, hf),
        "stripes" => stripes(p, accent, wf, hf),
        "dots" => dots(p, accent, wf, hf),
        "triangles" => triangles(p, accent, wf, hf),
        "mesh" => mesh(p, accent, w, h),
        "aurora" => aurora(p, accent, w, wf, hf),
        "sunset" => sunset(p, accent, wf, hf),
        "topography" => topography(p, accent, w, h),
        "mountains" => mountains(p, accent, w, wf, hf),
        _ => return None,
    })
}

/// Two companion colors that always differ from the accent.
fn companions(accent: &str) -> (&'static str, &'static str) {
    (
        if accent == "blue" { "lavender" } else { "blue" },
        if accent == "pink" { "mauve" } else { "pink" },
    )
}

/// 1px anti-aliased coverage of everything below a per-column edge.
fn below(y: f64, edge: f64) -> f64 {
    clamp01(y - edge + 0.5)
}

fn glow(p: Palette, accent: &str, w: f64, h: f64) -> PixelFn {
    let (crust, base) = (p.get("crust"), p.get("base"));
    let (second, third) = companions(accent);
    // (center x, center y) as a fraction of the screen, radius, color, strength
    let glows = [
        ((0.82, 0.85), 0.55, p.get(accent), 0.35),
        ((0.12, 0.10), 0.50, p.get(second), 0.22),
        ((0.50, 1.10), 0.45, p.get(third), 0.12),
    ];
    Box::new(move |x, y| {
        let (fx, fy) = (x as f64 / w, y as f64 / h);
        // Gradient: Crust at both ends of the diagonal, Base in the middle.
        let t = fx * 0.4 + fy * 0.6;
        let mut c = mix(crust, base, (1.0 - (t - 0.5).abs() * 2.0).powf(0.5));
        for ((gx, gy), r, col, a) in glows {
            // 1.78 ≈ 16/9 keeps the glows round on a widescreen display.
            let d = ((fx - gx) * 1.78).hypot(fy - gy) / r;
            if d < 1.0 {
                c = mix(c, col, a * (1.0 - d).powf(2.2));
            }
        }
        c
    })
}

fn waves(p: Palette, accent: &str, w: usize, wf: f64, h: f64) -> PixelFn {
    let (crust, base, mantle, acc) = (
        p.get("crust"),
        p.get("base"),
        p.get("mantle"),
        p.get(accent),
    );
    // Five hills, back to front: (base height, amplitude, frequency, phase,
    // amount of accent). Back hills are faint, the front one is the boldest.
    let layers = [
        (0.50, 0.050, 0.9, 0.10, 0.10),
        (0.60, 0.045, 1.3, 0.55, 0.20),
        (0.70, 0.040, 1.7, 0.25, 0.32),
        (0.79, 0.035, 2.2, 0.80, 0.42),
        (0.88, 0.030, 2.8, 0.40, 0.54),
    ];
    let hills: Vec<(Vec<f64>, Rgb)> = layers
        .iter()
        .map(|&(b, amp, f, ph, k)| {
            let edge = (0..w)
                .map(|x| {
                    let fx = x as f64 / wf;
                    h * (b
                        + amp * (2.0 * PI * (fx * f + ph)).sin()
                        + amp * 0.35 * (2.0 * PI * (fx * f * 2.3 + ph * 1.7)).sin())
                })
                .collect();
            (edge, mix(mantle, acc, k))
        })
        .collect();
    Box::new(move |x, y| {
        let yf = y as f64;
        let mut c = mix(crust, base, yf / h); // sky: Crust at the top to Base
        for (edge, col) in &hills {
            let cover = below(yf, edge[x]);
            if cover > 0.0 {
                c = mix(c, *col, cover);
            }
        }
        c
    })
}

fn stripes(p: Palette, accent: &str, w: f64, h: f64) -> PixelFn {
    let (crust, base) = (p.get("crust"), p.get("base"));
    // The accent in the middle with two neighbors on each side, in the
    // order of the official accent list (which wraps around).
    let accents = crate::palette::accents();
    let n = accents.len() as i64;
    let i = accents.iter().position(|a| *a == accent).unwrap() as i64;
    let cols: Vec<Rgb> = (-2..=2)
        .map(|k| p.get(accents[(i + k).rem_euclid(n) as usize]))
        .collect();
    let sw = h * 0.04; // stripe width, across the stripe
    let start = h * 0.98; // where the ribbon begins (lower right)
    let inv = 1.0 / 2f64.sqrt();
    Box::new(move |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        let bg = mix(crust, base, clamp01(xf / w * 0.5 + yf / h * 0.5));
        let u = (xf + yf) * inv - start * 1.4; // distance along the 45° diagonal
        let k = u / sw;
        let idx = k.floor();
        if idx < 0.0 || idx >= cols.len() as f64 {
            return bg;
        }
        let idx = idx as usize;
        // Blend over about one pixel into the neighbor (or the background)
        // on whichever edge of the stripe this pixel is closer to.
        let frac = k - idx as f64;
        if frac < 0.5 {
            let other = if idx > 0 { cols[idx - 1] } else { bg };
            mix(other, cols[idx], clamp01(frac * sw + 0.5))
        } else {
            let other = if idx < cols.len() - 1 {
                cols[idx + 1]
            } else {
                bg
            };
            mix(other, cols[idx], clamp01((1.0 - frac) * sw + 0.5))
        }
    })
}

fn dots(p: Palette, accent: &str, w: f64, h: f64) -> PixelFn {
    let (base, mantle, surface0, acc) = (
        p.get("base"),
        p.get("mantle"),
        p.get("surface0"),
        p.get(accent),
    );
    let s = h / 26.0; // grid spacing
    let maxd = w.hypot(h) * 0.75;
    Box::new(move |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        let mut c = mix(mantle, base, yf / h);
        let (cx, cy) = (((xf / s).floor() + 0.5) * s, ((yf / s).floor() + 0.5) * s);
        // Dot size grows toward the bottom-right corner, fading to nothing.
        let mut f = clamp01(1.0 - (w - cx).hypot(h - cy) / maxd);
        f *= f;
        let r = s * 0.46 * f;
        if r > 0.3 {
            let d = (xf + 0.5 - cx).hypot(yf + 0.5 - cy);
            let cover = clamp01(r - d + 0.5);
            if cover > 0.0 {
                c = mix(c, mix(surface0, acc, 0.25 + 0.75 * f), cover);
            }
        }
        c
    })
}

fn triangles(p: Palette, accent: &str, w: f64, h: f64) -> PixelFn {
    let (crust, mantle, base, surface0) = (
        p.get("crust"),
        p.get("mantle"),
        p.get("base"),
        p.get("surface0"),
    );
    let acc = p.get(accent);
    let s = h / 7.0; // square cell, split into two triangles
    Box::new(move |x, y| {
        let (gx, gy) = (x as f64 / s, y as f64 / s);
        let (i, j) = (gx.floor() as i64, gy.floor() as i64);
        let (fx, fy) = (gx - i as f64, gy - j as f64);
        // Alternate the diagonal's direction from cell to cell so the
        // triangles don't line up into a plain grid.
        let upper = if (i + j).rem_euclid(2) == 0 {
            fx > fy
        } else {
            fx + fy < 1.0
        };
        let v = hash01(i * 2 + i64::from(upper), j);
        let mut c = mix(crust, mix(mantle, surface0, v), 0.55 + 0.45 * v);
        c = mix(c, base, 0.35);
        // Accent tint toward the bottom-right corner.
        let g = clamp01((i as f64 + 0.5) * s / w * 0.6 + (j as f64 + 0.5) * s / h * 0.6 - 0.45);
        mix(c, acc, g * g * 0.55)
    })
}

/// Sample `f` every `step` px over the image (plus a border) in parallel, for
/// smooth fields that are then bilinearly interpolated per pixel.
fn sample_grid<T: Send>(
    w: usize,
    h: usize,
    step: usize,
    f: impl Fn(usize, usize) -> T + Sync,
) -> Vec<Vec<T>> {
    (0..h / step + 2)
        .into_par_iter()
        .map(|j| (0..w / step + 2).map(|i| f(i * step, j * step)).collect())
        .collect()
}

fn mesh(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let base = p.get("base");
    let (second, third) = companions(accent);
    // Color points: (x, y) as a fraction of the screen, color.
    let points = [
        ((0.80, 0.80), p.get(accent)),
        ((0.15, 0.20), p.get(second)),
        ((0.25, 0.90), p.get(third)),
        ((0.75, 0.10), p.get("crust")),
        ((0.45, 0.45), base),
    ];
    let calm = if p.is_dark() { 0.62 } else { 0.45 }; // how much Base tones it down
    let (wf, hf) = (w as f64, h as f64);
    let blend = |x: usize, y: usize| {
        let (mut fx, mut fy) = (x as f64 / wf * 1.78, y as f64 / hf);
        // A gentle warp makes the blend look organic instead of radial.
        fx += (vnoise(fx * 1.5, fy * 1.5, 3) - 0.5) * 0.25;
        fy += (vnoise(fx * 1.5, fy * 1.5, 9) - 0.5) * 0.25;
        let (mut tw, mut c) = (0.0, [0.0; 3]);
        for ((px, py), col) in points {
            let d2 = (fx - px * 1.78).powi(2) + (fy - py).powi(2);
            let wgt = 1.0 / (d2 + 0.02).powf(1.6);
            tw += wgt;
            for i in 0..3 {
                c[i] += col[i] * wgt;
            }
        }
        mix([c[0] / tw, c[1] / tw, c[2] / tw], base, calm)
    };
    // The blend is very smooth, so compute it every 4 px and interpolate.
    let step = 4;
    let grid = sample_grid(w, h, step, blend);
    Box::new(move |x, y| {
        let (i, j) = (x / step, y / step);
        let tx = (x % step) as f64 / step as f64;
        let ty = (y % step) as f64 / step as f64;
        let top = mix(grid[j][i], grid[j][i + 1], tx);
        mix(top, mix(grid[j + 1][i], grid[j + 1][i + 1], tx), ty)
    })
}

fn aurora(p: Palette, accent: &str, w: usize, wf: f64, h: f64) -> PixelFn {
    let (crust, mantle, base) = (p.get("crust"), p.get("mantle"), p.get("base"));
    let other = if accent == "teal" { "green" } else { "teal" };
    // Two curtains that take turns along the screen: where one is lit the
    // other fades, so they never stack into dark bands.
    // (height of the lower edge, wave amplitude, frequency, phase, color, brightness)
    let curtains = [
        (0.46, 0.09, 1.2, 0.20, p.get(other), 0.90),
        (0.38, 0.08, 1.7, 0.65, p.get(accent), 1.00),
    ];
    let column =
        |f: &dyn Fn(f64) -> f64| -> Vec<f64> { (0..w).map(|x| f(x as f64 / wf)).collect() };
    let edges: Vec<Vec<f64>> = curtains
        .iter()
        .map(|&(b, amp, f, ph, _, _)| {
            column(&|fx| {
                h * (b
                    + amp * (2.0 * PI * (fx * f + ph)).sin()
                    + amp * 0.45 * (2.0 * PI * (fx * f * 2.7 + ph * 3.0)).sin())
            })
        })
        .collect();
    let env = column(&|fx| clamp01((fbm(fx * 2.5, 0.5, 2, 11) - 0.5) * 3.0 + 0.5));
    let lit = [env.clone(), env.iter().map(|v| 1.0 - v).collect::<Vec<_>>()];
    // Fine vertical streaks shared by every curtain.
    let rays = column(&|fx| 0.3 + 0.7 * fbm(fx * 70.0, 0.5, 2, 21).powf(1.5));
    let (tall, sharp) = (h * 0.20, h * 0.05);
    let ground = column(&|fx| {
        h * (0.84
            + 0.025 * (2.0 * PI * (fx * 1.3 + 0.2)).sin()
            + 0.012 * (2.0 * PI * (fx * 4.1)).sin())
    });
    let dark = p.is_dark();
    let ground_col = if dark {
        mix(crust, mantle, 0.2)
    } else {
        mix(p.get("surface1"), crust, 0.3)
    };
    Box::new(move |x, y| {
        let yf = y as f64;
        let mut c = mix(crust, base, yf / h * 0.8);
        for (n, &(_, _, _, _, col, bright)) in curtains.iter().enumerate() {
            let dy = edges[n][x] - yf;
            // Bright lower edge, soft fade upward, quicker fade downward.
            let k = if dy >= 0.0 {
                (-dy / tall).exp()
            } else {
                (-(dy / sharp).powi(2)).exp()
            };
            let k = (k * bright * rays[x] * lit[n][x]).min(1.0);
            if k > 0.004 {
                c = if dark {
                    // Screen blend: overlapping curtains add up like light.
                    [0, 1, 2].map(|i| 255.0 - (255.0 - c[i]) * (255.0 - col[i] * k) / 255.0)
                } else {
                    mix(c, col, k)
                };
            }
        }
        mix(c, ground_col, below(yf, ground[x]))
    })
}

fn sunset(p: Palette, accent: &str, w: f64, h: f64) -> PixelFn {
    let (crust, mantle, acc) = (p.get("crust"), p.get("mantle"), p.get(accent));
    let sun_top = p.get(if accent == "yellow" {
        "peach"
    } else {
        "yellow"
    });
    let hz = h * 0.64; // horizon
    let (cx, cy, r) = (w * 0.5, hz - h * 0.10, h * 0.22);
    let lw = (h / 900.0).max(1.0); // grid line width in pixels
    let k_depth = h * 0.36 * 1.5; // perspective of horizontal lines
    let k_side = (h * 0.36) / (w * 0.07); // spacing of converging lines
    Box::new(move |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        if yf < hz {
            let t = yf / hz;
            let mut c = mix(crust, mantle, t);
            c = mix(c, acc, 0.28 * t.powi(3)); // warm haze near the horizon
            let d = (xf + 0.5 - cx).hypot(yf + 0.5 - cy);
            let mut cover = clamp01(r - d + 0.5);
            if cover > 0.0 {
                // Horizontal cuts in the lower half, wider toward the horizon.
                if yf > cy {
                    let period = h * 0.045;
                    let gap = period * (0.12 + 0.55 * (yf - cy) / r);
                    cover *= clamp01((yf - cy).rem_euclid(period) - gap + 0.5);
                }
                let sun = mix(sun_top, acc, clamp01((yf - (cy - r)) / (2.0 * r)));
                c = mix(c, sun, cover);
            }
            return c;
        }
        let dy = yf - hz + 1.0;
        let c = mix(mantle, crust, (yf - hz) / (h - hz));
        let fade = clamp01(dy / (h * 0.16)).powi(2); // hide the dense lines near the horizon
        let depth = k_depth / dy + 0.3;
        let f = depth - depth.floor();
        let line_h = clamp01(lw - f.min(1.0 - f) / (k_depth / dy.powi(2)) + 0.5);
        let u = (xf - cx) * k_side / dy;
        let f = u - u.floor();
        let line_v = clamp01(lw - f.min(1.0 - f) / (k_side / dy) + 0.5);
        mix(c, acc, line_h.max(line_v) * fade * 0.85)
    })
}

fn topography(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let (mantle, base, surface1, acc) = (
        p.get("mantle"),
        p.get("base"),
        p.get("surface1"),
        p.get(accent),
    );
    let (wf, hf) = (w as f64, h as f64);
    let step = 4; // the field is sampled every 4 px
    let scale = hf / 2.4;
    let field = sample_grid(w, h, step, |x, y| {
        fbm(x as f64 / scale, y as f64 / scale, 4, 5)
    });
    let levels = 18.0;
    let lw = (hf / 1100.0).max(0.9);
    Box::new(move |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        let mut c = mix(mantle, base, yf / hf);
        let (i, j) = (x / step, y / step);
        let tx = (x % step) as f64 / step as f64;
        let ty = (y % step) as f64 / step as f64;
        let (f00, f10, f01, f11) = (
            field[j][i],
            field[j][i + 1],
            field[j + 1][i],
            field[j + 1][i + 1],
        );
        let v = (f00 * (1.0 - tx) + f10 * tx) * (1.0 - ty) + (f01 * (1.0 - tx) + f11 * tx) * ty;
        let dvx = ((f10 - f00) * (1.0 - ty) + (f11 - f01) * ty) / step as f64;
        let dvy = ((f01 - f00) * (1.0 - tx) + (f11 - f10) * tx) / step as f64;
        let g = dvx.hypot(dvy) * levels;
        let lv = v * levels;
        let n = (lv + 0.5).floor();
        let dist = (lv - n).abs() / g.max(1e-6); // distance to the line in pixels
        let index = (n as i64).rem_euclid(5) == 0; // every fifth line is bolder
        let cover = clamp01(lw * if index { 1.6 } else { 1.0 } - dist + 0.5);
        if cover > 0.0 {
            // Lines get more accent toward the lower right.
            let t = clamp01(xf / wf * 0.5 + yf / hf * 0.6 - 0.1);
            let col = mix(surface1, acc, if index { t } else { t * 0.6 });
            c = mix(c, col, cover * if index { 0.9 } else { 0.6 });
        }
        c
    })
}

fn mountains(p: Palette, accent: &str, w: usize, wf: f64, h: f64) -> PixelFn {
    let (crust, mantle, base, acc) = (
        p.get("crust"),
        p.get("mantle"),
        p.get("base"),
        p.get(accent),
    );
    let (far, near, moon) = if p.is_dark() {
        (
            mix(p.get("surface1"), acc, 0.35),
            crust,
            mix(p.get("rosewater"), base, 0.1),
        )
    } else {
        (
            mix(p.get("surface0"), acc, 0.25),
            mix(p.get("overlay0"), acc, 0.15),
            mix(acc, base, 0.35),
        )
    };
    let n_layers = 4;
    let ridges: Vec<(Vec<f64>, Rgb)> = (0..n_layers)
        .map(|l| {
            let lf = l as f64;
            let (top, amp, freq) = (0.40 + lf * 0.12, 0.30 - lf * 0.045, 2.2 + lf * 1.1);
            let row = (0..w)
                .map(|x| {
                    let v = fbm(x as f64 / wf * freq, lf * 3.7, 5, 40 + l);
                    let ridge = 1.0 - (2.0 * v - 1.0).abs(); // ridged noise: sharp peaks
                    h * (top + amp * (1.0 - ridge.powf(2.5)))
                })
                .collect();
            (row, mix(far, near, lf / (n_layers - 1) as f64))
        })
        .collect();
    let (mx, my, mr) = (wf * 0.74, h * 0.22, h * 0.07);
    Box::new(move |x, y| {
        let (xf, yf) = (x as f64, y as f64);
        let t = yf / h;
        let mut c = mix(crust, mantle, t);
        c = mix(c, acc, 0.22 * clamp01((t - 0.2) / 0.5).powi(2)); // glow behind the peaks
        let d = (xf + 0.5 - mx).hypot(yf + 0.5 - my);
        if d < mr * 3.0 {
            c = mix(c, moon, 0.18 * clamp01(1.0 - (d - mr) / (mr * 2.0)).powi(2)); // halo
            c = mix(c, moon, clamp01(mr - d + 0.5));
        }
        for (ridge, col) in &ridges {
            let cover = below(yf, ridge[x]);
            if cover > 0.0 {
                c = mix(c, *col, cover);
            }
        }
        c
    })
}
