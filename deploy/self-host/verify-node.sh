#!/usr/bin/env bash
# Post-boot checklist for one node. Exit 0 only when every check passes.
#
# Usage: verify-node.sh [--url http://localhost:8080] [--container <name>] [--version X.Y.Z]
#                       [--role all|gateway|admin]
#
# --role matches the node's server.role (default all). A gateway node serves
# only /v1, the bridge, OAuth and health: it has no admin console, public
# site, downloads or MCP child server, so those checks are skipped. An admin
# node has no /v1 gateway, so /v1/messages must not answer 401 there.
#
# --container enables the in-container checks (binary version, MCP child
# server, migration ledger) via `docker exec`.
# Portable across macOS and Linux (curl, sha256sum or shasum, docker).
set -uo pipefail

URL="http://localhost:8080" CONTAINER="" VERSION="" ROLE="all"
while [ $# -gt 0 ]; do
    case "$1" in
        --url) URL="${2%/}"; shift 2 ;;
        --container) CONTAINER="$2"; shift 2 ;;
        --version) VERSION="$2"; shift 2 ;;
        --role) ROLE="$2"; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

case "$ROLE" in all|gateway|admin) ;; *) echo "--role must be all, gateway or admin" >&2; exit 2 ;; esac
serves_site() { [ "$ROLE" != "gateway" ]; }
serves_gateway() { [ "$ROLE" != "admin" ]; }

PASS=0 FAIL=0
ok()  { printf '  PASS  %s\n' "$1"; PASS=$((PASS+1)); }
bad() { printf '  FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
sha() { if command -v sha256sum >/dev/null; then sha256sum | cut -d' ' -f1; else shasum -a 256 | cut -d' ' -f1; fi; }
CURL=(curl -fsS -k --max-time 30)

echo "── node: $URL (role $ROLE) ──"
body="$("${CURL[@]}" "$URL/api/v1/health" 2>/dev/null || true)"
case "$body" in
    *healthy*) ok "GET /api/v1/health → $body" ;;
    *) bad "GET /api/v1/health → '${body:-no response}'" ;;
esac

if serves_site; then
code="$(curl -sk -o /dev/null -w '%{http_code}' "$URL/admin/login" || true)"
[ "$code" = "200" ] && ok "GET /admin/login → 200" || bad "GET /admin/login → $code"

code="$(curl -sk -o /dev/null -w '%{http_code}' "$URL/documentation/" || true)"
[ "$code" = "200" ] && ok "GET /documentation/ (public site rendered on this node) → 200" || bad "GET /documentation/ → $code (the publish pipeline has not run on this node)"

for asset in astound-bridge-windows.exe astound-bridge-macos.dmg \
             astound-bridge-linux-x86_64.tar.gz astound-bridge-linux-aarch64.tar.gz; do
    want="$("${CURL[@]}" "$URL/files/downloads/$asset.sha256" 2>/dev/null | cut -d' ' -f1 || true)"
    got="$("${CURL[@]}" "$URL/files/downloads/$asset" 2>/dev/null | sha || true)"
    if [ -n "$want" ] && [ "$want" = "$got" ]; then
        ok "download $asset matches its .sha256"
    else
        bad "download $asset (sha256 want=${want:-none} got=${got:-none})"
    fi
done
code="$(curl -sk -o /dev/null -w '%{http_code}' "$URL/files/downloads/install.sh" || true)"
[ "$code" = "200" ] && ok "download install.sh → 200" || bad "download install.sh → $code"
fi

code="$(curl -sk -o /dev/null -w '%{http_code}' -X POST "$URL/v1/messages" -H 'content-type: application/json' -d '{}' || true)"
if serves_gateway; then
    [ "$code" = "401" ] && ok "POST /v1/messages without credentials → 401" || bad "POST /v1/messages without credentials → $code (expected 401)"
else
    [ "$code" != "401" ] && [ "$code" != "200" ] && ok "POST /v1/messages not mounted on an admin node → $code" || bad "POST /v1/messages answered $code on an admin node (the gateway is mounted)"
fi

if [ -n "$CONTAINER" ]; then
    x() { docker exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml "$CONTAINER" "$@"; }
    v="$(x /app/bin/systemprompt --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
    if [ -n "$VERSION" ]; then
        [ "$v" = "$VERSION" ] && ok "binary version $v" || bad "binary version '$v' ≠ expected $VERSION"
    else
        [ -n "$v" ] && ok "binary version $v" || bad "binary version unreadable"
    fi
    # MCP servers speak streamable HTTP under /mcp; a bare GET without the MCP
    # Accept header is answered 406 by a live server and refused by a dead one.
    serves_site && for port in 5010:systemprompt; do
        p="${port%%:*}"; n="${port#*:}"
        code="$(x curl -s -o /dev/null -w '%{http_code}' --max-time 5 "http://127.0.0.1:$p/mcp" 2>/dev/null || echo 000)"
        case "$code" in
            000|5*) bad "MCP server $n on :$p not answering (HTTP $code)" ;;
            *) ok "MCP server $n on :$p alive (HTTP $code)" ;;
        esac
    done
    n="$(x sh -c 'psql "$DATABASE_URL" -tAc "select count(*) from extension_migrations"' 2>/dev/null | tr -d '[:space:]')"
    [ -n "$n" ] && [ "$n" -gt 0 ] 2>/dev/null && ok "migration ledger: $n applied" || bad "migration ledger unreadable"
    n="$(x sh -c 'psql "$DATABASE_URL" -tAc "select count(*) from users"' 2>/dev/null | tr -d '[:space:]')"
    [ -n "$n" ] && [ "$n" -gt 0 ] 2>/dev/null && ok "bootstrap admin present (users=$n)" || bad "no users row — admin bootstrap did not run"
fi

echo "── $PASS pass, $FAIL fail ──"
[ "$FAIL" -eq 0 ]
