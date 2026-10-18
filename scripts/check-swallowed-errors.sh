#!/usr/bin/env bash
# A `map_err` at an HTTP, database or IO boundary must not drop its cause:
# a closure that binds the error as `_`/`_name` and logs nothing turns an
# upstream failure into an unexplained fixed message. Log the cause (redacted
# where it carries a URL or secret) before mapping it. The scanner lives in
# core (scripts/rust-contracts) so both repos share one definition.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [ ! -d "$root/../systemprompt-core" ]; then
    echo "check-swallowed-errors.sh: ../systemprompt-core not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
fi
exec bash "$root/../systemprompt-core/scripts/lint-swallowed-errors.sh" "$root/extensions" "$root/src" "$root/bridge/src"
