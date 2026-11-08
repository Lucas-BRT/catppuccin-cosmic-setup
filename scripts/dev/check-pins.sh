#!/usr/bin/env bash
# =============================================================================
# Report upstream repositories pinned in src/apply.rs (Papirus and its
# Catppuccin folders) that have newer commits on their default branch.
# Dependabot cannot see these pins.
#
#   scripts/dev/check-pins.sh
#
# Exit status: 0 all current, 10 something is behind (listed on stdout),
# anything else is an error. Needs the gh CLI. Used by .github/workflows/pins.yml.
# =============================================================================
set -euo pipefail
cd "$(dirname "$0")/../.."
command -v gh >/dev/null || { echo "gh is required" >&2; exit 1; }

# Each pin is `const NAME: (&str, &str) = ("owner/repo", "commit");`
# spread over four lines.
pins=$(awk '/^const [A-Z_]+: \(&str, &str\) = \($/ {n=1; next}
    n==1 {gsub(/[",[:space:]]/, ""); slug=$0; n=2; next}
    n==2 {gsub(/[",[:space:]]/, ""); print slug, $0; n=0}' src/apply.rs)
[ -n "$pins" ] || { echo "no pins found in src/apply.rs" >&2; exit 1; }

behind=0
while read -r slug rev; do
    # "ahead_by" is how many commits the default branch has beyond the pin.
    ahead=$(gh api "repos/$slug/compare/$rev...HEAD" -q .ahead_by)
    if [ "$ahead" -gt 0 ]; then
        echo "$slug: pinned $rev is $ahead commit(s) behind (update it in src/apply.rs)"
        behind=1
    fi
done <<<"$pins"
[ "$behind" -eq 0 ] || exit 10
