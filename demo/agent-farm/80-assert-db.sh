#!/bin/bash
# Step 80 — the run left an attributed trail: gateway requests charged to the
# user's PAT, hook events for the kit plugin, and governance decisions.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 80 — database trail"
load_user_pat
[[ -s "$STATE_RUN_ID" && -s "$STATE_RUN_STARTED" ]] || { fail "no run recorded; run 70-run.sh first"; exit 1; }
run_id=$(cat "$STATE_RUN_ID"); since=$(cat "$STATE_RUN_STARTED")
u=$(sql_q "$FARM_USER_ID"); s=$(sql_q "$since"); r=$(sql_q "$run_id")

step "1. ai_requests attributed to the user's PAT"
assert_min "$(db_count "SELECT COUNT(*) FROM ai_requests WHERE user_id = $u AND api_key_id IS NOT NULL AND created_at >= $s::timestamptz")" 1 "ai_requests with api_key_id since run start"

step "2. plugin_usage_events for $FARM_PLUGIN"
assert_min "$(db_count "SELECT COUNT(*) FROM plugin_usage_events WHERE user_id = $u AND plugin_id = $(sql_q "$FARM_PLUGIN") AND created_at >= $s::timestamptz")" 1 "plugin_usage_events since run start"
tagged=$(db_count "SELECT COUNT(*) FROM plugin_usage_events WHERE user_id = $u AND metadata::text LIKE '%' || $r || '%'")
info "events carrying farm_run_id=$run_id: $tagged"
(( tagged >= 1 )) && pass "farm_run_id present in event metadata" || warn "no event metadata names the run id"

step "3. Governance decisions"
if (( $(db_count "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'governance_decisions'") == 1 )); then
  assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE user_id = $u AND created_at >= $s::timestamptz")" 1 "governance_decisions since run start"
  cli_json infra db query "SELECT decision, policy, COUNT(*) AS n FROM governance_decisions WHERE user_id = $u AND created_at >= $s::timestamptz GROUP BY 1,2 ORDER BY 3 DESC" \
    | jq -r '.items[]? | "    \(.decision)  \(.policy)  \(.n)"'
else
  warn "governance_decisions table absent; skipped"
fi
