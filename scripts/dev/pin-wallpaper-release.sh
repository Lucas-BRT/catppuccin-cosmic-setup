#!/usr/bin/env bash
# =============================================================================
# Pin the installer to a published ctp-wallpaper release.
#
#   scripts/dev/pin-wallpaper-release.sh v0.1.0
#
# Downloads the release's SHA256SUMS and writes wallpaper/RELEASE and
# wallpaper/SHA256SUMS. Review the diff and commit it: from then on the
# installer only runs binaries matching these hashes.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/../.."

tag=${1:?usage: $0 vX.Y.Z}
repo=Lucas-BRT/catppuccin-cosmic-setup
sums=$(curl -fsSL --proto '=https' --tlsv1.2 "https://github.com/$repo/releases/download/$tag/SHA256SUMS")
for arch in x86_64 aarch64; do
    grep -q " ctp-wallpaper-$arch-linux\$" <<<"$sums" || { echo "release $tag has no $arch binary" >&2; exit 1; }
done
printf '%s\n' "$sums" > wallpaper/SHA256SUMS
printf '%s\n' "$tag" > wallpaper/RELEASE
echo "Pinned $tag:"
cat wallpaper/SHA256SUMS
