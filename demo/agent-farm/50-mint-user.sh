#!/bin/bash
# Step 50 — the farm user: created if absent, added to project odoo-delivery,
# and given a PAT. Writes .user_id and .token.user (gitignored). Re-running
# reuses the user and membership and keeps an existing PAT unless ROTATE=1.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 50 — mint user" "$FARM_USER_EMAIL in project $FARM_PROJECT"

step "1. Create user (idempotent)"
run_cli_indented admin users create --name "$FARM_USER_NAME" --email "$FARM_USER_EMAIL" --if-not-exists >/dev/null || true
uid=$(cli_json infra db query "SELECT id FROM users WHERE email = $(sql_q "$FARM_USER_EMAIL") LIMIT 1" | jq -r '.items[0].id // empty')
assert_nonempty "$uid" "user id for $FARM_USER_EMAIL"
printf '%s' "$uid" > "$STATE_USER_ID"
info "user id $uid"

step "2. Project membership ($FARM_PROJECT)"
have=$(db_count "SELECT COUNT(*) FROM project_members WHERE project_id = $(sql_q "$FARM_PROJECT") AND user_id = $(sql_q "$uid") AND revoked_at IS NULL")
if (( have == 0 )); then
  "$CLI" infra db execute --yes "INSERT INTO project_members (project_id, user_id, source) VALUES ($(sql_q "$FARM_PROJECT"), $(sql_q "$uid"), 'manual') ON CONFLICT (project_id, user_id, source) DO UPDATE SET revoked_at = NULL" --profile "$PROFILE" >/dev/null
fi
assert_min "$(db_count "SELECT COUNT(*) FROM project_members WHERE project_id = $(sql_q "$FARM_PROJECT") AND user_id = $(sql_q "$uid") AND revoked_at IS NULL")" 1 "active $FARM_PROJECT membership"

step "3. Personal access token"
if [[ -s "$STATE_USER_TOKEN" && "${ROTATE:-0}" != "1" ]]; then
  code=$(curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $(cat "$STATE_USER_TOKEN")" "$BASE_URL/api/public/agent-farm/agents" || echo 000)
  if [[ "$code" == "200" ]]; then pass "existing PAT still accepted"; exit 0; fi
  warn "existing PAT rejected (HTTP $code); minting a new one"
fi
name="agent-farm-e2e-$(date +%Y%m%d%H%M%S)"
budget_args=()
[[ -n "${PAT_BUDGET_MICRODOLLARS:-}" ]] && budget_args=(--budget-microdollars "$PAT_BUDGET_MICRODOLLARS")
out=$("$CLI" --json admin users api-key issue --user "$uid" --name "$name" ${budget_args[@]+"${budget_args[@]}"} --profile "$PROFILE" 2>&1)
# The CLI renders a presentation card ({sections:[{heading,content}]}) or a flat
# object; accept either, then fall back to the secret's own shape.
pat=$(printf '%s' "$out" | jq -r '[(.sections[]? | select(.heading == "secret") | .content), (.. | objects | (.secret?, .token?, .key?, .api_key?))] | map(strings) | first // empty' 2>/dev/null || true)
[[ -z "$pat" ]] && pat=$(printf '%s' "$out" | grep -oE 'sp[-_][a-z]+[-_][A-Za-z0-9._-]{16,}' | head -1)
assert_nonempty "$pat" "api-key issue returned a token"
( umask 077; printf '%s' "$pat" > "$STATE_USER_TOKEN" )
pass "PAT $name written to demo/agent-farm/.token.user"
