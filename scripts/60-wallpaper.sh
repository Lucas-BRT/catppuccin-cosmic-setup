#!/usr/bin/env bash
# =============================================================================
# 60 · Wallpaper
# Puts the wallpaper in ~/Pictures/Wallpapers/: copied from wallpaper/ when
# the repo ships it (Mocha Mauve, glow style), otherwise rendered with
# wallpaper/generate.py (pure Python, about 10-20 seconds). The style comes
# from WALLPAPER_STYLE: glow (default), waves, stripes, dots or triangles. Then
# installs config/cosmic/background-all.ron as the background for every
# display.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Wallpaper ($FLAVOR, $ACCENT accent, $WALLPAPER_STYLE)"
mkdir -p "$(dirname "$WALLPAPER")"
src="$REPO_DIR/wallpaper/$(basename "$WALLPAPER")"
if [ -f "$src" ]; then
    cp "$src" "$WALLPAPER"
else
    python3 -I "$REPO_DIR/wallpaper/generate.py" "$WALLPAPER" \
        --style "$WALLPAPER_STYLE" --flavor "$FLAVOR" --accent "$ACCENT"
fi

cosmic_set com.system76.CosmicBackground same-on-all true
strip_ron_comments "$REPO_DIR/config/cosmic/background-all.ron" |
    sed "s#@WALLPAPER@#$WALLPAPER#" > "$COSMIC_DIR/com.system76.CosmicBackground/v1/all"
ok "Background: $WALLPAPER"
