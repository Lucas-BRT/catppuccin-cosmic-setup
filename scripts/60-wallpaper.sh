#!/usr/bin/env bash
# =============================================================================
# 60 · Wallpaper
#
# One style (default): puts catppuccin-<flavor>-<accent>[-<style>].png in
# ~/Pictures/Wallpapers/ and sets it as the background for every display.
# Images are rendered by ctp-wallpaper (wallpaper/, Rust; well under a
# second each), downloaded prebuilt and hash-checked, or built with cargo.
#
# WALLPAPER_STYLE=all: renders every style into
# ~/Pictures/Wallpapers/catppuccin-<flavor>-<accent>/ and adds that folder to
# COSMIC Settings → Wallpaper, so you can pick an image or turn on the
# slideshow there. The current background is left alone, unless...
#
# WALLPAPER_ROTATE=5m|10m|15m|30m|1h|2h (with WALLPAPER_STYLE=all): also sets
# that folder as the background in slideshow mode, changing image at that
# interval.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

settings="$COSMIC_DIR/com.system76.CosmicSettings.Wallpaper/v1"

# Point the background of every display at an image or a folder.
set_background() {
    local source=$1 rotation=${2:-3600}
    cosmic_set com.system76.CosmicBackground same-on-all true
    strip_ron_comments "$REPO_DIR/config/cosmic/background-all.ron" |
        sed -e "s#@SOURCE@#$source#" -e "s#@ROTATION@#$rotation#" \
        > "$COSMIC_DIR/com.system76.CosmicBackground/v1/all"
}

# Add the folder to the wallpaper picker the way COSMIC Settings does:
# "recent-folders" keeps at most five entries (oldest dropped first), and
# "current-folder" is the one the picker opens on.
register_folder() {
    mkdir -p "$settings"
    python3 -I - "$settings" "$1" <<'PY'
import os, re, sys
settings, folder = sys.argv[1], sys.argv[2]
path = os.path.join(settings, "recent-folders")
try:
    with open(path) as f:
        folders = re.findall(r'"((?:[^"\\]|\\.)*)"', f.read())
except FileNotFoundError:
    folders = []
if folder not in folders:
    while len(folders) > 4:
        folders.pop(0)
    folders.append(folder)
with open(path, "w") as f:
    f.write("[\n" + "".join('    "%s",\n' % p for p in folders) + "]")
with open(os.path.join(settings, "current-folder"), "w") as f:
    f.write('Some("%s")' % folder)
PY
}

wallpaper_bin

if [ "$WALLPAPER_STYLE" != all ]; then
    info "Wallpaper ($FLAVOR, $ACCENT accent, $WALLPAPER_STYLE)"
    mkdir -p "$(dirname "$WALLPAPER")"
    "$WALLPAPER_BIN" "$WALLPAPER" --style "$WALLPAPER_STYLE" --flavor "$FLAVOR" --accent "$ACCENT"
    set_background "$WALLPAPER"
    ok "Background: $WALLPAPER"
    exit 0
fi

info "Wallpapers: every style ($FLAVOR, $ACCENT accent)"
"$WALLPAPER_BIN" --all "$WALLPAPER_DIR" --flavor "$FLAVOR" --accent "$ACCENT"
ok "$(wc -w <<<"$WALLPAPER_STYLES") wallpapers in $WALLPAPER_DIR"

register_folder "$WALLPAPER_DIR"
ok "Folder added to Settings → Wallpaper"

if [ -n "$ROTATE_SECONDS" ]; then
    # Keep the Settings dropdown in sync with the interval actually used.
    printf '%s' "$ROTATE_SECONDS" > "$settings/rotation-frequency"
    set_background "$WALLPAPER_DIR" "$ROTATE_SECONDS"
    ok "Background: slideshow of $WALLPAPER_DIR every $WALLPAPER_ROTATE"
else
    todo "Background unchanged. Pick an image or turn on the slideshow in"
    todo "Settings → Wallpaper, or rerun with WALLPAPER_ROTATE=30m (5m ... 2h)."
fi
