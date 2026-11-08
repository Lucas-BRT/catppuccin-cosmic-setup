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
