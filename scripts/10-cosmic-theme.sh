#!/usr/bin/env bash
# =============================================================================
# 10 · COSMIC desktop theme
#
# `ctp-cosmic theme install` generates the theme in the COSMIC 1.10 format
# (ThemeBuilder v2) and applies it with `cosmic-settings appearance import`,
# the same code path as the Import button under Appearance, without opening
# the window. It keeps the interface density chosen under Appearance and reads
# the current Frosted Glass setting from COSMIC's config.
#
# Frosted Glass (blurred transparency), via FROSTED:
#   FROSTED=auto    (default) keep the current setting: if it is already on,
#                   reuse its strength and the places it applies to
#   FROSTED=off     turn it off
#   FROSTED=medium  turn it on at this strength (extremely_low ... extremely_high_2)
# Where it applies when on, via FROST_ON (comma-separated):
#   windows,system_interface,panel,applets,maximized_apps
#   (default: everything except maximized_apps)
# Opacity: ALPHA_OFFSET=0.1 is more opaque, -0.1 more transparent.
#
# Also writes every flavor and accent, solid and frosted, to
# ~/.local/share/catppuccin/themes/cosmic-settings/ for manual import.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "COSMIC theme ($FLAVOR, $ACCENT accent)"
args=(--flavor "$FLAVOR" --accent "$ACCENT" --alpha-offset "${ALPHA_OFFSET:-0}" --frosted "${FROSTED:-auto}")
[ -z "${FROST_ON:-}" ] || args+=(--frost-on "$FROST_ON")
[ "${NO_APPLY:-0}" != 1 ] || args+=(--no-apply)
out=$(ctp theme install "${args[@]}")
label=${out%%$'\n'*}
theme=${out#*$'\n'}

ok "Style: $label"
ok "Themes generated in $(dirname "$theme")"
if [ "${NO_APPLY:-0}" = 1 ]; then
    todo "NO_APPLY=1: to apply, run  cosmic-settings appearance import $theme"
else
    ok "Theme applied ($(basename "$theme"))"
fi
