#!/bin/bash
# Step 00 — check the kit signing seed matches the public key recorded in
# deploy/kit/known-kits.json. Needs BUNDLE_SIGNING_SEED (base64) or
# BUNDLE_SIGNING_SEED_FILE. Read-only.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 00 — signing key" "kit: $FARM_ID"

if [[ -z "${BUNDLE_SIGNING_SEED:-}" && -n "${BUNDLE_SIGNING_SEED_FILE:-}" ]]; then
  [[ -f "$BUNDLE_SIGNING_SEED_FILE" ]] || { fail "BUNDLE_SIGNING_SEED_FILE not found: $BUNDLE_SIGNING_SEED_FILE"; exit 1; }
  BUNDLE_SIGNING_SEED=$(tr -d '[:space:]' < "$BUNDLE_SIGNING_SEED_FILE")
fi
assert_nonempty "${BUNDLE_SIGNING_SEED:-}" "BUNDLE_SIGNING_SEED (or _FILE) is set"
export BUNDLE_SIGNING_SEED

recorded=$(jq -r --arg k "$FARM_ID" '.kits[] | select(.name==$k) | .public_key // empty' "$PROJECT_DIR/deploy/kit/known-kits.json")
assert_nonempty "$recorded" "known-kits.json records a public key for $FARM_ID"

# Derive the Ed25519 public key from the seed when a crypto lib is present.
derived=$(python3 -I - <<'PY' 2>/dev/null || true
import base64, os
seed = base64.b64decode(os.environ["BUNDLE_SIGNING_SEED"])
try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization
    pk = Ed25519PrivateKey.from_private_bytes(seed[:32]).public_key().public_bytes(
        serialization.Encoding.Raw, serialization.PublicFormat.Raw)
except ImportError:
    from nacl.signing import SigningKey
    pk = bytes(SigningKey(seed[:32]).verify_key)
print(base64.b64encode(pk).decode())
PY
)
if [[ -z "$derived" ]]; then
  warn "No python 'cryptography' or 'nacl' module; skipping key-match check."
  warn "The instance will reject a mismatched signature at import (step 30)."
else
  assert_eq "$derived" "$recorded" "seed derives the recorded public key"
fi
pass "signing key ready"
