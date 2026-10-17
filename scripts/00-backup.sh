#!/usr/bin/env bash
# =============================================================================
# 00 · Backup
# Copia tudo o que os outros passos vão alterar para
# ~/.config/catppuccin-backup-AAAA-MM-DD-HHMMSS/ e anota o caminho em
# ~/.cache/catppuccin-mocha-cosmic/last-backup (usado pelo uninstall.sh).
# O tema atual também é exportado como tema-anterior.ron.
# =============================================================================
source "$(dirname "$0")/lib.sh"

dest="$HOME/.config/catppuccin-backup-$(date +%F-%H%M%S)"
info "Backup em $dest"
mkdir -p "$dest" "$CACHE_DIR"

for p in "$COSMIC_DIR" "$HOME/.bashrc" "$HOME/.config/zed/settings.json"; do
    [ -e "$p" ] && cp -a "$p" "$dest/"
done

# Também exporta o tema atual num .ron que dá para reimportar sozinho:
#   cosmic-settings appearance import <backup>/tema-anterior.ron
cosmic-settings appearance export "$dest/tema-anterior.ron" >/dev/null 2>&1 || true

echo "$dest" > "$CACHE_DIR/last-backup"
ok "Backup salvo"
