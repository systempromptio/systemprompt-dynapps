#!/usr/bin/env bash
# Refuse unless gates.yml concluded success on exactly this commit, with the
# `Gates passed` aggregate job green. release.yml does not re-run the gates,
# so this is what stands between a red or unrun `next` push and a release.
set -euo pipefail
sha="${1:?usage: check-gates-green.sh <full-sha>}"
repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
run_id="$(gh run list -R "$repo" --workflow gates.yml --commit "$sha" --status success \
    --limit 1 --json databaseId --jq '.[0].databaseId // empty')"
if [ -z "$run_id" ]; then
    echo "ERROR: no successful gates.yml run on $sha — push next and wait for 'Gates passed' (gh run list --commit $sha)" >&2
    exit 1
fi
passed="$(gh run view "$run_id" -R "$repo" --json jobs \
    --jq '[.jobs[] | select(.name == "Gates passed" and .conclusion == "success")] | length')"
if [ "$passed" != "1" ]; then
    echo "ERROR: gates.yml run $run_id on $sha has no green 'Gates passed' job" >&2
    exit 1
fi
echo "✓ Gates passed on ${sha:0:9} (run $run_id)"
