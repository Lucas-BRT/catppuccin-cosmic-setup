#!/usr/bin/env bash
# =============================================================================
# Undoes the installation.
#
#   ./uninstall.sh            # restore the last backup made by install.sh
#   ./uninstall.sh <dir>      # restore a specific backup
#
# What happens:
#   * ~/.config/cosmic, ~/.bashrc and Zed's settings.json are restored;
#   * the "catppuccin-cosmic-setup" block is removed from ~/.bashrc (in case the
#     backup already had it);
#   * Papirus, downloaded themes and the wallpapers stay on disk but are no
#     longer used. Run with PURGE=1 to delete them too (including the
#     catppuccin-<flavor>-<accent>/ folders made by WALLPAPER_STYLE=all and
#     the copy of this setup that get.sh unpacked).
# =============================================================================
set -euo pipefail
# shellcheck source=scripts/lib.sh
source "$(dirname "$0")/scripts/lib.sh"

backup="${1:-$(cat "$CACHE_DIR/last-backup" 2>/dev/null || true)}"
[ -d "$backup" ] || { warn "Backup not found. Pass the directory: ./uninstall.sh ~/.config/catppuccin-backup-..."; exit 1; }

info "Restoring $backup"
if [ -d "$backup/cosmic" ]; then
    rm -rf "$COSMIC_DIR"
    cp -a "$backup/cosmic" "$COSMIC_DIR"
fi
[ -f "$backup/.bashrc" ] && cp -a "$backup/.bashrc" "$HOME/.bashrc"
[ -f "$backup/settings.json" ] && cp -a "$backup/settings.json" "$HOME/.config/zed/settings.json"
sed -i '/^# >>> catppuccin-cosmic-setup >>>$/,/^# <<< catppuccin-cosmic-setup <<<$/d' "$HOME/.bashrc"
ok "Settings restored"

if [ "${PURGE:-0}" = 1 ]; then
    rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark" \
           "$ICONS_DIR/Papirus-Catppuccin" "$ICONS_DIR/Papirus-Dark-Catppuccin" "$DATA_DIR" \
           "$HOME/.config/catppuccin" "$CACHE_DIR" \
           "$HOME"/Pictures/Wallpapers/catppuccin-*.png \
           "$HOME"/Pictures/Wallpapers/catppuccin-*/ \
           "${XDG_DATA_HOME:-$HOME/.local/share}/catppuccin-cosmic-setup"   # get.sh installs
    ok "Downloaded files removed"
fi
echo "Log out and back in for everything to return to normal."
