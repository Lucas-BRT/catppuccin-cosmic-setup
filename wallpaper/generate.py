#!/usr/bin/env python3
"""Generate a Catppuccin wallpaper (PNG, 2560x1440 by default).

Uses only the standard library (zlib/struct), so it runs on any Python 3
with nothing to install. Output is deterministic: the same options always
produce the exact same file.

Usage:
    python3 wallpaper/generate.py output.png
    python3 wallpaper/generate.py output.png --style waves --flavor latte --accent blue
    python3 wallpaper/generate.py output.png --size 3840x2160
    python3 wallpaper/generate.py --list-styles

Styles (every one uses only the chosen flavor's palette and accent):
  glow       diagonal Crust→Base gradient with soft glows: the accent bottom
             right, Blue top left and a faint Pink along the bottom edge
             (Lavender / Mauve stand in when the accent is Blue / Pink)
  waves      layered hills rising from the bottom, fading into the accent
  stripes    a diagonal ribbon of the accent and its neighbor colors
  dots       halftone dots in the accent, growing toward one corner
  triangles  low-poly triangles in surface tones, tinted by the accent
             toward one corner

A little fixed noise is added to every style to avoid visible banding.
"""
import argparse
import math
import os
import struct
import sys
import zlib

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "lib"))
from catppuccin import ACCENTS, FLAVORS, rgb  # noqa: E402


def mix(a, b, t):
    """Linear blend between two colors, t in 0..1."""
    return [a[i] + (b[i] - a[i]) * t for i in range(3)]


def clamp01(v):
    return 0.0 if v < 0 else 1.0 if v > 1 else v


def hash01(i, j):
    """Deterministic pseudo-random number in 0..1 for integer cell (i, j)."""
    n = (i * 374761393 + j * 668265263) & 0xFFFFFFFF
    n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((n ^ (n >> 16)) & 0xFFFF) / 0xFFFF


# --- styles ------------------------------------------------------------------
# Each style takes (flavor, accent, w, h) and returns a function
# (x, y) → [r, g, b] floats; noise and clamping are applied afterwards.

def style_glow(flavor, accent, w, h):
    crust, base = rgb(flavor, "crust"), rgb(flavor, "base")
    second = "lavender" if accent == "blue" else "blue"
    third = "mauve" if accent == "pink" else "pink"
    # (center x, center y) as a fraction of the screen, radius, color, strength
    glows = [
        ((0.82, 0.85), 0.55, rgb(flavor, accent), 0.35),
        ((0.12, 0.10), 0.50, rgb(flavor, second), 0.22),
        ((0.50, 1.10), 0.45, rgb(flavor, third), 0.12),
    ]

    def pixel(x, y):
        fx, fy = x / w, y / h
        # Gradient: Crust at both ends of the diagonal, Base in the middle.
        t = fx * 0.4 + fy * 0.6
        c = [crust[i] + (base[i] - crust[i]) * (1 - abs(t - 0.5) * 2) ** 0.5 for i in range(3)]
        for (gx, gy), r, col, a in glows:
            # 1.78 ≈ 16/9 keeps the glows round on a widescreen display.
            d = math.hypot((fx - gx) * 1.78, fy - gy) / r
            if d < 1:
                k = a * (1 - d) ** 2.2
                for i in range(3):
                    c[i] += (col[i] - c[i]) * k
        return c
    return pixel


def style_waves(flavor, accent, w, h):
    crust, base, mantle = rgb(flavor, "crust"), rgb(flavor, "base"), rgb(flavor, "mantle")
    acc = rgb(flavor, accent)
    # Five hills, back to front: (base height, amplitude, frequency, phase,
    # amount of accent). Back hills are faint, the front one is the boldest.
    layers = [
        (0.50, 0.050, 0.9, 0.10, 0.10),
        (0.60, 0.045, 1.3, 0.55, 0.20),
        (0.70, 0.040, 1.7, 0.25, 0.32),
        (0.79, 0.035, 2.2, 0.80, 0.42),
        (0.88, 0.030, 2.8, 0.40, 0.54),
    ]
    colors = [mix(mantle, acc, k) for *_, k in layers]
    # Height of every hill at every column, computed once.
    edges = [
        [h * (b + amp * math.sin(2 * math.pi * (x / w * f + ph))
              + amp * 0.35 * math.sin(2 * math.pi * (x / w * f * 2.3 + ph * 1.7)))
         for x in range(w)]
        for b, amp, f, ph, _ in layers
    ]

    def pixel(x, y):
        c = mix(crust, base, y / h)          # sky: Crust at the top to Base
        for edge, col in zip(edges, colors):
            cover = clamp01(y - edge[x] + 0.5)   # 1px anti-aliased edge
            if cover:
                c = mix(c, col, cover)
        return c
    return pixel


def style_stripes(flavor, accent, w, h):
    crust, base = rgb(flavor, "crust"), rgb(flavor, "base")
    # The accent in the middle with two neighbors on each side, in the
    # order of the official accent list (which wraps around).
    i = ACCENTS.index(accent)
    names = [ACCENTS[(i + k) % len(ACCENTS)] for k in (-2, -1, 0, 1, 2)]
    cols = [rgb(flavor, n) for n in names]
    sw = h * 0.04                       # stripe width, across the stripe
    start = h * 0.98                    # where the ribbon begins (lower right)
    inv = 1 / math.sqrt(2)

    def pixel(x, y):
        bg = mix(crust, base, clamp01((x / w) * 0.5 + (y / h) * 0.5))
        u = (x + y) * inv - start * 1.4  # distance along the 45° diagonal
        k = u / sw
        idx = math.floor(k)
        if not 0 <= idx < len(cols):
            return bg
        # Blend over about one pixel into the neighbor (or the background)
        # on whichever edge of the stripe this pixel is closer to.
        frac = k - idx
        if frac < 0.5:
            other = cols[idx - 1] if idx > 0 else bg
            return mix(other, cols[idx], clamp01(frac * sw + 0.5))
        other = cols[idx + 1] if idx < len(cols) - 1 else bg
        return mix(other, cols[idx], clamp01((1 - frac) * sw + 0.5))
    return pixel


def style_dots(flavor, accent, w, h):
    base, mantle, surface0 = rgb(flavor, "base"), rgb(flavor, "mantle"), rgb(flavor, "surface0")
    acc = rgb(flavor, accent)
    s = h / 26                          # grid spacing
    maxd = math.hypot(w, h) * 0.75

    def pixel(x, y):
        c = mix(mantle, base, y / h)
        cx, cy = (math.floor(x / s) + 0.5) * s, (math.floor(y / s) + 0.5) * s
        # Dot size grows toward the bottom-right corner, fading to nothing.
        f = clamp01(1 - math.hypot(w - cx, h - cy) / maxd)
        f = f * f
        r = s * 0.46 * f
        if r > 0.3:
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
            cover = clamp01(r - d + 0.5)
            if cover:
                c = mix(c, mix(surface0, acc, 0.25 + 0.75 * f), cover)
        return c
    return pixel


def style_triangles(flavor, accent, w, h):
    crust, mantle, base, surface0 = (rgb(flavor, n) for n in ("crust", "mantle", "base", "surface0"))
    acc = rgb(flavor, accent)
    s = h / 7                           # square cell, split into two triangles

    def pixel(x, y):
        i, j = math.floor(x / s), math.floor(y / s)
        fx, fy = x / s - i, y / s - j
        # Alternate the diagonal's direction from cell to cell so the
        # triangles don't line up into a plain grid.
        upper = fx > fy if (i + j) % 2 == 0 else fx + fy < 1
        v = hash01(i * 2 + upper, j)
        c = mix(crust, mix(mantle, surface0, v), 0.55 + 0.45 * v)
        c = mix(c, base, 0.35)
        # Accent tint toward the bottom-right corner.
        g = clamp01(((i + 0.5) * s / w) * 0.6 + ((j + 0.5) * s / h) * 0.6 - 0.45)
        c = mix(c, acc, g * g * 0.55)
        return c
    return pixel


STYLES = {
    "glow": style_glow,
    "waves": style_waves,
    "stripes": style_stripes,
    "dots": style_dots,
    "triangles": style_triangles,
}


# --- output ------------------------------------------------------------------

def render(w, h, flavor, accent, style="glow"):
    pixel = STYLES[style](flavor, accent, w, h)

    def finish(x, y):
        # Cheap fixed noise (a hash of the coordinates), between -2 and +2.
        n = ((x * 73856093 ^ y * 19349663) & 7) - 3.5
        return bytes(max(0, min(255, int(v + n * 0.6))) for v in pixel(x, y))

    return b"".join(b"\x00" + b"".join(finish(x, y) for x in range(w)) for y in range(h))


def png(w, h, flavor, accent, style="glow"):
    return png_from_raw(w, h, render(w, h, flavor, accent, style))


def png_from_raw(w, h, raw):
    """Wrap filtered RGB rows (each starting with a 0 filter byte) as a PNG."""
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 6))
        + chunk(b"IEND", b"")
    )


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("output", nargs="?")
    p.add_argument("--style", default="glow", choices=STYLES)
    p.add_argument("--flavor", default="mocha", choices=FLAVORS)
    p.add_argument("--accent", default="mauve", choices=ACCENTS)
    p.add_argument("--size", default="2560x1440", help="WIDTHxHEIGHT (default: 2560x1440)")
    p.add_argument("--list-styles", action="store_true", help="print the style names and exit")
    a = p.parse_args()
    if a.list_styles:
        print("\n".join(STYLES))
        return
    if not a.output:
        p.error("the output file is required")
    w, h = (int(v) for v in a.size.lower().split("x"))
    with open(a.output, "wb") as f:
        f.write(png(w, h, a.flavor, a.accent, a.style))


if __name__ == "__main__":
    main()
