#!/usr/bin/env bash
# No fallible value may be silently thrown away in extensions/ or bridge/src:
# `let _ =`, a trailing `.ok();`, `drop(<call>)`, or `unwrap_or_default()` on a
# Result. A swallowed error is an audit gap nobody sees. The scanner lives in
# core (scripts/rust-contracts) so both repos enforce one definition; a
# `// Why: discard-ok: <reason>` line above the statement carves it out.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [ ! -d "$root/../systemprompt-core" ]; then
    echo "check-discarded-results.sh: ../systemprompt-core not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
fi
exec bash "$root/../systemprompt-core/scripts/check-discarded-results.sh" "$root/extensions" "$root/bridge/src"
