#!/bin/bash
# Step 20 — pin the kit's channel tag into the local profile's services.sources[].
# services-pin.py rewrites an existing entry in place, so re-running is a no-op.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 20 — pin" "$FARM_ID @ $KIT_CHANNEL → $PROFILE"
[[ -f "$PROFILE_YAML" ]] || { fail "profile not found: $PROFILE_YAML"; exit 1; }
before=$(sha256_of "$PROFILE_YAML")
cmd "python3 scripts/services-pin.py $FARM_ID $KIT_CHANNEL ${PROFILE_YAML#$PROJECT_DIR/}"
( cd "$PROJECT_DIR" && python3 scripts/services-pin.py "$FARM_ID" "$KIT_CHANNEL" "$PROFILE_YAML" ) | sed 's/^/  /'
after=$(sha256_of "$PROFILE_YAML")
[[ "$before" == "$after" ]] && info "profile unchanged (already pinned)" || info "profile updated"
grep -q "$KIT_IMAGE" "$PROFILE_YAML" || { fail "profile does not reference $KIT_IMAGE"; exit 1; }
pass "profile pins $KIT_IMAGE:$KIT_CHANNEL"
