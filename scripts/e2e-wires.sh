#!/usr/bin/env bash
# The live wire matrix: every inbound surface the gateway serves, crossed with
# every outbound wire it can dispatch to, each proving a tool call survives the
# translation AND that the terminal reason declares it.
#
# The second assertion is the whole point. Gemini reports finishReason: STOP even
# when the candidate it returns is a functionCall, so the gateway used to render
# finish_reason: "stop" beside a fully-formed tool_calls array. Every
# OpenAI-contract client read "stop", ended the turn, and discarded the call --
# all tool use on Gemini through the OpenAI surface was dead, MCP included. A
# harness that only checked "is there a tool call in the body" went green
# throughout. So each cell here asserts both, and a cell that produces a tool
# call with the wrong terminal reason is a FAIL, not a warning.
#
# Core's offline matrix (crates/tests/integration/api/gateway_pipeline.rs) covers
# the same grid against wiremock and runs in CI with no credentials. This script
# is the other half of that claim: it proves the offline matrix is not lying
# about what real upstreams actually send.
#
# Usage: scripts/e2e-wires.sh <PAT> [GATEWAY] [--list]
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

GATEWAY="${2:-http://127.0.0.1:8080}"

# The matrix is derived from the live catalogue, not hardcoded. /v1/models is
# unauthenticated and Anthropic-shaped, so a model added to the gateway is
# covered here the same day and a stale list cannot silently stop covering the
# fleet.
if [ -n "${MODELS:-}" ]; then
    IFS=',' read -r -a MODEL_LIST <<< "$MODELS"
else
    # shellcheck disable=SC2207
    MODEL_LIST=($(curl -sS --max-time 15 "$GATEWAY/v1/models" 2>/dev/null | jq -r '.data[]?.id' 2>/dev/null))
fi
if [ "${#MODEL_LIST[@]}" -eq 0 ]; then
    echo "[FAIL] $GATEWAY/v1/models offered no models — is the server running?" >&2
    exit 1
fi
if [ "$LIST_ONLY" -eq 1 ]; then
    printf '%s\n' "${MODEL_LIST[@]}"
    exit 0
fi

PAT="${1:?usage: e2e-wires.sh <PAT> [GATEWAY] [--list]}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# A PAT alone is not enough: /v1/messages wants a gateway session too, and an
# arbitrary x-session-id is rejected with "unknown or revoked session" rather
# than being created on demand. Mint a real one up front so every cell below
# shares it and the audit rows land under a single session.
SESSION_ID=$(curl -sS -X POST "$GATEWAY/api/public/gateway/sessions" \
    -H "Authorization: Bearer $PAT" -H 'content-type: application/json' \
    -d '{}' | sed -n 's/.*"session_id":"\([^"]*\)".*/\1/p')
[ -n "$SESSION_ID" ] || { echo "[FAIL] could not mint a gateway session" >&2; exit 1; }

# Opened at the database's own clock, never the host's: the two can drift, and a
# window that starts late loses rows that were recorded correctly. Epoch seconds,
# not a timestamp literal -- a rendered timestamp contains a space, and trimming
# whitespace out of a scalar result silently corrupts it into a value every
# comparison rejects.
RUN_START=""
rc=0
fails=0
ok()   { printf '[OK  ] %s\n' "$*"; }
fail() { printf '[FAIL] %s\n' "$*"; rc=1; fails=$(( fails + 1 )); }
info() { printf '[INFO] %s\n' "$*"; }
say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

# The real tool, not a synthetic one. `systemprompt` is the single tool the
# systemprompt MCP server registers, and its schema below is that server's
# schema verbatim -- so a green cell proves the shape a client actually sends
# survives the translation, not the shape a harness invented. It is also
# unanswerable from memory: the model cannot guess a skill list.
TOOL_NAME=systemprompt
TOOL_DESC="Run a systemprompt CLI command and return its output."
PROMPT="Use the systemprompt tool with command 'core skills list' and reply with the first skill id it printed. Do not use any other tool."

# model id -> the outbound wire it exercises, for the report only. A model the
# catalogue grows that does not match here still runs; it is just labelled
# generically, which is the right failure mode for a cosmetic function.
wire_of() {
    case "$1" in
        claude-*)                echo anthropic ;;
        gemini-*)                echo gemini ;;
        o[0-9]*|gpt-5*)          echo openai-responses ;;
        gpt-*|o[0-9]*-mini)      echo openai-chat ;;
        deepseek*|kimi*|moonshot*|glm*|qwen*|zai*|*-oss-*) echo "openai-chat (compat)" ;;
        *)                       echo openai-chat ;;
    esac
}

dbq() { [ -n "${DBURL:-}" ] && psql "$DBURL" -tAc "$1" 2>/dev/null | tr -d '[:space:]'; }

post() {
    # post <path> <body-file> <out-file>; echoes the HTTP status.
    curl -sS -o "$3" -w '%{http_code}' -X POST "$GATEWAY$1" \
        -H "Authorization: Bearer $PAT" \
        -H "x-session-id: $SESSION_ID" \
        -H 'content-type: application/json' \
        -H 'anthropic-version: 2023-06-01' \
        --max-time 120 \
        --data-binary "@$2"
}

# ---------------------------------------------------------------- request bodies

body_messages() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, max_tokens: 1024, stream: $s,
        messages: [{role: "user", content: $p}],
        tools: [{
            name: $t,
            description: $d,
            input_schema: {
                type: "object",
                properties: {command: {type: "string"}},
                required: ["command"]
            }
        }]
    }'
}

body_chat() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, max_tokens: 1024, stream: $s,
        messages: [{role: "user", content: $p}],
        tools: [{
            type: "function",
            function: {
                name: $t,
                description: $d,
                parameters: {
                    type: "object",
                    properties: {command: {type: "string"}},
                    required: ["command"]
                }
            }
        }]
    }'
}

body_responses() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, stream: $s, input: $p,
        tools: [{
            type: "function",
            name: $t,
            description: $d,
            parameters: {
                type: "object",
                properties: {command: {type: "string"}},
                required: ["command"]
            }
        }]
    }'
}

# ------------------------------------------------------------------- assertions
#
# Each checker takes the response file and prints nothing; it returns 0 only when
# BOTH a tool call is present and the terminal reason names it. It reports which
# of the two failed, because "tool call, wrong reason" is the shipped bug and
# "no tool call at all" is a different problem with a different fix.

check_pair() {
    # check_pair <label> <saw-call:0|1> <saw-reason:0|1> <reason-seen>
    local label="$1" call="$2" reason="$3" seen="$4"
    if [ "$call" -eq 1 ] && [ "$reason" -eq 1 ]; then
        ok "$label"
    elif [ "$call" -eq 1 ]; then
        fail "$label — tool call present but terminal reason was '${seen:-<none>}', not a tool-use reason (this is the Gemini STOP bug's signature)"
    elif [ "$reason" -eq 1 ]; then
        fail "$label — terminal reason claims a tool call but no call is in the body"
    else
        fail "$label — no tool call and no tool-use terminal reason (reason '${seen:-<none>}')"
    fi
}

assert_messages_buffered() {
    local f="$2" call reason seen
    call=$(jq -r --arg t "$TOOL_NAME" \
        '[.content[]? | select(.type=="tool_use" and .name==$t)] | length' "$f" 2>/dev/null)
    seen=$(jq -r '.stop_reason // empty' "$f" 2>/dev/null)
    [ "${call:-0}" -gt 0 ] && call=1 || call=0
    [ "$seen" = "tool_use" ] && reason=1 || reason=0
    check_pair "$1" "$call" "$reason" "$seen"
}

assert_chat_buffered() {
    local f="$2" call reason seen
    call=$(jq -r --arg t "$TOOL_NAME" \
        '[.choices[0].message.tool_calls[]? | select(.function.name==$t)] | length' "$f" 2>/dev/null)
    seen=$(jq -r '.choices[0].finish_reason // empty' "$f" 2>/dev/null)
    [ "${call:-0}" -gt 0 ] && call=1 || call=0
    [ "$seen" = "tool_calls" ] && reason=1 || reason=0
    check_pair "$1" "$call" "$reason" "$seen"
}

assert_responses_buffered() {
    local f="$2" call reason seen
    call=$(jq -r --arg t "$TOOL_NAME" \
        '[.output[]? | select(.type=="function_call" and .name==$t)] | length' "$f" 2>/dev/null)
    # The responses surface carries no finish_reason: a completed response whose
    # output holds a function_call IS the tool-use terminal state, and
    # incomplete_details naming a cutoff is the way it says otherwise.
    seen=$(jq -r '.status // empty' "$f" 2>/dev/null)
    [ "${call:-0}" -gt 0 ] && call=1 || call=0
    [ "$seen" = "completed" ] && reason=1 || reason=0
    check_pair "$1" "$call" "$reason" "$seen"
}

assert_messages_stream() {
    local f="$2" call=0 reason=0 seen
    grep -q "\"type\":\"tool_use\"" "$f" && call=1
    seen=$(sed -n 's/.*"stop_reason":"\([a-z_]*\)".*/\1/p' "$f" | grep -v '^null$' | tail -1)
    [ "$seen" = "tool_use" ] && reason=1
    check_pair "$1" "$call" "$reason" "$seen"
}

assert_chat_stream() {
    local f="$2" call=0 reason=0 seen
    grep -q '"tool_calls"' "$f" && call=1
    seen=$(sed -n 's/.*"finish_reason":"\([a-z_]*\)".*/\1/p' "$f" | tail -1)
    [ "$seen" = "tool_calls" ] && reason=1
    check_pair "$1" "$call" "$reason" "$seen"
}

assert_responses_stream() {
    local f="$2" call=0 reason=0 seen
    grep -q '"function_call"' "$f" && call=1
    grep -q 'response.completed' "$f" && { reason=1; seen=completed; }
    check_pair "$1" "$call" "$reason" "${seen:-}"
}

# ------------------------------------------------------------------------ driver

cell() {
    # cell <surface> <model> <stream:true|false>
    local surface="$1" model="$2" stream="$3"
    local path body_fn assert_fn mode label out status
    case "$surface" in
        messages)  path=/v1/messages;         body_fn=body_messages
                   [ "$stream" = true ] && assert_fn=assert_messages_stream  || assert_fn=assert_messages_buffered ;;
        chat)      path=/v1/chat/completions; body_fn=body_chat
                   [ "$stream" = true ] && assert_fn=assert_chat_stream      || assert_fn=assert_chat_buffered ;;
        responses) path=/v1/responses;        body_fn=body_responses
                   [ "$stream" = true ] && assert_fn=assert_responses_stream || assert_fn=assert_responses_buffered ;;
    esac
    [ "$stream" = true ] && mode=streaming || mode=buffered
    label="$surface + $(wire_of "$model") [$model] $mode"

    "$body_fn" "$model" "$stream" > "$WORK/req.json"
    out="$WORK/resp"
    status=$(post "$path" "$WORK/req.json" "$out")

    if [ "$status" != "200" ]; then
        fail "$label — HTTP $status: $(head -c 300 "$out" | tr -d '\n')"
        record "$model" "$surface" "$mode" FAIL
        return
    fi
    local before=$fails
    "$assert_fn" "$label" "$out"
    [ "$fails" -eq "$before" ] && record "$model" "$surface" "$mode" PASS \
                              || record "$model" "$surface" "$mode" FAIL
}

# Per-cell verdicts, keyed model|surface|mode, for the table at the end. The
# running log says what went wrong; the table says how much of the fleet works.
CELLS=()
record() { CELLS+=("$1|$2|$3|$4"); }
verdict_of() {
    local key="$1|$2|$3" c
    for c in ${CELLS[@]+"${CELLS[@]}"}; do
        case "$c" in "$key|"*) printf '%s' "${c##*|}"; return ;; esac
    done
    printf '%s' '-'
}

DBURL=$(python3 -c 'import json;print(json.load(open(".systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)
command -v psql >/dev/null 2>&1 || DBURL=""
RUN_START=$(dbq "select extract(epoch from now())")

say "gateway $GATEWAY — $(( ${#MODEL_LIST[@]} * 3 * 2 )) cells"
info "models (${#MODEL_LIST[@]}): ${MODEL_LIST[*]}"

for surface in messages chat responses; do
    say "inbound /$([ "$surface" = messages ] && echo v1/messages || { [ "$surface" = chat ] && echo v1/chat/completions || echo v1/responses; })"
    for model in "${MODEL_LIST[@]}"; do
        cell "$surface" "$model" false
        cell "$surface" "$model" true
    done
done

# --------------------------------------------------------------- the audit trail
#
# A cell can be green on the wire and still leave the governance spine blind, so
# the run is not finished until the requests it made are visible in the audit
# tables the whole product is sold on.

say "audit trail"
# Resolve the CLI rather than requiring it on PATH: it usually is not, and an
# assertion that silently skips itself is indistinguishable from one that passed.
CLI=""
for c in ./target/debug/systemprompt ./target/release/systemprompt "$(command -v systemprompt 2>/dev/null)"; do
    [ -x "$c" ] && { CLI="$c"; break; }
done
if [ -z "$CLI" ]; then
    fail "no systemprompt binary found (target/{debug,release} or PATH) — audit trail unverified"
else
    info "using CLI $CLI"
    "$CLI" infra logs request list --limit 5 >/dev/null 2>&1 \
        && ok "infra logs request list is answering" \
        || fail "infra logs request list failed — the operational view over the audit spine is broken"
fi

if [ -z "${DBURL:-}" ] || [ -z "$RUN_START" ]; then
    fail "psql or database_url unavailable — the audit trail could not be verified"
else
    # Every count is scoped to this run. An all-time count would pass on
    # historical rows even if every cell recorded nothing, which is exactly the
    # kind of assertion that goes green while the feature is broken.
    for model in "${MODEL_LIST[@]}"; do
        # requested_model, not model: `model` holds the resolved upstream id
        # (gpt-4.1-2025-04-14, qwen/...-maas), so matching on it fails for every
        # model whose catalogue id is an alias -- which is most of them.
        N=$(dbq "select count(*) from ai_requests
                  where coalesce(requested_model, model) = '$model'
                    and extract(epoch from created_at) > $RUN_START")
        [ "${N:-0}" -gt 0 ] && ok "audited: $model (${N} requests)" \
                            || fail "no audit row for $model — the request dispatched but was not recorded"
    done

    # One tool call per cell that produced one. Cells that failed on the wire
    # legitimately have none, so the expectation is the number that passed, not
    # the size of the matrix -- otherwise an upstream 429 would be reported a
    # second time as an audit gap it did not cause.
    EXPECT=0
    for c in ${CELLS[@]+"${CELLS[@]}"}; do
        case "$c" in *"|PASS") EXPECT=$(( EXPECT + 1 )) ;; esac
    done
    N=$(dbq "select count(*) from ai_request_tool_calls where tool_name = '$TOOL_NAME' and extract(epoch from created_at) > $RUN_START")
    if [ "${N:-0}" -ge "$EXPECT" ]; then
        ok "ai_request_tool_calls recorded $N '$TOOL_NAME' calls this run (expected >= $EXPECT, one per passing cell)"
    else
        fail "ai_request_tool_calls holds ${N:-0} '$TOOL_NAME' calls this run, expected >= $EXPECT — tool turns dispatched but not audited"
    fi
fi

say "per-model results"
printf '%-30s %-18s %-18s %-18s\n' MODEL messages chat responses
printf '%-30s %-18s %-18s %-18s\n' '---' 'buf/stream' 'buf/stream' 'buf/stream'
for model in "${MODEL_LIST[@]}"; do
    printf '%-30s %-18s %-18s %-18s\n' "$model" \
        "$(verdict_of "$model" messages buffered)/$(verdict_of "$model" messages streaming)" \
        "$(verdict_of "$model" chat buffered)/$(verdict_of "$model" chat streaming)" \
        "$(verdict_of "$model" responses buffered)/$(verdict_of "$model" responses streaming)"
done

say "result"
if [ $rc -eq 0 ]; then
    echo "ALL WIRE MATRIX CELLS PASSED"
else
    echo "WIRE MATRIX FAILURES ABOVE"
fi
exit $rc
