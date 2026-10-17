#!/usr/bin/env python3
"""Render a preview grid of every wallpaper style in every flavor.

Rows are flavors, columns are styles, each thumbnail rendered at a small
size with the same code as the full wallpaper. Used for the README image.

Usage:
    python3 wallpaper/preview.py wallpaper/previews.png [--accent mauve] [--thumb 384x216]
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from generate import ACCENTS, FLAVORS, STYLES, png_from_raw, render, rgb  # noqa: E402

GAP = 8


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("output")
    p.add_argument("--accent", default="mauve", choices=ACCENTS)
    p.add_argument("--thumb", default="384x216", help="thumbnail WIDTHxHEIGHT")
    a = p.parse_args()
    tw, th = (int(v) for v in a.thumb.lower().split("x"))
    styles, flavors = list(STYLES), list(FLAVORS)
    w = len(styles) * tw + (len(styles) + 1) * GAP
    h = len(flavors) * th + (len(flavors) + 1) * GAP
    bg = bytes(rgb("mocha", "crust"))

    # Each thumbnail as a list of RGB rows (render() output minus the PNG
    # filter byte that starts each row).
    thumbs = {}
    for fl in flavors:
        for st in styles:
            raw = render(tw, th, fl, a.accent, st)
            stride = tw * 3 + 1
            thumbs[fl, st] = [raw[r * stride + 1:(r + 1) * stride] for r in range(th)]

    rows = []
    gap_row = b"\x00" + bg * w
    for fi, fl in enumerate(flavors):
        rows.extend([gap_row] * GAP)
        for r in range(th):
            line = bg * GAP
            for st in styles:
                line += thumbs[fl, st][r] + bg * GAP
            rows.append(b"\x00" + line)
    rows.extend([gap_row] * GAP)
    with open(a.output, "wb") as f:
        f.write(png_from_raw(w, h, b"".join(rows)))


if __name__ == "__main__":
    main()
