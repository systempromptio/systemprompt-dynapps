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
#   E2E_CLIENT=host     run the client on this host instead of in a container,
#                       under a throwaway HOME. Use when containers cannot
#                       reach the gateway. Default: container.
#   E2E_CLAUDE_BIN      path to the host `claude` (host mode only)
#   SKIP_BRIDGE_BUILD=1 reuse the bridge tarball already in dist/
#   CELL_TIMEOUT=180    seconds allowed for one `claude -p` turn
#   MCP_SERVER / MCP_TOOL / CELL_PROMPT
#                       the server, the tool and the prompt of every cell;
#                       default to the systemprompt tool and the prompt below
set -euo pipefail

# shellcheck source=lib/e2e-client.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/e2e-client.sh"
e2e_repo_root
IMAGE="astound-clean-client:local"
CONTAINER="astound-e2e-claude"
CELL_TIMEOUT="${CELL_TIMEOUT:-180}"

# The tool the model must reach, on both sides of the wire. Claude Code
# namespaces an MCP tool as mcp__<server>__<tool>; the server records the bare
# names.
MCP_SERVER="${MCP_SERVER:-systemprompt}"
MCP_TOOL="${MCP_TOOL:-systemprompt}"
CLAUDE_TOOL="mcp__${MCP_SERVER}__${MCP_TOOL}"

# Forcing the tool is prompt work: name the tool, name the argument, and forbid
# the two escape hatches a model reaches for first — its own memory, and a
# shell. The wording is shared verbatim with the OpenCode and wire harnesses, so
# a cross-harness difference is a client difference and not a prompt one.
#
# It no longer says "do not use any other tool". gpt-4.1 read that as "do not
# use any tool" and talked itself out of the call, which scored a model as
# refusing when the prompt had confused it.
CELL_PROMPT="${CELL_PROMPT:-Call the ${CLAUDE_TOOL} tool now with the argument command set to the string \"core skills list\", then reply with only the first skill id it printed. Do not use Bash and do not answer from memory.}"

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
HOST_GATEWAY="$(e2e_host_gateway_from "$GATEWAY")"

# Where the client runs. Container is the default and is the honest test: a
# clean machine with no state. Host mode exists because that stopped being
# possible — Docker Desktop lost its route into this WSL distro on every port,
# so a container can no longer reach the gateway at all. It runs the identical
# steps under a throwaway HOME, which keeps the isolation that matters (no
# inherited credentials, config, or plugins) while giving up the isolation that
# had become untestable.
CLIENT_MODE="${E2E_CLIENT:-container}"
case "$CLIENT_MODE" in
    container|host) ;;
    *) fail "E2E_CLIENT must be 'container' or 'host', not '$CLIENT_MODE'" ;;
esac
# In host mode there is no container, so the two views of the gateway collapse
# into one. Accept a docker-shaped URL and translate it rather than making the
# caller remember which mode wants which name.
[ "$CLIENT_MODE" = "host" ] && GATEWAY="$HOST_GATEWAY"

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

e2e_db_url || fail "no database_url — set SYSTEMPROMPT_DATABASE_URL"
e2e_db_reachable \
    || fail "cannot reach Postgres at the profile's database_url — \`just db-up\` first"

# `q` (one-column psql) and `gateway_up` come from the library. gateway_up is
# asked before every cell, because a sweep outlives the server it is testing:
# a peer moving the gateway to another port ended one run silently on cell
# 15, and the only evidence was a table that stopped.

# ── Client lifecycle ──────────────────────────────────────────────────────────
if [ "$CLIENT_MODE" = "container" ]; then

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

else # ── host mode ────────────────────────────────────────────────────────────

CLAUDE_BIN="${E2E_CLAUDE_BIN:-$HOME/.claude/local/claude}"
[ -x "$CLAUDE_BIN" ] || fail "no executable claude at $CLAUDE_BIN — set E2E_CLAUDE_BIN"

# The throwaway HOME, the bridge-process cleanup keyed on that HOME, the
# single-port proxy guard and the login-shell `cx` all live in the library;
# their reasoning is documented there.
e2e_host_client_home e2e-claude-home
cleanup() { e2e_host_cleanup; }
trap cleanup EXIT INT TERM
e2e_proxy_port_guard

info "client runs on this host under $CLIENT_HOME"
info "claude binary: $CLAUDE_BIN ($("$CLAUDE_BIN" --version 2>/dev/null | head -1))"

E2E_CLIENT_PATH="$(dirname "$CLAUDE_BIN")"
cx() { cx_host "$1"; }

fi

# The client installs the bridge by downloading the tarball the gateway serves,
# which is the path a real workstation takes. Rebuilding it is expensive and
# takes the shared cargo lock, so an existing tarball is reused by default.
TARBALL="$REPO_ROOT/dist/astound-bridge-linux-$(uname -m).tar.gz"
if [ "${SKIP_BRIDGE_BUILD:-0}" = "1" ] && [ -f "$TARBALL" ]; then
    info "reusing $TARBALL"
else
    say "packaging the Linux bridge tarball"
    "$REPO_ROOT/scripts/package-bridge-linux.sh"
fi

# ── Enrolment ─────────────────────────────────────────────────────────────────
say "enrolling the clean client (host: claude-code)"
# Fetch, then run — never `curl … | sh`. A pipeline's status is the last
# command's, so a 404 from curl feeds sh an empty script, sh exits 0, and the
# harness proceeds to blame the models for a client that was never installed.
# That is exactly what happened when the gateway moved to a tree with no
# generated downloads directory.
# Two very different faults land here, so separate them before blaming either.
# The host can reach a loopback-bound server that the container cannot: a
# container arrives over the Docker host gateway, not over the host's
# 127.0.0.1, so a server bound to 127.0.0.1 answers every check this script
# makes and still times out for every client it is meant to test.
if ! e2e_client_reaches_gateway; then
    fail "the client ($CLIENT_MODE mode) cannot reach $GATEWAY, although this
       host can reach $HOST_GATEWAY. The server is likely bound to 127.0.0.1;
       check with: ss -ltnp | grep ${GATEWAY##*:}
       Bind it to 0.0.0.0 and restart, or no client container will ever
       reach it."
fi
ENROL_RC=0
e2e_enrol_host claude-code "$PAT" || ENROL_RC=$?
case $ENROL_RC in
    0) ;;
    2) fail "the gateway answers but serves no install.sh at
       $GATEWAY/files/downloads/ — that tree has no generated downloads
       directory. Publish the bridge assets there before running this harness." ;;
    *) fail "the installer exited non-zero — nothing below would mean anything" ;;
esac

# ── Pre-flight: is the tool actually reachable? ───────────────────────────────
# A model that never sees the tool fails every cell for a reason that has
# nothing to do with the model. Ask the loopback MCP endpoint directly first, so
# a broken connector is reported once here instead of 32 times below.
say "pre-flight: listing tools on the loopback MCP endpoint"
TOOLS="$(e2e_mcp_tools "$MCP_SERVER")"
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

GATEWAY_LOST=0
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

    # A vanished gateway is not a model failure and must not be recorded as
    # one. Stop, and name every cell that never ran so the table cannot be
    # mistaken for a complete sweep.
    if ! gateway_up; then
        warn "the gateway at $HOST_GATEWAY stopped answering — abandoning the sweep"
        for UNRUN in ${MODEL_ARR[@]+"${MODEL_ARR[@]}"}; do
            grep -q "^$UNRUN|" "$RESULTS_FILE" 2>/dev/null && continue
            printf '%s|UNRUN|-|-|-|-|%s\n' "$UNRUN" \
                "never ran: the gateway stopped answering" >>"$RESULTS_FILE"
        done
        GATEWAY_LOST=1
        break
    fi

    printf '\n\033[1m[%d/%d] %s\033[0m\n' "$CELL_N" "$MODEL_COUNT" "$MODEL"

    # The cutoff comes from the database, not from the host clock: the two can
    # disagree, and a cutoff even slightly in the future silently hides every
    # row the cell is about to write.
    CUTOFF="$(q "select now()")"

    START="$(date +%s)"
    RAW="$LOG_DIR/${MODEL//\//_}.txt"
    set +e
    # --output-format json is for diagnostics only. The verdict below never
    # reads it: what the model says it did is the thing under test, so its own
    # account cannot be evidence. It is captured so that a cell which declines
    # to call the tool can say why in its own words.
    cx "cd \$HOME/work && timeout ${CELL_TIMEOUT} claude -p \"${CELL_PROMPT//\"/\\\"}\" \
          --model '$MODEL' \
          --permission-mode bypassPermissions \
          --output-format json \
          --allowedTools '$CLAUDE_TOOL' 2>&1" >"$RAW" 2>&1
    CLAUDE_RC=$?
    set -e
    ELAPSED=$(( $(date +%s) - START ))

    # Give the audit writes a moment to land: the gateway records the request
    # and its tool calls after the response has been streamed to the client, so
    # `claude` can exit before the rows exist.
    sleep 3

    # Proof 1 — the MCP server ran the tool.
    MCP_OK="$(e2e_count_mcp_exec "$CUTOFF" "$MCP_TOOL" "$MCP_SERVER")"

    # Proof 2 — an inference request on THIS model asked for it. The link
    # column is nullable and two different columns can carry it, so accept
    # either; requiring one would fail cells that genuinely passed.
    MODEL_ROW="$(e2e_tool_call_model_row "$CUTOFF" "$MCP_TOOL" "$MODEL")"

    REQ_COUNT="$(e2e_count_requests "$CUTOFF" "$MODEL")"

    CALL_OK=0; [ -n "$MODEL_ROW" ] && CALL_OK=1
    VERDICT=FAIL
    NOTE=""
    if [ "$MCP_OK" -gt 0 ] && [ "$CALL_OK" = "1" ]; then
        VERDICT=PASS
    elif [ "$REQ_COUNT" = "0" ]; then
        NOTE="no ai_requests row — the model never dispatched (rc=$CLAUDE_RC)"
    elif [ "$CALL_OK" = "0" ]; then
        # Quote what the model answered instead. An empty answer and a reasoned
        # refusal both fail here, and they are different faults: the first is
        # the gateway handing back nothing, the second is the model choosing.
        # Why: jq on a non-JSON transcript (a client that died mid-turn) exits
        # non-zero, and under pipefail + set -e that aborted the whole sweep at
        # the first failing cell instead of recording the failure.
        SAID="$( (jq -r '.result // empty' "$RAW" 2>/dev/null | tr '\n' ' ' | cut -c1-160) || true)"
        [ -n "$SAID" ] || SAID="(no assistant text — empty response body)"
        NOTE="dispatched but no tool call recorded on this model (rc=$CLAUDE_RC); said: $SAID"
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
UNRUN_N="$(grep -c '|UNRUN|' "$RESULTS_FILE" || true)"
TOTAL="$(wc -l <"$RESULTS_FILE" | tr -d ' ')"
echo
info "$PASSED/$TOTAL models drove a proven MCP tool call"
[ "$UNRUN_N" = "0" ] || warn "$UNRUN_N cells never ran — this is not a complete sweep"
info "transcripts: $LOG_DIR (copy them out before this script exits)"

# An abandoned sweep is a failed run even if every cell that ran passed.
[ "$GATEWAY_LOST" = "0" ] && [ "$PASSED" = "$TOTAL" ]
