#!/usr/bin/env bash
# Page loaders may not swallow a failed read behind a default.
#
# A server-rendered panel that logs a warning and renders its empty value
# turns a database outage into a page of zeros with a 200 status — on a
# spend or governance page that is wrong data presented as real. The one
# sanctioned fallback is admin-core's `ssr::faults::Faults`: `faults.take(..)`
# records what could not be read, the shell lists it above the content, and
# the log line is written once, centrally. So under the page-data roots below
# a `.inspect_err(` chain, an `unwrap_or_else(|e|` closure, or `.ok()` after
# `inspect_err` is a finding. A genuine exception — the value really is
# optional and its absence is not a fault — carries `// Why: fail-soft-ok:
# <reason>` on the line above the expression, which makes the decision
# visible in review.
#
# Portable to macOS and Linux: POSIX awk only, no grep -P.
set -euo pipefail

cd "$(dirname "$0")/.."

roots=(
    extensions/web/admin/*/src/handlers/ssr
    extensions/web/shared/admin-core/src/ssr
    extensions/web/src/router/admin
)

existing=()
for root in "${roots[@]}"; do
    [ -d "$root" ] && existing+=("$root")
done

findings=$(find "${existing[@]}" -name '*.rs' -type f -not -path '*/target/*' -print0 \
    | xargs -0 awk '
        FNR == 1 { exempt = 0 }
        /\/\/ Why: fail-soft-ok:/ { exempt = 3; next }
        /\.inspect_err\(|unwrap_or_else\(\|e\||unwrap_or_else\(\|err\|/ {
            if (exempt > 0) { exempt = 0; next }
            printf "%s:%d: %s\n", FILENAME, FNR, $0
            next
        }
        { if (exempt > 0) exempt-- }
    ')

if [ -n "$findings" ]; then
    echo "check-fail-soft: page data swallows a failed read behind a default:" >&2
    echo "$findings" >&2
    echo >&2
    echo "Hand the Result to a Faults collector (admin-core ssr::faults) and render" >&2
    echo "with render_page_with_faults, or mark a true exception with" >&2
    echo "'// Why: fail-soft-ok: <reason>' on the line above." >&2
    exit 1
fi
echo "check-fail-soft: no swallowed page reads"
