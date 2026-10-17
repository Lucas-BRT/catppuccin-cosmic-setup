#!/usr/bin/env bash
# =============================================================================
# 00 · Backup
# Copies everything the other steps change into
# ~/.config/catppuccin-backup-YYYY-MM-DD-HHMMSS/ and records that path in
# ~/.cache/catppuccin-mocha-cosmic/last-backup (used by uninstall.sh).
# The current theme is also exported as previous-theme.ron.
# =============================================================================
source "$(dirname "$0")/lib.sh"

dest="$HOME/.config/catppuccin-backup-$(date +%F-%H%M%S)"
info "Backing up to $dest"
mkdir -p "$dest" "$CACHE_DIR"

for p in "$COSMIC_DIR" "$HOME/.bashrc" "$HOME/.config/zed/settings.json"; do
    [ -e "$p" ] && cp -a "$p" "$dest/"
done

# Export the current theme as a standalone .ron that can be re-imported:
#   cosmic-settings appearance import <backup>/previous-theme.ron
cosmic-settings appearance export "$dest/previous-theme.ron" >/dev/null 2>&1 || true

echo "$dest" > "$CACHE_DIR/last-backup"
ok "Backup saved"
