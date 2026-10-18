#!/usr/bin/env bash
# A tracing message is a constant string; values go in structured fields.
# An interpolated message splits one event into as many distinct messages as
# it has values, which defeats grouping and search. The scanner lives in core
# (scripts/rust-contracts) so both repos share one definition.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [ ! -d "$root/../systemprompt-core" ]; then
    echo "check-tracing-messages.sh: ../systemprompt-core not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
fi
exec bash "$root/../systemprompt-core/scripts/lint-tracing-messages.sh" "$root/extensions" "$root/src" "$root/bridge/src"
