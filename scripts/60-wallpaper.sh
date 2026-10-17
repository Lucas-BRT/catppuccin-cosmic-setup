#!/usr/bin/env bash
# =============================================================================
# 60 · Wallpaper
# Copies wallpaper/catppuccin-mocha.png to ~/Pictures/Wallpapers/
# (regenerating it with wallpaper/generate.py if the PNG is missing from the
# repo) and installs config/cosmic/background-all.ron as the background for
# every display.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Wallpaper"
mkdir -p "$(dirname "$WALLPAPER")"
src="$REPO_DIR/wallpaper/catppuccin-mocha.png"
if [ -f "$src" ]; then
    cp "$src" "$WALLPAPER"
else
    python3 -I "$REPO_DIR/wallpaper/generate.py" "$WALLPAPER"
fi

cosmic_set com.system76.CosmicBackground same-on-all true
strip_ron_comments "$REPO_DIR/config/cosmic/background-all.ron" |
    sed "s#@WALLPAPER@#$WALLPAPER#" > "$COSMIC_DIR/com.system76.CosmicBackground/v1/all"
ok "Background: $WALLPAPER"
