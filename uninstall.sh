#!/usr/bin/env bash
# =============================================================================
# Undoes the installation.
#
#   ./uninstall.sh            # restore the last backup made by setup.sh
#   ./uninstall.sh <dir>      # restore a specific backup
#
# What happens:
#   * the files the install changed in ~/.config/cosmic, ~/.bashrc and Zed's
#     settings.json go back to what they were, unless you changed them since
#     (those are kept and listed);
#   * the "catppuccin-cosmic-setup" block is removed from ~/.bashrc (in case the
#     backup already had it);
#   * Papirus, downloaded themes and the wallpapers stay on disk but are no
#     longer used. Run with PURGE=1 to delete them too (including the
#     catppuccin-<flavor>-<accent>/ folders made by WALLPAPER_STYLE=all and
#     the copy of this setup that install.sh unpacked, and ~/.local/bin/ctp-cosmic).
# =============================================================================
set -euo pipefail
# shellcheck source=scripts/lib.sh
source "$(dirname "$0")/scripts/lib.sh"

backup="${1:-$(cat "$CACHE_DIR/last-backup" 2>/dev/null || true)}"
[ -d "$backup" ] || { warn "Backup not found. Pass the directory: ./uninstall.sh ~/.config/catppuccin-backup-..."; exit 1; }

info "Restoring $backup"
if [ -d "$backup/before" ]; then
    # Only the files the install changed, and only if they still hold what it left.
    ctp backup restore "$backup"
else
    # Backup made by an older version: restore whole folders.
    if [ -d "$backup/cosmic" ]; then
        rm -rf "$COSMIC_DIR"
        cp -a "$backup/cosmic" "$COSMIC_DIR"
    fi
    [ -f "$backup/.bashrc" ] && cp -a "$backup/.bashrc" "$HOME/.bashrc"
    [ -f "$backup/settings.json" ] && cp -a "$backup/settings.json" "$HOME/.config/zed/settings.json"
fi
ctp prompt remove
ok "Settings restored"

if [ "${PURGE:-0}" = 1 ]; then
    rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark" \
           "$ICONS_DIR/Papirus-Catppuccin" "$ICONS_DIR/Papirus-Dark-Catppuccin" "$DATA_DIR" \
           "$HOME/.config/catppuccin" "$CACHE_DIR" \
           "$HOME"/Pictures/Wallpapers/catppuccin-*.png \
           "$HOME"/Pictures/Wallpapers/catppuccin-*/ \
           "${XDG_DATA_HOME:-$HOME/.local/share}/catppuccin-cosmic-setup" \
           "$HOME/.local/bin/ctp-cosmic"   # install.sh installs these
    ok "Downloaded files removed"
fi
echo "Log out and back in for everything to return to normal."
