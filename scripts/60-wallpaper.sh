#!/usr/bin/env bash
# =============================================================================
# 60 · Papel de parede
# Copia wallpaper/catppuccin-mocha.png para ~/Pictures/Wallpapers/ (gera de
# novo com wallpaper/generate.py se o PNG não estiver no repositório) e
# instala config/cosmic/background-all.ron como fundo de todas as telas.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Papel de parede"
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
ok "Fundo: $WALLPAPER"
