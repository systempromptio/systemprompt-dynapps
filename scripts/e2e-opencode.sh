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
#   MCP_SERVER / MCP_TOOL / CELL_PROMPT
#                  the server, the tool and the prompt of every cell; default
#                  to the systemprompt tool and the prompt below
set -uo pipefail

# shellcheck source=lib/e2e-client.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/e2e-client.sh"

LIST_ONLY=0
ARGS=()
for a in "$@"; do
    case "$a" in
        --list) LIST_ONLY=1 ;;
        *)      ARGS+=("$a") ;;
    esac
done
set -- ${ARGS[@]+"${ARGS[@]}"}

# Where the OpenCode client runs. A container is the honest default: it proves a
# clean machine can enrol. But Docker Desktop's VM cannot reach this WSL distro
# on any address, so when that breaks the whole harness is unrunnable for a
# reason that has nothing to do with the product. `E2E_CLIENT=host` runs the
# identical steps against the host's own opencode, in a throwaway HOME, so the
# claim can still be checked.
CLIENT_MODE="${E2E_CLIENT:-container}"
NODE_BIN_DIR="$(dirname "$(command -v node 2>/dev/null || echo /usr/bin/node)")"
case "$CLIENT_MODE" in
    container|host) ;;
    *) echo "[FAIL] E2E_CLIENT must be 'container' or 'host', not '$CLIENT_MODE'" >&2; exit 1 ;;
esac
if [ "$CLIENT_MODE" = host ]; then
    GATEWAY="${2:-http://127.0.0.1:8081}"
else
    GATEWAY="${2:-http://host.docker.internal:8080}"
fi
# The container reaches the gateway through host.docker.internal; this script,
# the model list, and every database assertion run on the host, which cannot
# resolve that name. In host mode the two are already the same address.
HOST_GATEWAY="${HOST_GATEWAY:-$(printf '%s' "$GATEWAY" | sed 's|host\.docker\.internal|127.0.0.1|')}"
IMAGE="astound-clean-client:local"
CONTAINER="astound-e2e-opencode"
e2e_repo_root
# Host mode takes its opencode out of the container image, so both modes drive
# the same client bytes and neither needs the network. Whatever happens to be
# installed on the host can be years stale -- this machine carries 0.5.1, which
# rejects the `tools` key outright and so cannot run the toolset restriction the
# matrix depends on. Set OPENCODE_BIN to pin a specific binary instead.
OPENCODE_BIN="${OPENCODE_BIN:-}"
OPENCODE_PKG="${OPENCODE_PKG:-opencode-ai@latest}"
MCP_SERVER="${MCP_SERVER:-systemprompt}"
MCP_TOOL="${MCP_TOOL:-systemprompt}"
PROMPT="${CELL_PROMPT:-Call the ${MCP_TOOL} tool now with command 'core skills list', then reply with only the first skill id it printed.}"

# This harness accumulates failures rather than exiting on the first, and its
# section banners stand apart from the checks, so `fail` and `say` override
# the library's; `ok` and `info` are the library's.
rc=0
fail() { printf '[FAIL] %s\n' "$*"; rc=1; }
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
if ! e2e_db_url || ! command -v psql >/dev/null 2>&1; then
    echo "[FAIL] need psql and a local database_url to verify tool calls server-side" >&2
    exit 1
fi
# The whitespace-stripped scalar helper; `db_now` (epoch seconds, never a
# rendered timestamp — the space inside one would be stripped too) is the
# library's.
q() { q_scalar "$1"; }
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
# In host mode the sandbox HOME is the only thing to remove; there is no
# container, and calling docker here would fail noisily for no reason.
cleanup() {
    if [ "$CLIENT_MODE" = host ]; then
        # The bridge forks a proxy that outlives this script. Removing the
        # sandbox without stopping it leaves a process holding a loopback port
        # with its own config deleted underneath it, and the next run then
        # binds the next port up and talks to the wrong bridge.
        if [ -n "${SANDBOX:-}" ]; then
            pkill -f "^$SANDBOX/home/.local/bin/astound-bridge" >/dev/null 2>&1 || true
            rm -rf "$SANDBOX"
        fi
        return 0
    fi
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

if [ "$CLIENT_MODE" = container ]; then
    if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
        say "building $IMAGE"
        docker build --build-arg INSTALL_BROWSERS=0 \
            -f "$REPO_ROOT/deploy/clean-client/Dockerfile" \
            -t "$IMAGE" "$REPO_ROOT/deploy/clean-client"
    fi
fi

say "building and publishing the Linux bridge tarball"
( cd "$REPO_ROOT/bridge" && cargo build --release ) || { echo "[FAIL] bridge build failed" >&2; exit 1; }
"$REPO_ROOT/scripts/package-bridge-linux.sh" || { echo "[FAIL] bridge packaging failed" >&2; exit 1; }

if [ "$CLIENT_MODE" = container ]; then
    # The container is long-lived so each model is a separate `docker exec`
    # against one enrolment. Re-enrolling per model would burn a connect code
    # per cell and make the timing windows meaningless.
    say "starting $CONTAINER"
    docker run -d --rm --name "$CONTAINER" \
        --hostname e2e-opencode \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
        -e OPENCODE_DISABLE_AUTOUPDATE=1 \
        --entrypoint bash \
        "$IMAGE" -lc 'sleep infinity' >/dev/null || { echo "[FAIL] could not start $CONTAINER" >&2; exit 1; }
else
    # A throwaway HOME is what keeps this honest. The installer writes a config
    # directory, a bridge PAT and an opencode config, and every one of them must
    # land here rather than in the real home -- otherwise the harness quietly
    # edits the machine it is testing from, and a second run is no longer clean.
    SANDBOX=$(mktemp -d)
    mkdir -p "$SANDBOX/home"
    say "host client, sandbox HOME $SANDBOX/home"
    if [ -z "$OPENCODE_BIN" ]; then
        # The published package is a single self-contained ELF, so streaming it
        # out beats `docker cp`, which cannot recreate the package's internal
        # symlink under a Windows docker.exe and fails the copy halfway.
        if docker image inspect "$IMAGE" >/dev/null 2>&1; then
            info "taking opencode out of $IMAGE"
            docker run --rm --entrypoint bash "$IMAGE" \
                -c 'cat "$(readlink -f "$(command -v opencode)")"' </dev/null \
                > "$SANDBOX/opencode" 2>/dev/null
            if [ -s "$SANDBOX/opencode" ]; then
                chmod +x "$SANDBOX/opencode"
                OPENCODE_BIN="$SANDBOX/opencode"
            fi
        fi
    fi
    if [ -z "$OPENCODE_BIN" ]; then
        info "$IMAGE unavailable; installing $OPENCODE_PKG instead"
        npm install -g --prefix "$SANDBOX/npm" "$OPENCODE_PKG" \
            || { echo "[FAIL] no opencode: image absent and npm install failed (set OPENCODE_BIN)" >&2; exit 1; }
        OPENCODE_BIN="$SANDBOX/npm/bin/opencode"
    fi
    [ -x "$OPENCODE_BIN" ] || { echo "[FAIL] no opencode binary at $OPENCODE_BIN (set OPENCODE_BIN)" >&2; exit 1; }
    info "opencode: $OPENCODE_BIN ($("$OPENCODE_BIN" --version 2>/dev/null | head -1))"
fi

# Stdin is closed on every exec. A subprocess inheriting this script's stdin
# eats the rest of an enclosing here-doc, which silently truncates the caller
# and is maddening to diagnose from the far end.
cx() {
    if [ "$CLIENT_MODE" = host ]; then
        env -i HOME="$SANDBOX/home" USER="${USER:-e2e}" TERM=dumb \
            PATH="$(dirname "$OPENCODE_BIN"):${NODE_BIN_DIR:-}:/usr/local/bin:/usr/bin:/bin" \
            XDG_CONFIG_HOME="$SANDBOX/home/.config" \
            ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
            OPENCODE_DISABLE_AUTOUPDATE=1 \
            E2E_PROMPT="$PROMPT" \
            bash -c "$1" </dev/null
    else
        docker exec -e E2E_PROMPT="$PROMPT" "$CONTAINER" bash -lc "$1" </dev/null
    fi
}

# ------------------------------------------------------------- 1. enrolment

say "1. enrol"
# /etc/opencode is a system path and belongs only to the disposable container.
# In host mode the user config under the sandbox HOME carries everything.
PREP=""
[ "$CLIENT_MODE" = container ] && PREP="sudo install -d -o tester -g tester -m 0755 /etc/opencode && "
if e2e_enrol_host opencode "$PAT" "$PREP"; then
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

say "3. the loopback proxy, on the port it actually bound"
# The installer warns that the proxy did not come up whenever it is not on the
# port it first tried; it picks the next free one and says nothing about it. The
# config it wrote names the real port, so trust that and probe it -- otherwise
# the harness repeats a warning that is false.
PROXY_PORT=$(cx "$CFG_SNIPPET grep -hoE 'http://127\\.0\\.0\\.1:[0-9]+/mcp/' \"\$CFG\" \"\$USER_CFG\" 2>/dev/null | head -1 | sed 's|.*:\\([0-9]*\\)/.*|\\1|'" | tr -d '\r')
if [ -z "$PROXY_PORT" ]; then
    fail "no loopback proxy port in the opencode config — the bridge wrote no MCP entry"
else
    info "proxy port: $PROXY_PORT"
    if cx "curl -sS -o /dev/null --max-time 10 http://127.0.0.1:$PROXY_PORT/ 2>/dev/null || [ \$? -ne 7 ]"; then
        ok "the loopback proxy is listening on $PROXY_PORT"
    else
        fail "nothing is listening on the proxy port $PROXY_PORT the config names"
    fi
fi

say "4. MCP connectors synced into the user config"
MCP_URL=$(cx "$CFG_SNIPPET grep -hoE 'http://127\.0\.0\.1:[0-9]+/mcp/[a-z0-9_-]+' \"\$CFG\" \"\$USER_CFG\" 2>/dev/null | head -1" | tr -d '\r')
if [ -z "$MCP_URL" ]; then
    fail "no loopback MCP url in the user config"
else
    info "mcp url: $MCP_URL"
    # A bare tools/list is a malformed MCP request and answers 422. The
    # streamable-HTTP transport wants initialize, then the initialized
    # notification, then tools/list carrying the session id.
    TOOLS=$(e2e_mcp_tools_at "$MCP_URL" | tr -d '\r')
    info "mcp tools: ${TOOLS:-(none)}"
    case " $TOOLS " in
        *" $MCP_TOOL "*) ok "the $MCP_TOOL MCP tool is live" ;;
        *) fail "the MCP server did not list a $MCP_TOOL tool" ;;
    esac
fi

say "5. restrict the toolset to the MCP tool under test"
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
    jq '.tools = {\"*\": false, \"${MCP_SERVER}*\": true}' \"\$USER_CFG\" > \"\$USER_CFG.tmp\" \
      && mv \"\$USER_CFG.tmp\" \"\$USER_CFG\"" >/dev/null 2>&1
TOOLS_CFG=$(cx "$CFG_SNIPPET jq -c '.tools' \"\$USER_CFG\" 2>/dev/null" | tr -d '\r')
info "tools policy: ${TOOLS_CFG:-(none)}"
case "$TOOLS_CFG" in
    *'"*":false'*) ok "built-in tools are disabled; only the MCP tool is on offer" ;;
    *) fail "could not restrict the toolset — cells would measure tool selection, not the MCP path" ;;
esac

say "6. skills synced"
COUNT=$(cx 'find "$HOME/.config/opencode/skills" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l' | tr -d '[:space:]')
info "skills synced: ${COUNT:-0}"
[ "${COUNT:-0}" -gt 0 ] && ok "skills were synced" || fail "no skills were synced"

# ------------------------------------------------------- 5. one cell per model
#
# Attribution is by time window: cells are serialised, and each window opens at
# the database's own now() (never the host clock, which can drift from the
# container's and from Postgres') immediately before the cell runs. Nothing else
# is writing these tables during a cell.

say "7. the model matrix — ${#MODEL_LIST[@]} models, one forced MCP tool call each"
RESULTS=()
for model in "${MODEL_LIST[@]}"; do
    T=$(db_now)
    OUT=$(cx "opencode run --model 'systemprompt/$model' \"\$E2E_PROMPT\" 2>&1")
    status=$?

    # Scoped to a gateway session this model was actually dispatched under, not
    # merely to the time window. Another harness running against the same
    # database satisfied this half from its own rows, so a cell could report
    # "the tool ran" about someone else's tool call.
    MCP_ROWS=$(q "select count(*) from mcp_tool_executions e
                  where e.tool_name = '$MCP_TOOL' and e.status = 'success'
                    and extract(epoch from e.created_at) > $T
                    and e.session_id in (
                        select r.session_id from ai_requests r
                         where coalesce(r.requested_model, r.model) = '$model'
                           and extract(epoch from r.created_at) > $T)")
    AI_ROWS=$(q "select count(*) from ai_request_tool_calls c
                   join ai_requests r on r.request_id = c.request_id
                  where c.tool_name like '%$MCP_TOOL%'
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
