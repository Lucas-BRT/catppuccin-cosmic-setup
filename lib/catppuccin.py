#!/usr/bin/env python3
"""The Catppuccin palette: the single source of colors for this project.

Imported by themes/generate.py and used from the shell scripts through a
small CLI. (The wallpaper generator, written in Rust, uses the official
`catppuccin` crate; CI checks that both agree via `dump`.)

    lib/catppuccin.py check FLAVOR ACCENT     # exit 1 with a message if invalid
    lib/catppuccin.py name FLAVOR             # "Mocha", "Frappé", ...
    lib/catppuccin.py is-dark FLAVOR          # exit 0 if dark, 1 if light
    lib/catppuccin.py rgb FLAVOR COLOR        # "137;180;250" (for ANSI escapes)
    lib/catppuccin.py render FLAVOR ACCENT    # stdin → stdout, see below
    lib/catppuccin.py dump                    # every color as "flavor color rrggbb"

`render` replaces every @<color>@ placeholder (e.g. @blue@, @overlay0@) with
that color as "R;G;B", and @accent@ with the chosen accent. That is how the
bash prompt template gets the colors of any flavor.

Official palette: https://catppuccin.com/palette
"""
import re
import sys

FLAVORS = {
    "latte": dict(
        rosewater="dc8a78", flamingo="dd7878", pink="ea76cb", mauve="8839ef",
        red="d20f39", maroon="e64553", peach="fe640b", yellow="df8e1d",
        green="40a02b", teal="179299", sky="04a5e5", sapphire="209fb5",
        blue="1e66f5", lavender="7287fd", text="4c4f69", subtext1="5c5f77",
        subtext0="6c6f85", overlay2="7c7f93", overlay1="8c8fa1", overlay0="9ca0b0",
        surface2="acb0be", surface1="bcc0cc", surface0="ccd0da", base="eff1f5",
        mantle="e6e9ef", crust="dce0e8",
    ),
    "frappe": dict(
        rosewater="f2d5cf", flamingo="eebebe", pink="f4b8e4", mauve="ca9ee6",
        red="e78284", maroon="ea999c", peach="ef9f76", yellow="e5c890",
        green="a6d189", teal="81c8be", sky="99d1db", sapphire="85c1dc",
        blue="8caaee", lavender="babbf1", text="c6d0f5", subtext1="b5bfe2",
        subtext0="a5adce", overlay2="949cbb", overlay1="838ba7", overlay0="737994",
        surface2="626880", surface1="51576d", surface0="414559", base="303446",
        mantle="292c3c", crust="232634",
    ),
    "macchiato": dict(
        rosewater="f4dbd6", flamingo="f0c6c6", pink="f5bde6", mauve="c6a0f6",
        red="ed8796", maroon="ee99a0", peach="f5a97f", yellow="eed49f",
        green="a6da95", teal="8bd5ca", sky="91d7e3", sapphire="7dc4e4",
        blue="8aadf4", lavender="b7bdf8", text="cad3f5", subtext1="b8c0e0",
        subtext0="a5adcb", overlay2="939ab7", overlay1="8087a2", overlay0="6e738d",
        surface2="5b6078", surface1="494d64", surface0="363a4f", base="24273a",
        mantle="1e2030", crust="181926",
    ),
    "mocha": dict(
        rosewater="f5e0dc", flamingo="f2cdcd", pink="f5c2e7", mauve="cba6f7",
        red="f38ba8", maroon="eba0ac", peach="fab387", yellow="f9e2af",
        green="a6e3a1", teal="94e2d5", sky="89dceb", sapphire="74c7ec",
        blue="89b4fa", lavender="b4befe", text="cdd6f4", subtext1="bac2de",
        subtext0="a6adc8", overlay2="9399b2", overlay1="7f849c", overlay0="6c7086",
        surface2="585b70", surface1="45475a", surface0="313244", base="1e1e2e",
        mantle="181825", crust="11111b",
    ),
}
ACCENTS = [
    "rosewater", "flamingo", "pink", "mauve", "red", "maroon", "peach",
    "yellow", "green", "teal", "sky", "sapphire", "blue", "lavender",
]
# Display names, as used by other Catppuccin ports (e.g. Zed: "Catppuccin Frappé").
NAMES = {"latte": "Latte", "frappe": "Frappé", "macchiato": "Macchiato", "mocha": "Mocha"}


def is_dark(flavor):
    return flavor != "latte"


def rgb(flavor, color):
    """(r, g, b) tuple of 0-255 ints."""
    h = FLAVORS[flavor][color]
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def render(text, flavor, accent):
    def sub(m):
        name = accent if m.group(1) == "accent" else m.group(1)
        if name not in FLAVORS[flavor]:
            raise SystemExit("unknown placeholder: @%s@" % m.group(1))
        return "%d;%d;%d" % rgb(flavor, name)
    return re.sub(r"@([a-z0-9]+)@", sub, text)


def check(flavor, accent=None):
    if flavor not in FLAVORS:
        raise SystemExit("unknown flavor '%s' (choose: %s)" % (flavor, ", ".join(FLAVORS)))
    if accent is not None and accent not in ACCENTS:
        raise SystemExit("unknown accent '%s' (choose: %s)" % (accent, ", ".join(ACCENTS)))


def main(argv):
    if argv == ["dump"]:
        for flavor, colors in FLAVORS.items():
            for name, hexv in colors.items():
                print(flavor, name, hexv)
        return 0
    if len(argv) < 2:
        raise SystemExit(__doc__)
    cmd, args = argv[0], argv[1:]
    check(args[0], args[1] if cmd in ("check", "render") and len(args) > 1 else None)
    if cmd == "check":
        return 0
    if cmd == "name":
        print(NAMES[args[0]])
    elif cmd == "is-dark":
        return 0 if is_dark(args[0]) else 1
    elif cmd == "rgb":
        if args[1] not in FLAVORS[args[0]]:
            raise SystemExit("unknown color '%s'" % args[1])
        print("%d;%d;%d" % rgb(args[0], args[1]))
    elif cmd == "render":
        sys.stdout.write(render(sys.stdin.read(), args[0], args[1]))
    else:
        raise SystemExit(__doc__)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
