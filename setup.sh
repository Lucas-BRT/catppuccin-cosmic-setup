#!/usr/bin/env bash
# =============================================================================
# Installs Catppuccin (any flavor and accent; Mocha Mauve by default) on
# Pop!_OS 24.04 with the COSMIC desktop.
#
# Usage:
#   ./setup.sh                 # in a terminal: interactive setup (ctp-cosmic tui)
#   ./setup.sh icons wallpaper # only the listed steps, no questions
#
# The icons step (Papirus with Catppuccin folders) is optional: it only runs
# when named explicitly, e.g. `./setup.sh cosmic terminal bash zed icons wallpaper`.
#   ./setup.sh --list          # list the steps
#
# Without a terminal, or with any step or option below, everything runs
# directly (every step by default), so scripted installs need no input.
#
# Optional variables:
#   FLAVOR=mocha   ACCENT=mauve  # e.g. FLAVOR=latte ACCENT=lavender ./setup.sh
#   FROSTED=auto|off|medium|...  # Frosted Glass (default: keep the current setting)
#   FROST_ON=windows,panel,...   # where Frosted Glass applies
#   ALPHA_OFFSET=0.1             # more opaque (negative = more transparent)
#   WALLPAPER_STYLE=waves        # glow (default), waves, stripes, dots, triangles,
#                                # mesh, aurora, sunset, topography, mountains,
#                                # or all: every style in a folder for the COSMIC picker
#   WALLPAPER_ROTATE=30m         # with all: slideshow of that folder (5m ... 2h)
#   TERMINAL_FONT="JetBrainsMono Nerd Font"  # COSMIC Terminal font; list yours with
#                                # ctp-cosmic terminal font list. TERMINAL_FONT_SIZE=13
#   NO_APPLY=1                   # only generate the theme, don't apply it
#   NO_BACKUP=1                  # skip the backup
#   BUILD_FROM_SOURCE=1          # build ctp-cosmic with cargo instead of
#                                # downloading the prebuilt binary
#   NO_TUI=1                     # never open the interactive setup
#
# Safe to run any number of times: each step overwrites only what it
# installed itself and never duplicates anything.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/scripts"

# Interactive setup: only with no steps and no options, when there is a
# terminal to draw on (stdout) and to read keys from (/dev/tty, which also
# works under `curl ... | bash`).
interactive() {
    [ $# -eq 0 ] && [ "${NO_TUI:-0}" != 1 ] && [ -t 1 ] || return 1
    { : </dev/tty; } 2>/dev/null || return 1
    local v
    for v in FLAVOR ACCENT FROSTED FROST_ON ALPHA_OFFSET WALLPAPER_STYLE \
             WALLPAPER_ROTATE TERMINAL_FONT NO_APPLY NO_BACKUP; do
        [ -z "${!v:-}" ] || return 1
    done
}
# Decide before sourcing lib.sh: it sets FLAVOR, ACCENT and the like to their
# defaults, which interactive() would take for options given by the user.
want_tui=0; ! interactive "$@" || want_tui=1
# shellcheck source=scripts/lib.sh
source ./lib.sh   # finds (or builds) ctp-cosmic
if [ "$want_tui" = 1 ]; then
    exec "$CTP_BIN" tui --repo "$REPO_DIR"
fi

# The steps (name, script, required|optional) live in Rust: ctp-cosmic steps.
declare -A STEPS=() KIND=()
ORDER=()
while IFS=$'\t' read -r name script kind; do
    STEPS[$name]=$script; KIND[$name]=$kind; ORDER+=("$name")
done < <(ctp steps list)

if [ "${1:-}" = --list ]; then
    for s in "${ORDER[@]}"; do
        note=""; [ "${KIND[$s]}" = optional ] && note="  [optional, run only when named]"
        echo "$s  (scripts/${STEPS[$s]})$note"
    done
    exit 0
fi

if [ $# -eq 0 ]; then
    selected=()
    for s in "${ORDER[@]}"; do
        [ "${KIND[$s]}" = optional ] || selected+=("$s")
    done
else
    selected=("$@")
fi
for s in "${selected[@]}"; do
    [ -n "${STEPS[$s]:-}" ] || { echo "Unknown step: $s (see --list)" >&2; exit 1; }
done

if [ "${NO_BACKUP:-0}" != 1 ]; then
    ./00-backup.sh
    # Even if a step fails, record what changed so uninstall.sh can undo it.
    trap 'ctp backup seal >/dev/null || warn "Could not record the changes; uninstall.sh cannot undo this run"' EXIT
fi
for s in "${ORDER[@]}"; do
    [[ " ${selected[*]} " == *" $s "* ]] && "./${STEPS[$s]}"
done

echo
echo "Done. If icons or the wallpaper don't change right away, log out and back in."
