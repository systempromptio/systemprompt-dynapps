#!/bin/bash
# Step 30 — make sure the server is up, then refresh remote services sources so
# the pinned kit bundle is fetched, verified and swapped in.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 30 — start and import" "$BASE_URL"
need just
if ! curl -sf -o /dev/null "$BASE_URL/api/v1/health" && ! curl -sf -o /dev/null "$BASE_URL/"; then
  step "Server not reachable; starting it"
  ( cd "$PROJECT_DIR" && just start ) | tail -20 | sed 's/^/  /'
  for _ in $(seq 1 60); do curl -sf -o /dev/null "$BASE_URL/" && break; sleep 2; done
fi
curl -sf -o /dev/null "$BASE_URL/" || { fail "server not reachable at $BASE_URL"; exit 1; }
pass "server reachable"

auth=$(farm_admin_auth_header)
step "POST /api/v1/admin/services/refresh"
body=$(http_json POST "$BASE_URL/api/v1/admin/services/refresh" "$auth" '{}')
echo "$body" | jq -C . 2>/dev/null | head -30 | sed 's/^/  /' || echo "  $body"
assert_http "$HTTP_CODE" 2xx "services refresh"

step "GET /api/v1/admin/services/status"
status=$(http_json GET "$BASE_URL/api/v1/admin/services/status" "$auth")
assert_http "$HTTP_CODE" 200 "services status"
echo "$status" | jq -C . 2>/dev/null | head -40 | sed 's/^/  /'
if printf '%s' "$status" | grep -q "bundle:$FARM_ID\|$FARM_ID"; then
  pass "status names $FARM_ID"
else
  fail "services status does not mention $FARM_ID"; exit 1
fi
