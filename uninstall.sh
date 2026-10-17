#!/usr/bin/env bash
# =============================================================================
# Desfaz a instalação.
#
#   ./uninstall.sh            # restaura o último backup feito pelo install.sh
#   ./uninstall.sh <pasta>    # restaura um backup específico
#
# O que acontece:
#   * ~/.config/cosmic, ~/.bashrc e o settings.json do Zed voltam ao backup;
#   * o bloco "catppuccin-mocha" sai do ~/.bashrc (caso o backup já o tivesse);
#   * Papirus, temas baixados e o papel de parede ficam no disco, mas deixam
#     de ser usados. Para apagá-los também, rode com PURGE=1.
# =============================================================================
set -euo pipefail
source "$(dirname "$0")/scripts/lib.sh"

backup="${1:-$(cat "$CACHE_DIR/last-backup" 2>/dev/null || true)}"
[ -d "$backup" ] || { warn "Backup não encontrado. Passe a pasta: ./uninstall.sh ~/.config/catppuccin-backup-..."; exit 1; }

info "Restaurando $backup"
if [ -d "$backup/cosmic" ]; then
    rm -rf "$COSMIC_DIR"
    cp -a "$backup/cosmic" "$COSMIC_DIR"
fi
[ -f "$backup/.bashrc" ] && cp -a "$backup/.bashrc" "$HOME/.bashrc"
[ -f "$backup/bashrc" ]  && cp -a "$backup/bashrc" "$HOME/.bashrc"   # backup manual antigo
[ -f "$backup/settings.json" ] && cp -a "$backup/settings.json" "$HOME/.config/zed/settings.json"
[ -f "$backup/zed/settings.json" ] && cp -a "$backup/zed/settings.json" "$HOME/.config/zed/settings.json"
sed -i '/^# >>> catppuccin-mocha >>>$/,/^# <<< catppuccin-mocha <<<$/d' "$HOME/.bashrc"
ok "Configurações restauradas"

if [ "${PURGE:-0}" = 1 ]; then
    rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark" "$DATA_DIR" \
           "$HOME/.config/catppuccin" "$WALLPAPER" "$CACHE_DIR"
    ok "Arquivos baixados removidos"
fi
echo "Saia e entre na sessão para tudo voltar ao normal."
