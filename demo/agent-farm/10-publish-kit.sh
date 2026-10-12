#!/bin/bash
# Step 10 — import the kit (Anthropic format) into a services tree, validate it,
# pack a signed bundle and publish it to GHCR as :vVERSION and :stable.
# Env: KIT_DIR, VERSION (default: kit's latest tag or 0.1.0), GHCR_USER,
# BUNDLE_SIGNING_SEED[_FILE]. SKIP_PUBLISH=1 stops after packing.
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
header "Agent Farm 10 — publish kit" "$KIT_DIR → $KIT_IMAGE"
need git
[[ -d "$KIT_DIR" ]] || { fail "kit checkout not found: $KIT_DIR (set KIT_DIR)"; exit 1; }
if [[ -z "${BUNDLE_SIGNING_SEED:-}" && -n "${BUNDLE_SIGNING_SEED_FILE:-}" ]]; then
  BUNDLE_SIGNING_SEED=$(tr -d '[:space:]' < "$BUNDLE_SIGNING_SEED_FILE")
fi
assert_nonempty "${BUNDLE_SIGNING_SEED:-}" "signing seed present"
export BUNDLE_SIGNING_SEED

commit=$(git -C "$KIT_DIR" rev-parse HEAD)
if [[ -z "${VERSION:-}" ]]; then
  VERSION=$(git -C "$KIT_DIR" describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || true)
  VERSION="${VERSION:-0.1.0}"
fi
info "kit commit $commit, version $VERSION"
[[ -z "$(git -C "$KIT_DIR" status --porcelain)" ]] || warn "kit checkout has uncommitted changes; bundle records HEAD $commit"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
tree="$work/services"
bundle="$FARM_DIR/.bundle/$FARM_ID-$VERSION.tar.gz"
mkdir -p "$FARM_DIR/.bundle"

step "1. Import the kit into a services tree (--strict)"
run_cli_indented core marketplace import --from "$KIT_DIR" --into "$tree" --strict

step "2. Validate the tree against this instance's services"
run_cli_indented core services validate --root "$tree" --base "$PROJECT_DIR/services" --strict

step "3. Pack a signed bundle"
rm -f "$bundle"
run_cli_indented core services bundle --root "$tree" --out "$bundle" --marketplace-only \
  --version "$VERSION" --sign-key env:BUNDLE_SIGNING_SEED \
  --source-repo "https://github.com/systempromptio/systemprompt-agent-farm" --source-commit "$commit"
[[ -s "$bundle" ]] || { fail "bundle not written: $bundle"; exit 1; }
info "bundle sha256 $(sha256_of "$bundle")"
printf '%s' "$VERSION" > "$STATE_VERSION"

if [[ "${SKIP_PUBLISH:-0}" == "1" ]]; then
  warn "SKIP_PUBLISH=1 — not pushing to GHCR"; exit 0
fi

step "4. Publish to GHCR"
need gh
GHCR_USER="${GHCR_USER:-$(gh api user --jq .login)}"
GHCR_AUTH="$GHCR_USER:$(gh auth token)"
export GHCR_AUTH
digest=""
for tag in "v$VERSION" "$KIT_CHANNEL"; do
  cmd "systemprompt core services publish --bundle $bundle --to oci://$KIT_IMAGE:$tag --auth env:GHCR_AUTH --json"
  out=$("$CLI" --json core services publish --bundle "$bundle" --to "oci://$KIT_IMAGE:$tag" \
    --auth env:GHCR_AUTH --profile "$PROFILE" 2>&1) || { fail "publish $tag failed"; echo "$out" | sed 's/^/    /'; exit 1; }
  d=$(printf '%s' "$out" | jq -r '.. | objects | .digest? // empty' 2>/dev/null | head -1)
  [[ -z "$d" ]] && d=$(printf '%s' "$out" | grep -oE 'sha256:[0-9a-f]{64}' | head -1)
  pass "published :$tag ${d:-}"
  digest="${digest:-$d}"
done
assert_nonempty "$digest" "registry returned a manifest digest"
printf '%s' "$digest" > "$STATE_DIGEST"
