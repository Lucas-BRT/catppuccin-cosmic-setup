#!/usr/bin/env bash
# =============================================================================
# Pin installs from a git clone to a published release's ctp-cosmic binaries.
#
#   scripts/dev/pin-release.sh v0.2.0
#
# Downloads the release's SHA256SUMS and writes ctp-cosmic/RELEASE and
# ctp-cosmic/SHA256SUMS. Review the diff and commit it: from then on a clone
# only runs binaries matching these hashes. (Installs through get.sh use the
# binary bundled in the release package instead.)
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/../.."

tag=${1:?usage: $0 vX.Y.Z}
repo=Lucas-BRT/catppuccin-cosmic-setup
sums=$(curl -fsSL --proto '=https' --tlsv1.2 "https://github.com/$repo/releases/download/$tag/SHA256SUMS")
for arch in x86_64 aarch64; do
    grep -q " ctp-cosmic-$arch-linux\$" <<<"$sums" || { echo "release $tag has no $arch binary" >&2; exit 1; }
done
grep " ctp-cosmic-" <<<"$sums" > ctp-cosmic/SHA256SUMS
printf '%s\n' "$tag" > ctp-cosmic/RELEASE
echo "Pinned $tag:"
cat ctp-cosmic/SHA256SUMS
