#!/usr/bin/env bash
# =============================================================================
# 50 · Icons
# 1. Copies Papirus and Papirus-Dark into ~/.local/share/icons (no sudo).
# 2. Adds the colored folders from catppuccin/papirus-folders.
# 3. Uses the papirus-folders script to switch the folder color to
#    cat-<flavor>-<accent> (e.g. cat-mocha-mauve).
# 4. Sets the COSMIC icon theme: Papirus-Dark for dark flavors, Papirus
#    (light) for Latte, so icons stay readable on the background.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "$ICON_THEME icons (cat-$FLAVOR-$ACCENT folders)"
fetch_pinned "$PAPIRUS_REPO" "$PAPIRUS_REV" "$CACHE_DIR/papirus"
fetch_pinned "$CTP_FOLDERS_REPO" "$CTP_FOLDERS_REV" "$CACHE_DIR/ctp-folders"
fetch_pinned "$PAPIRUS_FOLDERS_REPO" "$PAPIRUS_FOLDERS_REV" "$CACHE_DIR/papirus-folders"

mkdir -p "$ICONS_DIR"
# Papirus-Dark points at Papirus files through symlinks; cp -a keeps them.
rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark"
cp -a "$CACHE_DIR/papirus/Papirus" "$CACHE_DIR/papirus/Papirus-Dark" "$ICONS_DIR/"
cp -r "$CACHE_DIR/ctp-folders/src/"* "$ICONS_DIR/Papirus/"

# papirus-folders finds the home directory through the user database, not
# $HOME, so point it at our icons directory explicitly.
USER_HOME="$HOME" XDG_DATA_HOME="$(dirname "$ICONS_DIR")" \
    bash "$CACHE_DIR/papirus-folders/papirus-folders" -t "$ICON_THEME" -C "cat-$FLAVOR-$ACCENT" >/dev/null
gtk-update-icon-cache -q "$ICONS_DIR/Papirus" 2>/dev/null || true
gtk-update-icon-cache -q "$ICONS_DIR/Papirus-Dark" 2>/dev/null || true

cosmic_set com.system76.CosmicTk icon_theme "\"$ICON_THEME\""
ok "Icons applied"
