#!/usr/bin/env bash
# =============================================================================
# 60 · Wallpaper
#
# One style (default): puts catppuccin-<flavor>-<accent>[-<style>].png in
# ~/Pictures/Wallpapers/ and sets it as the background for every display.
# Images are rendered by `ctp-cosmic wallpaper` (well under a second each).
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
set_background() { ctp background set "$@"; }

if [ "$WALLPAPER_STYLE" != all ]; then
    info "Wallpaper ($FLAVOR, $ACCENT accent, $WALLPAPER_STYLE)"
    mkdir -p "$(dirname "$WALLPAPER")"
    ctp wallpaper "$WALLPAPER" --style "$WALLPAPER_STYLE" --flavor "$FLAVOR" --accent "$ACCENT"
    set_background "$WALLPAPER"
    ok "Background: $WALLPAPER"
    exit 0
fi

info "Wallpapers: every style ($FLAVOR, $ACCENT accent)"
ctp wallpaper --all "$WALLPAPER_DIR" --flavor "$FLAVOR" --accent "$ACCENT"
ok "$(wc -w <<<"$WALLPAPER_STYLES") wallpapers in $WALLPAPER_DIR"

# Same keys COSMIC Settings writes: recent-folders (at most five) and
# current-folder, the one the picker opens on.
ctp register-wallpaper-folder "$settings" "$WALLPAPER_DIR"
ok "Folder added to Settings → Wallpaper"

if [ -n "$ROTATE_SECONDS" ]; then
    # Also keeps the Settings dropdown in sync with the interval used.
    set_background "$WALLPAPER_DIR" --rotation "$ROTATE_SECONDS"
    ok "Background: slideshow of $WALLPAPER_DIR every $WALLPAPER_ROTATE"
else
    todo "Background unchanged. Pick an image or turn on the slideshow in"
    todo "Settings → Wallpaper, or rerun with WALLPAPER_ROTATE=30m (5m ... 2h)."
fi
