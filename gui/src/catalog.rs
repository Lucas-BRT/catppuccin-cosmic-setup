// SPDX-License-Identifier: MIT

//! The 50 wallpaper styles, grouped for browsing, with a plain name wherever
//! the generator's own name is jargon.

/// (category id, [(style, display name)]). Category names are translated in
/// the .ftl as `cat-<id>`.
pub const CATEGORIES: [(&str, &[(&str, &str)]); 5] = [
    (
        "calm",
        &[
            ("glow", "Glow"),
            ("mesh", "Mesh"),
            ("waves", "Waves"),
            ("aurora", "Aurora"),
            ("sunset", "Sunset"),
            ("mountains", "Mountains"),
            ("plasma", "Plasma"),
            ("warp", "Liquid"),
            ("marble", "Marble"),
            ("topography", "Contours"),
        ],
    ),
    (
        "patterns",
        &[
            ("stripes", "Stripes"),
            ("dots", "Dots"),
            ("triangles", "Triangles"),
            ("hexgrid", "Hexagons"),
            ("truchet", "Arcs"),
            ("opart", "Op art"),
            ("halftone", "Halftone"),
            ("lowpoly", "Low poly"),
            ("circlepack", "Bubbles"),
            ("identicon", "Pixel blocks"),
            ("hilbert", "Maze curve"),
            ("grating", "Fine lines"),
        ],
    ),
    (
        "fractals",
        &[
            ("julia", "Julia set"),
            ("mandelbrot", "Mandelbrot"),
            ("burningship", "Burning ship"),
            ("newton", "Newton basins"),
            ("sierpinski", "Sierpinski"),
            ("xorfractal", "XOR grid"),
            ("lsystem", "Ferns"),
            ("automaton", "Cellular"),
            ("inversion", "Circle inversion"),
            ("domain", "Complex field"),
        ],
    ),
    (
        "lines",
        &[
            ("spirograph", "Spirograph"),
            ("lissajous", "Lissajous"),
            ("roses", "Roses"),
            ("superformula", "Superflower"),
            ("mandala", "Mandala"),
            ("kaleidoscope", "Kaleidoscope"),
            ("spiral", "Spiral"),
            ("phyllotaxis", "Sunflower"),
            ("clifford", "Clifford attractor"),
            ("dejong", "De Jong attractor"),
            ("flame", "Flame"),
        ],
    ),
    (
        "organic",
        &[
            ("voronoi", "Cells"),
            ("grayscott", "Coral"),
            ("flowlines", "Flow"),
            ("moire", "Moiré"),
            ("quasicrystal", "Crystal waves"),
            ("ripples", "Ripples"),
            ("gyroid", "Lattice"),
        ],
    ),
];

/// The display name of a style.
pub fn display(style: &str) -> &'static str {
    CATEGORIES
        .iter()
        .flat_map(|(_, items)| items.iter())
        .find(|(raw, _)| *raw == style)
        .map_or("", |(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_style_is_in_exactly_one_category() {
        let listed: Vec<_> = ctp_cosmic::styles::list()
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        let mut seen: Vec<&str> = CATEGORIES
            .iter()
            .flat_map(|(_, items)| items.iter().map(|(raw, _)| *raw))
            .collect();
        seen.sort_unstable();
        let mut want = listed;
        want.sort_unstable();
        assert_eq!(seen, want);
    }
}
