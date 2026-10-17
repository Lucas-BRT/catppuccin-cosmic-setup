# shellcheck shell=bash
# =============================================================================
# Shared functions and variables. Not meant to be run directly: every
# scripts/NN-*.sh sources this file.
# =============================================================================
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COSMIC_DIR="$HOME/.config/cosmic"
DATA_DIR="$HOME/.local/share/catppuccin"   # generated and downloaded .ron themes
ICONS_DIR="$HOME/.local/share/icons"       # Papirus goes here (no sudo)
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/catppuccin-cosmic-setup"

# Catppuccin flavor and accent. Override with environment variables:
#   FLAVOR=latte ACCENT=lavender ./install.sh
FLAVOR="${FLAVOR:-mocha}"
ACCENT="${ACCENT:-mauve}"

# Palette helper (lib/catppuccin.py): check, name, is-dark, rgb, render.
ctp() { python3 -I "$REPO_DIR/lib/catppuccin.py" "$@"; }
ctp check "$FLAVOR" "$ACCENT"

# Wallpaper style (see `wallpaper/generate.py --list-styles`), or "all" to
# render every style into a folder for COSMIC's wallpaper picker. The
# default "glow" keeps the plain catppuccin-<flavor>-<accent>.png name.
WALLPAPER_STYLE="${WALLPAPER_STYLE:-glow}"
WALLPAPER_STYLES=$(python3 -I "$REPO_DIR/wallpaper/generate.py" --list-styles)
if [ "$WALLPAPER_STYLE" != all ] && ! grep -qx "$WALLPAPER_STYLE" <<<"$WALLPAPER_STYLES"; then
    echo "unknown wallpaper style '$WALLPAPER_STYLE' (choose: $(paste -sd, <<<"$WALLPAPER_STYLES" | sed 's/,/, /g'), all)" >&2
    exit 1
fi
suffix=""; [ "$WALLPAPER_STYLE" = glow ] || suffix="-$WALLPAPER_STYLE"
WALLPAPER="$HOME/Pictures/Wallpapers/catppuccin-$FLAVOR-$ACCENT$suffix.png"
WALLPAPER_DIR="$HOME/Pictures/Wallpapers/catppuccin-$FLAVOR-$ACCENT"   # used by "all"
unset suffix

# Slideshow interval for WALLPAPER_STYLE=all. Only the intervals offered by
# COSMIC Settings → Wallpaper are accepted, so the UI shows the same value.
WALLPAPER_ROTATE="${WALLPAPER_ROTATE:-}"
ROTATE_SECONDS=""
if [ -n "$WALLPAPER_ROTATE" ]; then
    case "$WALLPAPER_ROTATE" in
        5m|300) ROTATE_SECONDS=300 ;;     10m|600) ROTATE_SECONDS=600 ;;
        15m|900) ROTATE_SECONDS=900 ;;    30m|1800) ROTATE_SECONDS=1800 ;;
        1h|3600) ROTATE_SECONDS=3600 ;;   2h|7200) ROTATE_SECONDS=7200 ;;
        *) echo "invalid WALLPAPER_ROTATE '$WALLPAPER_ROTATE' (choose: 5m, 10m, 15m, 30m, 1h, 2h)" >&2; exit 1 ;;
    esac
    if [ "$WALLPAPER_STYLE" != all ]; then
        echo "WALLPAPER_ROTATE needs WALLPAPER_STYLE=all (a slideshow needs several images)" >&2
        exit 1
    fi
fi
# Papirus has a dark and a light variant; Latte is the only light flavor.
if ctp is-dark "$FLAVOR"; then ICON_THEME=Papirus-Dark; else ICON_THEME=Papirus; fi

# Upstream repositories pinned to exact commits so installs are
# reproducible. To update, replace the hash with the commit you want.
CTP_COSMIC_REPO=https://github.com/catppuccin/cosmic-desktop.git
CTP_COSMIC_REV=95e81098042dd2102f0b258f6990f886c5759692
CTP_FOLDERS_REPO=https://github.com/catppuccin/papirus-folders.git
CTP_FOLDERS_REV=f83671d17ea67e335b34f8028a7e6d78bca735d7
PAPIRUS_REPO=https://github.com/PapirusDevelopmentTeam/papirus-icon-theme.git
PAPIRUS_REV=bf539287ef5dc18529424a02cccee76175920a6f
PAPIRUS_FOLDERS_REPO=https://github.com/PapirusDevelopmentTeam/papirus-folders.git
PAPIRUS_FOLDERS_REV=0f838ee5679229e3a3e97e3b333c222c9e9615b4

# Colored messages using the palette itself.
info() { printf '\e[38;2;137;180;250m==>\e[0m %s\n' "$*"; }   # blue
ok()   { printf '\e[38;2;166;227;161m ✓\e[0m %s\n' "$*"; }    # green
warn() { printf '\e[38;2;249;226;175m !\e[0m %s\n' "$*"; }    # yellow
todo() { printf '\e[38;2;203;166;247m ➜\e[0m %s\n' "$*"; }    # mauve

# Clone a repository at an exact commit, downloading only that commit.
# Skips the download when the cache is already at that commit.
fetch_pinned() {
    local url=$1 rev=$2 dest=$3
    if [ "$(git -C "$dest" rev-parse HEAD 2>/dev/null)" = "$rev" ]; then
        return 0
    fi
    rm -rf "$dest"
    mkdir -p "$dest"
    git -C "$dest" init -q
    git -C "$dest" fetch -q --depth 1 "$url" "$rev"
    git -C "$dest" -c advice.detachedHead=false checkout -q FETCH_HEAD
}

# Write a cosmic-config key. Each key is a file holding one RON value.
cosmic_set() {
    local component=$1 key=$2 value=$3
    mkdir -p "$COSMIC_DIR/$component/v1"
    printf '%s' "$value" > "$COSMIC_DIR/$component/v1/$key"
}

# Strip // comments from one of the repo's RON files before installing it.
strip_ron_comments() { sed -e 's#[[:space:]]*//.*$##' -e '/^[[:space:]]*$/d' "$1"; }
