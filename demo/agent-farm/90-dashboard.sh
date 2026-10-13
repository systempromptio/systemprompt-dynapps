#!/bin/bash
# Step 90 — every farm admin page renders (200) for the admin session and shows
# the run, agent and user it should.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 90 — dashboard"
load_user_pat
run_id=$(cat "$STATE_RUN_ID" 2>/dev/null || true)
assert_nonempty "$run_id" "run id from 70-run.sh"
jar=$(farm_admin_cookie)

# path|text the body must contain (empty = status only)
pages=(
  "/admin/farm|$FARM_ID"
  "/admin/farm/$FARM_ID|$FARM_AGENT"
  "/admin/farm/$FARM_ID/agents/$FARM_AGENT|$FARM_AGENT"
  "/admin/farm/runs|$run_id"
  "/admin/farm/runs/$run_id|$run_id"
  "/admin/farm/evals|$FARM_AGENT"
  "/admin/farm/people|$FARM_USER_NAME"
  "/admin/marketplaces/$FARM_ID|$FARM_PLUGIN"
  "/admin/sync|$FARM_ID"
)
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
bad=0
for entry in "${pages[@]}"; do
  path="${entry%%|*}"; want="${entry#*|}"
  code=$(curl -s -o "$tmp" -w '%{http_code}' -b "$jar" "$BASE_URL$path" || echo 000)
  if [[ "$code" != "200" ]]; then fail "$path → HTTP $code"; bad=$((bad+1)); continue; fi
  if [[ -n "$want" ]] && ! grep -q -- "$want" "$tmp"; then
    # The people page may show the e-mail instead of the username.
    if [[ "$path" == "/admin/farm/people" ]] && grep -q -- "$FARM_USER_EMAIL" "$tmp"; then :; else
      fail "$path → 200 but missing '$want'"; bad=$((bad+1)); continue; fi
  fi
  pass "$path → 200${want:+ (has $want)}"
done
(( bad == 0 )) || { fail "$bad dashboard page(s) failed"; exit 1; }
info "Open: $ADMIN_URL/admin/farm/runs/$run_id"
