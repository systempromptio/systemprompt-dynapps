#!/bin/bash
# Step 70 — start one farm run as the user (PAT), poll it to a terminal status
# (timeout RUN_TIMEOUT, default 900 s), then print the result and an event tail.
# Env: CASE (else the first case of farm/evals/<agent>.yaml), FARM_AGENT.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 70 — run" "$FARM_AGENT"
load_user_pat
case_id=$(farm_first_case "$FARM_AGENT")
assert_nonempty "$case_id" "eval case id (set CASE or KIT_DIR)"
auth="Authorization: Bearer $USER_PAT"

started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
payload=$(jq -nc --arg f "$FARM_ID" --arg a "$FARM_AGENT" --arg c "$case_id" '{farm:$f, agent:$a, case:$c}')
cmd "curl -X POST $BASE_URL/api/public/agent-farm/runs -H 'Authorization: Bearer \$PAT' -d '$payload'"
created=$(http_json POST "$BASE_URL/api/public/agent-farm/runs" "$auth" "$payload")
assert_http "$HTTP_CODE" 2xx "create run"
run_id=$(printf '%s' "$created" | jq -r '.id // empty')
assert_nonempty "$run_id" "run id"
printf '%s' "$run_id" > "$STATE_RUN_ID"
printf '%s' "$started" > "$STATE_RUN_STARTED"
info "run $run_id ($(printf '%s' "$created" | jq -r '.status // "?"'))"

deadline=$(( $(date +%s) + RUN_TIMEOUT ))
status=""; last=""
while :; do
  run=$(http_json GET "$BASE_URL/api/public/agent-farm/runs/$run_id" "$auth")
  status=$(printf '%s' "$run" | jq -r '.status // empty')
  [[ "$status" != "$last" ]] && { info "$(date +%H:%M:%S) status=$status"; last="$status"; }
  case "$status" in succeeded|failed|cancelled|timed_out) break ;; esac
  if (( $(date +%s) >= deadline )); then fail "run $run_id not terminal after ${RUN_TIMEOUT}s (last: $status)"; exit 1; fi
  sleep 5
done

echo ""
printf '%s' "$run" | jq -C '{id, status, cost_usd, num_turns, claude_session_id, transcript_url, assertions: (.assertions // .assertion_results // null)}' | sed 's/^/  /'

step "Event tail"
events=$(http_json GET "$BASE_URL/api/public/agent-farm/runs/$run_id/events?after=0" "$auth")
printf '%s' "$events" | jq -r '(.events // .items // . ) | if type=="array" then .[-15:][] else empty end
  | "\(.seq // .id // "-")  \(.kind // .type // "?")  \((.summary // .text // .tool // "") | tostring | .[0:100])"' 2>/dev/null | sed 's/^/  /' || true

assert_eq "$status" "succeeded" "run terminal status"
failed=$(printf '%s' "$run" | jq '[(.assertions // .assertion_results // [])[] | select((.passed // .ok // true) == false)] | length' 2>/dev/null || echo 0)
assert_eq "$failed" "0" "failed assertions"
assert_nonempty "$(printf '%s' "$run" | jq -r '.claude_session_id // empty')" "claude_session_id recorded"
