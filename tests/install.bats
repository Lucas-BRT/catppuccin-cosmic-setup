#!/usr/bin/env bats
# Tests of install.sh (the bootstrap) and of what a real `ctp-cosmic apply`
# and `uninstall` do to a home directory. Every test runs against a throwaway
# HOME and never touches the network: the binary is installed from a file.
#
#   cargo build --release
#   bats tests
# CTP_COSMIC_BIN points at another ctp-cosmic binary.

setup() {
    REPO=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
    CTP=${CTP_COSMIC_BIN:-$REPO/target/release/ctp-cosmic}
    [ -x "$CTP" ] || skip "build ctp-cosmic first (see the top of this file)"

    SANDBOX=$(mktemp -d)
    export HOME="$SANDBOX/home" XDG_CACHE_HOME="$SANDBOX/cache" NO_TUI=1 NO_COLOR=1
    unset XDG_DATA_HOME XDG_CONFIG_HOME FLAVOR ACCENT CATPPUCCIN_SETUP_VERIFY CATPPUCCIN_SETUP_SHA256
    export CATPPUCCIN_SETUP_BINARY="$CTP" CATPPUCCIN_SETUP_BINDIR="$SANDBOX/bin"
    CACHE=$XDG_CACHE_HOME/catppuccin-cosmic-setup
    mkdir -p "$HOME" "$SANDBOX/fakebin"
    # A cosmic-settings that accepts everything, and no real gh by default.
    printf '#!/bin/sh\nexit 0\n' > "$SANDBOX/fakebin/cosmic-settings"
    chmod +x "$SANDBOX/fakebin/cosmic-settings"
    export PATH="$SANDBOX/fakebin:$PATH"
}

teardown() { rm -rf "$SANDBOX"; }

fake_gh() {  # fake_gh EXIT_CODE
    printf '#!/bin/sh\nexit %s\n' "$1" > "$SANDBOX/fakebin/gh"
    chmod +x "$SANDBOX/fakebin/gh"
}

# An existing desktop: a COSMIC config and a symlinked ~/.bashrc.
existing_desktop() {
    mkdir -p "$HOME/.config/cosmic/x" "$SANDBOX/dots"
    echo original > "$HOME/.config/cosmic/x/file"
    echo "# my bashrc" > "$SANDBOX/dots/bashrc"
    ln -s "$SANDBOX/dots/bashrc" "$HOME/.bashrc"
}

@test "installs the binary and hands the arguments to apply" {
    run "$REPO/install.sh" --list
    [ "$status" -eq 0 ]
    [ -x "$SANDBOX/bin/ctp-cosmic" ]
    [[ $output == *"wallpaper"* ]]
}

@test "output is plain text without a terminal" {
    run "$REPO/install.sh" --list
    [[ $output != *$'\e'* ]]
}

@test "a binary that does not match CATPPUCCIN_SETUP_SHA256 is refused" {
    run env CATPPUCCIN_SETUP_SHA256=00 "$REPO/install.sh" --list
    [ "$status" -ne 0 ]
    [[ $output == *"CATPPUCCIN_SETUP_SHA256"* ]]
    [ ! -e "$SANDBOX/bin/ctp-cosmic" ]
}

@test "a binary that matches CATPPUCCIN_SETUP_SHA256 is accepted" {
    sum=$(sha256sum "$CTP")
    run env CATPPUCCIN_SETUP_SHA256="${sum%% *}" "$REPO/install.sh" --list
    [ "$status" -eq 0 ]
    [ -x "$SANDBOX/bin/ctp-cosmic" ]
}

@test "provenance: verified, warned about, or required" {
    fake_gh 0
    run "$REPO/install.sh" --list
    [ "$status" -eq 0 ]
    [[ $output == *"Provenance verified"* ]]

    fake_gh 1
    run "$REPO/install.sh" --list
    [ "$status" -eq 0 ]
    [[ $output == *"could not verify"* ]]

    run env CATPPUCCIN_SETUP_VERIFY=1 "$REPO/install.sh" --list
    [ "$status" -ne 0 ]
    [[ $output == *"no valid provenance"* ]]
}

@test "apply keeps a symlinked ~/.bashrc, is idempotent, and uninstall restores it" {
    existing_desktop
    "$REPO/install.sh" bash
    "$REPO/install.sh" bash
    [ -L "$HOME/.bashrc" ]
    [ "$(grep -c '>>> catppuccin-cosmic-setup' "$SANDBOX/dots/bashrc")" -eq 1 ]
    grep -q "# my bashrc" "$SANDBOX/dots/bashrc"

    run "$SANDBOX/bin/ctp-cosmic" uninstall
    [ "$status" -eq 0 ]
    [ -L "$HOME/.bashrc" ]
    [ "$(cat "$SANDBOX/dots/bashrc")" = "# my bashrc" ]
}

@test "a second run at the same time is refused" {
    "$REPO/install.sh" --list >/dev/null
    mkdir -p "$CACHE"
    flock "$CACHE/lock" sleep 20 &
    holder=$!
    sleep 0.5
    run "$SANDBOX/bin/ctp-cosmic" apply bash
    kill "$holder"
    [ "$status" -ne 0 ]
    [[ $output == *"in progress"* ]]
}

# --- apps (declarative manifests) -------------------------------------------

ctp() { "$SANDBOX/bin/ctp-cosmic" "$@"; }

# A throwaway "demo" app of your own that owns one file.
demo_app() {
    local dir=$HOME/.config/catppuccin-cosmic-setup/apps/demo
    mkdir -p "$dir"
    cat > "$dir/app.toml" <<TOML
name = "demo"
[[file]]
target = "~/.config/demo/theme"
template = "theme.in"
mode = "file"
TOML
    echo 'bg=@base@ fg=@text@ accent=@accent@' > "$dir/theme.in"
}

@test "your own app is themed by apply and undone by uninstall" {
    "$REPO/install.sh" --list >/dev/null
    demo_app
    run ctp apply demo
    [ "$status" -eq 0 ]
    [ "$(cat "$HOME/.config/demo/theme")" = "bg=#1e1e2e fg=#cdd6f4 accent=#cba6f7" ]
    run ctp apply demo --flavor latte --accent blue
    [ "$(cat "$HOME/.config/demo/theme")" = "bg=#eff1f5 fg=#4c4f69 accent=#1e66f5" ]
    run ctp uninstall
    [ "$status" -eq 0 ]
    [ "$(cat "$HOME/.config/demo/theme")" = "bg=#1e1e2e fg=#cdd6f4 accent=#cba6f7" ]
}

@test "a manifest that writes outside your home is refused, and does not break the others" {
    "$REPO/install.sh" --list >/dev/null
    demo_app
    mkdir -p "$HOME/.config/catppuccin-cosmic-setup/apps/evil"
    cat > "$HOME/.config/catppuccin-cosmic-setup/apps/evil/app.toml" <<'TOML'
name = "evil"
[[file]]
target = "~/../../tmp/pwned"
template = "t"
mode = "file"
TOML
    echo x > "$HOME/.config/catppuccin-cosmic-setup/apps/evil/t"
    run ctp app list
    [ "$status" -eq 0 ]
    [[ $output == *"demo"* && $output == *"skipped"* ]]
    run ctp apply evil
    [ "$status" -ne 0 ]
    [[ $output == *"unknown step 'evil'"* ]]
    run ctp apply demo
    [ "$status" -eq 0 ]
}

@test "firefox: your profile content is kept, and uninstall --only firefox restores it" {
    "$REPO/install.sh" --list >/dev/null
    profile=$HOME/.mozilla/firefox/abc.default-release
    mkdir -p "$profile/chrome"
    printf '[Install1]\nDefault=abc.default-release\n' > "$HOME/.mozilla/firefox/profiles.ini"
    echo 'user_pref("mine", 1);' > "$profile/user.js"
    echo '/* my css */' > "$profile/chrome/userChrome.css"
    printf '#!/bin/sh\nexit 0\n' > "$SANDBOX/fakebin/firefox"
    chmod +x "$SANDBOX/fakebin/firefox"

    run ctp apply firefox
    [ "$status" -eq 0 ]
    grep -q 'user_pref("mine", 1);' "$profile/user.js"
    grep -q legacyUserProfileCustomizations "$profile/user.js"
    grep -q '/\* my css \*/' "$profile/chrome/userChrome.css"
    grep -q -- '--toolbar-bgcolor: #1e1e2e' "$profile/chrome/userChrome.css"
    run ctp app check firefox
    [ "$status" -eq 0 ]

    run ctp uninstall --only firefox
    [ "$status" -eq 0 ]
    [ "$(cat "$profile/user.js")" = 'user_pref("mine", 1);' ]
    [ "$(cat "$profile/chrome/userChrome.css")" = '/* my css */' ]
    [ ! -e "$profile/chrome/userContent.css" ]
}

@test "firefox without a profile asks you to start it once" {
    "$REPO/install.sh" --list >/dev/null
    mkdir -p "$HOME/snap/firefox"
    run ctp apply firefox
    [ "$status" -eq 0 ]
    [[ $output == *"start Firefox once"* ]]
}
