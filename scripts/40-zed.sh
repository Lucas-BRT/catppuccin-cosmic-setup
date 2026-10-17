#!/usr/bin/env bash
# =============================================================================
# 40 · Zed
# Zed uses the official "Catppuccin" (colors) and "Catppuccin Icons" (icons)
# extensions. Extensions can only be installed from Zed itself
# (Ctrl+Shift+X). This step checks settings.json and reports what is
# missing, without rewriting the file: it holds comments and other
# preferences of yours.
#
# Expected snippet in ~/.config/zed/settings.json (Mocha shown; Latte goes
# under "light" instead of "dark"):
#   "icon_theme": "Catppuccin Mocha",
#   "theme": { "mode": "system", "light": "One Light", "dark": "Catppuccin Mocha" }
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "Zed"
settings="$HOME/.config/zed/settings.json"
name="Catppuccin $(ctp palette name "$FLAVOR")"   # e.g. "Catppuccin Frappé"
if ctp palette is-dark "$FLAVOR"; then mode=dark; else mode=light; fi
if ! command -v zed >/dev/null && ! command -v zeditor >/dev/null; then
    warn "Zed not found, skipping"; exit 0
fi
ext="$HOME/.local/share/zed/extensions/installed"
for e in catppuccin catppuccin-icons; do
    if [ -d "$ext/$e" ]; then
        ok "Extension $e installed"
    else
        todo "Install the \"$e\" extension in Zed (Ctrl+Shift+X)"
    fi
done
if grep -q "\"$mode\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Editor theme: $name"
else
    todo "In $settings, set \"theme\": { \"$mode\": \"$name\" }"
fi
if grep -q "\"icon_theme\": *\"$name\"" "$settings" 2>/dev/null; then
    ok "Editor icons: $name"
else
    todo "In $settings, set \"icon_theme\": \"$name\""
fi
