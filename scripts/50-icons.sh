#!/usr/bin/env bash
# =============================================================================
# 50 · Ícones
# 1. Copia o Papirus e o Papirus-Dark para ~/.local/share/icons (sem sudo).
# 2. Acrescenta as pastas coloridas do catppuccin/papirus-folders.
# 3. Usa o script papirus-folders para trocar a cor das pastas para
#    cat-<sabor>-<acento> (ex.: cat-mocha-mauve).
# 4. Define Papirus-Dark como tema de ícones do COSMIC.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Ícones Papirus-Dark (pastas cat-$FLAVOR-$ACCENT)"
fetch_pinned "$PAPIRUS_REPO" "$PAPIRUS_REV" "$CACHE_DIR/papirus"
fetch_pinned "$CTP_FOLDERS_REPO" "$CTP_FOLDERS_REV" "$CACHE_DIR/ctp-folders"
fetch_pinned "$PAPIRUS_FOLDERS_REPO" "$PAPIRUS_FOLDERS_REV" "$CACHE_DIR/papirus-folders"

mkdir -p "$ICONS_DIR"
# Papirus-Dark aponta para arquivos do Papirus por links simbólicos;
# cp -a preserva esses links.
rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark"
cp -a "$CACHE_DIR/papirus/Papirus" "$CACHE_DIR/papirus/Papirus-Dark" "$ICONS_DIR/"
cp -r "$CACHE_DIR/ctp-folders/src/"* "$ICONS_DIR/Papirus/"

bash "$CACHE_DIR/papirus-folders/papirus-folders" -t Papirus-Dark -C "cat-$FLAVOR-$ACCENT" >/dev/null
gtk-update-icon-cache -q "$ICONS_DIR/Papirus" 2>/dev/null || true
gtk-update-icon-cache -q "$ICONS_DIR/Papirus-Dark" 2>/dev/null || true

cosmic_set com.system76.CosmicTk icon_theme '"Papirus-Dark"'
ok "Ícones aplicados"
