#!/usr/bin/env bash
# =============================================================================
# 50 · Icons (optional: only runs when named, e.g. ./setup.sh icons)
# Downloads the pinned Papirus and catppuccin/papirus-folders sources, then
# `ctp-cosmic icons apply`:
# 1. Copies Papirus and Papirus-Dark into ~/.local/share/icons (no sudo).
# 2. Adds the colored folders and switches them to cat-<flavor>-<accent>
#    (e.g. cat-mocha-mauve).
# 3. Creates a tiny theme, <Papirus theme>-Catppuccin, that inherits it but
#    keeps COSMIC's own window controls (close, minimize, maximize, restore).
#    Papirus ships those icons drawn small inside 22/24 px canvases, which
#    COSMIC shrinks into its 16 px title bar buttons.
# 4. Sets that theme in COSMIC: based on Papirus-Dark for dark flavors and
#    Papirus (light) for Latte, so icons stay readable on the background.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "Papirus icons (cat-$FLAVOR-$ACCENT folders)"
fetch_pinned "$PAPIRUS_REPO" "$PAPIRUS_REV" "$CACHE_DIR/papirus"
fetch_pinned "$CTP_FOLDERS_REPO" "$CTP_FOLDERS_REV" "$CACHE_DIR/ctp-folders"

theme=$(ctp icons apply "$FLAVOR" "$ACCENT" \
    --papirus "$CACHE_DIR/papirus" --folders "$CACHE_DIR/ctp-folders" \
    --icons-dir "$ICONS_DIR" --config "$COSMIC_DIR")
ok "Icons applied ($theme)"
