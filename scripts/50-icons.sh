#!/usr/bin/env bash
# =============================================================================
# 50 · Icons
# 1. Copies Papirus and Papirus-Dark into ~/.local/share/icons (no sudo).
# 2. Adds the colored folders from catppuccin/papirus-folders.
# 3. Uses the papirus-folders script to switch the folder color to
#    cat-<flavor>-<accent> (e.g. cat-mocha-mauve).
# 4. Creates a tiny theme, <Papirus theme>-Catppuccin, that inherits it but
#    keeps COSMIC's own window controls (close, minimize, maximize, restore).
#    Papirus ships those icons drawn small inside 22/24 px canvases, which
#    COSMIC shrinks into its 16 px title bar buttons.
# 5. Sets that theme in COSMIC: based on Papirus-Dark for dark flavors and
#    Papirus (light) for Latte, so icons stay readable on the background.
# =============================================================================
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"

info "$ICON_THEME icons (cat-$FLAVOR-$ACCENT folders)"
fetch_pinned "$PAPIRUS_REPO" "$PAPIRUS_REV" "$CACHE_DIR/papirus"
fetch_pinned "$CTP_FOLDERS_REPO" "$CTP_FOLDERS_REV" "$CACHE_DIR/ctp-folders"
fetch_pinned "$PAPIRUS_FOLDERS_REPO" "$PAPIRUS_FOLDERS_REV" "$CACHE_DIR/papirus-folders"

mkdir -p "$ICONS_DIR"
# Papirus-Dark points at Papirus files through symlinks; cp -a keeps them.
rm -rf "$ICONS_DIR/Papirus" "$ICONS_DIR/Papirus-Dark"
cp -a "$CACHE_DIR/papirus/Papirus" "$CACHE_DIR/papirus/Papirus-Dark" "$ICONS_DIR/"
cp -r "$CACHE_DIR/ctp-folders/src/"* "$ICONS_DIR/Papirus/"

# papirus-folders finds the home directory through the user database, not
# $HOME, so point it at our icons directory explicitly.
USER_HOME="$HOME" XDG_DATA_HOME="$(dirname "$ICONS_DIR")" \
    bash "$CACHE_DIR/papirus-folders/papirus-folders" -t "$ICON_THEME" -C "cat-$FLAVOR-$ACCENT" >/dev/null
gtk-update-icon-cache -q "$ICONS_DIR/Papirus" 2>/dev/null || true
gtk-update-icon-cache -q "$ICONS_DIR/Papirus-Dark" 2>/dev/null || true

# The native window controls, from the cosmic-icons package.
native=""
IFS=: read -ra data_dirs <<<"${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
for dir in "${data_dirs[@]}"; do
    if [ -f "$dir/icons/Cosmic/scalable/actions/window-close-symbolic.svg" ]; then
        native="$dir/icons/Cosmic/scalable/actions"
        break
    fi
done

theme="$ICON_THEME"
if [ -n "$native" ]; then
    theme="$ICON_THEME-Catppuccin"
    rm -rf "${ICONS_DIR:?}/$theme"
    mkdir -p "$ICONS_DIR/$theme/scalable/actions"
    for icon in window-close window-minimize window-maximize window-restore; do
        if [ -f "$native/$icon-symbolic.svg" ]; then
            cp "$native/$icon-symbolic.svg" "$ICONS_DIR/$theme/scalable/actions/"
        fi
    done
    cat > "$ICONS_DIR/$theme/index.theme" <<THEME
[Icon Theme]
Name=$theme
Comment=$ICON_THEME with COSMIC's window controls (catppuccin-cosmic-setup)
Inherits=$ICON_THEME,Cosmic,hicolor
Directories=scalable/actions

[scalable/actions]
Context=Actions
Size=16
MinSize=8
MaxSize=512
Type=Scalable
THEME
else
    warn "COSMIC icons not found; window controls will use $ICON_THEME's"
fi

cosmic_set com.system76.CosmicTk icon_theme "\"$theme\""
ok "Icons applied ($theme)"
