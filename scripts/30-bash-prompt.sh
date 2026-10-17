#!/usr/bin/env bash
# =============================================================================
# 30 · Prompt do bash
# Instala config/bash/catppuccin-prompt.bash em ~/.config/catppuccin/ e
# adiciona ao ~/.bashrc um bloco marcado que carrega esse arquivo.
# Os marcadores permitem trocar ou remover o bloco sem mexer no resto.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Prompt do bash"
mkdir -p "$HOME/.config/catppuccin"
cp "$REPO_DIR/config/bash/catppuccin-prompt.bash" "$HOME/.config/catppuccin/"

start='# >>> catppuccin-mocha >>>'
end='# <<< catppuccin-mocha <<<'
touch "$HOME/.bashrc"
# Tira um bloco antigo (deste instalador ou o bloco manual "--- Catppuccin
# Mocha prompt ---" da primeira versão) antes de escrever o novo.
sed -i -e "/^$start\$/,/^$end\$/d" \
       -e '/^# --- Catppuccin Mocha prompt ---$/,/^# --- fim Catppuccin ---$/d' "$HOME/.bashrc"
cat >> "$HOME/.bashrc" <<BLOCK
$start
[ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
$end
BLOCK
ok "Prompt instalado (abra um terminal novo)"
