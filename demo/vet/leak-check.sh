#!/bin/bash
# Leak check for the vet demo: dump the rows the seed owns and grep them for
# real-tenant markers and for every email and display name in a reference
# database (default: the local Astound copy). Expected output: 0 hits.
#
#   ./demo/vet/leak-check.sh [reference_db_url]
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
TARGET="$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\(postgres[^"]*\)".*/\1/p' \
  "$HERE/../../.systemprompt/profiles/local/secrets.json" | head -1)"
REF="${1:-postgres://systemprompt:123@localhost:5447/sp_prod_1007}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

psql "$TARGET" -At > "$TMP/dump.txt" <<'SQL'
CREATE TEMP TABLE v AS SELECT id FROM users WHERE email LIKE '%@vetgroup-demo.example';
SELECT row_to_json(x)::text FROM users x WHERE id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM groups x WHERE id LIKE 'vg-%';
SELECT row_to_json(x)::text FROM projects x WHERE id LIKE 'vg-%';
SELECT row_to_json(x)::text FROM access_control_rules x WHERE justification LIKE '[vetgroup-demo]%';
SELECT row_to_json(x)::text FROM ai_requests x WHERE user_id IN (SELECT id FROM v);
SELECT row_to_json(m)::text FROM ai_request_messages m JOIN ai_requests r ON r.id = m.request_id WHERE r.user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM governance_decisions x WHERE user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM plugin_usage_events x WHERE user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM mcp_tool_executions x WHERE user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM ai_safety_findings x JOIN ai_requests r ON r.id = x.ai_request_id WHERE r.user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM agent_farms x WHERE id = 'vetgroup-ops';
SELECT row_to_json(x)::text FROM agent_farm_agents x WHERE farm_id = 'vetgroup-ops';
SELECT row_to_json(x)::text FROM agent_farm_runs x WHERE farm_id = 'vetgroup-ops';
SELECT row_to_json(x)::text FROM agent_farm_run_reports x JOIN agent_farm_runs r ON r.id = x.run_id WHERE r.farm_id = 'vetgroup-ops';
SELECT row_to_json(x)::text FROM agent_runs x WHERE user_id IN (SELECT id FROM v);
SELECT row_to_json(e)::text FROM agent_run_events e JOIN agent_runs x ON x.id = e.run_id WHERE x.user_id IN (SELECT id FROM v);
SELECT row_to_json(x)::text FROM clients x WHERE id LIKE 'vg-%';
SQL
echo "dumped $(wc -l < "$TMP/dump.txt") rows"

{ echo "dynapps.be"; echo "astound"; echo "systemprompt.io"; echo "dynapps.es"; echo "dynapps.local"; echo "dynapps.test";
  psql "$REF" -At -c "SELECT lower(email) FROM users WHERE email NOT LIKE '%@anonymous.local'
                      UNION SELECT lower(display_name) FROM users WHERE length(coalesce(display_name,'')) >= 4
                      UNION SELECT lower(full_name) FROM users WHERE length(coalesce(full_name,'')) >= 4"
} | sort -u > "$TMP/needles.txt"
echo "checking $(wc -l < "$TMP/needles.txt") needles"
hits="$(grep -i -F -o -f "$TMP/needles.txt" "$TMP/dump.txt" | sort | uniq -c || true)"
if [[ -n "$hits" ]]; then
  echo "LEAKS:"; echo "$hits"; exit 1
fi
echo "0 hits"
