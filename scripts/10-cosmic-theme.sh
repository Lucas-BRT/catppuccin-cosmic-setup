#!/usr/bin/env bash
# =============================================================================
# 10 · COSMIC desktop theme
#
# Generates the theme in the COSMIC 1.10 format (ThemeBuilder v2) with
# themes/generate.py and applies it with `cosmic-settings appearance import`,
# the same code path as the Import button under Appearance, without opening
# the window.
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

builder="$COSMIC_DIR/com.system76.CosmicTheme.Dark.Builder/v2"
[ "$FLAVOR" = latte ] && builder="$COSMIC_DIR/com.system76.CosmicTheme.Light.Builder/v2"

# Read the current Frosted Glass setting. COSMIC stores each option in its
# own file: frosted = "Medium", frosted_windows = "true", ...
current_frost() {
    local on=() t
    for t in windows system_interface panel applets maximized_apps; do
        [ "$(cat "$builder/frosted_$t" 2>/dev/null)" = true ] && on+=("$t")
    done
    [ ${#on[@]} -gt 0 ] || return 1
    # "ExtremelyLow2" → "extremely_low_2", the name the generator expects.
    FROSTED=$(sed -E 's/([a-z])([A-Z0-9])/\1_\2/g' "$builder/frosted" 2>/dev/null | tr '[:upper:]' '[:lower:]')
    FROSTED=${FROSTED:-medium}
    FROST_ON=$(IFS=,; echo "${on[*]}")
}

FROSTED="${FROSTED:-auto}"
if [ "$FROSTED" = auto ]; then
    current_frost || FROSTED=off
fi

args=(--flavor "$FLAVOR" --accent "$ACCENT" --alpha-offset "${ALPHA_OFFSET:-0}")
if [ "$FROSTED" = off ]; then
    label="solid"
else
    args+=(--frosted "$FROSTED" --frost-on "${FROST_ON:-windows,system_interface,panel,applets}")
    label="Frosted Glass $FROSTED on ${FROST_ON:-windows,system_interface,panel,applets}"
fi

info "COSMIC theme ($FLAVOR, $ACCENT accent, $label)"
gen="$REPO_DIR/themes/generate.py"
out="$DATA_DIR/themes/cosmic-settings"
mkdir -p "$out"
python3 -I "$gen" --all "$out" 2>/dev/null
theme="$out/catppuccin-$FLAVOR-$ACCENT+custom.ron"
python3 -I "$gen" "${args[@]}" -o "$theme"
ok "Themes generated in $out"

if [ "${NO_APPLY:-0}" = 1 ]; then
    todo "NO_APPLY=1: to apply, run  cosmic-settings appearance import $theme"
    exit 0
fi
cosmic-settings appearance import "$theme" >/dev/null
ok "Theme applied ($(basename "$theme"))"
