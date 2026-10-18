#!/usr/bin/env bash
# Production crates compile in one shape. Ported from core's lint-test-seams.sh.
#
# A test-only seam makes the released artifact differ from the one the suite
# exercises: a `test*` Cargo feature, a `cfg(feature = "test…")` or
# `cfg(any(test, …))` branch, a `test_api` module, an `unreachable_pub`
# suppression, or a `SYSTEMPROMPT_TEST_` env redirect. Tests live in `tests/`
# and reach public items; collaborators are injected through constructors or
# config. Scope: extensions/, src/, bridge/src and their manifests, tracked or
# not, so an untracked new file cannot pass vacuously.
set -uo pipefail
cd "$(dirname "$0")/.."

MATCHES=""
while IFS= read -r file; do
    case "$file" in
        tests/*) continue ;;
    esac

    case "$file" in
        *Cargo.toml)
            FOUND=$(awk '
                /^[[:space:]]*\[/ { in_features = ($0 ~ /^[[:space:]]*\[features\][[:space:]]*$/); next }
                in_features && /^[[:space:]]*"?test[A-Za-z0-9_-]*"?[[:space:]]*=/ {
                    print FILENAME ":" FNR ": test-only Cargo feature (" $0 ")"
                }
                /unreachable_pub[[:space:]]*=[[:space:]]*"allow"/ {
                    print FILENAME ":" FNR ": unreachable_pub suppressed (" $0 ")"
                }
            ' "$file")
            ;;
        *)
            FOUND=$(grep -nE 'cfg(_attr)?!?\((not\()?feature[[:space:]]*=[[:space:]]*"test|cfg(_attr)?!?\((not\()?(any|all)\(([^)]*[^A-Za-z0-9_)])?test([^A-Za-z0-9_]|$)|(^|[^A-Za-z0-9_])mod[[:space:]]+([a-z_]*_)?test_api|test_api::|#\[path[[:space:]]*=[[:space:]]*".*_test_api\.rs"|(allow|expect)\([^)]*unreachable_pub|SYSTEMPROMPT_TEST_' "$file" \
                | sed "s|^|$file:|; s|:\([0-9]*\):|:\1: test-only seam in production source: |")
            ;;
    esac
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard \
    'extensions/*.rs' 'extensions/**/*.rs' 'src/*.rs' 'src/**/*.rs' \
    'bridge/src/*.rs' 'bridge/src/**/*.rs' \
    'Cargo.toml' 'extensions/**/Cargo.toml' 'bridge/Cargo.toml' | sort -u)

if [ -z "$MATCHES" ]; then
    echo "lint-test-seams: OK (no test-only seams in production sources)"
    exit 0
fi

echo "lint-test-seams: production crates compile in one shape."
echo "Make the item \`pub\` in a \`pub mod\`, inject collaborators through constructors"
echo "or config, and move test conveniences into \`crates/tests/\`."
echo "See AGENTS.md."
echo ""
printf '%s' "$MATCHES"
exit 1
