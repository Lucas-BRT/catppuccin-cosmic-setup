#!/usr/bin/env bash
# =============================================================================
# 20 · COSMIC Terminal color scheme
# `ctp-cosmic terminal apply` adds the flavor's scheme to COSMIC Terminal's
# config (color_schemes_dark or _light, whichever matches the flavor) and
# selects it for syntax colors: the same keys the terminal's Color schemes…
# → Import fills, so nothing is left to do by hand. Schemes you added
# yourself are kept. Colors follow catppuccin/cosmic-desktop's cosmic-term
# schemes.
#
# The scheme does not set `background`: the terminal uses the desktop
# theme's background, so it also turns translucent with Frosted Glass.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "COSMIC Terminal"
ctp terminal apply "$FLAVOR"
ok "Terminal colors: Catppuccin $(ctp palette name "$FLAVOR")"
