#!/usr/bin/env bash
# =============================================================================
# Installs Catppuccin (any flavor and accent; Mocha Mauve by default) on
# Pop!_OS 24.04 with the COSMIC desktop.
#
# Usage:
#   ./install.sh                 # every step (backs up first)
#   ./install.sh icons wallpaper # only the listed steps
#   ./install.sh --list          # list the steps
#
# Optional variables:
#   FLAVOR=mocha   ACCENT=mauve  # e.g. FLAVOR=latte ACCENT=lavender ./install.sh
#   FROSTED=auto|off|medium|...  # Frosted Glass (default: keep the current setting)
#   FROST_ON=windows,panel,...   # where Frosted Glass applies
#   ALPHA_OFFSET=0.1             # more opaque (negative = more transparent)
#   WALLPAPER_STYLE=waves        # glow (default), waves, stripes, dots, triangles
#   NO_APPLY=1                   # only generate the theme, don't apply it
#   NO_BACKUP=1                  # skip the backup
#
# Safe to run any number of times: each step overwrites only what it
# installed itself and never duplicates anything.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/scripts"

declare -A STEPS=(
    [cosmic]=10-cosmic-theme.sh
    [terminal]=20-terminal.sh
    [bash]=30-bash-prompt.sh
    [zed]=40-zed.sh
    [icons]=50-icons.sh
    [wallpaper]=60-wallpaper.sh
)
ORDER=(cosmic terminal bash zed icons wallpaper)

if [ "${1:-}" = --list ]; then
    for s in "${ORDER[@]}"; do echo "$s  (scripts/${STEPS[$s]})"; done
    exit 0
fi

if [ $# -eq 0 ]; then selected=("${ORDER[@]}"); else selected=("$@"); fi
for s in "${selected[@]}"; do
    [ -n "${STEPS[$s]:-}" ] || { echo "Unknown step: $s (see --list)" >&2; exit 1; }
done

[ "${NO_BACKUP:-0}" = 1 ] || ./00-backup.sh
for s in "${ORDER[@]}"; do
    [[ " ${selected[*]} " == *" $s "* ]] && "./${STEPS[$s]}"
done

echo
echo "Done. If icons or the wallpaper don't change right away, log out and back in."
