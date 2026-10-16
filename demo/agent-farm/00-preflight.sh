#!/bin/bash
# Step 00 — the target is ready for an unattended run as Ed: the CLI reaches its
# database, Ed exists, his dynapps-odoo and dynbrain connector grants are on
# file and not revoked, and he is entitled to the farm's marketplace, both MCP
# servers and the accounting auditor's plugin. Nothing is written.
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 00 — preflight" "$BASE_URL (profile $PROFILE)"
require_pat
reset_step 00

step "1. Target and identity"
code=$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/api/v1/health" || echo 000)
record 00 "target answers" "$([[ "$code" =~ ^2 ]] && echo true || echo false)" "GET /api/v1/health → $code"
uid=$(ed_user_id)
record 00 "Ed's user row ($ED_EMAIL)" "$([[ -n "$uid" ]] && echo true || echo false)" "${uid:-not found; is the $PROFILE admin session logged in?}"
[[ -n "$uid" ]] || exit 1
u=$(sql_q "$uid")

step "2. Connector grants (mcp_connector_credentials)"
for p in "$ODOO_SERVER" "$BRAIN_SERVER"; do
  q=$(sql_q "$p")
  row=$(sql_rows "SELECT c.updated_at,
      (SELECT a.action FROM connector_credential_audit a WHERE a.subject_user_id = $u AND a.provider = $q ORDER BY a.created_at DESC LIMIT 1) AS last_action
    FROM mcp_connector_credentials c WHERE c.user_id = $u AND c.provider = $q")
  at=$(printf '%s' "$row" | jq -r '.[0].updated_at // empty')
  last=$(printf '%s' "$row" | jq -r '.[0].last_action // "none"')
  ok=false; [[ -n "$at" && "$last" != *revoke* && "$last" != *delete* ]] && ok=true
  record 00 "grant $p" "$ok" "${at:+updated $at, }last audit action: $last"
done

step "3. Entitlements (access_control_rules)"
roles_sql="(SELECT unnest(roles) FROM users WHERE id = $u)"
groups_sql="(SELECT group_id FROM group_members WHERE user_id = $u AND revoked_at IS NULL
  AND (valid_from IS NULL OR valid_from <= NOW()) AND (valid_until IS NULL OR valid_until > NOW()))"
console=$(db_count "SELECT COUNT(*) FROM users WHERE id = $u AND roles && ARRAY['admin','platform_admin','super_admin']::text[]")
for ent in "marketplace:$FARM_MARKETPLACE" "mcp_server:$ODOO_SERVER" "mcp_server:$BRAIN_SERVER" "plugin:dynapps-farm"; do
  t="${ent%%:*}"; e="${ent#*:}"
  row=$(sql_rows "SELECT
      COUNT(*) FILTER (WHERE access = 'allow' AND ((rule_type = 'user' AND rule_value = $u) OR (rule_type = 'role' AND rule_value IN $roles_sql) OR (rule_type = 'group' AND rule_value IN $groups_sql))) AS allows,
      COUNT(*) FILTER (WHERE access = 'deny' AND rule_type = 'user' AND rule_value = $u) AS denies
    FROM access_control_rules WHERE entity_type = $(sql_q "$t") AND entity_id = $(sql_q "$e")")
  a=$(printf '%s' "$row" | jq -r '.[0].allows // 0'); d=$(printf '%s' "$row" | jq -r '.[0].denies // 0')
  how="allow rules matching Ed: $a, user denies: $d"
  # The farm treats console roles as holding every farm marketplace.
  if [[ "$t" == marketplace && "$a" == 0 && "$console" -ge 1 ]]; then a=1; how="$how; held through a console role"; fi
  record 00 "entitled to $t/$e" "$([[ "$a" -ge 1 && "$d" == 0 ]] && echo true || echo false)" "$how"
done

step "4. The farm agent is visible to Ed's PAT"
agents=$(http_json GET "$BASE_URL/api/public/agent-farm/agents")
seen=$(printf '%s' "$agents" | jq --arg f "$FARM_ID" --arg a "$FARM_AGENT" \
  '[(.agents // .items // .)[]? | select(((.farm_id // .farm // "") == $f) and ((.agent_id // .id // .name // "") == $a))] | length' 2>/dev/null || echo 0)
record 00 "$FARM_ID/$FARM_AGENT listed" "$([[ "$(http_code)" == 200 && "$seen" -ge 1 ]] && echo true || echo false)" "GET /api/public/agent-farm/agents → $(http_code), matches: $seen"

step_result 00
