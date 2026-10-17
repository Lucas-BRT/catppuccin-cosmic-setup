# shellcheck shell=bash
# =============================================================================
# Shared functions and variables. Not meant to be run directly: every
# scripts/NN-*.sh sources this file.
# =============================================================================
# The variables defined here are used by the scripts that source this file.
# shellcheck disable=SC2034
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COSMIC_DIR="$HOME/.config/cosmic"
DATA_DIR="$HOME/.local/share/catppuccin"   # generated and downloaded .ron themes
ICONS_DIR="$HOME/.local/share/icons"       # Papirus goes here (no sudo)
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/catppuccin-cosmic-setup"
GITHUB_REPO=Lucas-BRT/catppuccin-cosmic-setup

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

# --- ctp-cosmic -----------------------------------------------------------
# ctp-cosmic (ctp-cosmic/, Rust) does everything the scripts compute: palette
# lookups, desktop themes and wallpapers. Release packages installed with
# get.sh include it in bin/. In a git clone, the prebuilt binary is
# downloaded from the release named in ctp-cosmic/RELEASE and only run if it
# matches the hash pinned in ctp-cosmic/SHA256SUMS; without a pinned binary
# for this machine, or with BUILD_FROM_SOURCE=1, it is built with cargo.
find_ctp() {
    local tag arch asset expected bin
    if [ -x "$REPO_DIR/bin/ctp-cosmic" ] && [ "${BUILD_FROM_SOURCE:-0}" != 1 ]; then
        CTP_BIN="$REPO_DIR/bin/ctp-cosmic"
        return 0
    fi
    tag=$(cat "$REPO_DIR/ctp-cosmic/RELEASE" 2>/dev/null || true)
    case "$(uname -m)" in
        x86_64 | amd64) arch=x86_64 ;;
        aarch64 | arm64) arch=aarch64 ;;
        *) arch="" ;;
    esac
    asset="ctp-cosmic-$arch-linux"
    expected=$(awk -v a="$asset" '$2 == a { print $1 }' "$REPO_DIR/ctp-cosmic/SHA256SUMS" 2>/dev/null || true)

    if [ "${BUILD_FROM_SOURCE:-0}" != 1 ] && [ -n "$tag" ] && [ -n "$arch" ] && [ -n "$expected" ]; then
        bin="$CACHE_DIR/bin/$tag/$asset"
        if ! sha256sum -c --status <<<"$expected  $bin" 2>/dev/null; then
            mkdir -p "$(dirname "$bin")"
            if curl -fsSL --proto '=https' --tlsv1.2 -o "$bin.part" \
                "https://github.com/$GITHUB_REPO/releases/download/$tag/$asset"; then
                if ! sha256sum -c --status <<<"$expected  $bin.part"; then
                    rm -f "$bin.part"
                    echo "ctp-cosmic download does not match ctp-cosmic/SHA256SUMS; refusing to run it" >&2
                    exit 1
                fi
                chmod +x "$bin.part"
                mv "$bin.part" "$bin"
            else
                rm -f "$bin.part"
                warn "Could not download ctp-cosmic $tag; trying a local build"
                bin=""
            fi
        fi
        if [ -n "$bin" ]; then
            CTP_BIN=$bin
            return 0
        fi
    fi

    if ! command -v cargo >/dev/null; then
        echo "no prebuilt ctp-cosmic for $(uname -m) and cargo is not installed;" >&2
        echo "install Rust from https://rustup.rs and run the installer again" >&2
        exit 1
    fi
    CTP_BIN="$CACHE_DIR/target/release/ctp-cosmic"
    [ -x "$CTP_BIN" ] || info "Building ctp-cosmic with cargo (first time takes a minute)"
    cargo build --release --locked --quiet \
        --manifest-path "$REPO_DIR/ctp-cosmic/Cargo.toml" --target-dir "$CACHE_DIR/target"
}
find_ctp
ctp() { "$CTP_BIN" "$@"; }

# Catppuccin flavor and accent. Override with environment variables:
#   FLAVOR=latte ACCENT=lavender ./install.sh
FLAVOR="${FLAVOR:-mocha}"
ACCENT="${ACCENT:-mauve}"
ctp palette check "$FLAVOR" "$ACCENT"

# Papirus has a dark and a light variant; Latte is the only light flavor.
if ctp palette is-dark "$FLAVOR"; then ICON_THEME=Papirus-Dark; else ICON_THEME=Papirus; fi

# Wallpaper style, or "all" to render every style into a folder for COSMIC's
# wallpaper picker. The default "glow" keeps the plain
# catppuccin-<flavor>-<accent>.png name.
WALLPAPER_STYLE="${WALLPAPER_STYLE:-glow}"
WALLPAPER_STYLES=$(ctp wallpaper --list-styles)
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

# Download a GitHub repository at an exact commit (as a tarball, so git is
# not needed) into dest. Skips the download when dest already holds that
# commit.
fetch_pinned() {
    local url=$1 rev=$2 dest=$3 slug
    if [ "$(cat "$dest/.pinned-rev" 2>/dev/null)" = "$rev" ]; then
        return 0
    fi
    slug=${url#https://github.com/}
    slug=${slug%.git}
    rm -rf "$dest"
    mkdir -p "$dest"
    curl -fsSL --proto '=https' --tlsv1.2 "https://codeload.github.com/$slug/tar.gz/$rev" |
        tar -xz -C "$dest" --strip-components=1
    printf '%s\n' "$rev" > "$dest/.pinned-rev"
}

# Write a cosmic-config key. Each key is a file holding one RON value.
cosmic_set() {
    local component=$1 key=$2 value=$3
    mkdir -p "$COSMIC_DIR/$component/v1"
    printf '%s' "$value" > "$COSMIC_DIR/$component/v1/$key"
}

# Strip // comments from one of the repo's RON files before installing it.
strip_ron_comments() { sed -e 's#[[:space:]]*//.*$##' -e '/^[[:space:]]*$/d' "$1"; }
