#!/usr/bin/env bash
# =============================================================================
# Build the release package that get.sh downloads.
#
#   scripts/dev/package.sh v0.1.0 x86_64 path/to/ctp-wallpaper out-dir/
#
# Writes out-dir/catppuccin-cosmic-setup-<arch>-linux.tar.gz holding
# catppuccin-cosmic-setup-<version>/ with the installer, its data and the
# ctp-wallpaper binary in bin/. Used by .github/workflows/release.yml.
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

mkdir -p "$work/$name/bin" "$work/$name/wallpaper" "$work/$name/scripts"
cp -r install.sh uninstall.sh LICENSE README.md docs lib themes config "$work/$name/"
cp scripts/*.sh "$work/$name/scripts/"
cp wallpaper/styles.txt "$work/$name/wallpaper/"
install -m 755 "$binary" "$work/$name/bin/ctp-wallpaper"
printf '%s\n' "$version" > "$work/$name/VERSION"
find "$work/$name" -name __pycache__ -prune -exec rm -rf {} +

mkdir -p "$outdir"
tar -C "$work" -czf "$outdir/catppuccin-cosmic-setup-$arch-linux.tar.gz" "$name"
echo "$outdir/catppuccin-cosmic-setup-$arch-linux.tar.gz"
