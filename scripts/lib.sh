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

WALLPAPER="$HOME/Pictures/Wallpapers/catppuccin-$FLAVOR-$ACCENT.png"
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
