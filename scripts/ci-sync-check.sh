#!/usr/bin/env bash
# Gate: the CI layout shared by systemprompt-astound and systemprompt-dynapps
# is byte-identical in both. Every divergence these repos ever had was a fix
# that landed in one and not the other; this makes such a fix a red build.
#
#   scripts/ci-sync-check.sh             check SHARED against .github/ci-shared.sha256
#   scripts/ci-sync-check.sh --update    re-record the manifest after a deliberate change
#   scripts/ci-sync-check.sh --sibling DIR
#                                        also diff every shared file against DIR's copy
#
# The manifest is itself shared, so it can only move in both repos at once:
# a change recorded here and not ported leaves the sibling red until it is.
# Brand values live in .github/ci-params.env, the one CI file allowed to
# differ; this checks its key set instead of its values.
set -euo pipefail

cd "$(dirname "$0")/.."

SHARED=(
    .github/workflows/gates.yml
    .github/workflows/coverage.yml
    .github/workflows/release.yml
    .github/actions/core-checkout/action.yml
    .github/actions/apple-signing/action.yml
    .github/actions/free-disk/action.yml
    ci.just
    scripts/ci-sync-check.sh
    scripts/ci-retry.sh
    scripts/coverage.sh
    scripts/coverage-check.sh
    scripts/coverage-badge.sh
    scripts/check-file-size.sh
)
PARAMS=.github/ci-params.env
PARAM_KEYS="CI_BRIDGE_DISPLAY_NAME CI_BRIDGE_PREFIX CI_BUNDLE_SIGNING_SECRET_NAME CI_GITHUB_REPO CI_HOMEPAGE CI_IMAGE CI_PRODUCT_NAME CI_RELEASE_GATE_EMAIL CI_REPO_NAME CI_RUNNER_IMAGE CI_SERVER_BINS"
MANIFEST=.github/ci-shared.sha256

mode=check
sibling=""
while [ $# -gt 0 ]; do
    case "$1" in
        --update) mode=update ;;
        --sibling) sibling="${2:?--sibling needs a directory}"; shift ;;
        *) echo "ci-sync-check: unknown argument $1" >&2; exit 2 ;;
    esac
    shift
done

failed=0
fail() { echo "ci-sync-check: $*" >&2; failed=1; }

keys="$(grep -v '^#' "$PARAMS" | sed -n 's/^\([A-Z_]*\)=.*/\1/p' | sort | tr '\n' ' ' | sed 's/ $//')"
[ "$keys" = "$PARAM_KEYS" ] || fail "$PARAMS keys are [$keys], expected [$PARAM_KEYS]"
while IFS='=' read -r k v; do
    case "$k" in ''|\#*) continue ;; esac
    [ -n "$v" ] || fail "$PARAMS: $k is empty"
done < "$PARAMS"

for f in "${SHARED[@]}"; do [ -f "$f" ] || fail "shared file $f is missing"; done

if [ "$mode" = update ]; then
    [ "$failed" -eq 0 ] || exit 1
    sha256sum "${SHARED[@]}" > "$MANIFEST"
    echo "ci-sync-check: recorded ${#SHARED[@]} files in $MANIFEST — port the same change to the sibling repo"
    exit 0
fi

[ -f "$MANIFEST" ] || { fail "$MANIFEST missing (run with --update)"; exit 1; }
recorded="$(awk '{print $2}' "$MANIFEST" | tr '\n' ' ')"
[ "$recorded" = "${SHARED[*]} " ] || fail "$MANIFEST lists [$recorded], expected [${SHARED[*]} ]"
if ! sha256sum --quiet -c "$MANIFEST"; then
    fail "shared CI files differ from $MANIFEST: land the change in both repos and re-record with --update"
fi

if [ -n "$sibling" ]; then
    for f in "${SHARED[@]}" "$MANIFEST"; do
        cmp -s "$f" "$sibling/$f" || { fail "$f differs from $sibling/$f"; diff -u "$sibling/$f" "$f" | head -40 >&2 || true; }
    done
    other="$(grep -v '^#' "$sibling/$PARAMS" | sed -n 's/^\([A-Z_]*\)=.*/\1/p' | sort | tr '\n' ' ' | sed 's/ $//')"
    [ "$other" = "$keys" ] || fail "$sibling/$PARAMS keys [$other] differ from [$keys]"
fi

[ "$failed" -eq 0 ] || exit 1
echo "ci-sync-check: ${#SHARED[@]} shared CI files match $MANIFEST${sibling:+ and $sibling}"
