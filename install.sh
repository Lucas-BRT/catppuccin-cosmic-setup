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
#   CATPPUCCIN_SETUP_SHA256=hash       require the binary to have this SHA-256
#                                      (take it from a source you trust; the
#                                      release's SHA256SUMS only catches a
#                                      corrupted download)
#   CATPPUCCIN_SETUP_VERIFY=1          require a valid signed provenance
#                                      attestation (needs the gh CLI); without
#                                      it, gh is used when installed and a
#                                      failure only warns
#   NO_COLOR=1                         plain output
# =============================================================================
set -euo pipefail

# Everything is inside main(), called on the last line, so a partially
# downloaded script never runs.
main() {
    local repo=Lucas-BRT/catppuccin-cosmic-setup
    local version=${CATPPUCCIN_SETUP_VERSION:-latest}
    local bindir=${CATPPUCCIN_SETUP_BINDIR:-$HOME/.local/bin}
    local arch asset base src sum

    local on='' off=''
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then on=$'\e[38;2;203;166;247m'; off=$'\e[0m'; fi
    say() { printf '%s==>%s %s\n' "$on" "$off" "$*"; }
    warn() { printf 'warning: %s\n' "$*" >&2; }
    die() { printf 'error: %s\n' "$*" >&2; exit 1; }

    [ "$(id -u)" -ne 0 ] || die "do not run this as root: it configures your own user's desktop (no sudo needed)"
    [ "$(uname -s)" = Linux ] || die "this setup is for Linux (Pop!_OS with COSMIC)"
    case "$(uname -m)" in
        x86_64 | amd64) arch=x86_64 ;;
        aarch64 | arm64) arch=aarch64 ;;
        *) die "no release for $(uname -m)" ;;
    esac
    asset="ctp-cosmic-$arch-linux"

    tmp=$(mktemp -d)   # global: the EXIT trap runs after main returns
    trap 'rm -rf "$tmp"' EXIT
    trap 'exit 130' INT TERM

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
        curl -fsSL --retry 3 --proto '=https' --tlsv1.2 -o "$tmp/$asset" "$base/$asset" ||
            die "download failed: $base/$asset"
        curl -fsSL --retry 3 --proto '=https' --tlsv1.2 -o "$tmp/SHA256SUMS" "$base/SHA256SUMS" ||
            die "download failed: $base/SHA256SUMS"
        (cd "$tmp" && grep " $asset\$" SHA256SUMS | sha256sum -c --status) ||
            die "checksum mismatch for $asset"
        src="$tmp/$asset"
    fi

    if [ -n "${CATPPUCCIN_SETUP_SHA256:-}" ]; then
        sum=$(sha256sum "$src")
        [ "${sum%% *}" = "$CATPPUCCIN_SETUP_SHA256" ] || die "binary does not match CATPPUCCIN_SETUP_SHA256"
    fi

    # Signed build provenance (GitHub artifact attestation): proves the file
    # was built by this repository's release workflow.
    if command -v gh >/dev/null; then
        if gh attestation verify "$src" --repo "$repo" >/dev/null 2>&1; then
            say "Provenance verified (gh attestation)"
        elif [ "${CATPPUCCIN_SETUP_VERIFY:-0}" = 1 ]; then
            die "no valid provenance attestation for $(basename "$src")"
        else
            warn "could not verify the provenance attestation (older release, or gh is not signed in); continuing with the checksum only"
        fi
    elif [ "${CATPPUCCIN_SETUP_VERIFY:-0}" = 1 ]; then
        die "CATPPUCCIN_SETUP_VERIFY=1 needs the gh CLI"
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
