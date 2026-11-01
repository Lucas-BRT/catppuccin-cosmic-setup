#!/usr/bin/env bash
# =============================================================================
# catppuccin-cosmic-setup installer.
#
#   curl -fsSL https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/latest/download/install.sh | bash
#
# Options are environment variables, steps are arguments:
#   curl -fsSL .../install.sh | FLAVOR=latte ACCENT=blue bash
#   curl -fsSL .../install.sh | bash -s -- icons wallpaper
#
# Downloads the release package for this machine, checks it against the
# release's SHA256SUMS, unpacks it into ~/.local/share/catppuccin-cosmic-setup
# (so uninstall.sh stays available), installs the ctp-cosmic binary into
# ~/.local/bin so it can be run again without this script, and runs setup.sh.
#
#   CATPPUCCIN_SETUP_VERSION=v0.1.0   install a specific release (default: latest)
#   CATPPUCCIN_SETUP_PACKAGE=file     install from a local package instead
# =============================================================================
set -euo pipefail

# Everything is inside main(), called on the last line, so a partially
# downloaded script never runs.
main() {
    local repo=Lucas-BRT/catppuccin-cosmic-setup
    local version=${CATPPUCCIN_SETUP_VERSION:-latest}
    local root=${XDG_DATA_HOME:-$HOME/.local/share}/catppuccin-cosmic-setup
    local arch asset base pkg dir bindir
    bindir=${CATPPUCCIN_SETUP_BINDIR:-$HOME/.local/bin}

    say() { printf '\033[38;2;203;166;247m==>\033[0m %s\n' "$*"; }
    die() { printf '\033[38;2;243;139;168merror:\033[0m %s\n' "$*" >&2; exit 1; }

    [ "$(uname -s)" = Linux ] || die "this setup is for Linux (Pop!_OS with COSMIC)"
    case "$(uname -m)" in
        x86_64 | amd64) arch=x86_64 ;;
        aarch64 | arm64) arch=aarch64 ;;
        *) die "no release package for $(uname -m)" ;;
    esac
    for cmd in curl tar sha256sum; do
        command -v "$cmd" >/dev/null || die "$cmd is required"
    done

    tmp=$(mktemp -d)   # global: the EXIT trap runs after main returns
    trap 'rm -rf "$tmp"' EXIT
    asset="catppuccin-cosmic-setup-$arch-linux.tar.gz"

    if [ -n "${CATPPUCCIN_SETUP_PACKAGE:-}" ]; then
        pkg=$CATPPUCCIN_SETUP_PACKAGE
        say "Using local package $pkg"
    else
        if [ "$version" = latest ]; then
            base="https://github.com/$repo/releases/latest/download"
        else
            base="https://github.com/$repo/releases/download/$version"
        fi
        say "Downloading catppuccin-cosmic-setup ($version, $arch)"
        curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/$asset" "$base/$asset" ||
            die "download failed: $base/$asset"
        curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/SHA256SUMS" "$base/SHA256SUMS" ||
            die "download failed: $base/SHA256SUMS"
        (cd "$tmp" && grep " $asset\$" SHA256SUMS | sha256sum -c --status) ||
            die "checksum mismatch for $asset"
        pkg="$tmp/$asset"
    fi

    # The package holds a single catppuccin-cosmic-setup-<version>/ directory.
    dir=$(tar -tzf "$pkg" | sed -n 1p)
    dir=${dir%%/*}
    case "$dir" in
        catppuccin-cosmic-setup-*) ;;
        *) die "unexpected package layout" ;;
    esac
    mkdir -p "$root"
    rm -rf "${root:?}/$dir"
    tar -xzf "$pkg" -C "$root"
    ln -sfn "$dir" "$root/current"
    say "Unpacked to $root/current ($(cat "$root/current/VERSION"))"

    # Copy to a temporary name and rename, so a running ctp-cosmic is not
    # overwritten in place.
    mkdir -p "$bindir"
    install -m 755 "$root/current/bin/ctp-cosmic" "$bindir/.ctp-cosmic.part"
    mv -f "$bindir/.ctp-cosmic.part" "$bindir/ctp-cosmic"
    say "Installed $bindir/ctp-cosmic (run it again any time: ctp-cosmic tui)"
    case ":$PATH:" in
        *":$bindir:"*) ;;
        *) say "$bindir is not in your PATH; add it to run ctp-cosmic by name" ;;
    esac

    "$root/current/setup.sh" "$@" </dev/null

    echo
    say "To undo: $root/current/uninstall.sh"
}

main "$@"
