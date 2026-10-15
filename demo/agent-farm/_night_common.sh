#!/bin/bash
# Shared helpers for the night-audit e2e (demo/agent-farm/*.sh).
# Sourced, never run: every step script does
#   source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
# Portable across macOS (BSD) and Linux (GNU): no grep -P, no sed -i, no
# head -n -1, no GNU date arithmetic (python3 does the time maths).
#
# Secrets come from the environment only: ED_PAT (Ed's personal access token on
# the target) is never written to disk. Database reads go through the CLI with
# the production profile's admin session (`systemprompt admin session login`).

NIGHT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PROFILE="${PROFILE:-production}"
TARGET="${TARGET:-https://dynapps.systemprompt.io}"
export BASE_URL="${BASE_URL:-$TARGET}"
# shellcheck source=../_common.sh
source "$NIGHT_DIR/../_common.sh"

FARM_ID="${FARM_ID:-dynapps-night-audit}"
FARM_AGENT="${FARM_AGENT:-accounting_night_audit}"
FARM_MARKETPLACE="${FARM_MARKETPLACE:-dynapps-agent-farm}"
CASE="${CASE:-nightly_previous_month}"
TRIGGER="${TRIGGER:-scheduler}"
ED_EMAIL="${ED_EMAIL:-ed@systemprompt.io}"
RUN_TIMEOUT="${RUN_TIMEOUT:-2400}"
ODOO_SERVER="dynapps-odoo"
BRAIN_SERVER="dynbrain"
# Every dynapps-odoo tool the audit may call. Anything else on that server
# counts as a write, so a new tool is flagged until someone classifies it.
ODOO_READ_TOOLS="whoami orm_search_read odoo_list_models odoo_describe_model odoo_get_selection odoo_aggregate odoo_resolve odoo_attachment_search"
BRAIN_WRITE_RE='^docs_(create|create_tree|update|trash|move|asset_upload|asset_link|request_upload|propose_changes|review_proposal|merge_proposal)$'

STATE_DIR="$NIGHT_DIR/.state"
mkdir -p "$STATE_DIR"
ASSERTIONS="$STATE_DIR/assertions.jsonl"

need() {
  local c
  for c in "$@"; do
    command -v "$c" >/dev/null 2>&1 || { fail "missing required tool: $c"; exit 1; }
  done
}
need curl jq python3

require_pat() {
  [[ -n "${ED_PAT:-}" ]] || { fail "ED_PAT is not set (Ed's PAT on $BASE_URL; export it, never commit it)"; exit 1; }
}

# HTTP helper: prints the body; the status lands in a file because callers
# capture the body with $(...) and a subshell cannot set a global.
#   http_json METHOD URL [BODY]   (authenticated as Ed's PAT)
HTTP_CODE_FILE="$STATE_DIR/http_code"
http_json() {
  local method="$1" url="$2" body="${3:-}" tmp code
  tmp=$(mktemp)
  if [[ -n "$body" ]]; then
    code=$(curl -s -o "$tmp" -w '%{http_code}' -X "$method" -H "Authorization: Bearer $ED_PAT" \
      -H 'Content-Type: application/json' --data "$body" "$url" || echo 000)
  else
    code=$(curl -s -o "$tmp" -w '%{http_code}' -X "$method" -H "Authorization: Bearer $ED_PAT" "$url" || echo 000)
  fi
  printf '%s' "$code" > "$HTTP_CODE_FILE"
  cat "$tmp"; rm -f "$tmp"
}
http_code() { cat "$HTTP_CODE_FILE" 2>/dev/null || echo 000; }

sql_q() { printf "'%s'" "$(printf '%s' "$1" | sed "s/'/''/g")"; }

# Rows of a read-only query as a JSON array (empty array on failure).
# Database reads. Remote-routed CLI commands cap each argument at 256 bytes,
# too short for these queries, so they run locally against the tenant's
# external database URL: NIGHT_DATABASE_URL from the environment, else the
# profile's `external_database_url` secret, read at call time and never stored.
night_db_url() {
  if [[ -n "${NIGHT_DATABASE_URL:-}" ]]; then printf '%s' "$NIGHT_DATABASE_URL"; return; fi
  local secrets="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/secrets.json"
  [[ -f "$secrets" ]] && jq -r '.external_database_url // empty' "$secrets"
}
db_query() {
  local url; url=$(night_db_url)
  if [[ -n "$url" ]]; then "$CLI" --json infra db query "$1" --database-url "$url" 2>/dev/null
  else cli_json infra db query "$1"; fi
}
# The CLI prints the table document and then a status message; keep the table.
sql_rows() {
  local rows
  rows=$(db_query "$1" | jq -sc '[.[] | objects | select(has("items")) | .items] | first // []' 2>/dev/null)
  printf '%s' "${rows:-[]}"
}
db_count() {
  local n
  n=$(sql_rows "$1" | jq -r '(.[0] // {} | to_entries[0].value) // 0' 2>/dev/null)
  [[ "$n" =~ ^[0-9]+$ ]] && echo "$n" || echo 0
}

ed_user_id() {
  if [[ -s "$STATE_DIR/ed_user_id" ]]; then cat "$STATE_DIR/ed_user_id"; return; fi
  local id
  id=$(sql_rows "SELECT id FROM users WHERE email = $(sql_q "$ED_EMAIL")" | jq -r '.[0].id // empty')
  [[ -n "$id" ]] && printf '%s' "$id" > "$STATE_DIR/ed_user_id"
  printf '%s' "$id"
}

load_run_id() {
  [[ -s "$STATE_DIR/run_id" ]] || { fail "no run recorded; run 10-run.sh first"; exit 1; }
  RUN_ID=$(cat "$STATE_DIR/run_id")
}

# SQL fragments scoping a query to the run's identity and wall-clock window.
# The identity is the agent run's user (run_as), which for this farm is Ed.
run_scope_sql() {
  local r; r=$(sql_q "$RUN_ID")
  RUN_USER_SQL="(SELECT COALESCE(ar.user_id, fr.user_id) FROM agent_farm_runs fr LEFT JOIN agent_runs ar ON ar.id = fr.agent_run_id WHERE fr.id = $r)"
  RUN_FROM_SQL="(SELECT COALESCE(started_at, created_at) FROM agent_farm_runs WHERE id = $r)"
  RUN_TO_SQL="(SELECT COALESCE(finished_at, NOW()) FROM agent_farm_runs WHERE id = $r)"
}

# Normalises a tool name to its bare form (mcp__server__tool -> tool).
bare_tool_sql() { printf "regexp_replace(%s, '^mcp__.*__', '')" "$1"; }

# Which portal server a row belongs to. Farm runs are observed twice: the
# plugin hook records Claude Code's name (mcp__dynapps-odoo__orm_search_read,
# server hook_claude_code) and the gateway proxy may record the bare tool
# under the server's own name. Rows from neither server map to their server.
server_sql() {
  local srv="${1:-NULL}"
  printf "CASE WHEN tool_name LIKE 'mcp\\_\\_%s\\_\\_%%' OR %s ILIKE '%%%s%%' THEN '%s' WHEN tool_name LIKE 'mcp\\_\\_%s\\_\\_%%' OR %s ILIKE '%%%s%%' THEN '%s' ELSE COALESCE(%s, 'other') END" \
    "$ODOO_SERVER" "$srv" "$ODOO_SERVER" "$ODOO_SERVER" "$BRAIN_SERVER" "$srv" "$BRAIN_SERVER" "$BRAIN_SERVER" "$srv"
}

odoo_read_list_sql() {
  local t out=""
  for t in $ODOO_READ_TOOLS; do out="$out${out:+,}'$t'"; done
  printf '(%s)' "$out"
}

# Records one assertion (step, name, pass true/false, detail) and prints it.
record() {
  local step="$1" name="$2" ok="$3" detail="${4:-}"
  jq -nc --arg s "$step" --arg n "$name" --argjson ok "$ok" --arg d "$detail" \
    '{step:$s, name:$n, pass:$ok, detail:$d}' >> "$ASSERTIONS"
  if [[ "$ok" == "true" ]]; then pass "$name${detail:+ — $detail}"; else fail "$name${detail:+ — $detail}"; fi
}

# Exit status of a step: non-zero when any of its recorded assertions failed.
step_result() {
  local n
  n=$(jq -s --arg s "$1" '[.[] | select(.step == $s and .pass == false)] | length' "$ASSERTIONS" 2>/dev/null || echo 1)
  [[ "$n" == "0" ]]
}

# Drops a step's earlier assertions so a re-run reports only the latest result.
reset_step() {
  [[ -f "$ASSERTIONS" ]] || return 0
  local tmp; tmp=$(mktemp)
  jq -c --arg s "$1" 'select(.step != $s)' "$ASSERTIONS" > "$tmp" && mv "$tmp" "$ASSERTIONS"
}
