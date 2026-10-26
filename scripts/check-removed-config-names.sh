#!/usr/bin/env bash
# Gate: a name the core 0.73 config model removed does not come back here.
# The name list is core's (../systemprompt-core/scripts/check-removed-config-names.sh
# at bridge/CORE_REF); this install allows only history and the pre-0.73
# fixtures the golden migration test reads.
set -euo pipefail

cd "$(dirname "$0")/.."

names='surface_for|canonicalize_org_uuid|ProviderEntry|YAML_SOURCE|DASHBOARD_SOURCE|legacy_id|find_served_model|by_scope'

allowed=(
    # History.
    'CHANGELOG.md'
    'schema/migrations/'
    'tests/fixtures/schema/'
    # The pre-0.73 catalog the golden migration test reads.
    'tests/fixtures/catalog/'
    # This gate names them.
    'scripts/check-removed-config-names.sh'
)

hits=$(git ls-files -- ':!:*.lock' \
    | while read -r f; do
        for a in "${allowed[@]}"; do
            case "$f" in *"$a"*) continue 2 ;; esac
        done
        [ -f "$f" ] && grep -nwE "$names" "$f" 2>/dev/null | sed "s#^#$f:#"
    done || true)

if [ -n "$hits" ]; then
    echo "check-removed-config-names: a name the 0.73 config model removed is back:"
    printf '  %s\n' "$hits"
    exit 1
fi
echo "check-removed-config-names: OK"
