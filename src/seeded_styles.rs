//! More wallpaper styles, each drawn from a fixed internal seed: strange attractors, flame fractals, flow lines, domain warping,
//! rose curves, a Hilbert curve, identicons, halftone, superformula outlines,
//! mandalas, a kaleidoscope, low-poly triangles, circle packing, L-systems and
//! Turing patterns. Colors come only from the flavor's palette and the accent.

use std::f64::consts::PI;

use rayon::prelude::*;

use crate::math_styles::{crisp, density, ramp, splat};
use crate::noise::{Rng, clamp01, fbm, hash_seed, mix};
use crate::palette::{Palette, Rgb, accents};
use crate::styles::{PixelFn, companions};

const SEED: u64 = 0;

pub fn build(style: &str, p: Palette, accent: &str, w: usize, h: usize) -> Option<PixelFn> {
    // Fixed for now: the look of these styles does not depend on the machine.
    let seed = SEED;
    Some(match style {
        "dejong" => attractor(p, accent, w, h, Rng::new(seed, 1), |x, y, a| {
            (
                (a[0] * y).sin() - (a[1] * x).cos(),
                (a[2] * x).sin() - (a[3] * y).cos(),
            )
        }),
        "flame" => flame(p, accent, w, h, seed),
        "flowlines" => flowlines(p, accent, w, h, seed),
        "warp" => warp(p, accent, w, h, seed),
        "roses" => roses(p, accent, w, h, seed),
        "hilbert" => hilbert(p, accent, w, h),
        "identicon" => identicon(p, accent, w, h, seed),
        "halftone" => halftone(p, accent, w, h, seed),
        "superformula" => superformula(p, accent, w, h, seed),
        "mandala" => mandala(p, accent, w, h, seed),
        "kaleidoscope" => kaleidoscope(p, accent, w, h, seed),
        "lowpoly" => lowpoly(p, accent, w, h, seed),
        "circlepack" => circlepack(p, accent, w, h, seed),
        "lsystem" => lsystem(p, accent, w, h, seed),
        "grayscott" => grayscott(p, accent, w, h, seed),
        _ => return None,
    })
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

/// `k` accent colors, starting at the chosen accent and stepping around the wheel.
fn cycle(accent: &str, p: Palette, k: usize) -> Vec<Rgb> {
    let all = accents();
    let a = all.iter().position(|x| *x == accent).unwrap_or(0);
    (0..k)
        .map(|i| p.get(all[(a + i * 3) % all.len()]))
        .collect()
}

fn along(cols: &[Rgb], t: f64) -> Rgb {
    let x = clamp01(t) * (cols.len() - 1) as f64;
    let i = (x as usize).min(cols.len() - 2);
    mix(cols[i], cols[i + 1], x - i as f64)
}

fn sky(p: Palette, y: f64, h: f64) -> Rgb {
    mix(p.get("mantle"), p.get("base"), y / h)
}

// ------------------------------------------------------------------ colored line/point buffers

struct Painted {
    w: usize,
    cnt: Vec<f32>,
    tsum: Vec<f32>,
}

impl Painted {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            cnt: vec![0.0; w * h],
            tsum: vec![0.0; w * h],
        }
    }
    /// A soft dot of the given radius at (x, y) carrying the tone `t`.
    fn dot(&mut self, x: f64, y: f64, t: f64, radius: i64) {
        let h = self.cnt.len() / self.w;
        let s2 = (radius as f64 * 0.6).powi(2).max(0.25);
        let (ix, iy) = (x.round() as i64, y.round() as i64);
        for py in iy - radius..=iy + radius {
            for px in ix - radius..=ix + radius {
                if px >= 0 && py >= 0 && (px as usize) < self.w && (py as usize) < h {
                    let d2 = (px as f64 - x).powi(2) + (py as f64 - y).powi(2);
                    let wgt = (-d2 / (2.0 * s2)).exp() as f32;
                    let k = py as usize * self.w + px as usize;
                    self.cnt[k] += wgt;
                    self.tsum[k] += wgt * t as f32;
                }
            }
        }
    }
    fn line(&mut self, a: (f64, f64), b: (f64, f64), t0: f64, t1: f64, radius: i64) {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let n = (len / 0.6).ceil().max(1.0) as usize;
        for i in 0..=n {
            let u = i as f64 / n as f64;
            self.dot(
                a.0 + (b.0 - a.0) * u,
                a.1 + (b.1 - a.1) * u,
                t0 + (t1 - t0) * u,
                radius,
            );
        }
    }
}

/// The painted buffer over the sky: tone along `cols`, opacity from the density.
fn painted(p: Palette, w: usize, h: usize, b: Painted, cols: Vec<Rgb>, gain: f64) -> PixelFn {
    Box::new(move |x, y| {
        let k = y * w + x;
        let c = f64::from(b.cnt[k]);
        let bg = sky(p, y as f64, h as f64);
        if c <= 1e-6 {
            return bg;
        }
        let t = f64::from(b.tsum[k]) / c;
        mix(bg, along(&cols, t), clamp01(c * gain))
    })
}

fn fit(points: &[(f64, f64)], w: usize, h: usize, margin: f64, tail: f64) -> Vec<(f64, f64)> {
    let pct = |v: &mut Vec<f64>, q: f64| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[((v.len() - 1) as f64 * q) as usize]
    };
    let mut xs: Vec<f64> = points.iter().map(|q| q.0).collect();
    let mut ys: Vec<f64> = points.iter().map(|q| q.1).collect();
    let (x0, x1, y0, y1) = (
        pct(&mut xs, tail),
        pct(&mut xs, 1.0 - tail),
        pct(&mut ys, tail),
        pct(&mut ys, 1.0 - tail),
    );
    let s = (w as f64 * margin / (x1 - x0).max(1e-9)).min(h as f64 * margin / (y1 - y0).max(1e-9));
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    points
        .iter()
        .map(|q| {
            (
                w as f64 / 2.0 + (q.0 - cx) * s,
                h as f64 / 2.0 + (q.1 - cy) * s,
            )
        })
        .collect()
}

// ------------------------------------------------------------------ attractors and flames

/// A 2D iterated map with random parameters, drawn as a density.
fn attractor(
    p: Palette,
    accent: &str,
    w: usize,
    h: usize,
    mut rng: Rng,
    step: fn(f64, f64, &[f64; 4]) -> (f64, f64),
) -> PixelFn {
    let mut prm = [0.0; 4];
    for _ in 0..300 {
        prm.iter_mut().for_each(|k| *k = rng.uni(-2.2, 2.2));
        let (mut x, mut y) = (rng.uni(-1.0, 1.0), rng.uni(-1.0, 1.0));
        let mut pts = Vec::new();
        for i in 0..4030 {
            (x, y) = step(x, y, &prm);
            if i > 30 {
                pts.push((x, y));
            }
        }
        let n = pts.len() as f64;
        let (mx, my) = (
            pts.iter().map(|q| q.0).sum::<f64>() / n,
            pts.iter().map(|q| q.1).sum::<f64>() / n,
        );
        let (sx, sy) = (
            (pts.iter().map(|q| (q.0 - mx).powi(2)).sum::<f64>() / n).sqrt(),
            (pts.iter().map(|q| (q.1 - my).powi(2)).sum::<f64>() / n).sqrt(),
        );
        let cells: std::collections::HashSet<(i64, i64)> = pts
            .iter()
            .map(|q| ((q.0 * 20.0) as i64, (q.1 * 20.0) as i64))
            .collect();
        if pts.iter().all(|q| q.0.is_finite() && q.1.is_finite())
            && sx > 0.35
            && sy > 0.35
            && cells.len() > 300
        {
            break;
        }
    }
    let (mut x, mut y) = (0.1, 0.1);
    let n = (w * h * 2 / 5).clamp(20_000, 1_500_000);
    let orbit: Vec<(f64, f64)> = (0..n)
        .map(|_| {
            (x, y) = step(x, y, &prm);
            (x, y)
        })
        .collect();
    let buf = splat(w, h, fit(&orbit, w, h, 0.88, 0.002).into_iter(), 1);
    let max = f64::from(buf.iter().copied().fold(1.0, f32::max));
    density(p, accent, w, buf, move |d| {
        0.03 + 0.97 * ((1.0 + d).ln() / (1.0 + max * 0.5).ln()).min(1.0).powf(0.8)
    })
}

/// Random affine maps with non-linear variations, iterated by the chaos game.
fn flame(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 2);
    type Var = fn(f64, f64) -> (f64, f64);
    let vars: [Var; 5] = [
        |x, y| (x.sin(), y.sin()),
        |x, y| {
            let d = x * x + y * y + 1e-9;
            (x / d, y / d)
        },
        |x, y| {
            let d = x * x + y * y;
            (x * d.sin() - y * d.cos(), x * d.cos() + y * d.sin())
        },
        |x, y| (y.atan2(x) / PI, x.hypot(y) - 1.0),
        |x, y| (x, y),
    ];
    // Random maps often collapse to a few streaks: draw sets until one fills the frame.
    type Map = ([f64; 6], Var, f64);
    let mut best: Option<(usize, Vec<Map>)> = None;
    for _ in 0..60 {
        let k = r.int(3, 6) as usize;
        let maps: Vec<Map> = (0..k)
            .map(|_| {
                let mut m = [0.0; 6];
                m.iter_mut().for_each(|v| *v = r.uni(-1.0, 1.0));
                let norm = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2] + m[3] * m[3]).sqrt();
                if norm > 0.9 {
                    m[..4].iter_mut().for_each(|v| *v *= 0.9 / norm);
                }
                (m, vars[r.int(0, vars.len() as i64) as usize], r.f())
            })
            .collect();
        let (mut x, mut y) = (r.uni(-1.0, 1.0), r.uni(-1.0, 1.0));
        let mut cells = std::collections::HashSet::new();
        for i in 0..6000 {
            let (m, f, _) = &maps[r.int(0, k as i64) as usize];
            (x, y) = f(m[0] * x + m[1] * y + m[4], m[2] * x + m[3] * y + m[5]);
            (x, y) = (x.clamp(-50.0, 50.0), y.clamp(-50.0, 50.0));
            if i > 30 {
                cells.insert(((x * 12.0) as i64, (y * 12.0) as i64));
            }
        }
        if best.as_ref().is_none_or(|b| cells.len() > b.0) {
            best = Some((cells.len(), maps));
        }
        if cells.len() > 1500 {
            break;
        }
    }
    let maps = best.expect("one set").1;
    let k = maps.len();
    let n = (w * h / 2).clamp(40_000, 2_000_000);
    let (mut x, mut y, mut c) = (r.uni(-1.0, 1.0), r.uni(-1.0, 1.0), r.f());
    let mut pts = Vec::with_capacity(n);
    let mut cs = Vec::with_capacity(n);
    for i in 0..n + 30 {
        let (m, f, col) = &maps[r.int(0, k as i64) as usize];
        let (ax, ay) = (m[0] * x + m[1] * y + m[4], m[2] * x + m[3] * y + m[5]);
        (x, y) = f(ax, ay);
        (x, y) = (x.clamp(-50.0, 50.0), y.clamp(-50.0, 50.0));
        c = (c + col) / 2.0;
        if i >= 30 {
            pts.push((x, y));
            cs.push(c);
        }
    }
    let mut b = Painted::new(w, h);
    for (q, c) in fit(&pts, w, h, 0.9, 0.04).into_iter().zip(cs) {
        b.dot(q.0, q.1, c, 1);
    }
    let max = b.cnt.iter().copied().fold(1.0, f32::max) as f64;
    let g = 4.0 / max.max(4.0);
    painted(p, w, h, b, cycle(accent, p, k.max(2)), (g * 6.0).max(0.1))
}

// ------------------------------------------------------------------ noise

fn flowlines(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 3);
    let sd = r.int(1, 999);
    let (wf, hf) = (w as f64, h as f64);
    let s = 3.4;
    let n = (w * h / 100).clamp(1000, 9000);
    let steps = 260;
    let mut b = vec![0f32; w * h];
    let st = 1.6 * hf / 720.0;
    for _ in 0..n {
        let (mut x, mut y) = (r.f() * wf, r.f() * hf);
        for _ in 0..steps {
            let a = fbm(x / hf * s, y / hf * s, 4, sd) * 4.0 * PI;
            x += st * a.cos();
            y += st * a.sin();
            if x >= 0.0 && y >= 0.0 && (x as usize) < w && (y as usize) < h {
                b[y as usize * w + x as usize] += 1.0;
            }
        }
    }
    // a little blur so the strands read as soft light
    let blurred: Vec<f32> = (0..w * h)
        .map(|k| {
            let (x, y) = (k % w, k / w);
            let mut acc = 0.0;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (xx, yy) = (x as i64 + dx, y as i64 + dy);
                    if xx >= 0 && yy >= 0 && (xx as usize) < w && (yy as usize) < h {
                        acc += b[yy as usize * w + xx as usize]
                            * if dx == 0 && dy == 0 { 0.4 } else { 0.075 };
                    }
                }
            }
            acc
        })
        .collect();
    density(p, accent, w, blurred, |d| {
        (1.0 - (-(1.0 + d).ln() * 0.9).exp()).powf(0.8)
    })
}

/// Domain warping: fbm(p + fbm(p + fbm(p))).
fn warp(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 4);
    let sd = r.int(1, 1000);
    let (ox, oy) = (r.uni(0.0, 10.0), r.uni(0.0, 10.0));
    let hf = h as f64;
    let wf = w as f64;
    let (mantle, surface1, acc) = (p.get("mantle"), p.get("surface1"), p.get(accent));
    let (_, third) = companions(accent);
    let comp = p.get(third);
    Box::new(move |x, y| {
        let (u, v) = ((x as f64 - wf / 2.0) / hf, (y as f64 - hf / 2.0) / hf);
        let (px, py) = (u * 3.5 + ox, v * 3.5 + oy);
        let (qx, qy) = (fbm(px, py, 5, sd), fbm(px + 5.2, py + 1.3, 5, sd + 1));
        let (rx, ry) = (
            fbm(px + 4.0 * qx + 1.7, py + 4.0 * qy + 9.2, 5, sd + 2),
            fbm(px + 4.0 * qx + 8.3, py + 4.0 * qy + 2.8, 5, sd + 3),
        );
        let f = fbm(px + 4.0 * rx, py + 4.0 * ry, 5, sd + 4);
        let _ = ry;
        let mut c = mix(mantle, surface1, f);
        c = mix(c, acc, clamp01(qx * qy * 2.2));
        c = mix(c, comp, clamp01(rx * 1.2 - 0.35) * 0.6);
        let k = 0.55 + 0.7 * f * f;
        [c[0] * k, c[1] * k, c[2] * k]
    })
}

fn kaleidoscope(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 5);
    let sym = r.int(3, 12) as f64;
    let sd = r.int(1, 999);
    let (wf, hf) = (w as f64, h as f64);
    let rm = ramp(p, accent);
    let px = 1.0 / hf;
    let seg = 2.0 * PI / sym;
    let field = move |u: f64, v: f64| {
        let (rr, th) = (u.hypot(v), v.atan2(u));
        let th = ((th.rem_euclid(seg)) - seg / 2.0).abs();
        let (x, y) = (rr * th.cos() * 7.6 + 3.1, rr * th.sin() * 7.6 + 7.7);
        let q = fbm(x, y, 5, sd);
        fbm(
            x + 2.2 * q,
            y + 2.2 * fbm(x + 5.0, y + 1.0, 5, sd + 1),
            5,
            sd + 2,
        )
    };
    Box::new(move |x, y| {
        let (u, v) = (
            (x as f64 + 0.5 - wf / 2.0) / hf,
            (y as f64 + 0.5 - hf / 2.0) / hf,
        );
        let t = crisp(field, u, v, px, 9.0);
        let dim = 1.1 - 0.5 * clamp01(u.hypot(v) * 1.3);
        let c = rm(t);
        [c[0] * dim, c[1] * dim, c[2] * dim]
    })
}

// ------------------------------------------------------------------ curves

fn roses(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 6);
    let (wf, hf) = (w as f64, h as f64);
    let sym = r.int(3, 11);
    let mut pts = Vec::new();
    for i in 0..(sym / 2 + 2) {
        let k = r.int(2, 9) as f64 / r.int(1, 5) as f64;
        let n = h * 80;
        let a = i as f64 * 2.0 * PI / (sym as f64 + 1.0) / 3.0;
        for j in 0..n {
            let t = j as f64 / n as f64 * 2.0 * PI * 8.0;
            let rad = (k * t).cos() * (1.0 - i as f64 * 0.12);
            pts.push((
                wf / 2.0 + hf * 0.46 * rad * (t + a).cos(),
                hf / 2.0 + hf * 0.46 * rad * (t + a).sin(),
            ));
        }
    }
    let buf = splat(w, h, pts.into_iter(), 1);
    density(p, accent, w, buf, |d| {
        0.04 + 0.96 * (1.0 - (-d * 0.9).exp())
    })
}

fn hilbert(p: Palette, accent: &str, w: usize, h: usize) -> PixelFn {
    let order = 5;
    let n = 1usize << order;
    let cell = h as f64 * 0.9 / n as f64;
    let ox = (w as f64 - n as f64 * cell) / 2.0;
    let pos = |d: usize| {
        let (mut x, mut y, mut t, mut s) = (0usize, 0usize, d, 1usize);
        while s < n {
            let rx = 1 & (t / 2);
            let ry = 1 & (t ^ rx);
            if ry == 0 {
                if rx == 1 {
                    x = s - 1 - x;
                    y = s - 1 - y;
                }
                std::mem::swap(&mut x, &mut y);
            }
            x += s * rx;
            y += s * ry;
            t /= 4;
            s *= 2;
        }
        (
            ox + (x as f64 + 0.5) * cell,
            h as f64 * 0.05 + (y as f64 + 0.5) * cell,
        )
    };
    let mut b = Painted::new(w, h);
    let total = (n * n - 1) as f64;
    for d in 0..n * n - 1 {
        b.line(
            pos(d),
            pos(d + 1),
            d as f64 / total,
            (d + 1) as f64 / total,
            2,
        );
    }
    let cols = {
        let mut c = cycle(accent, p, 4);
        c.push(c[0]);
        c
    };
    painted(p, w, h, b, cols, 1.2)
}

fn lsystem(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 7);
    type Lsys<'a> = (&'a str, &'a [(char, &'a str)], f64, usize, f64);
    let systems: [Lsys; 5] = [
        (
            "X",
            &[('X', "F+[[X]-X]-F[-FX]+X"), ('F', "FF")],
            25.0,
            5,
            90.0,
        ),
        ("F", &[('F', "F+F-F-F+F")], 90.0, 4, 0.0),
        ("F-G-G", &[('F', "F-G+F+G-F"), ('G', "GG")], 120.0, 6, 0.0),
        ("A", &[('A', "B-A-B"), ('B', "A+B+A")], 60.0, 7, 0.0),
        ("FX", &[('X', "X+YF+"), ('Y', "-FX-Y")], 90.0, 12, 0.0),
    ];
    let (axiom, rules, ang, iters, start) = systems[r.int(0, 5) as usize];
    let mut s = axiom.to_string();
    for _ in 0..iters {
        s = s
            .chars()
            .map(|c| {
                rules
                    .iter()
                    .find(|(k, _)| *k == c)
                    .map_or(c.to_string(), |(_, v)| v.to_string())
            })
            .collect();
        if s.len() > 300_000 {
            break;
        }
    }
    let a = (ang + r.uni(-1.5, 1.5)).to_radians();
    let (mut x, mut y, mut th) = (0.0f64, 0.0f64, start.to_radians());
    let mut stack = Vec::new();
    let mut seg: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for c in s.chars() {
        match c {
            'F' | 'G' | 'A' | 'B' => {
                let (nx, ny) = (x + th.cos(), y + th.sin());
                seg.push(((x, y), (nx, ny)));
                (x, y) = (nx, ny);
            }
            '+' => th += a,
            '-' => th -= a,
            '[' => stack.push((x, y, th)),
            ']' => {
                if let Some(q) = stack.pop() {
                    (x, y, th) = q;
                }
            }
            _ => {}
        }
    }
    let ends: Vec<(f64, f64)> = seg.iter().flat_map(|q| [q.0, q.1]).collect();
    let f = fit(&ends, w, h, 0.92, 0.002);
    let mut b = Painted::new(w, h);
    let n = seg.len() as f64;
    for (i, pair) in f.chunks(2).enumerate() {
        b.line(pair[0], pair[1], i as f64 / n, (i + 1) as f64 / n, 1);
    }
    let cols = vec![
        p.get("surface2"),
        p.get(companions(accent).0),
        p.get(accent),
        p.get(companions(accent).1),
    ];
    painted(p, w, h, b, cols, 2.5)
}

// ------------------------------------------------------------------ per-pixel patterns

fn identicon(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let cw = (h / 18).max(4);
    let nx = w / cw + 2;
    let cols = [
        p.get("mantle"),
        p.get("surface0"),
        p.get(accent),
        p.get(companions(accent).0),
    ];
    Box::new(move |x, y| {
        let (i, j) = (x / cw, y / cw);
        let m = i.min(nx - 1 - i.min(nx - 1)); // mirror about the middle
        let v = hash_seed(m as i64, j as i64, seed);
        cols[if v < 0.45 {
            0
        } else if v < 0.67 {
            1
        } else if v < 0.88 {
            2
        } else {
            3
        }]
    })
}

fn halftone(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 8);
    let sd = r.int(1, 99);
    let ang = r.uni(0.3, 1.1);
    let (wf, hf) = (w as f64, h as f64);
    let rm = ramp(p, accent);
    let mantle = p.get("mantle");
    let s = hf / 48.0;
    Box::new(move |x, y| {
        let (u, v) = ((x as f64 - wf / 2.0) / hf, (y as f64 - hf / 2.0) / hf);
        let f = fbm(u * 2.7 + 1.0, v * 2.7 + 1.0, 5, sd);
        let (xf, yf) = (x as f64, y as f64);
        let (xr, yr) = (
            xf * ang.cos() + yf * ang.sin(),
            -xf * ang.sin() + yf * ang.cos(),
        );
        let (fx, fy) = (
            (xr / s).rem_euclid(1.0) - 0.5,
            (yr / s).rem_euclid(1.0) - 0.5,
        );
        let rad = 0.12 + 0.42 * f.powf(1.3);
        let cover = clamp01((rad - fx.hypot(fy)) * s + 0.5);
        mix(mantle, rm(0.4 + 0.6 * f), cover)
    })
}

fn superformula(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 9);
    let m = r.int(3, 12) as f64;
    let (n1, n2, n3) = (r.uni(0.3, 2.5), r.uni(0.5, 6.0), r.uni(0.5, 6.0));
    let (wf, hf) = (w as f64, h as f64);
    let cols = cycle(accent, p, 6);
    let px = 1.0 / hf;
    let radius = move |th: f64| {
        ((m * th / 4.0).cos().abs().powf(n2) + (m * th / 4.0).sin().abs().powf(n3)).powf(-1.0 / n1)
    };
    // normalize so the biggest outline stays inside the picture
    let rmax = (0..720)
        .map(|i| radius(i as f64 / 720.0 * 2.0 * PI))
        .filter(|v| v.is_finite())
        .fold(1e-9, f64::max);
    let q_of =
        move |u: f64, v: f64| u.hypot(v) / (radius(v.atan2(u)) / rmax * 0.46).max(1e-3) * 14.0;
    let bgc = [p.get("crust"), p.get("mantle")];
    Box::new(move |x, y| {
        let (u, v) = (
            (x as f64 + 0.5 - wf / 2.0) / hf,
            (y as f64 + 0.5 - hf / 2.0) / hf,
        );
        let q = q_of(u, v);
        let bg = mix(bgc[0], bgc[1], y as f64 / hf);
        if !q.is_finite() || q > 16.0 {
            return bg;
        }
        let g = ((q_of(u + px, v) - q).abs() + (q_of(u, v + px) - q).abs()).max(1e-6); // per pixel
        let dist = (q - q.round()).abs() / g * 1.0;
        let line = clamp01(1.4 - dist + 0.5);
        mix(bg, cols[(q.round() as i64).rem_euclid(6) as usize], line)
    })
}

fn mandala(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 10);
    let sym = r.int(3, 11);
    let rings: Vec<(i64, f64)> = (0..7)
        .map(|_| (sym * r.int(1, 3), r.uni(1.0, 2.2)))
        .collect();
    let (wf, hf) = (w as f64, h as f64);
    let cols = cycle(accent, p, 5);
    let (bg0, bg1, surface0) = (p.get("mantle"), p.get("base"), p.get("surface0"));
    Box::new(move |x, y| {
        let (u, v) = (
            (x as f64 + 0.5 - wf / 2.0) / hf,
            (y as f64 + 0.5 - hf / 2.0) / hf,
        );
        let (rr, th) = (u.hypot(v), v.atan2(u));
        let mut c = mix(bg0, bg1, y as f64 / hf);
        for (k, &(n, stretch)) in rings.iter().enumerate() {
            let ring = (k + 1) as f64;
            let seg = 2.0 * PI / n as f64;
            let a = (th - ring * 0.07).rem_euclid(seg) - seg / 2.0;
            let (px, py) = (rr * a.cos() - (ring * 0.062 + 0.04), rr * a.sin());
            let d = (px * stretch).hypot(py) - (0.03 + ring * 0.0075);
            let fill = 1.0 - smooth(-1.0 / hf, 1.0 / hf, d);
            let rim = 1.0 - smooth(0.6 / hf, 2.2 / hf, d.abs());
            c = mix(c, surface0, fill * 0.55);
            c = mix(c, cols[(k + 1) % 5], rim);
        }
        c
    })
}

fn lowpoly(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 11);
    let sd = r.int(1, 99);
    let nx = 22usize;
    let cw = w as f64 / nx as f64;
    let ny = (h as f64 / cw) as usize + 2;
    let mut verts = vec![(0.0, 0.0); (nx + 1) * (ny + 1)];
    for j in 0..=ny {
        for i in 0..=nx {
            let jit = if i == 0 || i == nx || j == 0 || j == ny {
                0.0
            } else {
                0.35
            };
            verts[j * (nx + 1) + i] = (
                (i as f64 + r.uni(-jit, jit)) * cw,
                (j as f64 + r.uni(-jit, jit)) * cw,
            );
        }
    }
    let hf = h as f64;
    let rm = ramp(p, accent);
    let tone: Vec<f64> = (0..nx * ny * 2)
        .map(|k| {
            let (q, tri) = (k / 2, k % 2);
            let (i, j) = (q % nx, q / nx);
            let v = |a: usize, b: usize| verts[(j + b) * (nx + 1) + i + a];
            let t = if tri == 0 {
                [v(0, 0), v(1, 0), v(1, 1)]
            } else {
                [v(0, 0), v(1, 1), v(0, 1)]
            };
            let (cx, cy) = (
                (t[0].0 + t[1].0 + t[2].0) / 3.0,
                (t[0].1 + t[1].1 + t[2].1) / 3.0,
            );
            fbm(cx / hf * 2.2, cy / hf * 2.2, 3, sd) * 0.8
                + hash_seed(i as i64, (j * 2 + tri) as i64, seed) * 0.25
        })
        .collect();
    Box::new(move |x, y| {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        let (i0, j0) = ((px / cw) as i64, (py / cw) as i64);
        for j in (j0 - 1).max(0)..=(j0 + 1).min(ny as i64 - 1) {
            for i in (i0 - 1).max(0)..=(i0 + 1).min(nx as i64 - 1) {
                let v = |a: usize, b: usize| verts[(j as usize + b) * (nx + 1) + i as usize + a];
                for tri in 0..2usize {
                    let t = if tri == 0 {
                        [v(0, 0), v(1, 0), v(1, 1)]
                    } else {
                        [v(0, 0), v(1, 1), v(0, 1)]
                    };
                    let den = (t[1].1 - t[2].1) * (t[0].0 - t[2].0)
                        + (t[2].0 - t[1].0) * (t[0].1 - t[2].1);
                    if den.abs() < 1e-9 {
                        continue;
                    }
                    let l0 = ((t[1].1 - t[2].1) * (px - t[2].0)
                        + (t[2].0 - t[1].0) * (py - t[2].1))
                        / den;
                    let l1 = ((t[2].1 - t[0].1) * (px - t[2].0)
                        + (t[0].0 - t[2].0) * (py - t[2].1))
                        / den;
                    if l0 >= -0.001 && l1 >= -0.001 && 1.0 - l0 - l1 >= -0.001 {
                        return rm(tone[(j as usize * nx + i as usize) * 2 + tri]);
                    }
                }
            }
        }
        rm(0.0)
    })
}

fn circlepack(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 12);
    let sd = r.int(1, 99);
    let (wf, hf) = (w as f64, h as f64);
    let max_r = hf * 0.16;
    let cell = (max_r).max(8.0);
    let gw = (wf / cell) as usize + 2;
    let gh = (hf / cell) as usize + 2;
    let mut grid: Vec<Vec<(f64, f64, f64)>> = vec![Vec::new(); gw * gh];
    let mut circles = Vec::new();
    let reach = (max_r * 2.0 / cell).ceil() as i64 + 1;
    let candidates = (w * h / 15).clamp(2000, 80_000);
    let min_r = (hf / 144.0).max(1.5);
    for _ in 0..candidates {
        let (x, y) = (r.f() * wf, r.f() * hf);
        let mut rad = x.min(y).min(wf - x).min(hf - y).min(max_r);
        let (cx, cy) = ((x / cell) as i64, (y / cell) as i64);
        for gj in (cy - reach).max(0)..=(cy + reach).min(gh as i64 - 1) {
            for gi in (cx - reach).max(0)..=(cx + reach).min(gw as i64 - 1) {
                for &(ox, oy, orad) in &grid[gj as usize * gw + gi as usize] {
                    rad = rad.min((x - ox).hypot(y - oy) - orad);
                }
            }
        }
        if rad < min_r {
            continue;
        }
        rad -= hf / 360.0;
        grid[cy as usize * gw + cx as usize].push((x, y, rad + hf / 360.0));
        circles.push((x, y, rad));
    }
    let mut buf: Vec<Rgb> = (0..w * h).map(|k| sky(p, (k / w) as f64, hf)).collect();
    let cols: Vec<Rgb> = cycle(accent, p, 4)
        .into_iter()
        .chain([p.get("surface1"), p.get("surface2")])
        .collect();
    for (x, y, rad) in circles {
        let f = fbm(x / hf * 2.0, y / hf * 2.0, 3, sd);
        let k = (0.7 + 0.5 * (rad / max_r).sqrt()).min(1.4);
        let c = cols[(f * cols.len() as f64) as usize % cols.len()];
        let c = [c[0] * k, c[1] * k, c[2] * k];
        let (x0, x1) = (
            ((x - rad - 1.0).max(0.0)) as usize,
            ((x + rad + 2.0).min(wf)) as usize,
        );
        let (y0, y1) = (
            ((y - rad - 1.0).max(0.0)) as usize,
            ((y + rad + 2.0).min(hf)) as usize,
        );
        for yy in y0..y1 {
            for xx in x0..x1 {
                let m = clamp01(rad - (xx as f64 + 0.5 - x).hypot(yy as f64 + 0.5 - y) + 0.5);
                if m > 0.0 {
                    buf[yy * w + xx] = mix(buf[yy * w + xx], c, m);
                }
            }
        }
    }
    Box::new(move |x, y| buf[y * w + x])
}

/// Gray-Scott reaction-diffusion: Turing spots and worms, with crisp edges.
fn grayscott(p: Palette, accent: &str, w: usize, h: usize, seed: u64) -> PixelFn {
    let mut r = Rng::new(seed, 13);
    let (f, k) = [
        (0.0367, 0.0649),
        (0.0545, 0.062),
        (0.029, 0.057),
        (0.039, 0.058),
        (0.026, 0.055),
    ][r.int(0, 5) as usize];
    let (gw, gh) = ((w / 4).max(16), (h / 4).max(9));
    let mut u = vec![1.0f64; gw * gh];
    let mut v = vec![0.0f64; gw * gh];
    for _ in 0..(14 + r.int(0, 8)) {
        let (cx, cy) = (
            r.int(5, gw as i64 - 5) as usize,
            r.int(5, gh as i64 - 5) as usize,
        );
        for yy in cy - 3..cy + 3 {
            for xx in cx - 3..cx + 3 {
                u[yy * gw + xx] = 0.5;
                v[yy * gw + xx] = 0.25;
            }
        }
    }
    let lap = |z: &Vec<f64>, x: usize, y: usize| {
        let at = |xx: usize, yy: usize| z[(yy % gh) * gw + xx % gw];
        at(x + 1, y) + at(x + gw - 1, y) + at(x, y + 1) + at(x, y + gh - 1) - 4.0 * at(x, y)
    };
    for _ in 0..6000 {
        let (nu, nv): (Vec<f64>, Vec<f64>) = (0..gw * gh)
            .into_par_iter()
            .map(|c| {
                let (x, y) = (c % gw, c / gw);
                let uvv = u[c] * v[c] * v[c];
                (
                    u[c] + 0.16 * lap(&u, x, y) - uvv + f * (1.0 - u[c]),
                    v[c] + 0.08 * lap(&v, x, y) + uvv - (f + k) * v[c],
                )
            })
            .unzip();
        u = nu;
        v = nv;
    }
    let sample = move |x: f64, y: f64| {
        let (fx, fy) = (
            (x / w as f64 * gw as f64 - 0.5).clamp(0.0, gw as f64 - 1.001),
            (y / h as f64 * gh as f64 - 0.5).clamp(0.0, gh as f64 - 1.001),
        );
        let (i, j, tx, ty) = (fx as usize, fy as usize, fx.fract(), fy.fract());
        let g = |a: usize, b: usize| v[b * gw + a];
        let t = (g(i, j) * (1.0 - tx) + g(i + 1, j) * tx) * (1.0 - ty)
            + (g(i, j + 1) * (1.0 - tx) + g(i + 1, j + 1) * tx) * ty;
        clamp01(t * 3.2)
    };
    let rm = ramp(p, accent);
    Box::new(move |x, y| rm(crisp(&sample, x as f64 + 0.5, y as f64 + 0.5, 1.0, 3.0)))
}
