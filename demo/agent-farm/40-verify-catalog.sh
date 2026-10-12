#!/bin/bash
# Step 40 — the imported marketplace is visible: CLI catalog + admin page.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 40 — verify catalog" "marketplace $FARM_ID, plugin $FARM_PLUGIN"
jar=$(farm_admin_cookie)
for path in "/admin/marketplaces/$FARM_ID" "/admin/sync"; do
  tmp=$(mktemp)
  code=$(curl -s -o "$tmp" -w '%{http_code}' -b "$jar" "$BASE_URL$path" || echo 000)
  assert_http "$code" 200 "GET $path"
  if grep -q "$FARM_PLUGIN\|$FARM_ID" "$tmp"; then pass "$path mentions the kit"
  else fail "$path does not mention $FARM_ID/$FARM_PLUGIN"; rm -f "$tmp"; exit 1; fi
  rm -f "$tmp"
done
