#!/bin/bash
# VET-GROUP DEMO SEED — a fictional veterinary group on the LOCAL instance.
#
# Writes ~40 days of synthetic usage ending Sunday 2026-10-18 20:00 (CEST):
# entity groups mapped from Entra-style AD groups, projects, access rules,
# gateway requests (Claude models only), MCP tool calls, hook events and
# skill invocations, governance decisions in warn mode (with would-block
# safety findings) and Agent Farm runs with reports. Every person and company
# is invented; users live on the reserved domain vetgroup-demo.example.
#
# Idempotent: every run deletes the rows it owns (cleanup.sql) and re-inserts
# the same deterministic set (generate.py, fixed seed) in one transaction.
#
# Refuses to run against anything but the local profile's localhost database.
#
# The farm is not declared in services/web/config/agent-farms.yaml (that file
# ships to production), and server boot removes undeclared farms. Run this
# AFTER `just start`; re-run it after any server restart.
#
#   ./demo/vet/seed.sh            seed (or re-seed)
#   ./demo/vet/seed.sh --clean    remove the demo rows only
#   ./demo/vet/seed.sh --hide-nondemo   seed, then move every non-demo user's
#                                       usage into schema demo_hidden
#   ./demo/vet/seed.sh --unhide-nondemo restore what --hide-nondemo moved
set -euo pipefail
export PGOPTIONS="${PGOPTIONS:-} -c client_min_messages=warning"

HERE="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$HERE/../.." && pwd)"
PROFILE="${PROFILE:-local}"

if [[ "$PROFILE" != "local" ]]; then
  echo "refusing: PROFILE=$PROFILE — this seed only targets the local profile" >&2
  exit 1
fi
SECRETS="$PROJECT_DIR/.systemprompt/profiles/local/secrets.json"
DB_URL="$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\(postgres[^"]*\)".*/\1/p' "$SECRETS" | head -1)"
if [[ -z "$DB_URL" ]]; then
  echo "refusing: no database_url in $SECRETS" >&2
  exit 1
fi
HOST="$(printf '%s' "$DB_URL" | sed -E 's#^postgres(ql)?://[^@]*@([^:/]+).*#\2#')"
if [[ "$HOST" != "localhost" && "$HOST" != "127.0.0.1" ]]; then
  echo "refusing: database host '$HOST' is not local" >&2
  exit 1
fi
command -v psql >/dev/null || { echo "psql not found" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 not found" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

{
  echo "\\set ON_ERROR_STOP on"
  echo "BEGIN;"
  echo "SET LOCAL systemprompt.audit_maintenance = 'on';"
  echo "SET LOCAL application_name = 'vetgroup-demo-seed';"
  cat "$HERE/cleanup.sql"
} > "$TMP/seed.sql"

run_tx() {
  { echo "\\set ON_ERROR_STOP on"; echo "BEGIN;"; echo "SET LOCAL systemprompt.audit_maintenance = 'on';"
    cat "$1"; echo "COMMIT;"; } | psql "$DB_URL" -q > /dev/null
}
if [[ "${1:-}" == "--unhide-nondemo" ]]; then
  run_tx "$HERE/unhide-nondemo.sql"
  echo "non-demo rows restored"
  exit 0
fi

if [[ "${1:-}" == "--clean" ]]; then
  echo "COMMIT;" >> "$TMP/seed.sql"
  psql "$DB_URL" -q -f "$TMP/seed.sql"
  echo "vet demo rows removed"
  exit 0
fi

echo "generating dataset…"
python3 -I "$HERE/generate.py" >> "$TMP/seed.sql" 2> "$TMP/counts.json"
cat "$HERE/post.sql" >> "$TMP/seed.sql"
echo "COMMIT;" >> "$TMP/seed.sql"

echo "loading into $HOST (one transaction)…"
psql "$DB_URL" -q -f "$TMP/seed.sql" > "$TMP/psql.log"
echo "generated: $(cat "$TMP/counts.json")"
echo "refreshing conversation and trace facts…"
psql "$DB_URL" -q -v ON_ERROR_STOP=1 -f "$HERE/facts.sql" > /dev/null

psql "$DB_URL" -At -F ' ' <<'SQL'
SELECT 'users', count(*)::text FROM users WHERE email LIKE '%@vetgroup-demo.example'
UNION ALL SELECT 'ai_requests', count(*)::text FROM ai_requests r JOIN users u ON u.id = r.user_id
  WHERE u.email LIKE '%@vetgroup-demo.example'
UNION ALL SELECT 'governance_warn', count(*)::text FROM governance_decisions g JOIN users u ON u.id = g.user_id
  WHERE u.email LIKE '%@vetgroup-demo.example' AND g.decision = 'warn'
UNION ALL SELECT 'farm_runs', count(*)::text FROM agent_farm_runs WHERE farm_id = 'vetgroup-ops'
UNION ALL SELECT 'last_request', max(r.created_at)::text FROM ai_requests r JOIN users u ON u.id = r.user_id
  WHERE u.email LIKE '%@vetgroup-demo.example';
SQL

if [[ "${1:-}" == "--hide-nondemo" ]]; then
  run_tx "$HERE/hide-nondemo.sql"
  echo "non-demo usage moved to schema demo_hidden (undo: $0 --unhide-nondemo)"
fi
