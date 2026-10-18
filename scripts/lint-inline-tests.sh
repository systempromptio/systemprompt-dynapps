#!/usr/bin/env bash
# Inline `#[cfg(test)] mod tests` is banned in production crates. Ported from
# core's lint-inline-tests.sh. Tests live in the separate `tests/` workspace so
# they compile against the public surface a consumer has; extensions/, src/
# and bridge/src are scanned.
set -uo pipefail
cd "$(dirname "$0")/.."

fail=0
while IFS=: read -r file line _; do
    case "$file" in
        tests/*) continue ;;
    esac
    echo "$file:$line: inline \`#[cfg(test)] mod tests\` — move it to tests/"
    fail=1
done < <(grep -rn --include='*.rs' -A1 '#\[cfg(test)\]' extensions src bridge/src 2>/dev/null \
    | grep -B0 'mod tests' \
    | sed -E 's/-([0-9]+)-/:\1:/' \
    | grep -E '^[^:]+:[0-9]+:.*mod tests')

if [ "$fail" -eq 0 ]; then
    echo "lint-inline-tests: ok"
fi
exit "$fail"
