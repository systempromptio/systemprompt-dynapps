#!/bin/bash
# Step 60 — sync the farm definitions (agents, evals, fixtures) from the kit
# repo at its pinned ref, then confirm the user can list the four agents.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 60 — farm sync" "$FARM_ID"
auth=$(farm_admin_auth_header)
body=$(http_json POST "$BASE_URL/api/public/admin/agent-farm/farms/$FARM_ID/sync" "$auth" '{}')
assert_http "$HTTP_CODE" 2xx "farm sync"
echo "$body" | jq -C '{sha: (.sha // .kit_sha // .commit // null), agents: (.agents // .agent_count // null), status: (.status // null)}' 2>/dev/null | sed 's/^/  /' || true

load_user_pat
agents=$(http_json GET "$BASE_URL/api/public/agent-farm/agents" "Authorization: Bearer $USER_PAT")
assert_http "$HTTP_CODE" 200 "GET /api/public/agent-farm/agents (user PAT)"
for a in $FARM_AGENTS; do
  if printf '%s' "$agents" | grep -q "\"$a\""; then pass "agent $a listed"
  else fail "agent $a missing from /agents"; exit 1; fi
done
