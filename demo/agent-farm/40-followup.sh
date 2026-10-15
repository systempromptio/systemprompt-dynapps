#!/bin/bash
# Step 40 — the morning follow-up. A headless Claude Code session (`claude -p`)
# signed in through the Dynapps bridge as Ed fetches the night's summary from
# DynBrain and proposes ONE fix. Without APPROVE_FIX=1 it stops at the
# proposal and has no Odoo write tool at all. With APPROVE_FIX=1 the same
# session is resumed and allowed exactly one tool, orm_write, to apply it; the
# write is then looked up in mcp_tool_executions and governance_decisions.
#
# Needs `claude` on PATH configured through the bridge (`just connect <code>`
# or `just claude <code>`), so the session is governed and recorded like Ed's
# own. ODOO_TOOL_PREFIX / BRAIN_TOOL_PREFIX override how the bridge names the
# two servers' tools (default mcp__dynapps-odoo__ / mcp__dynbrain__).
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 40 — follow-up" "APPROVE_FIX=${APPROVE_FIX:-0}"
need claude
reset_step 40
ODOO_TOOL_PREFIX="${ODOO_TOOL_PREFIX:-mcp__${ODOO_SERVER}__}"
BRAIN_TOOL_PREFIX="${BRAIN_TOOL_PREFIX:-mcp__${BRAIN_SERVER}__}"
FOLLOWUP_MODEL="${FOLLOWUP_MODEL:-claude-opus-5-5}"
article="${DYNBRAIN_ARTICLE_ID:-$(cat "$STATE_DIR/dynbrain_article_id" 2>/dev/null || true)}"
[[ -n "$article" ]] || { record 40 "DynBrain article id known" false "run 30-assert.sh or set DYNBRAIN_ARTICLE_ID"; exit 1; }
work=$(mktemp -d "${TMPDIR:-/tmp}/night-followup.XXXXXX")
started=$(date -u +%Y-%m-%dT%H:%M:%SZ)

reads=""
for t in $ODOO_READ_TOOLS; do reads="$reads,${ODOO_TOOL_PREFIX}$t"; done
allowed="${BRAIN_TOOL_PREFIX}docs_get,${BRAIN_TOOL_PREFIX}docs_search,${BRAIN_TOOL_PREFIX}docs_metadata$reads"
denied="${ODOO_TOOL_PREFIX}orm_create,${ODOO_TOOL_PREFIX}orm_write,${ODOO_TOOL_PREFIX}orm_unlink,${ODOO_TOOL_PREFIX}orm_execute_kw"

cat > "$work/propose.txt" <<PROMPT
Fetch DynBrain document $article with docs_get: it is last night's accounting
audit of Dynapps (Odoo master, company 1). Read its prioritised correction
proposal, confirm the record's current value with one orm_search_read on
dynapps-odoo, and propose exactly ONE fix. Do not change anything.
End your answer with a fenced json block and nothing after it:
{"model": "...", "record_id": 0, "field": "...", "current_value": ..., "proposed_value": ..., "reason": "..."}
PROMPT

step "1. Proposal (read-only session)"
cmd "claude -p <propose.txt> --model $FOLLOWUP_MODEL --output-format json --allowedTools <reads> --disallowedTools <writes>"
( cd "$work" && claude -p "$(cat propose.txt)" --model "$FOLLOWUP_MODEL" --output-format json --max-turns 15 \
    --allowedTools "$allowed" --disallowedTools "$denied" < /dev/null > propose.json 2> propose.err )
python3 -I - "$work/propose.json" "$STATE_DIR/followup.json" "$article" "$started" <<'PY'
import json, re, sys
src, dst, article, started = sys.argv[1:5]
try:
    res = json.load(open(src))
except Exception as exc:
    res = {"is_error": True, "result": f"unreadable claude output: {exc}"}
text = res.get("result") or ""
blocks = re.findall(r"```json\s*(\{.*?\})\s*```", text, re.S)
fix = None
if blocks:
    try:
        fix = json.loads(blocks[-1])
    except ValueError:
        fix = None
json.dump({"article_id": article, "started_at": started,
           "proposal": {"session_id": res.get("session_id"), "is_error": res.get("is_error"),
                        "num_turns": res.get("num_turns"), "cost_usd": res.get("total_cost_usd"),
                        "duration_ms": res.get("duration_ms"), "fix": fix, "text": text[-4000:]},
           "applied": None}, open(dst, "w"), indent=2)
PY
fix=$(jq -c '.proposal.fix // empty' "$STATE_DIR/followup.json")
record 40 "one fix proposed from the DynBrain summary" "$([[ -n "$fix" && "$fix" != null ]] && echo true || echo false)" "${fix:-$(head -c 300 "$work/propose.err")}"
[[ -n "$fix" && "$fix" != null ]] || exit 1
printf '%s' "$fix" | jq -C . | sed 's/^/  /'

if [[ "${APPROVE_FIX:-0}" != 1 ]]; then
  info "APPROVE_FIX is not 1: stopping at the proposal; nothing was written."
  step_result 40; exit $?
fi

step "2. Approved: apply through dynapps-odoo"
session=$(jq -r '.proposal.session_id // empty' "$STATE_DIR/followup.json")
applied_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
( cd "$work" && claude -p "Approved by Ed. Apply exactly that one fix with a single orm_write on dynapps-odoo, then read the field back with orm_search_read and report the new value. Change nothing else." \
    --resume "$session" --model "$FOLLOWUP_MODEL" --output-format json --max-turns 6 \
    --allowedTools "${ODOO_TOOL_PREFIX}orm_write,${ODOO_TOOL_PREFIX}orm_search_read" < /dev/null > apply.json 2> apply.err )
tmp=$(mktemp)
jq --slurpfile a "$work/apply.json" --arg at "$applied_at" \
  '.applied = ($a[0] | {session_id, is_error, num_turns, cost_usd: .total_cost_usd, duration_ms, at: $at, text: ((.result // "")[-2000:])})' \
  "$STATE_DIR/followup.json" > "$tmp" 2>/dev/null && mv "$tmp" "$STATE_DIR/followup.json"

uid=$(ed_user_id); u=$(sql_q "$uid"); s=$(sql_q "$applied_at")
nw=$(db_count "SELECT COUNT(*) FROM mcp_tool_executions WHERE server_name ILIKE '%$ODOO_SERVER%' AND $(bare_tool_sql tool_name) = 'orm_write' AND status = 'success' AND user_id = $u AND created_at >= $s::timestamptz")
record 40 "one orm_write landed on dynapps-odoo" "$([[ "$nw" == 1 ]] && echo true || echo false)" "$nw successful writes since approval"
ng=$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE tool_name ILIKE 'mcp__%orm_write' AND user_id = $u AND created_at >= $s::timestamptz")
record 40 "the write was governed and recorded" "$([[ "$ng" -ge 1 ]] && echo true || echo false)" "$ng governance decisions"
step_result 40
