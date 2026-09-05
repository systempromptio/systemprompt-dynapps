#!/usr/bin/env bash
# End-to-end proof that OpenCode works through the gateway, for EVERY model the
# gateway serves: enrolment, model discovery, the synced skills, a live MCP
# tools/list, and then one cell per model in which the model must call the
# `systemprompt` MCP tool.
#
# The other harness (clean-client-install.sh) proves the installer writes the
# right files. This one proves the client can then actually be used, which is a
# different claim and the one that matters: a provider block can be present and
# correct while the model it names is undispatchable, or while the MCP server it
# registers never answers.
#
# What a cell asserts has deliberately moved off the transcript. Grepping
# OpenCode's stdout for a tool name proves only that a string was printed --
# it went green while the model narrated calling the tool and never did. A cell
# now passes only when the server, independently, recorded both halves of the
# call: a successful mcp_tool_executions row, and an ai_request_tool_calls row
# hanging off an ai_requests row for THAT model. Nothing the model can say
# fabricates either.
#
# Usage: scripts/e2e-opencode.sh <PAT> [GATEWAY] [--list]
#   MODELS=a,b,c   run only these models instead of every one /v1/models offers
#   --list         print the model list and exit
set -uo pipefail

LIST_ONLY=0
ARGS=()
for a in "$@"; do
    case "$a" in
        --list) LIST_ONLY=1 ;;
        *)      ARGS+=("$a") ;;
    esac
done
set -- ${ARGS[@]+"${ARGS[@]}"}

GATEWAY="${2:-http://host.docker.internal:8080}"
# The container reaches the gateway through host.docker.internal; this script,
# the model list, and every database assertion run on the host, which cannot
# resolve that name.
HOST_GATEWAY="${HOST_GATEWAY:-$(printf '%s' "$GATEWAY" | sed 's|host\.docker\.internal|127.0.0.1|')}"
IMAGE="astound-clean-client:local"
CONTAINER="astound-e2e-opencode"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMPT="Call the systemprompt tool now with command 'core skills list', then reply with only the first skill id it printed."

rc=0
ok()   { printf '[OK  ] %s\n' "$*"; }
fail() { printf '[FAIL] %s\n' "$*"; rc=1; }
info() { printf '[INFO] %s\n' "$*"; }
say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

# ------------------------------------------------------------------ model list
#
# /v1/models is unauthenticated and Anthropic-shaped. Deriving the matrix from it
# is the point: a model added to the catalogue is in this harness the same day,
# and a hardcoded list cannot silently stop covering the fleet.
if [ -n "${MODELS:-}" ]; then
    IFS=',' read -r -a MODEL_LIST <<< "$MODELS"
else
    MODELS_JSON=$(curl -sS --max-time 15 "$HOST_GATEWAY/v1/models" 2>/dev/null)
    # shellcheck disable=SC2207
    MODEL_LIST=($(printf '%s' "$MODELS_JSON" | jq -r '.data[]?.id' 2>/dev/null))
fi
if [ "${#MODEL_LIST[@]}" -eq 0 ]; then
    echo "[FAIL] $HOST_GATEWAY/v1/models offered no models — is the server running?" >&2
    exit 1
fi
if [ "$LIST_ONLY" -eq 1 ]; then
    printf '%s\n' "${MODEL_LIST[@]}"
    exit 0
fi

PAT="${1:?usage: e2e-opencode.sh <PAT> [GATEWAY] [--list]}"

# ------------------------------------------------------------------- database
#
# The assertions are server-side, so a missing database is a failed run and not
# a skipped one: a check that quietly disappears is indistinguishable from one
# that passed.
DBURL="${SYSTEMPROMPT_DATABASE_URL:-$(python3 -c 'import json;print(json.load(open("'"$REPO_ROOT"'/.systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)}"
if [ -z "${DBURL:-}" ] || ! command -v psql >/dev/null 2>&1; then
    echo "[FAIL] need psql and a local database_url to verify tool calls server-side" >&2
    exit 1
fi
q() { psql "$DBURL" -tAc "$1" 2>/dev/null | tr -d '[:space:]'; }
# Epoch seconds, not a rendered timestamp: a timestamp literal contains a space,
# and trimming whitespace out of a scalar result corrupts it into a value every
# comparison rejects -- silently, as an empty window that looks like an audit gap.
db_now() { q "select extract(epoch from now())"; }
if [ -z "$(db_now)" ]; then
    echo "[FAIL] cannot reach the database at the local profile's database_url" >&2
    exit 1
fi

# Three ways this bites, all of which left a live container behind and failed
# the next run on a name conflict. The docker shim on PATH mangles a bare `-f`,
# so removal is spelled `--force`. `docker rm` under a Windows docker.exe
# reached through WSL has its argument path-mangled into
# \\wsl.localhost\...\astound-e2e-opencode and answers "No such container",
# so `stop` (which is not mangled, and auto-removes a --rm container) is tried
# first. And removal is asynchronous, so the name outlives the command that
# removed it -- wait the name out rather than racing it.
cleanup() {
    docker stop "$CONTAINER" >/dev/null 2>&1 || true
    docker rm --force "$CONTAINER" >/dev/null 2>&1 || true
    local i=0
    while docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER"; do
        i=$(( i + 1 )); [ "$i" -gt 30 ] && break
        sleep 1
    done
}
trap cleanup EXIT
cleanup

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    say "building $IMAGE"
    docker build --build-arg INSTALL_BROWSERS=0 \
        -f "$REPO_ROOT/deploy/clean-client/Dockerfile" \
        -t "$IMAGE" "$REPO_ROOT/deploy/clean-client"
fi

say "building and publishing the Linux bridge tarball"
( cd "$REPO_ROOT/bridge" && cargo build --release ) || { echo "[FAIL] bridge build failed" >&2; exit 1; }
"$REPO_ROOT/scripts/package-bridge-linux.sh" || { echo "[FAIL] bridge packaging failed" >&2; exit 1; }

# The container is long-lived so each model is a separate `docker exec` against
# one enrolment. Re-enrolling per model would burn a connect code per cell and
# make the timing windows meaningless.
say "starting $CONTAINER"
docker run -d --rm --name "$CONTAINER" \
    --hostname e2e-opencode \
    --add-host host.docker.internal:host-gateway \
    -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
    -e OPENCODE_DISABLE_AUTOUPDATE=1 \
    --entrypoint bash \
    "$IMAGE" -lc 'sleep infinity' >/dev/null || { echo "[FAIL] could not start $CONTAINER" >&2; exit 1; }

# Stdin is closed on every exec. A docker exec inheriting this script's stdin
# eats the rest of an enclosing here-doc, which silently truncates the caller
# and is maddening to diagnose from the far end.
cx() { docker exec "$CONTAINER" bash -lc "$1" </dev/null; }

# ------------------------------------------------------------- 1. enrolment

say "1. enrol"
if cx "sudo install -d -o tester -g tester -m 0755 /etc/opencode && curl -fsSL '$GATEWAY/files/downloads/install.sh' | sh -s -- --download-base '$GATEWAY/files/downloads' --gateway '$GATEWAY' --pat '$PAT' --host opencode"; then
    ok "installer enrolled the host"
else
    fail "installer exited non-zero"
    exit 1
fi

# The bridge has written the user config as both opencode.json and
# opencode.jsonc across versions; pin to whichever exists rather than to a name
# that was right once.
CFG_SNIPPET='CFG=/etc/opencode/opencode.json; [ -f "$CFG" ] || CFG="$HOME/.config/opencode/opencode.json"; USER_CFG=""; for f in "$HOME/.config/opencode/opencode.jsonc" "$HOME/.config/opencode/opencode.json"; do [ -f "$f" ] && { USER_CFG="$f"; break; }; done; [ -n "$USER_CFG" ] || USER_CFG="$HOME/.config/opencode/opencode.json";'

say "2. what the provider block offers"
OFFERED=$(cx "$CFG_SNIPPET jq -r '.provider.systemprompt.models | keys[]?' \"\$CFG\" 2>/dev/null" | tr -d '\r')
info "offered: $(printf '%s' "$OFFERED" | tr '\n' ' ')"
MISSING=""
for m in "${MODEL_LIST[@]}"; do
    printf '%s\n' "$OFFERED" | grep -qx -- "$m" || MISSING="$MISSING $m"
done
if [ -z "$MISSING" ]; then
    ok "every gateway model is offered to OpenCode"
else
    fail "not offered to OpenCode:$MISSING — the surface filter is narrower than /v1/models"
fi

say "3. MCP connectors synced into the user config"
MCP_URL=$(cx "$CFG_SNIPPET grep -hoE 'http://127\.0\.0\.1:[0-9]+/mcp/[a-z0-9_-]+' \"\$CFG\" \"\$USER_CFG\" 2>/dev/null | head -1" | tr -d '\r')
if [ -z "$MCP_URL" ]; then
    fail "no loopback MCP url in the user config"
else
    info "mcp url: $MCP_URL"
    # A bare tools/list is a malformed MCP request and answers 422. The
    # streamable-HTTP transport wants initialize, then the initialized
    # notification, then tools/list carrying the session id.
    TOOLS=$(cx '
      set -e
      URL="'"$MCP_URL"'"
      KEY=$(cat "${XDG_CONFIG_HOME:-$HOME/.config}/astound/bridge-loopback.key")
      MH=(-H content-type:application/json -H "accept:application/json,text/event-stream" -H "Authorization: Bearer $KEY")
      curl -sS -D /tmp/mcp-h.txt -o /dev/null -X POST "$URL" "${MH[@]}" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"e2e\",\"version\":\"1\"}}}"
      SID=$(grep -i "mcp-session-id" /tmp/mcp-h.txt | tr -d "\r" | awk "{print \$2}")
      [ -n "$SID" ] || exit 1
      curl -sS -X POST "$URL" "${MH[@]}" -H "mcp-session-id: $SID" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" >/dev/null
      curl -sS -X POST "$URL" "${MH[@]}" -H "mcp-session-id: $SID" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}" \
        | sed -n "s/^data: //p" | jq -r ".result.tools[]?.name" 2>/dev/null
    ' | tr -d '\r' | tr '\n' ' ')
    info "mcp tools: ${TOOLS:-(none)}"
    case " $TOOLS " in
        *" systemprompt "*) ok "the systemprompt MCP tool is live" ;;
        *) fail "the MCP server did not list a systemprompt tool" ;;
    esac
fi

say "4. restrict the toolset to the MCP tool under test"
# OpenCode offers its own built-ins alongside the MCP server, and gpt-4.1 was
# answering in a dozen output tokens without calling anything while choosing
# among fourteen tools. That is a tool-selection result, not an MCP result, and
# this harness is not the place to measure it -- the wire matrix, offering the
# one tool, gets the same model to call it. Disable everything and re-enable
# only the MCP tool, so a failed cell means the MCP path did not work.
#
# `tools` takes a wildcard map, last-match-wins. Verified on opencode 1.18.29:
# {"*": false} offers nothing, {"*": false, "bash": true} offers exactly bash.
cx "$CFG_SNIPPET
    mkdir -p \"\$(dirname \"\$USER_CFG\")\"
    [ -s \"\$USER_CFG\" ] || printf '{}' > \"\$USER_CFG\"
    jq '.tools = {\"*\": false, \"systemprompt*\": true}' \"\$USER_CFG\" > \"\$USER_CFG.tmp\" \
      && mv \"\$USER_CFG.tmp\" \"\$USER_CFG\"" >/dev/null 2>&1
TOOLS_CFG=$(cx "$CFG_SNIPPET jq -c '.tools' \"\$USER_CFG\" 2>/dev/null" | tr -d '\r')
info "tools policy: ${TOOLS_CFG:-(none)}"
case "$TOOLS_CFG" in
    *'"*":false'*) ok "built-in tools are disabled; only the MCP tool is on offer" ;;
    *) fail "could not restrict the toolset — cells would measure tool selection, not the MCP path" ;;
esac

say "5. skills synced"
COUNT=$(cx 'find "$HOME/.config/opencode/skills" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l' | tr -d '[:space:]')
info "skills synced: ${COUNT:-0}"
[ "${COUNT:-0}" -gt 0 ] && ok "skills were synced" || fail "no skills were synced"

# ------------------------------------------------------- 5. one cell per model
#
# Attribution is by time window: cells are serialised, and each window opens at
# the database's own now() (never the host clock, which can drift from the
# container's and from Postgres') immediately before the cell runs. Nothing else
# is writing these tables during a cell.

say "6. the model matrix — ${#MODEL_LIST[@]} models, one forced MCP tool call each"
RESULTS=()
for model in "${MODEL_LIST[@]}"; do
    T=$(db_now)
    OUT=$(docker exec "$CONTAINER" bash -lc "opencode run --model 'systemprompt/$model' \"\$1\" 2>&1" _ "$PROMPT" </dev/null)
    status=$?

    # Scoped to a gateway session this model was actually dispatched under, not
    # merely to the time window. Another harness running against the same
    # database satisfied this half from its own rows, so a cell could report
    # "the tool ran" about someone else's tool call.
    MCP_ROWS=$(q "select count(*) from mcp_tool_executions e
                  where e.tool_name = 'systemprompt' and e.status = 'success'
                    and extract(epoch from e.created_at) > $T
                    and e.session_id in (
                        select r.session_id from ai_requests r
                         where coalesce(r.requested_model, r.model) = '$model'
                           and extract(epoch from r.created_at) > $T)")
    AI_ROWS=$(q "select count(*) from ai_request_tool_calls c
                   join ai_requests r on r.request_id = c.request_id
                  where c.tool_name like '%systemprompt%'
                    and coalesce(r.requested_model, r.model) = '$model'
                    and extract(epoch from c.created_at) > $T")
    MCP_ROWS=${MCP_ROWS:-0}; AI_ROWS=${AI_ROWS:-0}

    if [ "$status" -ne 0 ]; then
        verdict="FAIL"; why="opencode exit $status: $(printf '%s' "$OUT" | tr -d '\n' | tail -c 160)"
    elif [ "$MCP_ROWS" -gt 0 ] && [ "$AI_ROWS" -gt 0 ]; then
        verdict="PASS"; why="mcp_tool_executions=$MCP_ROWS ai_request_tool_calls=$AI_ROWS"
    elif [ "$MCP_ROWS" -gt 0 ]; then
        verdict="FAIL"; why="the tool ran ($MCP_ROWS) but no ai_request_tool_calls row is attributed to $model"
    elif [ "$AI_ROWS" -gt 0 ]; then
        verdict="FAIL"; why="the request recorded a tool call but no successful mcp_tool_executions row followed"
    else
        # Quote what it said instead. "the model answered without one" is a
        # symptom; the answer itself distinguishes a model that declined from one
        # that claims it called the tool, or fabricates the output it was asked
        # to report -- and those have different fixes.
        SAID=$(printf '%s' "$OUT" | tr -d '\r' | tr '\n' ' ' | sed 's/  */ /g' | tail -c 300)
        verdict="FAIL"; why="no server-side tool call at all — the model answered without one; it said: ${SAID:-<nothing>}"
    fi
    [ "$verdict" = PASS ] && ok "$model — $why" || fail "$model — $why"
    RESULTS+=("$model|$verdict|$why")
done

say "result"
printf '%-34s %-6s %s\n' MODEL VERDICT DETAIL
printf '%-34s %-6s %s\n' '---' '---' '---'
for row in "${RESULTS[@]}"; do
    printf '%-34s %-6s %s\n' "${row%%|*}" "$(printf '%s' "$row" | cut -d'|' -f2)" "$(printf '%s' "$row" | cut -d'|' -f3-)"
done
echo
if [ $rc -eq 0 ]; then echo "ALL OPENCODE E2E CHECKS PASSED"; else echo "OPENCODE E2E FAILURES ABOVE"; fi
exit $rc
