#!/usr/bin/env python3
"""Gera temas Catppuccin no formato do COSMIC 1.10 (ThemeBuilder v2).

Por que existe: os .ron oficiais de catppuccin/cosmic-desktop foram feitos
para o formato antigo (ThemeBuilder v1). No COSMIC 1.10 eles ainda abrem,
mas perdem coisas:

  * `is_frosted` não existe mais e é ignorado em silêncio. A transparência
    agora usa `frosted` (intensidade do desfoque), `frosted_windows`,
    `frosted_system_interface`, `frosted_panel`, `frosted_applets`,
    `frosted_maximized_apps` e `alpha_map` (opacidade de cada intensidade).
    Como esses campos faltam, importar o tema antigo DESLIGA o Frosted
    Glass de quem estava com ele ligado.
  * A paleta perdeu `blue`, `red`, `green`, `yellow` e `gray_3`
    (também ignorados).
  * O COSMIC grava cores como "#RRGGBBAA". O formato antigo
    (red: 0.5, green: ...) ainda é aceito, mas este gerador já usa o novo,
    igual ao que `cosmic-settings appearance export` produz.

As cores e a escolha de qual cor do Catppuccin vai em cada campo seguem o
template oficial (templates/cosmic-settings.tera do catppuccin/cosmic-desktop).

Uso:
    themes/generate.py -o tema.ron                     # Mocha Mauve, sólido
    themes/generate.py --accent blue --frosted medium -o tema.ron
    themes/generate.py --all themes/cosmic-settings    # todos os sabores e acentos
    themes/generate.py --help

Para aplicar sem abrir as Configurações:
    cosmic-settings appearance import tema.ron
"""
import argparse
import os
import sys

# Paleta oficial: https://catppuccin.com/palette
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

# Campos da paleta do COSMIC → cor do Catppuccin, na ordem em que o
# COSMIC 1.10 grava (CosmicPaletteInner no libcosmic).
PALETTE_MAP = [
    ("bright_red", "red"),
    ("bright_green", "green"),
    ("bright_orange", "peach"),
    ("gray_1", "mantle"),          # superfícies
    ("gray_2", "base"),
    ("neutral_0", "crust"),        # escala de neutros, do mais escuro ao mais claro
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
    ("accent_blue", "blue"),       # os 9 acentos que aparecem como bolinhas em Aparência
    ("accent_indigo", "mauve"),
    ("accent_purple", "lavender"),
    ("accent_pink", "pink"),
    ("accent_red", "red"),
    ("accent_orange", "peach"),
    ("accent_yellow", "yellow"),
    ("accent_green", "green"),
    ("accent_warm_grey", "overlay2"),
    ("ext_warm_grey", "overlay2"),  # cores extras usadas por alguns apps
    ("ext_orange", "peach"),
    ("ext_yellow", "yellow"),
    ("ext_blue", "blue"),
    ("ext_purple", "lavender"),
    ("ext_pink", "pink"),
    ("ext_indigo", "mauve"),
]

ROUNDNESS = {
    # radius_0, xs, s, m, l, xl
    "round": (0, 4, 8, 16, 32, 160),          # padrão do COSMIC
    "slightlyround": (0, 2, 8, 8, 8, 8),
    "square": (0, 2, 2, 2, 2, 2),
}

# Intensidades do desfoque (enum BlurStrength do libcosmic), da mais fraca
# para a mais forte, e a opacidade padrão de cada uma (AlphaMap::default).
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

# Onde o Frosted Glass vale, na mesma ordem dos interruptores em Aparência.
FROST_TARGETS = ["windows", "system_interface", "panel", "applets", "maximized_apps"]
FROST_DEFAULT = ["windows", "system_interface", "panel", "applets"]


def hexa(rgb, alpha=1.0):
    """'cba6f7' → '#CBA6F7FF', o formato que o COSMIC grava."""
    return "#%s%02X" % (rgb.upper(), round(alpha * 255))


def build(flavor, accent, roundness="round", window_hint=None, frosted=None,
          frost_on=None, alpha_offset=0.0, gaps=(0, 8), active_hint=3):
    """Monta o texto RON de um ThemeBuilder v2."""
    c = FLAVORS[flavor]
    dark = flavor != "latte"
    frost_on = set(frost_on if frost_on is not None else (FROST_DEFAULT if frosted else []))
    blur_key = frosted or "medium"   # com tudo desligado a intensidade não importa

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

    # Contêineres e tinta dos controles só no escuro, como no tema oficial;
    # no Latte o COSMIC calcula a partir da paleta.
    opt("neutral_tint", "overlay1" if dark else None)
    opt("bg_color", "base")                  # fundo das janelas
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
    opt("window_hint", window_hint or accent)  # borda da janela ativa
    for t in FROST_TARGETS:
        lines.append("    frosted_%s: %s," % (t, "true" if t in frost_on else "false"))

    # Opacidade de cada intensidade. O controle de opacidade em Aparência
    # soma um deslocamento a todos os valores; --alpha-offset faz o mesmo.
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
    p.add_argument("--window-hint", choices=ACCENTS, help="cor da borda da janela ativa (padrão: o acento)")
    p.add_argument("--frosted", choices=[k for _, k, _ in BLUR],
                   help="liga o Frosted Glass com esta intensidade (ex.: medium)")
    p.add_argument("--frost-on", default=",".join(FROST_DEFAULT),
                   help="onde aplicar o Frosted Glass, separado por vírgula: %s" % ",".join(FROST_TARGETS))
    p.add_argument("--alpha-offset", type=float, default=0.0,
                   help="soma à opacidade (ex.: 0.1 = mais opaco, -0.1 = mais transparente)")
    p.add_argument("--gaps", default="0,8", help="espaço externo,interno entre janelas")
    p.add_argument("--active-hint", type=int, default=3, help="espessura da borda da janela ativa")
    p.add_argument("-o", "--output", help="arquivo de saída (padrão: tela)")
    p.add_argument("--all", metavar="DIR",
                   help="gera todos os sabores e acentos, sólidos e com Frosted Glass, em DIR")
    a = p.parse_args()

    frost_on = [t.strip() for t in a.frost_on.split(",") if t.strip()]
    bad = set(frost_on) - set(FROST_TARGETS)
    if bad:
        p.error("--frost-on desconhecido: %s" % ", ".join(sorted(bad)))
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
        print("%d temas em %s" % (n, a.all), file=sys.stderr)
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
