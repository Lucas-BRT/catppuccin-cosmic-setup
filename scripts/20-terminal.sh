#!/usr/bin/env bash
# =============================================================================
# 20 · COSMIC Terminal color scheme
# The file comes from catppuccin/cosmic-desktop
# (themes/cosmic-term/catppuccin-mocha.ron). Its format did not change in
# COSMIC 1.10. The terminal has no command to import schemes, so this part
# still goes through its UI.
#
# The scheme does not set `background`: the terminal uses the desktop
# theme's background (Base #1e1e2e), so it also turns translucent with
# Frosted Glass.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "COSMIC Terminal"
fetch_pinned "$CTP_COSMIC_REPO" "$CTP_COSMIC_REV" "$CACHE_DIR/cosmic-desktop"
mkdir -p "$DATA_DIR/themes"
cp -r "$CACHE_DIR/cosmic-desktop/themes/cosmic-term" "$DATA_DIR/themes/"
scheme="$DATA_DIR/themes/cosmic-term/catppuccin-$FLAVOR.ron"

if grep -rqs "Catppuccin" "$COSMIC_DIR/com.system76.CosmicTerm/v1/"; then
    ok "Catppuccin scheme already imported in the terminal"
else
    todo "In COSMIC Terminal: View → Color schemes… → Import and pick:"
    todo "  $scheme"
fi
