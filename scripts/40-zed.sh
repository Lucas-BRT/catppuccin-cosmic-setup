#!/usr/bin/env bash
# =============================================================================
# 40 · Zed
# Zed uses the official "Catppuccin" (colors) and "Catppuccin Icons" (icons)
# extensions. Extensions can only be installed from Zed itself
# (Ctrl+Shift+X). This step applies the theme and icon theme to
# ~/.config/zed/settings.json using ctp-cosmic zed, preserving existing comments
# and preferences.
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
export PATH="$HOME/.local/bin:$PATH"
if ! command -v zed >/dev/null && ! command -v zeditor >/dev/null && [ ! -x "$HOME/.local/bin/zed" ]; then
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

ctp zed apply "$FLAVOR" --settings "$settings" >/dev/null
ok "Editor theme: $name"
ok "Editor icons: $name"
