//! Small math helpers and deterministic noise shared by the styles.

use crate::palette::Rgb;

/// Linear blend between two colors, `t` in 0..1.
pub fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// Deterministic pseudo-random number in 0..1 for an integer cell (i, j).
pub fn hash01(i: i64, j: i64) -> f64 {
    let n = u64::from(
        i.wrapping_mul(374_761_393)
            .wrapping_add(j.wrapping_mul(668_265_263)) as u32,
    );
    let n = ((n ^ (n >> 13)) * 1_274_126_177) & 0xFFFF_FFFF;
    ((n ^ (n >> 16)) & 0xFFFF) as f64 / 65535.0
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Smooth value noise in 0..1 (bilinear blend of hashed lattice points).
pub fn vnoise(x: f64, y: f64, seed: i64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (tx, ty) = (smooth(x - xf), smooth(y - yf));
    let (xi, yi) = (xf as i64 + seed * 7919, yf as i64);
    let (a, b) = (hash01(xi, yi), hash01(xi + 1, yi));
    let (c, d) = (hash01(xi, yi + 1), hash01(xi + 1, yi + 1));
    let top = a + (b - a) * tx;
    top + (c + (d - c) * tx - top) * ty
}

/// Fractal noise: several octaves of value noise, normalized to 0..1.
pub fn fbm(mut x: f64, mut y: f64, octaves: i64, seed: i64) -> f64 {
    let (mut total, mut amp, mut norm) = (0.0, 1.0, 0.0);
    for o in 0..octaves {
        total += amp * vnoise(x, y, seed + o);
        norm += amp;
        x *= 2.03;
        y *= 2.03;
        amp *= 0.5;
    }
    total / norm
}

/// Fixed per-pixel dither in -3.5..3.5, scaled when applied, so gradients
/// don't band.
pub fn dither(x: usize, y: usize) -> f64 {
    let n = ((x as u64).wrapping_mul(73_856_093) ^ (y as u64).wrapping_mul(19_349_663)) & 7;
    n as f64 - 3.5
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values from the original Python generator, which produced
    // the same pixels; they pin the noise so images never change by accident.
    #[test]
    fn noise_matches_reference() {
        assert_eq!(hash01(0, 0), 0.0);
        assert_eq!(hash01(12345, -678), 0.783_138_780_804_150_4);
        assert_eq!(vnoise(3.7, 1.2, 5), 0.621_361_161_882_963_2);
        assert_eq!(fbm(0.3, 0.9, 4, 5), 0.538_798_053_950_044_8);
    }

    #[test]
    fn dither_stays_small() {
        for (x, y) in [(0, 0), (1, 0), (7, 13), (2559, 1439)] {
            assert!((-3.5..=3.5).contains(&dither(x, y)));
        }
    }
}
