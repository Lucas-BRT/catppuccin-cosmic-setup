#!/usr/bin/env bash
# =============================================================================
# 20 · Esquema de cores do COSMIC Terminal
# O arquivo vem de catppuccin/cosmic-desktop
# (themes/cosmic-term/catppuccin-mocha.ron). O formato não mudou no
# COSMIC 1.10. O terminal não tem comando para importar esquemas, então
# esta parte continua pela interface.
#
# O esquema não define `background`: o terminal usa o fundo do tema do
# desktop (Base #1e1e2e), e assim também fica translúcido com Frosted Glass.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "COSMIC Terminal"
fetch_pinned "$CTP_COSMIC_REPO" "$CTP_COSMIC_REV" "$CACHE_DIR/cosmic-desktop"
mkdir -p "$DATA_DIR/themes"
cp -r "$CACHE_DIR/cosmic-desktop/themes/cosmic-term" "$DATA_DIR/themes/"
scheme="$DATA_DIR/themes/cosmic-term/catppuccin-$FLAVOR.ron"

if grep -rqs "Catppuccin" "$COSMIC_DIR/com.system76.CosmicTerm/v1/"; then
    ok "Esquema Catppuccin já importado no terminal"
else
    todo "No COSMIC Terminal: View → Color schemes… → Import e escolha:"
    todo "  $scheme"
fi
