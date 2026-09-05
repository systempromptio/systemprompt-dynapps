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
# Usage: scripts/e2e-wires.sh <PAT> [GATEWAY]
set -uo pipefail

PAT="${1:?usage: e2e-wires.sh <PAT> [GATEWAY]}"
GATEWAY="${2:-http://127.0.0.1:8080}"
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

rc=0
ok()   { printf '[OK  ] %s\n' "$*"; }
fail() { printf '[FAIL] %s\n' "$*"; rc=1; }
info() { printf '[INFO] %s\n' "$*"; }
say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

# One tool, deliberately trivial and deliberately not answerable from memory:
# the model has no way to produce a plausible reply except by calling it.
TOOL_NAME=get_weather
PROMPT='What is the weather in Paris? You must call the get_weather tool. Do not answer from memory.'

# model id -> the outbound wire it exercises, for the report only.
wire_of() {
    case "$1" in
        claude-*)     echo anthropic ;;
        gemini-*)     echo gemini ;;
        o4-mini)      echo openai-responses ;;
        gpt-oss-120b) echo "openai-chat (cerebras)" ;;
        *)            echo "openai-chat (openai)" ;;
    esac
}

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
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, max_tokens: 1024, stream: $s,
        messages: [{role: "user", content: $p}],
        tools: [{
            name: $t,
            description: "Look up the current weather for a city.",
            input_schema: {
                type: "object",
                properties: {city: {type: "string", description: "City name"}},
                required: ["city"]
            }
        }]
    }'
}

body_chat() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, max_tokens: 1024, stream: $s,
        messages: [{role: "user", content: $p}],
        tools: [{
            type: "function",
            function: {
                name: $t,
                description: "Look up the current weather for a city.",
                parameters: {
                    type: "object",
                    properties: {city: {type: "string", description: "City name"}},
                    required: ["city"]
                }
            }
        }]
    }'
}

body_responses() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg p "$PROMPT" --argjson s "$2" '{
        model: $m, stream: $s, input: $p,
        tools: [{
            type: "function",
            name: $t,
            description: "Look up the current weather for a city.",
            parameters: {
                type: "object",
                properties: {city: {type: "string", description: "City name"}},
                required: ["city"]
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
        return
    fi
    "$assert_fn" "$label" "$out"
}

MODELS=(claude-haiku-4-5 gemini-2.5-flash gpt-4.1-mini gpt-oss-120b o4-mini)

say "gateway $GATEWAY — $(( ${#MODELS[@]} * 3 * 2 )) cells"
info "models: ${MODELS[*]}"

for surface in messages chat responses; do
    say "inbound /$([ "$surface" = messages ] && echo v1/messages || { [ "$surface" = chat ] && echo v1/chat/completions || echo v1/responses; })"
    for model in "${MODELS[@]}"; do
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
    REQS=$("$CLI" infra logs request list --limit 40 2>/dev/null)
    if printf '%s' "$REQS" | grep -q .; then
        ok "infra logs request list returned rows"
        for model in "${MODELS[@]}"; do
            printf '%s' "$REQS" | grep -q -- "$model" \
                && ok "audited: $model" \
                || fail "no audit row for $model — the request dispatched but was not recorded"
        done
    else
        fail "infra logs request list returned nothing after $(( ${#MODELS[@]} * 6 )) requests"
    fi

    DBURL=$(python3 -c 'import json,sys;print(json.load(open(".systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)
    if [ -n "${DBURL:-}" ] && command -v psql >/dev/null 2>&1; then
        # Scoped to this run. An all-time count would pass on historical rows
        # even if every cell in this run recorded nothing, which is exactly the
        # kind of assertion that goes green while the feature is broken.
        N=$(psql "$DBURL" -tAc "select count(*) from ai_request_tool_calls \
            where tool_name = '$TOOL_NAME' and created_at > now() - interval '30 minutes'" 2>/dev/null)
        EXPECT=$(( ${#MODELS[@]} * 6 ))
        if [ -n "$N" ] && [ "$N" -ge "$EXPECT" ]; then
            ok "ai_request_tool_calls recorded $N '$TOOL_NAME' calls in the last 30m (expected >= $EXPECT)"
        else
            fail "ai_request_tool_calls holds ${N:-0} recent '$TOOL_NAME' calls, expected >= $EXPECT — tool turns dispatched but not audited"
        fi
    else
        info "psql or database_url unavailable — skipping the ai_request_tool_calls assertion"
    fi
fi

say "result"
if [ $rc -eq 0 ]; then
    echo "ALL WIRE MATRIX CELLS PASSED"
else
    echo "WIRE MATRIX FAILURES ABOVE"
fi
exit $rc
