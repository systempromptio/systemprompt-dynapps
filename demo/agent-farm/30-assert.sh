#!/bin/bash
# Step 30 — the run did what the nightly flow promises:
#   artifacts  dashboard, dated 11-section report and receipt in the run outputs;
#              the eval case passed
#   read-only  every dynapps-odoo call in the run window was a read
#   trail      ai_requests and MCP trace rows for the run's identity
#   delivery   a successful dynbrain docs_create, whose article id is kept
# Every check is recorded; the step fails if any did.
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 30 — assert"
load_run_id
run_scope_sql
reset_step 30
r=$(sql_q "$RUN_ID")
win="created_at BETWEEN $RUN_FROM_SQL AND $RUN_TO_SQL AND user_id = $RUN_USER_SQL"

step "1. Run row"
row=$(sql_rows "SELECT status, triggered_via, started_at, finished_at FROM agent_farm_runs WHERE id = $r")
printf '%s' "$row" > "$STATE_DIR/run_row.json"
trig=$(printf '%s' "$row" | jq -r '.[0].triggered_via // empty')
record 30 "trigger recorded as $TRIGGER" "$([[ "$trig" == "$TRIGGER" ]] && echo true || echo false)" "triggered_via=${trig:-none}"

step "2. Artifacts"
outputs=$(jq -c '[.outputs[]?.path]' "$STATE_DIR/report.json" 2>/dev/null || echo '[]')
for f in out/dashboard.md out/DELIVERY.md; do
  record 30 "artifact $f" "$(printf '%s' "$outputs" | jq --arg f "$f" 'index($f) != null')"
done
dated=$(printf '%s' "$outputs" | jq -r '[.[] | select(test("^out/informe_auditoria_Dynapps_20[0-9]{2}_(0[1-9]|1[0-2])\\.md$"))][0] // empty')
record 30 "artifact out/informe_auditoria_Dynapps_YYYY_MM.md" "$([[ -n "$dated" ]] && echo true || echo false)" "${dated:-none in $outputs}"
# The 11 sections are asserted by the eval (DELIVERY.md `sections: 11`); when
# the run directory is on this machine (local profile) the file is counted too.
run_dir=$(jq -r '.run.run_dir // .run_dir // empty' "$STATE_DIR/run.json" 2>/dev/null)
if [[ -n "$dated" && -n "$run_dir" && -f "$run_dir/$dated" ]]; then
  n=$(grep -cE '^## (1[01]|[1-9])\. ' "$run_dir/$dated" || true)
  record 30 "report has 11 numbered sections" "$([[ "$n" == 11 ]] && echo true || echo false)" "$n found in $dated"
fi
ev_passed=$(jq -r '.eval_passed // "null"' "$STATE_DIR/report.json" 2>/dev/null)
ev_score=$(jq -r '.eval_score // "null"' "$STATE_DIR/report.json" 2>/dev/null)
record 30 "eval case $CASE passed" "$([[ "$ev_passed" == true ]] && echo true || echo false)" "eval_passed=$ev_passed score=$ev_score"

step "3. dynapps-odoo was read-only"
# Attempts come from governance_decisions (one per PreToolUse, denied ones
# included), so a write the policy refused still counts against the run.
bt=$(bare_tool_sql tool_name)
calls=$(sql_rows "SELECT $bt AS tool, COUNT(*) AS n FROM governance_decisions
  WHERE $(server_sql) = '$ODOO_SERVER' AND $win GROUP BY 1 ORDER BY 2 DESC")
printf '%s' "$calls" > "$STATE_DIR/odoo_calls.json"
reads=$(printf '%s' "$calls" | jq --arg r "$ODOO_READ_TOOLS" '[.[] | select(.tool as $t | ($r | split(" ") | index($t)) != null) | .n | tonumber] | add // 0')
writes=$(printf '%s' "$calls" | jq -r --arg r "$ODOO_READ_TOOLS" '[.[] | select(.tool as $t | ($r | split(" ") | index($t)) == null) | "\(.tool)×\(.n)"] | join(", ")')
record 30 "dynapps-odoo reads happened" "$([[ "$reads" -ge 1 ]] && echo true || echo false)" "$reads read attempts"
record 30 "zero dynapps-odoo write attempts" "$([[ -z "$writes" ]] && echo true || echo false)" "${writes:-none}"
nx=$(db_count "SELECT COUNT(*) FROM mcp_tool_executions WHERE $(server_sql server_name) = '$ODOO_SERVER'
  AND $bt NOT IN $(odoo_read_list_sql) AND $win")
record 30 "zero dynapps-odoo write executions" "$([[ "$nx" == 0 ]] && echo true || echo false)" "$nx"

step "4. Gateway and trace trail"
nreq=$(db_count "SELECT COUNT(*) FROM ai_requests WHERE $win")
record 30 "ai_requests rows" "$([[ "$nreq" -ge 1 ]] && echo true || echo false)" "$nreq"
ntr=$(db_count "SELECT COUNT(DISTINCT trace_id) FROM mcp_tool_executions WHERE trace_id IS NOT NULL AND $win")
record 30 "MCP trace rows" "$([[ "$ntr" -ge 1 ]] && echo true || echo false)" "$ntr distinct traces"

step "5. DynBrain delivery"
pub=$(sql_rows "SELECT output, status FROM mcp_tool_executions WHERE $(server_sql server_name) = '$BRAIN_SERVER'
  AND $(bare_tool_sql tool_name) = 'docs_create' AND output IS NOT NULL AND $win ORDER BY created_at DESC LIMIT 1")
pst=$(printf '%s' "$pub" | jq -r '.[0].status // empty')
art=$(printf '%s' "$pub" | jq -r '.[0].output // ""' | grep -oE '(article_id|\\?"id\\?")[^0-9]{1,6}[0-9]+' | head -1 | grep -oE '[0-9]+$' || true)
[[ -n "$art" ]] && printf '%s' "$art" > "$STATE_DIR/dynbrain_article_id"
record 30 "dynbrain docs_create succeeded" "$([[ "$pst" == success && -n "$art" ]] && echo true || echo false)" "status=${pst:-no call} article=${art:-none}"

step_result 30
