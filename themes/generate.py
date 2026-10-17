#!/usr/bin/env python3
"""Generate Catppuccin themes in the COSMIC 1.10 format (ThemeBuilder v2).

Why this exists: the official .ron files from catppuccin/cosmic-desktop
target the old format (ThemeBuilder v1). COSMIC 1.10 still imports them,
but things get lost:

  * `is_frosted` no longer exists and is silently ignored. Transparency
    now uses `frosted` (blur strength), `frosted_windows`,
    `frosted_system_interface`, `frosted_panel`, `frosted_applets`,
    `frosted_maximized_apps` and `alpha_map` (opacity for each strength).
    Since those fields are missing, importing the old theme TURNS OFF
    Frosted Glass for anyone who had it on.
  * The palette dropped `blue`, `red`, `green`, `yellow` and `gray_3`
    (also ignored).
  * COSMIC stores colors as "#RRGGBBAA". The old format
    (red: 0.5, green: ...) is still accepted, but this generator uses the
    new one, matching what `cosmic-settings appearance export` writes.

Colors, and which Catppuccin color goes into each field, follow the
official template (templates/cosmic-settings.tera in catppuccin/cosmic-desktop).

Usage:
    themes/generate.py -o theme.ron                    # Mocha Mauve, solid
    themes/generate.py --accent blue --frosted medium -o theme.ron
    themes/generate.py --all themes/cosmic-settings    # every flavor and accent
    themes/generate.py --help

To apply without opening Settings:
    cosmic-settings appearance import theme.ron
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "lib"))
from catppuccin import ACCENTS, FLAVORS, is_dark  # noqa: E402

# COSMIC palette field → Catppuccin color, in the order COSMIC 1.10
# writes them (CosmicPaletteInner in libcosmic).
PALETTE_MAP = [
    ("bright_red", "red"),
    ("bright_green", "green"),
    ("bright_orange", "peach"),
    ("gray_1", "mantle"),          # surfaces
    ("gray_2", "base"),
    ("neutral_0", "crust"),        # neutral scale, darkest to lightest
    ("neutral_1", "mantle"),
    ("neutral_2", "base"),
    ("neutral_3", "surface0"),
    ("neutral_4", "surface1"),
    ("neutral_5", "surface2"),
    ("neutral_6", "overlay0"),
    ("neutral_7", "overlay1"),
    ("neutral_8", "overlay2"),
    ("neutral_9", "subtext0"),
    ("neutral_10", "subtext1"),
    ("accent_blue", "blue"),       # the 9 accent swatches shown under Appearance
    ("accent_indigo", "mauve"),
    ("accent_purple", "lavender"),
    ("accent_pink", "pink"),
    ("accent_red", "red"),
    ("accent_orange", "peach"),
    ("accent_yellow", "yellow"),
    ("accent_green", "green"),
    ("accent_warm_grey", "overlay2"),
    ("ext_warm_grey", "overlay2"),  # extra colors used by some apps
    ("ext_orange", "peach"),
    ("ext_yellow", "yellow"),
    ("ext_blue", "blue"),
    ("ext_purple", "lavender"),
    ("ext_pink", "pink"),
    ("ext_indigo", "mauve"),
]

ROUNDNESS = {
    # radius_0, xs, s, m, l, xl
    "round": (0, 4, 8, 16, 32, 160),          # COSMIC default
    "slightlyround": (0, 2, 8, 8, 8, 8),
    "square": (0, 2, 2, 2, 2, 2),
}

# Blur strengths (libcosmic's BlurStrength enum), weakest to strongest,
# with the default opacity for each one (AlphaMap::default).
BLUR = [
    ("ExtremelyLow", "extremely_low", 0.90),
    ("ExtremelyLow2", "extremely_low_2", 0.87692),
    ("VeryLow", "very_low", 0.85385),
    ("VeryLow2", "very_low_2", 0.83076),
    ("Low", "low", 0.80769),
    ("Low2", "low_2", 0.78461),
    ("Medium", "medium", 0.76154),
    ("Medium2", "medium_2", 0.73846),
    ("High", "high", 0.71538),
    ("High2", "high_2", 0.69231),
    ("VeryHigh", "very_high", 0.66023),
    ("VeryHigh2", "very_high_2", 0.64615),
    ("ExtremelyHigh", "extremely_high", 0.62308),
    ("ExtremelyHigh2", "extremely_high_2", 0.6),
]
BLUR_BY_KEY = {key: name for name, key, _ in BLUR}

# Where Frosted Glass applies, in the same order as the toggles under Appearance.
FROST_TARGETS = ["windows", "system_interface", "panel", "applets", "maximized_apps"]
FROST_DEFAULT = ["windows", "system_interface", "panel", "applets"]


def hexa(rgb, alpha=1.0):
    """'cba6f7' → '#CBA6F7FF', the format COSMIC writes."""
    return "#%s%02X" % (rgb.upper(), round(alpha * 255))


def build(flavor, accent, roundness="round", window_hint=None, frosted=None,
          frost_on=None, alpha_offset=0.0, gaps=(0, 8), active_hint=3):
    """Build the RON text of a ThemeBuilder v2."""
    c = FLAVORS[flavor]
    dark = is_dark(flavor)
    frost_on = set(frost_on if frost_on is not None else (FROST_DEFAULT if frosted else []))
    blur_key = frosted or "medium"   # strength is irrelevant when everything is off

    lines = ["("]
    lines.append("    palette: %s((" % ("Dark" if dark else "Light"))
    lines.append('        name: "Catppuccin-%s-%s",' % (flavor.capitalize(), accent.capitalize()))
    for field, color in PALETTE_MAP:
        lines.append('        %s: "%s",' % (field, hexa(c[color])))
    lines.append("    )),")

    lines.append("    spacing: (")
    for k, v in zip(
        ["none", "xxxs", "xxs", "xs", "s", "m", "l", "xl", "xxl", "xxxl"],
        [0, 4, 8, 12, 16, 24, 32, 48, 64, 128],
    ):
        lines.append("        space_%s: %d," % (k, v))
    lines.append("    ),")

    lines.append("    corner_radii: (")
    for k, r in zip(["0", "xs", "s", "m", "l", "xl"], ROUNDNESS[roundness]):
        lines.append("        radius_%s: (%.1f, %.1f, %.1f, %.1f)," % (k, r, r, r, r))
    lines.append("    ),")

    def opt(name, color):
        lines.append('    %s: %s,' % (name, 'Some("%s")' % hexa(c[color]) if color else "None"))

    # Containers and control tint only for dark flavors, as in the official
    # theme; for Latte COSMIC derives them from the palette.
    opt("neutral_tint", "overlay1" if dark else None)
    opt("bg_color", "base")                  # window background
    opt("primary_container_bg", "surface0" if dark else None)
    opt("secondary_container_bg", "surface1" if dark else None)
    opt("text_tint", "text")
    opt("accent", accent)
    opt("success", "green")
    opt("warning", "yellow")
    opt("destructive", "red")

    lines.append("    frosted: %s," % BLUR_BY_KEY[blur_key])
    lines.append("    gaps: (%d, %d)," % gaps)
    lines.append("    active_hint: %d," % active_hint)
    opt("window_hint", window_hint or accent)  # active window outline
    for t in FROST_TARGETS:
        lines.append("    frosted_%s: %s," % (t, "true" if t in frost_on else "false"))

    # Opacity for each strength. The opacity slider under Appearance adds
    # one offset to every value; --alpha-offset does the same.
    lines.append("    alpha_map: (")
    for _, key, a in BLUR:
        lines.append("        %s: %s," % (key, round(min(1.0, max(0.0, a + alpha_offset)), 5)))
    lines.append("    ),")
    lines.append(")")
    return "\n".join(lines) + "\n"


def filename(flavor, accent, roundness, frosted):
    return "catppuccin-%s-%s+%s%s.ron" % (flavor, accent, roundness, "+frosted" if frosted else "")


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--flavor", default="mocha", choices=FLAVORS)
    p.add_argument("--accent", default="mauve", choices=ACCENTS)
    p.add_argument("--roundness", default="round", choices=ROUNDNESS)
    p.add_argument("--window-hint", choices=ACCENTS, help="active window outline color (default: the accent)")
    p.add_argument("--frosted", choices=[k for _, k, _ in BLUR],
                   help="turn on Frosted Glass at this strength (e.g. medium)")
    p.add_argument("--frost-on", default=",".join(FROST_DEFAULT),
                   help="where to apply Frosted Glass, comma-separated: %s" % ",".join(FROST_TARGETS))
    p.add_argument("--alpha-offset", type=float, default=0.0,
                   help="added to the opacity (e.g. 0.1 = more opaque, -0.1 = more transparent)")
    p.add_argument("--gaps", default="0,8", help="outer,inner gap between windows")
    p.add_argument("--active-hint", type=int, default=3, help="active window outline width")
    p.add_argument("-o", "--output", help="output file (default: stdout)")
    p.add_argument("--all", metavar="DIR",
                   help="write every flavor and accent, solid and frosted, into DIR")
    a = p.parse_args()

    frost_on = [t.strip() for t in a.frost_on.split(",") if t.strip()]
    bad = set(frost_on) - set(FROST_TARGETS)
    if bad:
        p.error("unknown --frost-on value: %s" % ", ".join(sorted(bad)))
    gaps = tuple(int(x) for x in a.gaps.split(","))
    common = dict(roundness=a.roundness, window_hint=a.window_hint, alpha_offset=a.alpha_offset,
                  gaps=gaps, active_hint=a.active_hint)

    if a.all:
        os.makedirs(a.all, exist_ok=True)
        n = 0
        for flavor in FLAVORS:
            for accent in ACCENTS:
                for frosted in (None, a.frosted or "medium"):
                    text = build(flavor, accent, frosted=frosted,
                                 frost_on=frost_on if frosted else [], **common)
                    with open(os.path.join(a.all, filename(flavor, accent, a.roundness, frosted)), "w") as f:
                        f.write(text)
                    n += 1
        print("%d themes in %s" % (n, a.all), file=sys.stderr)
        return

    text = build(a.flavor, a.accent, frosted=a.frosted,
                 frost_on=frost_on if a.frosted else [], **common)
    if a.output:
        with open(a.output, "w") as f:
            f.write(text)
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
