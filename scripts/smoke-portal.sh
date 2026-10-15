#!/usr/bin/env bash
# Manual live smoke check of the Dynapps portal MCP servers through the
# gateway, for someone holding a real account with a connected portal grant.
#
#   SMOKE_BASE_URL=https://dynapps.systemprompt.io SMOKE_PAT=<pat> scripts/smoke-portal.sh
#
# For each server (dynapps-odoo, dynbrain) it opens an MCP session at
# <base>/api/v1/mcp/<server>/mcp with the PAT, sends initialize, then
# tools/list, and prints PASS/FAIL. Exits nonzero when any check fails.
# Not run by CI: it needs a live tenant and a personal token.
# Portable to macOS and Linux: curl, grep -E, sed and tr only.
set -euo pipefail

base="${SMOKE_BASE_URL:-}"
pat="${SMOKE_PAT:-}"
servers="${SMOKE_SERVERS:-dynapps-odoo dynbrain}"

if [ -z "$base" ] || [ -z "$pat" ]; then
    echo "usage: SMOKE_BASE_URL=<https://host> SMOKE_PAT=<token> $0" >&2
    exit 2
fi
base="${base%/}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

failures=0

report() {
    if [ "$1" = ok ]; then
        printf 'PASS  %-14s %s\n' "$2" "$3"
    else
        printf 'FAIL  %-14s %s\n' "$2" "$3"
        failures=$((failures + 1))
    fi
}

post() {
    local url="$1" body="$2" out="$3" hdr="$4"
    shift 4
    curl -sS --max-time 30 -D "$hdr" -o "$out" -w '%{http_code}' -X POST "$url" \
        -H 'content-type: application/json' \
        -H 'accept: application/json, text/event-stream' \
        -H "Authorization: Bearer $pat" \
        "$@" -d "$body" || printf '000'
}

for server in $servers; do
    url="$base/api/v1/mcp/$server/mcp"
    hdr="$tmp/$server.hdr"
    out="$tmp/$server.init"
    init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"smoke-portal","version":"1"}}}'
    code="$(post "$url" "$init" "$out" "$hdr")"
    if [ "$code" != 200 ] || ! grep -qE '"protocolVersion"' "$out"; then
        report fail "$server" "initialize: HTTP $code $(head -c 200 "$out" | tr '\n' ' ')"
        continue
    fi
    report ok "$server" "initialize"

    sid="$(grep -iE '^mcp-session-id:' "$hdr" | tr -d '\r' | sed -n 's/^[^:]*:[[:space:]]*//p' | head -n 1)"
    session=()
    if [ -n "$sid" ]; then
        session=(-H "mcp-session-id: $sid")
    fi
    post "$url" '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        "$tmp/$server.note" "$tmp/$server.note.hdr" "${session[@]+"${session[@]}"}" >/dev/null

    out="$tmp/$server.tools"
    code="$(post "$url" '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' "$out" "$tmp/$server.tools.hdr" \
        "${session[@]+"${session[@]}"}")"
    count="$(grep -oE '"name"[[:space:]]*:' "$out" | wc -l | tr -d ' ')"
    if [ "$code" = 200 ] && grep -qE '"tools"' "$out" && [ "$count" -gt 0 ]; then
        report ok "$server" "tools/list ($count tools)"
    else
        report fail "$server" "tools/list: HTTP $code $(head -c 200 "$out" | tr '\n' ' ')"
    fi
done

if [ "$failures" -gt 0 ]; then
    echo "smoke-portal: $failures check(s) failed" >&2
    exit 1
fi
echo "smoke-portal: all checks passed"
