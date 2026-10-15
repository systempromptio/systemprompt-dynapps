#!/bin/bash
# Step 20 — poll the run to a terminal status (RUN_TIMEOUT, default 2400 s),
# then save the run, its report and its events under .state/ for 30 and 90.
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 20 — wait"
require_pat
load_run_id
reset_step 20

deadline=$(( $(date +%s) + RUN_TIMEOUT ))
status=""; last=""
while :; do
  run=$(http_json GET "$BASE_URL/api/public/agent-farm/runs/$RUN_ID")
  status=$(printf '%s' "$run" | jq -r '.status // .run.status // empty' 2>/dev/null)
  [[ "$status" != "$last" ]] && { info "$(date +%H:%M:%S) status=${status:-?} (HTTP $(http_code))"; last="$status"; }
  case "$status" in succeeded|failed|cancelled|timed_out) break ;; esac
  if (( $(date +%s) >= deadline )); then break; fi
  sleep 10
done
printf '%s' "$run" > "$STATE_DIR/run.json"

# The report and eval land a moment after the run turns terminal.
for _ in 1 2 3 4 5 6 7 8 9 10 11 12; do
  report=$(http_json GET "$BASE_URL/api/public/agent-farm/runs/$RUN_ID/report")
  [[ "$(http_code)" == 200 ]] && printf '%s' "$report" | jq -e '.eval_passed != null' >/dev/null 2>&1 && break
  sleep 10
done
printf '%s' "$report" > "$STATE_DIR/report.json"
http_json GET "$BASE_URL/api/public/agent-farm/runs/$RUN_ID/events?after=0" > "$STATE_DIR/events.json"

printf '%s' "$run" | jq -C '{id, status, cost_usd, num_turns: (.num_turns // .run.num_turns), claude_session_id, transcript_url, dashboard_url}' 2>/dev/null | sed 's/^/  /'
record 20 "run terminal" "$([[ "$status" == succeeded ]] && echo true || echo false)" "status ${status:-unknown}"
record 20 "run report available" "$(jq -e 'has("run_id")' "$STATE_DIR/report.json" >/dev/null 2>&1 && echo true || echo false)"
step_result 20
