#!/usr/bin/env bash
# =============================================================================
# 20 · Esquema de cores do COSMIC Terminal
# O arquivo vem do mesmo repositório do passo 10
# (themes/cosmic-term/catppuccin-mocha.ron). Assim como o tema do desktop,
# o terminal só aceita esquemas novos pela própria interface.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "COSMIC Terminal"
scheme="$DATA_DIR/themes/cosmic-term/catppuccin-$FLAVOR.ron"
[ -f "$scheme" ] || { warn "Rode o passo 10 antes (falta $scheme)"; exit 1; }

if grep -rqs "Catppuccin" "$COSMIC_DIR/com.system76.CosmicTerm/v1/"; then
    ok "Esquema Catppuccin já importado no terminal"
else
    todo "No COSMIC Terminal: View → Color schemes… → Import e escolha:"
    todo "  $scheme"
fi
