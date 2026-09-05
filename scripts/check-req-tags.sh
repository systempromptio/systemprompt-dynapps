#!/usr/bin/env bash
# Gate: no frozen requirement id may disappear from the Playwright suite.
#
# playwright/requirements.txt lists every @REQ-NNN the register expects to be
# proven. A page rewrite may move a tag between files; dropping one silently
# turns a signed requirement into an unproven one, which is exactly the kind of
# regression nobody notices until an audit.
#
# Read-only, no server, no node — safe in lint-gates.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIST="$ROOT/playwright/requirements.txt"
SPECS="$ROOT/playwright/tests"

[ -f "$LIST" ] || { echo "check-req-tags: missing $LIST"; exit 1; }
[ -d "$SPECS" ] || { echo "check-req-tags: missing $SPECS"; exit 1; }

tags="$(grep -rhoE '@REQ-[0-9]{3}' "$SPECS" | sort -u)"

missing=""
while IFS= read -r id; do
    case "$id" in ''|\#*) continue ;; esac
    printf '%s\n' "$tags" | grep -qxF "$id" || missing="$missing $id"
done < "$LIST"

if [ -n "$missing" ]; then
    echo "check-req-tags: FAILED — these frozen requirement ids are no longer tagged by any spec:"
    for id in $missing; do echo "  $id"; done
    echo
    echo "Re-tag the spec that now proves each one, or remove the id from"
    echo "playwright/requirements.txt and say in the PR why it is no longer proven."
    exit 1
fi

count="$(grep -cE '^@REQ-' "$LIST")"
echo "check-req-tags: OK ($count frozen ids, all tagged)"
