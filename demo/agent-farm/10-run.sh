#!/bin/bash
# Step 10 — start the night audit exactly as the 02:00 scheduler does: run the
# `farm_scheduled_run` job remotely on the tenant (it queues
# accounting_night_audit on dynapps-night-audit as Ed, triggered_via
# scheduler), then pick up the run it created. The public API cannot claim a
# scheduler trigger, by design. RUN_VIA=api launches over the API with Ed's PAT
# instead (triggered_via api). DYNBRAIN_PARENT_ID only reaches the agent on
# the API path; the scheduled run relies on the "Auditorías nocturnas" lookup.
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
RUN_VIA="${RUN_VIA:-scheduler}"
header "Night audit 10 — run" "$FARM_ID/$FARM_AGENT via $RUN_VIA"
reset_step 10
rm -f "$STATE_DIR/run_id" "$STATE_DIR/run.json" "$STATE_DIR/report.json" "$STATE_DIR/events.json" \
  "$STATE_DIR/followup.json" "$STATE_DIR/dynbrain_article_id"
launched=$(sql_rows "SELECT NOW() AS t" | jq -r '.[0].t // empty')
[[ -n "$launched" ]] || { record 10 "database reachable" false; exit 1; }

if [[ "$RUN_VIA" == api ]]; then
  require_pat
  prompt=""
  [[ -n "${DYNBRAIN_PARENT_ID:-}" ]] && prompt="dynbrain_parent_id: $DYNBRAIN_PARENT_ID"
  payload=$(jq -nc --arg f "$FARM_ID" --arg a "$FARM_AGENT" --arg c "$CASE" --arg p "$prompt" \
    '{farm:$f, agent:$a, case:$c} + (if $p == "" then {} else {prompt:$p} end)')
  cmd "curl -X POST $BASE_URL/api/public/agent-farm/runs -H 'Authorization: Bearer \$ED_PAT' -d '$payload'"
  created=$(http_json POST "$BASE_URL/api/public/agent-farm/runs" "$payload")
  run_id=$(printf '%s' "$created" | jq -r '.id // .run.id // empty' 2>/dev/null)
  record 10 "run created over the API" "$([[ "$(http_code)" =~ ^2 && -n "$run_id" ]] && echo true || echo false)" "HTTP $(http_code)"
else
  cmd "systemprompt infra jobs run farm_scheduled_run --yes --profile $PROFILE"
  out=$("$CLI" infra jobs run farm_scheduled_run --yes --profile "$PROFILE" 2>&1); rc=$?
  record 10 "farm_scheduled_run job ran" "$([[ $rc == 0 ]] && echo true || echo false)" "exit $rc"
  [[ $rc == 0 ]] || { printf '%s\n' "$out" | tail -15; exit 1; }
  run_id=""
  for _ in 1 2 3 4 5 6 7 8 9 10 11 12; do
    run_id=$(sql_rows "SELECT id FROM agent_farm_runs WHERE farm_id = $(sql_q "$FARM_ID") AND agent_id = $(sql_q "$FARM_AGENT")
      AND triggered_via = 'scheduler' AND created_at >= $(sql_q "$launched")::timestamptz ORDER BY created_at DESC LIMIT 1" | jq -r '.[0].id // empty')
    [[ -n "$run_id" ]] && break
    sleep 5
  done
  record 10 "scheduler-triggered run queued" "$([[ -n "$run_id" ]] && echo true || echo false)" "${run_id:-no run since $launched}"
fi
[[ -n "$run_id" ]] || exit 1
printf '%s' "$run_id" > "$STATE_DIR/run_id"
info "run $run_id"
step_result 10
