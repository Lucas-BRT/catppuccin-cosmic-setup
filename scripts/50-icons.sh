#!/usr/bin/env bash
# =============================================================================
# 50 · Icons
# 1. Copies Papirus and Papirus-Dark into ~/.local/share/icons (no sudo).
# 2. Adds the colored folders from catppuccin/papirus-folders.
# 3. Uses the papirus-folders script to switch the folder color to
#    cat-<flavor>-<accent> (e.g. cat-mocha-mauve).
# 4. Sets Papirus-Dark as the COSMIC icon theme.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Papirus-Dark icons (cat-$FLAVOR-$ACCENT folders)"
fetch_pinned "$PAPIRUS_REPO" "$PAPIRUS_REV" "$CACHE_DIR/papirus"
fetch_pinned "$CTP_FOLDERS_REPO" "$CTP_FOLDERS_REV" "$CACHE_DIR/ctp-folders"
fetch_pinned "$PAPIRUS_FOLDERS_REPO" "$PAPIRUS_FOLDERS_REV" "$CACHE_DIR/papirus-folders"

mkdir -p "$ICONS_DIR"
# Papirus-Dark points at Papirus files through symlinks; cp -a keeps them.
rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark"
cp -a "$CACHE_DIR/papirus/Papirus" "$CACHE_DIR/papirus/Papirus-Dark" "$ICONS_DIR/"
cp -r "$CACHE_DIR/ctp-folders/src/"* "$ICONS_DIR/Papirus/"

bash "$CACHE_DIR/papirus-folders/papirus-folders" -t Papirus-Dark -C "cat-$FLAVOR-$ACCENT" >/dev/null
gtk-update-icon-cache -q "$ICONS_DIR/Papirus" 2>/dev/null || true
gtk-update-icon-cache -q "$ICONS_DIR/Papirus-Dark" 2>/dev/null || true

cosmic_set com.system76.CosmicTk icon_theme '"Papirus-Dark"'
ok "Icons applied"
