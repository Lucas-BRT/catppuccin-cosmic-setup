#!/usr/bin/env bash
# =============================================================================
# Build the release package that get.sh downloads.
#
#   scripts/dev/package.sh v0.2.0 x86_64 path/to/ctp-cosmic out-dir/
#
# Writes out-dir/catppuccin-cosmic-setup-<arch>-linux.tar.gz holding
# catppuccin-cosmic-setup-<version>/ with the installer, its data and the
# ctp-cosmic binary in bin/. Used by .github/workflows/release.yml.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/../.."

version=${1:?usage: $0 VERSION ARCH BINARY OUTDIR}
arch=${2:?usage: $0 VERSION ARCH BINARY OUTDIR}
binary=${3:?usage: $0 VERSION ARCH BINARY OUTDIR}
outdir=${4:?usage: $0 VERSION ARCH BINARY OUTDIR}

name="catppuccin-cosmic-setup-$version"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/$name/bin" "$work/$name/scripts" "$work/$name/docs"
cp -r install.sh uninstall.sh LICENSE README.md themes config "$work/$name/"
cp docs/*.md "$work/$name/docs/"
cp scripts/*.sh "$work/$name/scripts/"
install -m 755 "$binary" "$work/$name/bin/ctp-cosmic"
printf '%s\n' "$version" > "$work/$name/VERSION"

mkdir -p "$outdir"
tar -C "$work" -czf "$outdir/catppuccin-cosmic-setup-$arch-linux.tar.gz" "$name"
echo "$outdir/catppuccin-cosmic-setup-$arch-linux.tar.gz"
