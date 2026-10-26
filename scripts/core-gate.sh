#!/usr/bin/env bash
# Runs one of core's source gates over this install's sources. The rules live
# once, in ../systemprompt-core at bridge/CORE_REF (CI checks that ref out via
# .github/actions/core-checkout; SYSTEMPROMPT_CORE_DIR overrides the location),
# and the install only supplies its roots and, where the gate takes one, its
# allowlist.
#
#   core-gate.sh <core-script.sh> [--allowlist FILE] ROOT...
#
# A core gate that does not yet take roots (its usage header has no `ROOT...`)
# would scan core's own tree and prove nothing about this install, so the
# wrapper reports `skip:` and exits 0 until the pinned core grows the
# interface. A missing core checkout is an error (exit 2), never a pass.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_DIR="${SYSTEMPROMPT_CORE_DIR:-$REPO_ROOT/../systemprompt-core}"

[ $# -ge 2 ] || { echo "usage: $0 <core-script.sh> [--allowlist FILE] ROOT..." >&2; exit 2; }
gate="$1"; shift
script="$CORE_DIR/scripts/$gate"

[ -d "$CORE_DIR" ] || {
    echo "$gate: $CORE_DIR not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
}
[ -f "$script" ] || { echo "$gate: $script does not exist at core $(tr -d '[:space:]' < "$REPO_ROOT/bridge/CORE_REF")" >&2; exit 1; }

if ! sed -n '1,80p' "$script" | grep -qE '^#.*ROOT\.\.\.'; then
    echo "skip: core gate $gate lacks root args at CORE_REF $(tr -d '[:space:]' < "$REPO_ROOT/bridge/CORE_REF")"
    exit 0
fi

args=()
if [ "${1:-}" = "--allowlist" ]; then
    [ -f "$REPO_ROOT/$2" ] || { echo "$gate: allowlist $2 not found" >&2; exit 1; }
    args+=(--allowlist "$REPO_ROOT/$2"); shift 2
fi
for root in "$@"; do
    [ -d "$REPO_ROOT/$root" ] || { echo "$gate: root $root not found under $REPO_ROOT" >&2; exit 1; }
    args+=("$REPO_ROOT/$root")
done
exec bash "$script" "${args[@]}"
