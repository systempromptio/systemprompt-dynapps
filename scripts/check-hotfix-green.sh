#!/usr/bin/env bash
set -euo pipefail
head="${1:?usage: check-hotfix-green.sh <head> <ref> <base>}"
ref="${2:?missing hotfix ref}"
base="${3:?missing main base}"
repo=systempromptio/systemprompt-dynapps
die() { echo "hotfix-proof: $*" >&2; exit 1; }
[[ "$head" =~ ^[0-9a-f]{40}$ && "$base" =~ ^[0-9a-f]{40}$ ]] || die 'expected full commit identities'
[[ "$ref" =~ ^hotfix/[0-9]+\.[0-9]+\.[0-9]+/$base$ ]] || die 'hotfix ref differs from frozen base'
get_proof() {
    gh api -X GET "repos/$repo/actions/workflows/gates.yml/runs" -f event=pull_request -f branch="$ref" -f head_sha="$head" -f per_page=1
}
proof="$(get_proof)"
# Why: GitHub empties a run's pull_requests once its PR merges, so after the
# merge the PR itself is matched by head branch, head and base instead.
merged="$(gh api -X GET "repos/$repo/pulls" -f state=closed -f head="${repo%%/*}:$ref" -f base=main -f per_page=5 |
    jq --arg head "$head" --arg base "$base" '[.[] | select(.merged_at != null and .head.sha == $head and .base.sha == $base)] | length')" || merged=0
identity="$(jq -er --arg head "$head" --arg ref "$ref" --arg base "$base" --argjson merged "${merged:-0}" '
    .workflow_runs[0] | select(.event == "pull_request" and .head_branch == $ref and .head_sha == $head and .status == "completed" and .conclusion == "success") |
    select(any(.pull_requests[]; .base.ref == "main" and .base.sha == $base and .head.sha == $head) or (.pull_requests == [] and $merged == 1)) |
    [.id,.run_attempt] | @tsv' <<<"$proof")" || die 'latest exact hotfix PR run is missing, pending, or unsuccessful'
IFS=$'\t' read -r run_id attempt <<<"$identity"
[[ "$run_id" =~ ^[0-9]+$ && "$attempt" =~ ^[1-9][0-9]*$ ]] || die 'missing run identity'
gh api --paginate "repos/$repo/actions/runs/$run_id/attempts/$attempt/jobs?per_page=100" | jq -se '
    [.[].jobs[]] as $jobs |
    ["Gates passed", "Verify frozen hotfix", "Static (fmt, sqlx cache, source gates)", "Lint (clippy ×3, rustdoc, MSRV)", "Test (unit, integration, admin contract)"] |
    all(.[]; . as $name | [$jobs[] | select(.name == $name)] | length == 1 and all(.[]; .status == "completed" and .conclusion == "success"))' >/dev/null || die 'every hotfix gate must succeed exactly once'
get_proof | jq -e --argjson id "$run_id" --argjson attempt "$attempt" '.workflow_runs[0] | .id == $id and .run_attempt == $attempt and .status == "completed" and .conclusion == "success"' >/dev/null || die 'proof changed during verification'
printf '%s\t%s\n' "$run_id" "$attempt"
