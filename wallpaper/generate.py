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
  mesh       a soft mesh gradient blending the accent with two other colors
  aurora     curtains of light in the accent and Teal over a night sky
  sunset     a retro sun cut by stripes over a perspective grid
  topography contour lines, like a topographic map, in the accent
  mountains  layered mountain ridges under a moon

A little fixed noise is added to every style to avoid visible banding.
"""
import argparse
import math
import os
import struct
import sys
import zlib

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "lib"))
from catppuccin import ACCENTS, FLAVORS, is_dark, rgb  # noqa: E402


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


def smooth(t):
    return t * t * (3 - 2 * t)


def vnoise(x, y, seed=0):
    """Smooth value noise in 0..1 (bilinear blend of hashed lattice points)."""
    xi, yi = math.floor(x), math.floor(y)
    tx, ty = smooth(x - xi), smooth(y - yi)
    xi += seed * 7919
    a, b = hash01(xi, yi), hash01(xi + 1, yi)
    c, d = hash01(xi, yi + 1), hash01(xi + 1, yi + 1)
    top = a + (b - a) * tx
    return top + (c + (d - c) * tx - top) * ty


def fbm(x, y, octaves=4, seed=0):
    """Fractal noise: several octaves of value noise, normalized to 0..1."""
    total, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        total += amp * vnoise(x, y, seed + o)
        norm += amp
        x, y, amp = x * 2.03, y * 2.03, amp * 0.5
    return total / norm


def second_colors(accent):
    """Two companion colors that always differ from the accent."""
    return ("lavender" if accent == "blue" else "blue",
            "mauve" if accent == "pink" else "pink")


# --- styles ------------------------------------------------------------------
# Each style takes (flavor, accent, w, h) and returns a function
# (x, y) → [r, g, b] floats; noise and clamping are applied afterwards.

def style_glow(flavor, accent, w, h):
    crust, base = rgb(flavor, "crust"), rgb(flavor, "base")
    second, third = second_colors(accent)
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


def style_mesh(flavor, accent, w, h):
    base = rgb(flavor, "base")
    second, third = second_colors(accent)
    # Color points: (x, y) as a fraction of the screen, color.
    points = [
        ((0.80, 0.80), rgb(flavor, accent)),
        ((0.15, 0.20), rgb(flavor, second)),
        ((0.25, 0.90), rgb(flavor, third)),
        ((0.75, 0.10), rgb(flavor, "crust")),
        ((0.45, 0.45), base),
    ]
    calm = 0.62 if is_dark(flavor) else 0.45   # how much Base tones it down

    def blend(x, y):
        fx, fy = x / w * 1.78, y / h
        # A gentle warp makes the blend look organic instead of radial.
        fx += (vnoise(fx * 1.5, fy * 1.5, 3) - 0.5) * 0.25
        fy += (vnoise(fx * 1.5, fy * 1.5, 9) - 0.5) * 0.25
        tw, c = 0.0, [0.0, 0.0, 0.0]
        for (px, py), col in points:
            d2 = (fx - px * 1.78) ** 2 + (fy - py) ** 2
            wgt = 1 / (d2 + 0.02) ** 1.6
            tw += wgt
            for i in range(3):
                c[i] += col[i] * wgt
        return mix([v / tw for v in c], base, calm)

    # The blend is very smooth, so compute it every 4 px and interpolate.
    step = 4
    grid = [[blend(i * step, j * step) for i in range(w // step + 2)] for j in range(h // step + 2)]

    def pixel(x, y):
        gx, gy = x / step, y / step
        i, j = int(gx), int(gy)
        tx, ty = gx - i, gy - j
        top = mix(grid[j][i], grid[j][i + 1], tx)
        return mix(top, mix(grid[j + 1][i], grid[j + 1][i + 1], tx), ty)
    return pixel


def style_aurora(flavor, accent, w, h):
    crust, mantle, base = rgb(flavor, "crust"), rgb(flavor, "mantle"), rgb(flavor, "base")
    other = "green" if accent == "teal" else "teal"
    # Two curtains that take turns along the screen: where one is lit the
    # other fades, so they never stack into dark bands.
    # (height of the lower edge, wave amplitude, frequency, phase, color, brightness)
    curtains = [
        (0.46, 0.09, 1.2, 0.20, rgb(flavor, other), 0.90),
        (0.38, 0.08, 1.7, 0.65, rgb(flavor, accent), 1.00),
    ]
    edges = [[h * (b + amp * math.sin(2 * math.pi * (x / w * f + ph))
                   + amp * 0.45 * math.sin(2 * math.pi * (x / w * f * 2.7 + ph * 3)))
              for x in range(w)]
             for b, amp, f, ph, _, _ in curtains]
    env = [clamp01((fbm(x / w * 2.5, 0.5, 2, 11) - 0.5) * 3 + 0.5) for x in range(w)]
    lit = [env, [1 - v for v in env]]
    # Fine vertical streaks shared by every curtain.
    rays = [0.3 + 0.7 * fbm(x / w * 70, 0.5, 2, 21) ** 1.5 for x in range(w)]
    tall, sharp = h * 0.20, h * 0.05
    ground = [h * (0.84 + 0.025 * math.sin(2 * math.pi * (x / w * 1.3 + 0.2))
                   + 0.012 * math.sin(2 * math.pi * (x / w * 4.1))) for x in range(w)]
    dark = is_dark(flavor)
    ground_col = mix(crust, mantle, 0.2) if dark else mix(rgb(flavor, "surface1"), crust, 0.3)

    def pixel(x, y):
        c = mix(crust, base, y / h * 0.8)
        for edge, row, (_, _, _, _, col, bright) in zip(edges, lit, curtains):
            dy = edge[x] - y
            # Bright lower edge, soft fade upward, quicker fade downward.
            k = math.exp(-dy / tall) if dy >= 0 else math.exp(-(dy / sharp) ** 2)
            k = min(k * bright * rays[x] * row[x], 1.0)
            if k > 0.004:
                if dark:
                    # Screen blend: overlapping curtains add up like light.
                    c = [255 - (255 - c[i]) * (255 - col[i] * k) / 255 for i in range(3)]
                else:
                    c = mix(c, col, k)
        return mix(c, ground_col, clamp01(y - ground[x] + 0.5))
    return pixel


def style_sunset(flavor, accent, w, h):
    crust, mantle = rgb(flavor, "crust"), rgb(flavor, "mantle")
    acc = rgb(flavor, accent)
    top_name = "peach" if accent == "yellow" else "yellow"
    sun_top = rgb(flavor, top_name)
    hz = h * 0.64                       # horizon
    cx, cy, r = w * 0.5, hz - h * 0.10, h * 0.22
    lw = max(1.0, h / 900)              # grid line width in pixels
    k_depth = h * 0.36 * 1.5            # perspective of horizontal lines
    k_side = (h * 0.36) / (w * 0.07)    # spacing of converging lines

    def pixel(x, y):
        if y < hz:
            t = y / hz
            c = mix(crust, mantle, t)
            c = mix(c, acc, 0.28 * t ** 3)          # warm haze near the horizon
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
            cover = clamp01(r - d + 0.5)
            if cover:
                # Horizontal cuts in the lower half, wider toward the horizon.
                if y > cy:
                    p = h * 0.045
                    gap = p * (0.12 + 0.55 * (y - cy) / r)
                    cover *= clamp01((y - cy) % p - gap + 0.5)
                sun = mix(sun_top, acc, clamp01((y - (cy - r)) / (2 * r)))
                c = mix(c, sun, cover)
            return c
        dy = y - hz + 1
        c = mix(mantle, crust, (y - hz) / (h - hz))
        fade = clamp01(dy / (h * 0.16)) ** 2   # hide the dense lines near the horizon
        depth = k_depth / dy + 0.3
        f = depth - math.floor(depth)
        line_h = clamp01(lw - min(f, 1 - f) / (k_depth / dy ** 2) + 0.5)
        u = (x - cx) * k_side / dy
        f = u - math.floor(u)
        line_v = clamp01(lw - min(f, 1 - f) / (k_side / dy) + 0.5)
        return mix(c, acc, max(line_h, line_v) * fade * 0.85)
    return pixel


def style_topography(flavor, accent, w, h):
    mantle, base, surface1 = rgb(flavor, "mantle"), rgb(flavor, "base"), rgb(flavor, "surface1")
    acc = rgb(flavor, accent)
    step = 4                            # the field is sampled every 4 px
    scale = h / 2.4
    gw, gh = w // step + 2, h // step + 2
    field = [[fbm(i * step / scale, j * step / scale, 4, 5) for i in range(gw)] for j in range(gh)]
    levels = 18
    lw = max(0.9, h / 1100)

    def pixel(x, y):
        c = mix(mantle, base, y / h)
        gx, gy = x / step, y / step
        i, j = int(gx), int(gy)
        tx, ty = gx - i, gy - j
        f00, f10 = field[j][i], field[j][i + 1]
        f01, f11 = field[j + 1][i], field[j + 1][i + 1]
        v = (f00 * (1 - tx) + f10 * tx) * (1 - ty) + (f01 * (1 - tx) + f11 * tx) * ty
        dvx = ((f10 - f00) * (1 - ty) + (f11 - f01) * ty) / step
        dvy = ((f01 - f00) * (1 - tx) + (f11 - f10) * tx) / step
        g = math.hypot(dvx, dvy) * levels
        lv = v * levels
        n = math.floor(lv + 0.5)
        dist = abs(lv - n) / max(g, 1e-6)       # distance to the line in pixels
        index = n % 5 == 0                      # every fifth line is bolder
        cover = clamp01((lw * (1.6 if index else 1.0)) - dist + 0.5)
        if cover:
            # Lines get more accent toward the lower right.
            t = clamp01((x / w) * 0.5 + (y / h) * 0.6 - 0.1)
            col = mix(surface1, acc, t if index else t * 0.6)
            c = mix(c, col, cover * (0.9 if index else 0.6))
        return c
    return pixel


def style_mountains(flavor, accent, w, h):
    crust, mantle, base = rgb(flavor, "crust"), rgb(flavor, "mantle"), rgb(flavor, "base")
    acc = rgb(flavor, accent)
    dark = is_dark(flavor)
    if dark:
        far, near = mix(rgb(flavor, "surface1"), acc, 0.35), crust
        moon = mix(rgb(flavor, "rosewater"), base, 0.1)
    else:
        far, near = mix(rgb(flavor, "surface0"), acc, 0.25), mix(rgb(flavor, "overlay0"), acc, 0.15)
        moon = mix(acc, base, 0.35)
    n_layers = 4
    ridges = []
    for l in range(n_layers):
        top = 0.40 + l * 0.12
        amp = 0.30 - l * 0.045
        freq = 2.2 + l * 1.1
        row = []
        for x in range(w):
            v = fbm(x / w * freq, l * 3.7, 5, 40 + l)
            ridge = 1 - abs(2 * v - 1)          # ridged noise: sharp peaks
            row.append(h * (top + amp * (1 - ridge ** 2.5)))
        ridges.append(row)
    colors = [mix(far, near, l / (n_layers - 1)) for l in range(n_layers)]
    mx, my, mr = w * 0.74, h * 0.22, h * 0.07

    def pixel(x, y):
        t = y / h
        c = mix(crust, mantle, t)
        c = mix(c, acc, 0.22 * clamp01((t - 0.2) / 0.5) ** 2)   # glow behind the peaks
        d = math.hypot(x + 0.5 - mx, y + 0.5 - my)
        if d < mr * 3:
            c = mix(c, moon, 0.18 * clamp01(1 - (d - mr) / (mr * 2)) ** 2)  # halo
            c = mix(c, moon, clamp01(mr - d + 0.5))
        for ridge, col in zip(ridges, colors):
            cover = clamp01(y - ridge[x] + 0.5)
            if cover:
                c = mix(c, col, cover)
        return c
    return pixel


STYLES = {
    "glow": style_glow,
    "waves": style_waves,
    "stripes": style_stripes,
    "dots": style_dots,
    "triangles": style_triangles,
    "mesh": style_mesh,
    "aurora": style_aurora,
    "sunset": style_sunset,
    "topography": style_topography,
    "mountains": style_mountains,
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
