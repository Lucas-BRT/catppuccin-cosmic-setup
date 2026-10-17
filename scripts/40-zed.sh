#!/usr/bin/env bash
# =============================================================================
# 40 · Zed
# O Zed usa as extensões oficiais "Catppuccin" (cores) e "Catppuccin Icons"
# (ícones). Extensões só são instaladas pelo próprio Zed (Ctrl+Shift+X).
# Este passo confere o settings.json e diz o que falta, sem reescrever o
# arquivo: ele tem comentários e outras preferências suas.
#
# Trecho esperado em ~/.config/zed/settings.json:
#   "icon_theme": "Catppuccin Mocha",
#   "theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Zed"
settings="$HOME/.config/zed/settings.json"
name="Catppuccin ${FLAVOR^}"   # mocha → Mocha
if ! command -v zed >/dev/null && ! command -v zeditor >/dev/null; then
    warn "Zed não encontrado, pulando"; exit 0
fi
ext="$HOME/.local/share/zed/extensions/installed"
for e in catppuccin catppuccin-icons; do
    [ -d "$ext/$e" ] && ok "Extensão $e instalada" || todo "Instale a extensão \"$e\" no Zed (Ctrl+Shift+X)"
done
if grep -q "\"dark\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Tema do editor: $name"
else
    todo "Em $settings, use \"theme\": { \"dark\": \"$name\" }"
fi
if grep -q "\"icon_theme\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Ícones do editor: $name"
else
    todo "Em $settings, use \"icon_theme\": \"$name\""
fi
