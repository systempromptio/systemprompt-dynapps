#!/usr/bin/env bash
# Drive a real Claude Code agent turn through the gateway, once per model the
# gateway advertises, and prove server-side that the model actually called the
# `systemprompt` MCP tool.
#
# Why server-side. The obvious check — grep the transcript for the tool name —
# has already produced a false pass: the model shelled out to `echo`, printed
# something that looked like tool output, and the harness went green. Claude
# Code's own transcript is the model's account of what it did, and that is
# exactly the thing under test. So nothing here reads the transcript for a
# verdict; the transcript is captured for diagnosis only.
#
# The proof is two rows that the server writes and the client cannot forge:
#
#   mcp_tool_executions    the MCP server ran the tool (status='success')
#   ai_request_tool_calls  an inference request on THIS model asked for it,
#                          joined to ai_requests for the model name
#
# Both must appear, and the ai_requests row must name the model of the cell, or
# the cell fails. That join is also how a cell is attributed: not by a
# timestamp window alone but by the model recorded on the request the tool call
# hangs off, so a stray execution from another agent on this box cannot make a
# cell pass.
#
# Usage:
#   scripts/e2e-claude-code.sh --list
#   scripts/e2e-claude-code.sh <PAT> [GATEWAY]
#
# Environment:
#   MODELS=a,b,c        run only these models instead of everything advertised
#   SYSTEMPROMPT_DATABASE_URL  override the Postgres URL (default: the local
#                       profile's secrets.json)
#   SKIP_BRIDGE_BUILD=1 reuse the bridge tarball already in dist/
#   CELL_TIMEOUT=180    seconds allowed for one `claude -p` turn
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="astound-clean-client:local"
CONTAINER="astound-e2e-claude"
CELL_TIMEOUT="${CELL_TIMEOUT:-180}"

# The tool the model must reach, on both sides of the wire. Claude Code
# namespaces an MCP tool as mcp__<server>__<tool>; the server records the bare
# names.
MCP_SERVER="systemprompt"
MCP_TOOL="systemprompt"
CLAUDE_TOOL="mcp__${MCP_SERVER}__${MCP_TOOL}"

# Forcing the tool is prompt work, and the prompt is deliberately narrow: name
# the tool, name the argument, and forbid the two escape hatches a model reaches
# for first (its own memory, and a shell).
CELL_PROMPT="Use the ${CLAUDE_TOOL} tool with the argument command set to the string \"core skills list\". Then reply with the first skill id it printed, and nothing else. You MUST call that tool. Do not use Bash, do not use any other tool, and do not answer from memory."

say()  { printf '\033[1m==>\033[0m %s\n' "$*"; }
info() { printf '[INFO] %s\n' "$*"; }
warn() { printf '\033[33mwarn:\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ── Arguments ─────────────────────────────────────────────────────────────────
# --list is accepted anywhere, so `... <PAT> --list` and `--list` both work; the
# sibling OpenCode harness takes it in either position and the two are used
# interchangeably.
LIST_ONLY=0
ARGS=()
for a in "$@"; do
    case "$a" in
        --list) LIST_ONLY=1 ;;
        *)      ARGS+=("$a") ;;
    esac
done
set -- ${ARGS[@]+"${ARGS[@]}"}

PAT="${1:-}"
GATEWAY="${2:-http://host.docker.internal:8080}"
[ "$LIST_ONLY" = "1" ] || [ -n "$PAT" ] \
    || fail "usage: e2e-claude-code.sh [--list] <PAT> [GATEWAY]

A PAT is minted with:
  systemprompt admin users api-key issue --user <uuid> --name e2e-claude
The user must hold the admin role: the systemprompt MCP server requires scope
'admin', and that scope is baked into the bridge JWT at issue time."

# The container reaches the gateway at host.docker.internal; this script runs on
# the host and cannot resolve that name, so it probes the same server directly.
HOST_GATEWAY="${GATEWAY//host.docker.internal/127.0.0.1}"
HOST_GATEWAY="${HOST_GATEWAY//localhost/127.0.0.1}"

command -v jq    >/dev/null 2>&1 || fail "jq is required"
command -v psql  >/dev/null 2>&1 || fail "psql is required — the pass is asserted against Postgres, not against the transcript"
command -v docker >/dev/null 2>&1 || fail "docker is required"

# ── The server under test ─────────────────────────────────────────────────────
curl -fsS --max-time 5 "$HOST_GATEWAY/v1/models" >/dev/null 2>&1 \
    || fail "no gateway answering at $HOST_GATEWAY — start the server (\`just start\`) and re-run.
       This harness deliberately does not start or restart anything."

ADVERTISED="$(curl -fsS "$HOST_GATEWAY/v1/models" | jq -r '.data[].id')"
[ -n "$ADVERTISED" ] || fail "the gateway advertises no models"

if [ -n "${MODELS:-}" ]; then
    MODEL_LIST="$(printf '%s' "$MODELS" | tr ',' '\n' | sed '/^[[:space:]]*$/d')"
else
    MODEL_LIST="$ADVERTISED"
fi
MODEL_COUNT="$(printf '%s\n' "$MODEL_LIST" | wc -l | tr -d ' ')"

if [ "$LIST_ONLY" = "1" ]; then
    printf '%s\n' "$ADVERTISED"
    exit 0
fi

DB_URL="${SYSTEMPROMPT_DATABASE_URL:-$(python3 -c 'import json;print(json.load(open("'"$REPO_ROOT"'/.systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)}"
[ -n "${DB_URL:-}" ] || fail "no database_url — set SYSTEMPROMPT_DATABASE_URL"
psql "$DB_URL" -tAc 'select 1' >/dev/null 2>&1 \
    || fail "cannot reach Postgres at the profile's database_url — \`just db-up\` first"

# One-column, no-header psql, so the callers below can treat output as a value.
q() { psql "$DB_URL" -tAF'|' -v ON_ERROR_STOP=1 -c "$1"; }

# ── Container lifecycle ───────────────────────────────────────────────────────
# A trap rather than --rm: the container is long-lived across the model loop, so
# an interrupt mid-sweep would otherwise leave it holding the state volume.
# Two hazards, both of which have already cost a whole sweep.
#
# `--force`, never `-f`: a common WSL shim forwards to Docker Desktop and
# path-translates the argument after `-f`, reading it as `docker build --file`.
# `docker rm -f <name>` then asks the daemon to remove a Windows path, which
# fails with "No such container" while the container keeps running — and the
# usual `|| true` swallows it. The long flag is untouched by that translation.
#
# And removal is asynchronous: `docker rm` returns when the kill is sent, not
# when the name is released, so a run started straight after a previous one
# loses the name to a container that is still dying. Wait for it to go.
cleanup() {
    docker rm --force "$CONTAINER" >/dev/null 2>&1 || true
    for _ in $(seq 1 50); do
        docker inspect "$CONTAINER" >/dev/null 2>&1 || return 0
        sleep 0.2
    done
    warn "$CONTAINER is still present after 10s — remove it by hand"
}
trap cleanup EXIT INT TERM
cleanup

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    say "building $IMAGE"
    docker build --build-arg INSTALL_BROWSERS=0 \
        -f "$REPO_ROOT/deploy/clean-client/Dockerfile" \
        -t "$IMAGE" "$REPO_ROOT/deploy/clean-client"
fi

# The client installs the bridge by downloading the tarball the gateway serves,
# which is the path a real workstation takes. Rebuilding it is expensive and
# takes the shared cargo lock, so an existing tarball is reused by default.
TARBALL="$REPO_ROOT/dist/astound-bridge-linux-$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch64/').tar.gz"
if [ "${SKIP_BRIDGE_BUILD:-0}" = "1" ] && [ -f "$TARBALL" ]; then
    info "reusing $TARBALL"
else
    say "packaging the Linux bridge tarball"
    "$REPO_ROOT/scripts/package-bridge-linux.sh"
fi

say "starting $CONTAINER"
docker run -d --name "$CONTAINER" \
    --hostname e2e-claude \
    --add-host host.docker.internal:host-gateway \
    -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
    --entrypoint bash \
    "$IMAGE" -lc 'sleep infinity' >/dev/null

# Run as the unprivileged tester through a login shell, because that is what
# puts the bridge's managed environment in front of `claude`.
# </dev/null is load-bearing. `docker exec` inherits and drains the caller's
# stdin, and the model loop below is fed from a here-doc — so without this the
# first cell swallowed the remaining 31 models and the sweep reported a clean
# 1/1 pass over a matrix of 32.
cx() { docker exec -u tester "$CONTAINER" bash -lc "$1" </dev/null; }

# ── Enrolment ─────────────────────────────────────────────────────────────────
say "enrolling the clean client (host: claude-code)"
cx "curl -fsSL '$GATEWAY/files/downloads/install.sh' \
      | sh -s -- --download-base '$GATEWAY/files/downloads' \
                 --gateway '$GATEWAY' --pat '$PAT' --host claude-code" \
    || fail "the installer exited non-zero — nothing below would mean anything"

# ── Pre-flight: is the tool actually reachable? ───────────────────────────────
# A model that never sees the tool fails every cell for a reason that has
# nothing to do with the model. Ask the loopback MCP endpoint directly first, so
# a broken connector is reported once here instead of 32 times below.
say "pre-flight: listing tools on the loopback MCP endpoint"
MCP_URL="http://127.0.0.1:48217/mcp/${MCP_SERVER}"
TOOLS="$(cx '
set -euo pipefail
KEY=$(cat "${XDG_CONFIG_HOME:-$HOME/.config}/astound/bridge-loopback.key")
MH="-H content-type:application/json -H accept:application/json,text/event-stream"
curl -sS -D /tmp/h.txt -o /dev/null -X POST '"$MCP_URL"' -H "Authorization: Bearer $KEY" $MH \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"e2e\",\"version\":\"1\"}}}"
SID=$(grep -i "mcp-session-id" /tmp/h.txt | tr -d "\r" | awk "{print \$2}")
[ -n "$SID" ] || { echo "NO_SESSION"; exit 0; }
curl -sS -X POST '"$MCP_URL"' -H "Authorization: Bearer $KEY" $MH -H "mcp-session-id: $SID" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" >/dev/null
curl -sS -X POST '"$MCP_URL"' -H "Authorization: Bearer $KEY" $MH -H "mcp-session-id: $SID" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}" \
  | sed -n "s/^data: //p" | jq -r "..|.tools? // empty | .[].name" 2>/dev/null | tr "\n" " "
' 2>/dev/null || true)"
info "tools: ${TOOLS:-(none)}"
case " $TOOLS " in
    *" $MCP_TOOL "*) info "the $MCP_TOOL tool is reachable" ;;
    *) fail "the $MCP_TOOL tool is not reachable over the loopback proxy — every cell below would fail for that reason, not for a model reason. Check the PAT's admin role: services/mcp/systemprompt.yaml requires scope 'admin'." ;;
esac

# Listed is not the same as working. The tool shells out to the real CLI, so a
# broken `services/` config, a bad profile, or a missing binary makes every call
# fail while tools/list keeps answering happily — and every cell below would
# then read as a model failure. Call it once, for real, and stop here if the
# call itself cannot succeed.
say "pre-flight: calling the $MCP_TOOL tool once"
PROBE_CUTOFF="$(q "select now()")"
cx "cd \$HOME/work && timeout 60 claude -p \"${CELL_PROMPT//\"/\\\"}\" \
      --model '$(printf '%s\n' "$MODEL_LIST" | head -1)' \
      --permission-mode bypassPermissions \
      --allowedTools '$CLAUDE_TOOL' >/dev/null 2>&1" || true
sleep 3
PROBE_FAIL="$(q "select error_message from mcp_tool_executions
                 where created_at >= '$PROBE_CUTOFF' and tool_name = '$MCP_TOOL'
                   and status <> 'success' order by created_at desc limit 1")"
PROBE_OK="$(q "select count(*) from mcp_tool_executions
               where created_at >= '$PROBE_CUTOFF' and tool_name = '$MCP_TOOL'
                 and status = 'success'")"
if [ "$PROBE_OK" = "0" ] && [ -n "$PROBE_FAIL" ]; then
    fail "the $MCP_TOOL tool is registered but every call fails, so no model
       could pass. This is an environment fault, not a model one. The server
       recorded:

$(printf '%s\n' "$PROBE_FAIL" | tail -6)"
fi
[ "$PROBE_OK" = "0" ] && warn "the pre-flight call recorded no execution — cells may fail for a non-model reason"

# Claude Code discovers the tool through the per-plugin .mcp.json the bridge
# sync wrote. If that file never landed, the tool exists but the agent cannot
# see it — a different failure with the same symptom, so distinguish them.
if ! cx "grep -rl '\"${MCP_SERVER}\"' \$HOME/.claude/plugins 2>/dev/null | head -1" | grep -q .; then
    warn "no plugin .mcp.json mentions ${MCP_SERVER} — Claude Code may not see the tool"
fi

# ── The sweep ─────────────────────────────────────────────────────────────────
# One cell per model, serialised. Serial is not a performance oversight: the
# proof reads rows written since a cutoff, and concurrent cells would interleave
# in that window.
RESULTS_FILE="$(mktemp)"
LOG_DIR="$(mktemp -d)"
trap 'cleanup; rm -f "$RESULTS_FILE"; rm -rf "$LOG_DIR"' EXIT INT TERM

say "running $MODEL_COUNT model cells (transcripts in $LOG_DIR)"
# Split into an array rather than piping the list into `while read`: a loop
# whose iterator IS stdin is one stray stdin-reading command away from ending
# early, and it ends early silently, reporting a pass over the cells it skipped.
OLD_IFS="$IFS"; IFS=$'\n'
set -f
MODEL_ARR=($MODEL_LIST)
set +f
IFS="$OLD_IFS"

CELL_N=0
for MODEL in ${MODEL_ARR[@]+"${MODEL_ARR[@]}"}; do
    [ -n "$MODEL" ] || continue
    CELL_N=$((CELL_N + 1))
    printf '\n\033[1m[%d/%d] %s\033[0m\n' "$CELL_N" "$MODEL_COUNT" "$MODEL"

    # The cutoff comes from the database, not from the host clock: the two can
    # disagree, and a cutoff even slightly in the future silently hides every
    # row the cell is about to write.
    CUTOFF="$(q "select now()")"

    START="$(date +%s)"
    RAW="$LOG_DIR/${MODEL//\//_}.txt"
    set +e
    cx "cd \$HOME/work && timeout ${CELL_TIMEOUT} claude -p \"${CELL_PROMPT//\"/\\\"}\" \
          --model '$MODEL' \
          --permission-mode bypassPermissions \
          --allowedTools '$CLAUDE_TOOL' 2>&1" >"$RAW" 2>&1
    CLAUDE_RC=$?
    set -e
    ELAPSED=$(( $(date +%s) - START ))

    # Give the audit writes a moment to land: the gateway records the request
    # and its tool calls after the response has been streamed to the client, so
    # `claude` can exit before the rows exist.
    sleep 3

    # Proof 1 — the MCP server ran the tool.
    MCP_OK="$(q "select count(*) from mcp_tool_executions
                 where created_at >= '$CUTOFF'
                   and tool_name = '$MCP_TOOL' and server_name = '$MCP_SERVER'
                   and status = 'success'")"

    # Proof 2 — an inference request on THIS model asked for it. The link
    # column is nullable and two different columns can carry it, so accept
    # either; requiring one would fail cells that genuinely passed.
    MODEL_ROW="$(q "select r.requested_model, r.model
                    from ai_request_tool_calls c
                    join ai_requests r on r.id = c.request_id
                    left join mcp_tool_executions e
                      on e.mcp_execution_id = c.mcp_execution_id
                      or (e.ai_tool_call_id is not null and e.ai_tool_call_id = c.ai_tool_call_id)
                    where c.created_at >= '$CUTOFF'
                      and c.tool_name like '%${MCP_TOOL}%'
                      and (r.requested_model = '$MODEL' or r.model = '$MODEL')
                    order by c.created_at desc limit 1")"

    REQ_COUNT="$(q "select count(*) from ai_requests
                    where created_at >= '$CUTOFF'
                      and (requested_model = '$MODEL' or model = '$MODEL')")"

    CALL_OK=0; [ -n "$MODEL_ROW" ] && CALL_OK=1
    VERDICT=FAIL
    NOTE=""
    if [ "$MCP_OK" -gt 0 ] && [ "$CALL_OK" = "1" ]; then
        VERDICT=PASS
    elif [ "$REQ_COUNT" = "0" ]; then
        NOTE="no ai_requests row — the model never dispatched (rc=$CLAUDE_RC)"
    elif [ "$CALL_OK" = "0" ]; then
        NOTE="dispatched but no tool call recorded on this model (rc=$CLAUDE_RC)"
    else
        NOTE="tool call recorded but no successful MCP execution (rc=$CLAUDE_RC)"
    fi
    [ -n "$NOTE" ] || NOTE="ok"

    printf '%s|%s|%s|%s|%s|%s|%s\n' \
        "$MODEL" "$VERDICT" "$REQ_COUNT" "$CALL_OK" "$MCP_OK" "${ELAPSED}s" "$NOTE" \
        >>"$RESULTS_FILE"
    printf '  %s  ai_request=%s tool_call=%s mcp_exec=%s  %ss  %s\n' \
        "$VERDICT" "$REQ_COUNT" "$CALL_OK" "$MCP_OK" "$ELAPSED" "$NOTE"
done

# ── Report ────────────────────────────────────────────────────────────────────
echo
say "results"
printf '%-40s %-6s %-11s %-10s %-9s %-8s %s\n' \
    MODEL VERDICT AI_REQUEST TOOL_CALL MCP_EXEC LATENCY NOTE
printf '%-40s %-6s %-11s %-10s %-9s %-8s %s\n' \
    "$(printf '%.0s-' $(seq 40))" ------ ----------- ---------- --------- -------- ----
while IFS='|' read -r m v r c e l n; do
    printf '%-40s %-6s %-11s %-10s %-9s %-8s %s\n' "$m" "$v" "$r" "$c" "$e" "$l" "$n"
done <"$RESULTS_FILE"

PASSED="$(grep -c '|PASS|' "$RESULTS_FILE" || true)"
TOTAL="$(wc -l <"$RESULTS_FILE" | tr -d ' ')"
echo
info "$PASSED/$TOTAL models drove a proven MCP tool call"
info "transcripts: $LOG_DIR (copy them out before this script exits)"

[ "$PASSED" = "$TOTAL" ]
