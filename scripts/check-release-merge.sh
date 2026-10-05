#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
repo=Astound-Digital/systemprompt-astound
merge="${1:?usage: check-release-merge.sh <main-sha>}"
die() { echo "release-proof: $*" >&2; exit 1; }
[[ "$merge" =~ ^[0-9a-f]{40}$ ]] || die 'expected full merge SHA'
[ "$(git rev-parse HEAD)" = "$merge" ] || die 'checkout differs from release commit'
parents="$(git show -s --format=%P "$merge")"
read -r base head extra <<<"$parents"
[ -n "$base" ] && [ -n "$head" ] && [ -z "$extra" ] || die 'release is not a two-parent promotion merge'
[ "$(git rev-parse "$merge^{tree}")" = "$(git rev-parse "$head^{tree}")" ] || die 'merged tree differs from candidate'
prs="$(gh api --paginate "repos/$repo/commits/$merge/pulls?per_page=100")"
pr="$(jq -sce --arg merge "$merge" --arg head "$head" --arg repo "$repo" '
    [ .[][] | select(.merged_at != null and .merge_commit_sha == $merge and .base.ref == "main" and .head.sha == $head and .head.repo.full_name == $repo) ] |
    if length == 1 then .[0] else error("expected exactly one merged promotion PR") end' <<<"$prs")" || die 'missing unambiguous merged promotion PR'
ref="$(jq -r .head.ref <<<"$pr")"
if [[ "$ref" =~ ^hotfix/([0-9]+\.[0-9]+\.[0-9]+)/([0-9a-f]{40})$ ]]; then
    version="${BASH_REMATCH[1]}"
    [ "${BASH_REMATCH[2]}" = "$base" ] || die 'hotfix base differs from frozen main'
    scripts/sync-release-version.sh "$version" --check
    scripts/sync-core-version.sh --check
    if [ "${HOTFIX_VALIDATE_IN_RELEASE:-0}" = 1 ]; then
        [ "${GITHUB_ACTIONS:-false}" = true ] && [ "${GITHUB_SHA:-}" = "$merge" ] || die 'release validation is Actions-only and must identify this merge'
        run_id="${GITHUB_RUN_ID:?missing release run identity}"
        attempt="${GITHUB_RUN_ATTEMPT:?missing release run attempt}"
        echo "Hotfix validated in this release workflow before publication"
    else
        identity="$(bash scripts/check-hotfix-green.sh "$head" "$ref" "$base")" || die 'hotfix proof did not succeed'
        IFS=$'\t' read -r run_id attempt <<<"$identity"
    fi
else
[[ "$ref" =~ ^promote/([0-9]+\.[0-9]+\.[0-9]+)/([0-9a-f]{40})/([0-9a-f]{40})$ ]] || die 'PR is not a frozen promotion'
[ "${BASH_REMATCH[2]}" = "$base" ] && [ "${BASH_REMATCH[3]}" = "$head" ] || die 'merge parents differ from frozen promotion'
version="${BASH_REMATCH[1]}"
scripts/sync-release-version.sh "$version" --check
scripts/sync-core-version.sh --check
bash scripts/check-gates-green.sh "$head"
identity="$(bash scripts/check-promotion-green.sh "$head" "$ref")" || die 'promotion proof did not succeed'
IFS=$'\t' read -r run_id attempt <<<"$identity"
fi
echo "Release merge $merge verified against frozen candidate $head, base $base, promotion run $run_id attempt $attempt"
