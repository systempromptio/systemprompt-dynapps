#!/bin/bash
# Step 10 — queue one run of the night audit as Ed, over the public API with
# his PAT, marked trigger=scheduler-sim so it reads as the 02:00 scheduler job.
# DYNBRAIN_PARENT_ID (optional) names the DynBrain document the summary is
# published under; without it the agent looks for "Auditorías nocturnas".
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 10 — run" "$FARM_ID/$FARM_AGENT case $CASE trigger $TRIGGER"
require_pat
reset_step 10
rm -f "$STATE_DIR/run_id" "$STATE_DIR/run.json" "$STATE_DIR/report.json" "$STATE_DIR/events.json" \
  "$STATE_DIR/followup.json" "$STATE_DIR/dynbrain_article_id"

prompt=""
[[ -n "${DYNBRAIN_PARENT_ID:-}" ]] && prompt="dynbrain_parent_id: $DYNBRAIN_PARENT_ID"
payload=$(jq -nc --arg f "$FARM_ID" --arg a "$FARM_AGENT" --arg c "$CASE" --arg t "$TRIGGER" --arg p "$prompt" \
  '{farm:$f, agent:$a, case:$c, trigger:$t} + (if $p == "" then {} else {prompt:$p} end)')
cmd "curl -X POST $BASE_URL/api/public/agent-farm/runs -H 'Authorization: Bearer \$ED_PAT' -d '$payload'"
created=$(http_json POST "$BASE_URL/api/public/agent-farm/runs" "$payload")
# The launch body is deny_unknown_fields; an instance that predates the
# trigger field refuses it, and the run is still worth taking as manual.
if [[ "$(http_code)" == 422 || "$(http_code)" == 400 ]] && printf '%s' "$created" | grep -q trigger; then
  warn "instance does not accept trigger; launching without it (the run records the API trigger)"
  payload=$(printf '%s' "$payload" | jq -c 'del(.trigger)')
  created=$(http_json POST "$BASE_URL/api/public/agent-farm/runs" "$payload")
fi
code=$(http_code)
run_id=$(printf '%s' "$created" | jq -r '.id // .run.id // empty' 2>/dev/null)
record 10 "run created" "$([[ "$code" =~ ^2 && -n "$run_id" ]] && echo true || echo false)" "HTTP $code${run_id:+, run $run_id}"
[[ -n "$run_id" ]] || { printf '%s\n' "$created" | head -c 600; echo; exit 1; }
printf '%s' "$run_id" > "$STATE_DIR/run_id"
date -u +%Y-%m-%dT%H:%M:%SZ > "$STATE_DIR/launched_at"
info "run $run_id ($(printf '%s' "$created" | jq -r '.status // .run.status // "?"'))"
step_result 10
