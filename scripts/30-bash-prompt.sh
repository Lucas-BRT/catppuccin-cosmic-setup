#!/usr/bin/env bash
# =============================================================================
# 30 · Bash prompt
# Installs config/bash/catppuccin-prompt.bash into ~/.config/catppuccin/ and
# adds a marked block to ~/.bashrc that sources it. The markers let the
# block be replaced or removed without touching the rest of the file.
# =============================================================================
source "$(dirname "$0")/lib.sh"

info "Bash prompt"
mkdir -p "$HOME/.config/catppuccin"
cp "$REPO_DIR/config/bash/catppuccin-prompt.bash" "$HOME/.config/catppuccin/"

start='# >>> catppuccin-mocha >>>'
end='# <<< catppuccin-mocha <<<'
touch "$HOME/.bashrc"
# Drop any block from a previous run before writing the new one.
sed -i "/^$start\$/,/^$end\$/d" "$HOME/.bashrc"
cat >> "$HOME/.bashrc" <<BLOCK
$start
[ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
$end
BLOCK
ok "Prompt installed (open a new terminal)"
