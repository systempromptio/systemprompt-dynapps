#!/usr/bin/env bash
# Shared library for the real-client e2e harnesses: e2e-claude-code.sh,
# e2e-opencode.sh and e2e-marketplaces.sh source this file.
#
# Everything here is a function or a documented variable; sourcing it has no
# side effects. The functions are the parts the harnesses had in common —
# reporting, the Postgres helper, the host-mode throwaway HOME and its `cx`,
# the loopback proxy port guard, enrolment with a PAT, the streamable-HTTP MCP
# pre-flight dance, and the server-side proof queries. Each was lifted out of
# one harness verbatim; the header comment on each function says which, and
# any behaviour that differs between harnesses stays in the harness.
#
# Variables a caller sets before using the relevant function:
#   REPO_ROOT       the repo checkout (e2e_repo_root sets it)
#   DB_URL          Postgres URL (e2e_db_url sets it)
#   GATEWAY         the gateway URL as the CLIENT sees it
#   HOST_GATEWAY    the gateway URL as THIS host sees it
#   CLIENT_HOME     the throwaway HOME (e2e_host_client_home sets it)
#   E2E_CLIENT_PATH extra PATH entries put in front of the client's PATH
#   E2E_CX_ENV      array of extra VAR=value pairs for the client's environment
#
# Contract for the loopback proxy: host mode binds ONE fixed port, so two
# host-mode harnesses cannot run at once. E2E_PROXY_PORT is that port.

E2E_PROXY_PORT="${E2E_PROXY_PORT:-48217}"

# ── Reporting ────────────────────────────────────────────────────────────────
# From e2e-claude-code.sh. `fail` exits: a harness that wants a non-exiting
# `fail` (e2e-opencode.sh accumulates an rc) redefines it after sourcing.
say()  { printf '\033[1m==>\033[0m %s\n' "$*"; }
info() { printf '[INFO] %s\n' "$*"; }
ok()   { printf '[OK  ] %s\n' "$*"; }
warn() { printf '\033[33mwarn:\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

e2e_repo_root() {
    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[1]}")/.." && pwd)"
}

# ── Postgres ─────────────────────────────────────────────────────────────────
# The pass is asserted against Postgres, never against a transcript, so no
# database is a failed run rather than a skipped one.
e2e_db_url() {
    DB_URL="${SYSTEMPROMPT_DATABASE_URL:-$(python3 -c 'import json;print(json.load(open("'"$REPO_ROOT"'/.systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)}"
    [ -n "${DB_URL:-}" ] || return 1
}

# One-column, no-header psql, so callers can treat output as a value.
# From e2e-claude-code.sh.
q() { psql "$DB_URL" -tAF'|' -v ON_ERROR_STOP=1 -c "$1"; }

# The whitespace-stripped scalar variant, from e2e-opencode.sh. Never use it
# on a rendered timestamp: the space inside one is stripped too, and the
# corrupted literal then fails every comparison silently.
q_scalar() { psql "$DB_URL" -tAc "$1" 2>/dev/null | tr -d '[:space:]'; }

# Epoch seconds from the database's own clock, which is the clock every
# created_at column is stamped with; the host clock can drift from it.
db_now() { q_scalar "select extract(epoch from now())"; }

e2e_db_reachable() { psql "$DB_URL" -tAc 'select 1' >/dev/null 2>&1; }

# ── The gateway ──────────────────────────────────────────────────────────────
# The container view of the gateway is host.docker.internal; the host cannot
# resolve that, so it probes the same server on the loopback name.
e2e_host_gateway_from() {
    local g="$1"
    g="${g//host.docker.internal/127.0.0.1}"
    printf '%s' "${g//localhost/127.0.0.1}"
}

# Asked before every cell, because a sweep outlives the server it is testing.
gateway_up() { curl -fsS --max-time 5 "$HOST_GATEWAY/v1/models" >/dev/null 2>&1; }

# ── Host mode: the throwaway HOME ────────────────────────────────────────────
# From e2e-claude-code.sh. A throwaway HOME is the whole isolation story in
# host mode, so it is created fresh and never reused: the installer writes
# ~/.claude, the bridge config and the PAT under it, and none of that may reach
# the real home directory, which is a developer's live client setup.
#   $1  a name stem for the temp dir (e.g. e2e-claude-home)
e2e_host_client_home() {
    CLIENT_HOME="$(mktemp -d "${TMPDIR:-/tmp}/${1:-e2e-client-home}.XXXXXX")"
    mkdir -p "$CLIENT_HOME/work" "$CLIENT_HOME/.cache" "$CLIENT_HOME/.config"
    # Refuse to run against the real home even if something above went wrong.
    case "$CLIENT_HOME" in
        "$HOME"|"$HOME"/*) fail "refusing to run: the throwaway HOME resolved inside the real one" ;;
    esac
}

# Kill only the processes this run started. They are identified by the
# throwaway HOME in their environment, never by name: `pkill dynapps-bridge`
# would take out a proxy the developer is using on this same machine.
e2e_host_kill_bridge() {
    local pid
    for pid in $(pgrep -f 'dynapps-bridge' 2>/dev/null || true); do
        if tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null | grep -qx "HOME=$CLIENT_HOME"; then
            kill "$pid" 2>/dev/null || true
        fi
    done
}

# Kill the bridge and discard the HOME. KEEP_CLIENT_HOME=1 keeps the directory
# for inspection.
e2e_host_cleanup() {
    [ -n "${CLIENT_HOME:-}" ] || return 0
    e2e_host_kill_bridge
    [ -n "${KEEP_CLIENT_HOME:-}" ] || rm -rf "$CLIENT_HOME"
}

# The loopback proxy port is a single fixed resource on this host, so two
# host-mode harnesses cannot run at once. Whoever binds it first keeps it; the
# second client silently talks to the first one's proxy, which is enrolled
# against a different config — and the symptom is an empty tools/list blamed
# on the PAT's role. Refuse instead of producing that.
# Why: under `set -o pipefail` an empty grep exits 1, which `set -e` would
# turn into a silent abort on the common case of a FREE port; `|| true`
# keeps the guard a guard.
e2e_proxy_port_guard() {
    local holder holder_home
    holder="$( (ss -ltnp 2>/dev/null | awk -v p="127.0.0.1:${E2E_PROXY_PORT}" '$0 ~ p {print}' \
                | grep -oE 'pid=[0-9]+' | head -1 | cut -d= -f2) || true)"
    if [ -n "$holder" ]; then
        holder_home="$(tr '\0' '\n' <"/proc/$holder/environ" 2>/dev/null \
                       | sed -n 's/^HOME=//p' | head -1)"
        fail "127.0.0.1:${E2E_PROXY_PORT} is already held by pid $holder (HOME=${holder_home:-unknown}).
       Host mode needs that port for its own bridge proxy, and there is only
       one, so two host-mode harnesses cannot run concurrently. Wait for the
       other run to finish, or stop that process, then re-run."
    fi
}

# Run a command as the client, on this host, under the throwaway HOME.
# From e2e-claude-code.sh. A login shell so the managed block the installer
# writes into the throwaway ~/.profile is sourced, exactly as it is in the
# container. ANTHROPIC_API_KEY is cleared because a key in the developer's
# environment would send the client straight to Anthropic and quietly bypass
# the gateway under test — the run would pass while proving nothing.
# </dev/null is load-bearing: a subprocess inheriting this script's stdin eats
# the rest of an enclosing here-doc and silently truncates the caller.
cx_host() {
    env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL \
        ${E2E_CX_ENV[@]+"${E2E_CX_ENV[@]}"} \
        HOME="$CLIENT_HOME" \
        XDG_CONFIG_HOME="$CLIENT_HOME/.config" \
        XDG_CACHE_HOME="$CLIENT_HOME/.cache" \
        DYNAPPS_BRIDGE_GATEWAY_URL="$GATEWAY" \
        PATH="${E2E_CLIENT_PATH:+$E2E_CLIENT_PATH:}$CLIENT_HOME/.local/bin:$PATH" \
        bash -lc "$1" </dev/null
}

# ── Enrolment ────────────────────────────────────────────────────────────────
# From e2e-claude-code.sh. Fetch, then run — never `curl … | sh`. A pipeline's
# status is the last command's, so a 404 from curl feeds sh an empty script,
# sh exits 0, and the harness proceeds to blame the models for a client that
# was never installed.
#   $1 host id (claude-code | opencode)   $2 PAT   $3 optional command prefix
# Uses the caller's `cx`. Returns 2 when the gateway serves no install.sh,
# 1 when the installer exits non-zero, 0 on success.
e2e_enrol_host() {
    local host="$1" pat="$2" prefix="${3:-}"
    local download_base="${E2E_DOWNLOAD_BASE:-$GATEWAY/files/downloads}"
    # This URL crosses into the client's shell. Refuse metacharacters before
    # quoting it; a release URL needs none of them.
    [[ "$download_base" =~ ^https?://[a-zA-Z0-9.:/_-]+$ ]] || return 2
    cx "${prefix}curl -fsSL -o \$HOME/install.sh '$download_base/install.sh'" || return 2
    cx "sh \$HOME/install.sh --download-base '$download_base' \
                           --gateway '$GATEWAY' --pat '$pat' --host $host" || return 1
}

# Two very different faults can look alike, so separate them before blaming
# either: the host can reach a loopback-bound server that a container cannot.
e2e_client_reaches_gateway() {
    cx "curl -fsS --max-time 10 -o /dev/null '$GATEWAY/v1/models'"
}

# ── The loopback MCP endpoint ────────────────────────────────────────────────
# The streamable-HTTP transport wants initialize, then the initialized
# notification, then the request carrying the session id; a bare tools/list is
# a malformed MCP request and answers 422. Everything runs inside the client
# (`cx`) because the loopback bearer lives in the client's XDG_CONFIG_HOME.
e2e_mcp_url() { printf 'http://127.0.0.1:%s/mcp/%s' "$E2E_PROXY_PORT" "$1"; }

# The shell fragment both MCP helpers start with: open a session at $URL and
# leave SID + MH set. Exits 3 when the proxy answers with no session.
E2E_MCP_SESSION_SH='
set -uo pipefail
KEY=$(cat "${XDG_CONFIG_HOME:-$HOME/.config}/dynapps/bridge-loopback.key")
MH=(-H content-type:application/json -H "accept:application/json,text/event-stream" -H "Authorization: Bearer $KEY")
HDR=$(mktemp)
curl -sS -D "$HDR" -o /dev/null -X POST "$URL" "${MH[@]}" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"e2e\",\"version\":\"1\"}}}"
SID=$(grep -i "mcp-session-id" "$HDR" | tr -d "\r" | awk "{print \$2}")
rm -f "$HDR"
[ -n "$SID" ] || exit 3
curl -sS -X POST "$URL" "${MH[@]}" -H "mcp-session-id: $SID" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" >/dev/null
'

# Print the tool names the endpoint at $1 lists, space-separated, or nothing.
e2e_mcp_tools_at() {
    cx 'URL="'"$1"'"'"$E2E_MCP_SESSION_SH"'
curl -sS -X POST "$URL" "${MH[@]}" -H "mcp-session-id: $SID" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}" \
  | sed -n "s/^data: //p" | jq -r "..|.tools? // empty | .[].name" 2>/dev/null | tr "\n" " "
' 2>/dev/null || true
}
e2e_mcp_tools() { e2e_mcp_tools_at "$(e2e_mcp_url "$1")"; }

# Call tool $2 at endpoint $1 with JSON arguments $3 and print the JSON-RPC
# result object (the SSE `data:` payload). Exit 3 on no session.
e2e_mcp_call_at() {
    local args="${3:-{\}}" body
    # Why: the body crosses into the client shell as script text, so a JSON
    # argument with spaces is word-split there; base64 carries it intact.
    body="$(printf '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"%s","arguments":%s}}' "$2" "$args" | base64 -w0)"
    cx 'URL="'"$1"'"'"$E2E_MCP_SESSION_SH"'
curl -sS -X POST "$URL" "${MH[@]}" -H "mcp-session-id: $SID" \
  -d "$(printf %s '"$body"' | base64 -d)" \
  | sed -n "s/^data: //p" | tail -1
'
}
e2e_mcp_call() { e2e_mcp_call_at "$(e2e_mcp_url "$1")" "$2" "${3:-}"; }

# The HTTP status the proxy answers /mcp/<server> with, for a bare initialize.
# A server the principal is not entitled to must not answer 2xx.
e2e_mcp_status() {
    cx 'KEY=$(cat "${XDG_CONFIG_HOME:-$HOME/.config}/dynapps/bridge-loopback.key" 2>/dev/null)
curl -sS -o /dev/null -w "%{http_code}" -X POST "'"$(e2e_mcp_url "$1")"'" \
  -H "Authorization: Bearer $KEY" -H content-type:application/json \
  -H "accept:application/json,text/event-stream" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"e2e\",\"version\":\"1\"}}}"' 2>/dev/null || printf '000'
}

# Pre-flight: is tool $2 listed by server $1, and (with $3, JSON args) does a
# real call succeed? Prints the tool list on [INFO] to stderr so a caller can
# capture stdout; with args, prints the call result on stdout. Returns 1 when the tool is not listed, 2 when the call
# fails, 3 when the proxy opens no session.
preflight_mcp() {
    local server="$1" tool="$2" args="${3:-}" tools result
    tools="$(e2e_mcp_tools "$server")"
    info "tools on $server: ${tools:-(none)}" >&2
    case " $tools " in
        *" $tool "*) ;;
        "  ") return 3 ;;
        *) return 1 ;;
    esac
    [ -n "$args" ] || return 0
    result="$(e2e_mcp_call "$server" "$tool" "$args")" || return 3
    printf '%s\n' "$result"
    printf '%s' "$result" | jq -e '.result and (.result.isError // false | not)' >/dev/null 2>&1 || return 2
}

# ── Server-side proof ────────────────────────────────────────────────────────
# From e2e-claude-code.sh. The proof is rows the server writes and the client
# cannot forge. Each takes a cutoff from `q "select now()"` — the database's
# clock, never the host's — and an optional user id that pins the rows to one
# principal when several share the database.

# Proof 1 — the MCP server ran the tool.
#   $1 cutoff  $2 tool  $3 server  [$4 user_id]
e2e_count_mcp_exec() {
    q "select count(*) from mcp_tool_executions
       where created_at >= '$1'
         and tool_name = '$2' and server_name = '$3'
         and status = 'success'${4:+ and user_id = '$4'}"
}

# Proof 2 — an inference request on THIS model asked for it. The link column
# is nullable and two different columns can carry it, so accept either.
#   $1 cutoff  $2 tool  $3 model  [$4 user_id]   → the model row, or nothing
e2e_tool_call_model_row() {
    q "select r.requested_model, r.model
       from ai_request_tool_calls c
       join ai_requests r on r.id = c.request_id
       left join mcp_tool_executions e
         on e.mcp_execution_id = c.mcp_execution_id
         or (e.ai_tool_call_id is not null and e.ai_tool_call_id = c.ai_tool_call_id)
       where c.created_at >= '$1'
         and c.tool_name like '%$2%'
         and (r.requested_model = '$3' or r.model = '$3')${4:+ and r.user_id = '$4'}
       order by c.created_at desc limit 1"
}

# How many requests the model dispatched at all: none means the client never
# reached the gateway, which is a different fault from a model declining.
#   $1 cutoff  $2 model  [$3 user_id]
e2e_count_requests() {
    q "select count(*) from ai_requests
       where created_at >= '$1'
         and (requested_model = '$2' or model = '$2')${3:+ and user_id = '$3'}"
}
