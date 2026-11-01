#!/usr/bin/env bash
# =============================================================================
# 30 · Bash prompt
# `ctp-cosmic prompt install` renders config/bash/catppuccin-prompt.bash.in
# (built into the binary) with the colors of $FLAVOR and $ACCENT, installs it
# as ~/.config/catppuccin/catppuccin-prompt.bash and adds a marked block to
# ~/.bashrc that sources it. The markers let the block be replaced or removed
# without touching the rest of the file.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "Bash prompt ($FLAVOR, $ACCENT accent)"
ctp prompt install "$FLAVOR" "$ACCENT"
ok "Prompt installed (open a new terminal)"
