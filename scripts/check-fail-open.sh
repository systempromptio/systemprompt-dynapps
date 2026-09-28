#!/usr/bin/env bash
# Guard fns must fail closed: `unwrap_or(true)`, `map_or(true, ..)`,
# `is_none_or` and a `_ => true` arm turn "could not decide" into "allow", and a
# guard that walks an inventory param must return Option/Result so an empty
# catalog withholds rather than permits. The scanner lives in core
# (scripts/rust-contracts) so both repos share one definition.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [ ! -d "$root/../systemprompt-core" ]; then
    echo "check-fail-open.sh: ../systemprompt-core not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
fi
exec bash "$root/../systemprompt-core/scripts/check-fail-open.sh" "$root/extensions" "$root/bridge/src"
