#!/usr/bin/env bash
# =============================================================================
# 40 · Zed
# Zed uses the official "Catppuccin" (colors) and "Catppuccin Icons" (icons)
# extensions. Extensions can only be installed from Zed itself
# (Ctrl+Shift+X). This step checks settings.json and reports what is
# missing, without rewriting the file: it holds comments and other
# preferences of yours.
#
# Expected snippet in ~/.config/zed/settings.json:
#   "icon_theme": "Catppuccin Mocha",
#   "theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Zed"
settings="$HOME/.config/zed/settings.json"
name="Catppuccin ${FLAVOR^}"   # mocha → Mocha
if ! command -v zed >/dev/null && ! command -v zeditor >/dev/null; then
    warn "Zed not found, skipping"; exit 0
fi
ext="$HOME/.local/share/zed/extensions/installed"
for e in catppuccin catppuccin-icons; do
    [ -d "$ext/$e" ] && ok "Extension $e installed" || todo "Install the \"$e\" extension in Zed (Ctrl+Shift+X)"
done
if grep -q "\"dark\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Editor theme: $name"
else
    todo "In $settings, set \"theme\": { \"dark\": \"$name\" }"
fi
if grep -q "\"icon_theme\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Editor icons: $name"
else
    todo "In $settings, set \"icon_theme\": \"$name\""
fi
