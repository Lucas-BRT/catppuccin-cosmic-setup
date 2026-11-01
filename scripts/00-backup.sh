#!/usr/bin/env bash
# =============================================================================
# 00 · Backup
# `ctp-cosmic backup create` copies what the other steps may change (COSMIC's
# config, ~/.bashrc, Zed's settings.json) into
# ~/.config/catppuccin-backup-YYYY-MM-DD-HHMMSS/ and records that path as the
# last backup. setup.sh seals it when the steps end, listing what changed, so
# uninstall.sh can undo exactly that. The current theme is also exported as
# previous-theme.ron.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

dest="$HOME/.config/catppuccin-backup-$(date +%F-%H%M%S)"
info "Backing up to $dest"
ctp backup create "$dest" >/dev/null

# Export the current theme as a standalone .ron that can be re-imported:
#   cosmic-settings appearance import <backup>/previous-theme.ron
cosmic-settings appearance export "$dest/previous-theme.ron" >/dev/null 2>&1 || true
ok "Backup saved"
