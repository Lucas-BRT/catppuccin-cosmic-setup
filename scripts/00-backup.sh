#!/usr/bin/env bash
# =============================================================================
# 00 · Backup
# Copia tudo o que os outros passos vão alterar para
# ~/.config/catppuccin-backup-AAAA-MM-DD-HHMMSS/ e anota o caminho em
# ~/.cache/catppuccin-mocha-cosmic/last-backup (usado pelo uninstall.sh).
# =============================================================================
source "$(dirname "$0")/lib.sh"

dest="$HOME/.config/catppuccin-backup-$(date +%F-%H%M%S)"
info "Backup em $dest"
mkdir -p "$dest" "$CACHE_DIR"

for p in "$COSMIC_DIR" "$HOME/.bashrc" "$HOME/.config/zed/settings.json"; do
    [ -e "$p" ] && cp -a "$p" "$dest/"
done

echo "$dest" > "$CACHE_DIR/last-backup"
ok "Backup salvo"
