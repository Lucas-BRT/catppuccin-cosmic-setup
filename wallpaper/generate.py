#!/usr/bin/env python3
"""Generate a Catppuccin wallpaper (PNG, 2560x1440 by default).

Uses only the standard library (zlib/struct), so it runs on any Python 3
with nothing to install. Output is deterministic: the same options always
produce the exact same file.

Usage:
    python3 wallpaper/generate.py output.png
    python3 wallpaper/generate.py output.png --flavor latte --accent blue --size 3840x2160

Design:
  * diagonal gradient between the flavor's Crust and Base;
  * three soft glows: the accent bottom right, Blue top left and a faint
    Pink along the bottom edge (Lavender / Mauve stand in when the accent
    is already Blue / Pink, so the three glows always differ);
  * a little fixed noise to avoid visible banding in the gradient.
"""
import argparse
import math
import os
import struct
import sys
import zlib

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "lib"))
from catppuccin import ACCENTS, FLAVORS, rgb  # noqa: E402


def glows(flavor, accent):
    """(center x, center y) as a fraction of the screen, radius, color, strength."""
    second = "lavender" if accent == "blue" else "blue"
    third = "mauve" if accent == "pink" else "pink"
    return [
        ((0.82, 0.85), 0.55, rgb(flavor, accent), 0.35),
        ((0.12, 0.10), 0.50, rgb(flavor, second), 0.22),
        ((0.50, 1.10), 0.45, rgb(flavor, third), 0.12),
    ]


def pixel(x, y, w, h, crust, base, glow_list):
    fx, fy = x / w, y / h
    # Gradient: Crust at both ends of the diagonal, Base in the middle.
    t = fx * 0.4 + fy * 0.6
    c = [crust[i] + (base[i] - crust[i]) * (1 - abs(t - 0.5) * 2) ** 0.5 for i in range(3)]
    for (gx, gy), r, col, a in glow_list:
        # 1.78 ≈ 16/9 keeps the glows round on a widescreen display.
        d = math.hypot((fx - gx) * 1.78, fy - gy) / r
        if d < 1:
            k = a * (1 - d) ** 2.2
            for i in range(3):
                c[i] += (col[i] - c[i]) * k
    # Cheap fixed noise (a hash of the coordinates), between -2 and +2.
    n = ((x * 73856093 ^ y * 19349663) & 7) - 3.5
    return bytes(max(0, min(255, int(v + n * 0.6))) for v in c)


def png(w, h, flavor, accent):
    crust, base, glow_list = rgb(flavor, "crust"), rgb(flavor, "base"), glows(flavor, accent)
    raw = b"".join(
        b"\x00" + b"".join(pixel(x, y, w, h, crust, base, glow_list) for x in range(w))
        for y in range(h)
    )

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
    p.add_argument("output")
    p.add_argument("--flavor", default="mocha", choices=FLAVORS)
    p.add_argument("--accent", default="mauve", choices=ACCENTS)
    p.add_argument("--size", default="2560x1440", help="WIDTHxHEIGHT (default: 2560x1440)")
    a = p.parse_args()
    w, h = (int(v) for v in a.size.lower().split("x"))
    with open(a.output, "wb") as f:
        f.write(png(w, h, a.flavor, a.accent))


if __name__ == "__main__":
    main()
