#!/usr/bin/env bash
# =============================================================================
# catppuccin-cosmic-setup installer.
#
#   curl -fsSL https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/latest/download/install.sh | bash
#
# Downloads ctp-cosmic for this machine, checks it against the release's
# SHA256SUMS, puts it in ~/.local/bin and runs `ctp-cosmic apply`. Everything
# after the first install is ctp-cosmic itself:
#
#   ctp-cosmic            change flavor, accent or steps (interactive)
#   ctp-cosmic update     install the latest release
#   ctp-cosmic uninstall  undo what apply changed
#
# Options for apply are arguments and environment variables:
#   curl -fsSL .../install.sh | FLAVOR=latte ACCENT=blue bash
#   curl -fsSL .../install.sh | bash -s -- bash wallpaper
#
#   CATPPUCCIN_SETUP_VERSION=v0.4.0    install a specific release (default: latest)
#   CATPPUCCIN_SETUP_BINARY=file       install this binary instead of downloading
#   CATPPUCCIN_SETUP_BINDIR=dir        where to put it (default: ~/.local/bin)
# =============================================================================
set -euo pipefail

# Everything is inside main(), called on the last line, so a partially
# downloaded script never runs.
main() {
    local repo=Lucas-BRT/catppuccin-cosmic-setup
    local version=${CATPPUCCIN_SETUP_VERSION:-latest}
    local bindir=${CATPPUCCIN_SETUP_BINDIR:-$HOME/.local/bin}
    local arch asset base src

    say() { printf '\033[38;2;203;166;247m==>\033[0m %s\n' "$*"; }
    die() { printf '\033[38;2;243;139;168merror:\033[0m %s\n' "$*" >&2; exit 1; }

    [ "$(uname -s)" = Linux ] || die "this setup is for Linux (Pop!_OS with COSMIC)"
    case "$(uname -m)" in
        x86_64 | amd64) arch=x86_64 ;;
        aarch64 | arm64) arch=aarch64 ;;
        *) die "no release for $(uname -m)" ;;
    esac
    asset="ctp-cosmic-$arch-linux"

    tmp=$(mktemp -d)   # global: the EXIT trap runs after main returns
    trap 'rm -rf "$tmp"' EXIT

    if [ -n "${CATPPUCCIN_SETUP_BINARY:-}" ]; then
        src=$CATPPUCCIN_SETUP_BINARY
        say "Using local binary $src"
    else
        for cmd in curl sha256sum; do
            command -v "$cmd" >/dev/null || die "$cmd is required"
        done
        if [ "$version" = latest ]; then
            base="https://github.com/$repo/releases/latest/download"
        else
            base="https://github.com/$repo/releases/download/$version"
        fi
        say "Downloading ctp-cosmic ($version, $arch)"
        curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/$asset" "$base/$asset" ||
            die "download failed: $base/$asset"
        curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/SHA256SUMS" "$base/SHA256SUMS" ||
            die "download failed: $base/SHA256SUMS"
        (cd "$tmp" && grep " $asset\$" SHA256SUMS | sha256sum -c --status) ||
            die "checksum mismatch for $asset"
        src="$tmp/$asset"
    fi

    # Copy to a temporary name and rename, so a running ctp-cosmic is not
    # overwritten in place.
    mkdir -p "$bindir"
    install -m 755 "$src" "$bindir/.ctp-cosmic.part"
    mv -f "$bindir/.ctp-cosmic.part" "$bindir/ctp-cosmic"
    say "Installed $bindir/ctp-cosmic"
    case ":$PATH:" in
        *":$bindir:"*) ;;
        *) say "$bindir is not in your PATH; add it to run ctp-cosmic by name" ;;
    esac

    "$bindir/ctp-cosmic" apply "$@" </dev/null

    echo
    say "Change it later: ctp-cosmic   Update: ctp-cosmic update   Undo: ctp-cosmic uninstall"
}

main "$@"
