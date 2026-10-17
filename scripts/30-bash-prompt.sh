#!/usr/bin/env bash
# =============================================================================
# 30 · Bash prompt
# Renders config/bash/catppuccin-prompt.bash.in with the colors of $FLAVOR
# and $ACCENT, installs it as ~/.config/catppuccin/catppuccin-prompt.bash and
# adds a marked block to ~/.bashrc that sources it. The markers let the
# block be replaced or removed without touching the rest of the file.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "Bash prompt ($FLAVOR, $ACCENT accent)"
mkdir -p "$HOME/.config/catppuccin"
ctp render "$FLAVOR" "$ACCENT" < "$REPO_DIR/config/bash/catppuccin-prompt.bash.in" \
    > "$HOME/.config/catppuccin/catppuccin-prompt.bash"

start='# >>> catppuccin-cosmic-setup >>>'
end='# <<< catppuccin-cosmic-setup <<<'
touch "$HOME/.bashrc"
# Drop any block from a previous run before writing the new one.
sed -i "/^$start\$/,/^$end\$/d" "$HOME/.bashrc"
cat >> "$HOME/.bashrc" <<BLOCK
$start
[ -f ~/.config/catppuccin/catppuccin-prompt.bash ] && . ~/.config/catppuccin/catppuccin-prompt.bash
$end
BLOCK
ok "Prompt installed (open a new terminal)"
