#!/usr/bin/env bash
# =============================================================================
# 10 · Tema do desktop COSMIC
# Baixa os temas oficiais (catppuccin/cosmic-desktop) para
# ~/.local/share/catppuccin/themes/ e liga o modo escuro.
#
# A importação em si precisa ser feita pelas Configurações: é o
# cosmic-settings que transforma o .ron num tema completo (cores de
# botões, contêineres, sombras...). Escrever só o .ron na config não basta.
# Por isso o script abre a página certa e mostra qual arquivo escolher.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Tema COSMIC ($FLAVOR, acento $ACCENT)"
fetch_pinned "$CTP_COSMIC_REPO" "$CTP_COSMIC_REV" "$CACHE_DIR/cosmic-desktop"
mkdir -p "$DATA_DIR"
cp -r "$CACHE_DIR/cosmic-desktop/themes" "$DATA_DIR/"

theme="$DATA_DIR/themes/cosmic-settings/catppuccin-$FLAVOR-$ACCENT+round.ron"
[ -f "$theme" ] || { warn "Tema não encontrado: $theme"; exit 1; }

# Mocha é um tema escuro: garante o modo escuro do COSMIC.
cosmic_set com.system76.CosmicTheme.Mode is_dark true
ok "Temas salvos em $DATA_DIR/themes"

if [ -n "$(ls -A "$COSMIC_DIR/com.system76.CosmicTheme.Dark.Builder/v1" 2>/dev/null)" ] &&
   grep -qi "catppuccin-$FLAVOR" "$COSMIC_DIR/com.system76.CosmicTheme.Dark.Builder/v1/palette" 2>/dev/null; then
    ok "Tema Catppuccin já importado"
else
    todo "Em Configurações → Desktop → Aparência, clique em Importar e escolha:"
    todo "  $theme"
    [ "${NO_GUI:-0}" = 1 ] || (cosmic-settings appearance >/dev/null 2>&1 &)
fi
