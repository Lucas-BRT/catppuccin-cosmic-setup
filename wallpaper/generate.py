#!/usr/bin/env python3
"""Generate the Catppuccin Mocha wallpaper (2560x1440 PNG).

Uses only the standard library (zlib/struct), so it runs on any Python 3
with nothing to install. Output is deterministic: running it again
produces the exact same file.

Usage:
    python3 wallpaper/generate.py output.png [width height]

Design:
  * diagonal gradient between Crust (#11111b) and Base (#1e1e2e);
  * three soft glows: Mauve bottom right, Blue top left and a faint Pink
    along the bottom edge;
  * a little fixed noise to avoid visible banding in the gradient.
"""
import math
import struct
import sys
import zlib

CRUST = (17, 17, 27)  # #11111b
BASE = (30, 30, 46)   # #1e1e2e

# (center x, center y) as a fraction of the screen, radius, color, strength
GLOWS = [
    ((0.82, 0.85), 0.55, (203, 166, 247), 0.35),  # Mauve #cba6f7
    ((0.12, 0.10), 0.50, (137, 180, 250), 0.22),  # Blue  #89b4fa
    ((0.50, 1.10), 0.45, (245, 194, 231), 0.12),  # Pink  #f5c2e7
]


def pixel(x, y, w, h):
    fx, fy = x / w, y / h
    # Gradient: darker (Crust) at both ends of the diagonal, Base in the middle.
    t = fx * 0.4 + fy * 0.6
    c = [CRUST[i] + (BASE[i] - CRUST[i]) * (1 - abs(t - 0.5) * 2) ** 0.5 for i in range(3)]
    for (gx, gy), r, col, a in GLOWS:
        # 1.78 ≈ 16/9 keeps the glows round on a widescreen display.
        d = math.hypot((fx - gx) * 1.78, fy - gy) / r
        if d < 1:
            k = a * (1 - d) ** 2.2
            for i in range(3):
                c[i] += (col[i] - c[i]) * k
    # Cheap fixed noise (a hash of the coordinates), between -2 and +2.
    n = ((x * 73856093 ^ y * 19349663) & 7) - 3.5
    return bytes(max(0, min(255, int(v + n * 0.6))) for v in c)


def png(w, h):
    raw = b"".join(
        b"\x00" + b"".join(pixel(x, y, w, h) for x in range(w)) for y in range(h)
    )

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 6))
        + chunk(b"IEND", b"")
    )


if __name__ == "__main__":
    out = sys.argv[1]
    w, h = (int(sys.argv[2]), int(sys.argv[3])) if len(sys.argv) == 4 else (2560, 1440)
    with open(out, "wb") as f:
        f.write(png(w, h))
