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
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
PROMPT="Call the systemprompt tool now with command 'core skills list', then reply with only the first skill id it printed."

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

# Reasoning models genuinely think for minutes on a trivial prompt --
# qwen.qwen3-next-thinking emitted 618 thinking deltas in its first 35 seconds --
# so a buffered cell can outlive a short client timeout and be recorded as a
# malformed stream. The fast models are unaffected by a generous ceiling.
MAX_TIME="${MAX_TIME:-600}"
# A reasoning model spends this budget thinking before it can emit the tool
# call, and 1024 left qwen.qwen3-next-thinking finishing on `length` with
# nothing to show. All three surfaces send the same ceiling so a cell that
# fails does not fail differently depending on which door it came in.
MAX_OUTPUT_TOKENS="${MAX_OUTPUT_TOKENS:-8192}"
CURL_RC=0
CURL_TIME=0

post() {
    # post <path> <body-file> <out-file>; echoes the HTTP status, sets CURL_RC
    # and CURL_TIME. Elapsed is not a nicety: a status of 000 with exit code 0
    # is undiagnosable after the fact without knowing whether it came back
    # instantly or sat for ten minutes.
    # Truncate first. curl leaves the output file untouched when it cannot
    # connect at all, so a transport failure would otherwise be reported with
    # the previous cell's response body attached -- which reads as a bizarre
    # model mix-up rather than as the server having gone away.
    : > "$3"
    local code
    local raw
    raw=$(curl -sS -o "$3" -w '%{http_code} %{time_total}' -X POST "$GATEWAY$1" \
        -H "Authorization: Bearer $PAT" \
        -H "x-session-id: $SESSION_ID" \
        -H 'content-type: application/json' \
        -H 'anthropic-version: 2023-06-01' \
        --max-time "$MAX_TIME" \
        --data-binary "@$2")
    CURL_RC=$?
    code=${raw%% *}
    CURL_TIME=${raw##* }
    [ "$code" = "$CURL_TIME" ] && CURL_TIME=0
    printf '%s' "$code"
}

# ------------------------------------------------------------------ tool_choice
#
# Every body forces the call. Without it a cell measures whether the model felt
# like using the tool -- gemini-2.5-flash-lite simply declined on two cells --
# and this matrix is about wire fidelity, not the model's whim. Forcing it also
# puts tool_choice translation itself under test on all four outbound wires
# (anthropic, gemini's toolConfig.functionCallingConfig mode ANY, openai-chat,
# openai-responses): if a wire drops the field, the cell still shows the model
# declining, and that is a real defect rather than noise.
#
# Forcing is not universal, so each cell walks a ladder from the strongest form
# to the weakest and stops at the first the upstream accepts:
#
#   named     name this exact tool          -- the strongest claim
#   any       call some tool                -- OpenAI spells this "required"
#   auto      may call a tool               -- no force at all
#
# Vertex MaaS rejects a named force outright with 400 "Forced tool calling is
# not supported for this model", so those models start at the second rung. That
# is derived from the catalogue rather than a list of model ids, so a model
# added to vertex-maas is handled without touching this script. The table
# records which rung each cell used, because a cell that had to fall back to
# `auto` proves something weaker than one that did not.

declare -A PROVIDER_OF
build_provider_map() {
    local yaml="$REPO_ROOT/services/ai/providers.yaml" line
    [ -f "$yaml" ] || return 0
    while IFS='|' read -r prov mid; do
        [ -n "$mid" ] && PROVIDER_OF["$mid"]="$prov"
    done < <(python3 - "$yaml" <<'PY' 2>/dev/null
import sys, yaml
d = yaml.safe_load(open(sys.argv[1])) or {}
for prov in d.get("providers") or []:
    pid = prov.get("id") or prov.get("name")
    for m in prov.get("models") or []:
        mid = m.get("id") or m.get("name")
        if pid and mid:
            print(f"{pid}|{mid}")
PY
)
}

provider_of() {
    if [ -n "${PROVIDER_OF[$1]:-}" ]; then
        printf '%s' "${PROVIDER_OF[$1]}"
    else
        # Fallback when the catalogue is not readable from here. Every
        # vertex-maas id in this catalogue is namespaced with a dot and no other
        # provider's is, so the id shape carries the same information.
        case "$1" in *.*) printf '%s' vertex-maas ;; *) printf '%s' unknown ;; esac
    fi
}

# ladder_for <surface> <model> -> the rungs to try, strongest first
ladder_for() {
    local rungs
    case "$1" in
        messages) rungs="named any auto" ;;
        *)        rungs="named required auto" ;;
    esac
    [ "$(provider_of "$2")" = vertex-maas ] && rungs="${rungs#named }"
    printf '%s' "$rungs"
}

# tool_choice_json <surface> <rung> -> the JSON value for that surface
tool_choice_json() {
    case "$1:$2" in
        messages:named) printf '{"type":"tool","name":"%s"}' "$TOOL_NAME" ;;
        messages:any)   printf '{"type":"any"}' ;;
        messages:auto)  printf '{"type":"auto"}' ;;
        chat:named)     printf '{"type":"function","function":{"name":"%s"}}' "$TOOL_NAME" ;;
        responses:named) printf '{"type":"function","name":"%s"}' "$TOOL_NAME" ;;
        *:required)     printf '"required"' ;;
        *:auto)         printf '"auto"' ;;
    esac
}

# A 400 that names forced tool calling is the upstream refusing the rung, not a
# defect -- step down. Anything else is a real failure and must not be retried
# into a weaker claim that then passes.
rung_rejected() {
    [ "$1" = 400 ] || return 1
    grep -qiE 'forced tool calling|tool_choice|function calling' "$2"
}

# ---------------------------------------------------------------- request bodies
#
# The tool schema is the shape Claude Code really sends, not a toy: a
# draft-2020-12 `$schema`, `additionalProperties: false`, a `format` hint, a
# `const`, a nullable union and — the one that took Gemini down on 2026-09-08 —
# a tuple-typed array (`prefixItems`) nested two levels under `items`. A wire
# that forwards any of these verbatim fails here the way it failed in
# production, so a "toolcall survives" pass means the sanitizer did its job.
# `command` stays the required field the prompt asks the model to fill.
TOOL_SCHEMA=$(jq -n '{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    type: "object",
    additionalProperties: false,
    properties: {
        command: {type: "string"},
        limit: {type: "integer", format: "uint32", minimum: 1},
        mode: {const: "fast", "$comment": "internal"},
        query: {
            type: "object",
            properties: {
                where: {
                    type: "array",
                    items: {
                        type: "array",
                        prefixItems: [{type: "string"}, {type: "string", enum: ["eq", "gt"]}, {}],
                        additionalItems: false
                    }
                },
                cursor: {type: ["string", "null"]}
            }
        }
    },
    required: ["command"]
}')


body_messages() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson schema "$TOOL_SCHEMA" \
          --argjson s "$2" --argjson tc "$3" --argjson mo "$MAX_OUTPUT_TOKENS" '{
        model: $m, max_tokens: $mo, stream: $s,
        messages: [{role: "user", content: $p}],
        tool_choice: $tc,
        tools: [{
            name: $t,
            description: $d,
            input_schema: $schema
        }]
    }'
}

body_chat() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson schema "$TOOL_SCHEMA" \
          --argjson s "$2" --argjson tc "$3" --argjson mo "$MAX_OUTPUT_TOKENS" '{
        model: $m, max_tokens: $mo, stream: $s,
        messages: [{role: "user", content: $p}],
        tool_choice: $tc,
        tools: [{
            type: "function",
            function: {
                name: $t,
                description: $d,
                parameters: $schema
            }
        }]
    }'
}

body_responses() {
    jq -n --arg m "$1" --arg t "$TOOL_NAME" --arg d "$TOOL_DESC" --arg p "$PROMPT" --argjson schema "$TOOL_SCHEMA" \
          --argjson s "$2" --argjson tc "$3" --argjson mo "$MAX_OUTPUT_TOKENS" '{
        model: $m, stream: $s, input: $p, max_output_tokens: $mo,
        tool_choice: $tc,
        tools: [{
            type: "function",
            name: $t,
            description: $d,
            parameters: $schema
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
    out="$WORK/resp"

    local rung used=""
    for rung in $(ladder_for "$surface" "$model"); do
        "$body_fn" "$model" "$stream" "$(tool_choice_json "$surface" "$rung")" > "$WORK/req.json"
        status=$(post "$path" "$WORK/req.json" "$out")
        used="$rung"
        rung_rejected "$status" "$out" || break
        info "$label — upstream refused tool_choice '$rung', stepping down"
    done

    if [ "$status" != "200" ]; then
        # A client-side timeout is not an upstream error and must not be read as
        # one: there is no response to quote, and the fix is a longer ceiling or
        # a faster model, not a gateway change.
        local diag="curl exit $CURL_RC after ${CURL_TIME}s"
        if [ "$CURL_RC" -eq 28 ]; then
            fail "$label [$used] — client-side timeout at the ${MAX_TIME}s ceiling ($diag); the upstream never finished and returned no error"
        elif [ "$CURL_RC" -ne 0 ]; then
            fail "$label [$used] — transport failure before any response ($diag)"
        else
            fail "$label [$used] — HTTP $status ($diag): $(head -c 300 "$out" | tr -d '\n')"
        fi
        record "$model" "$surface" "$mode" "FAIL" "$used"
        return
    fi
    local before=$fails
    "$assert_fn" "$label [$used]" "$out"
    [ "$fails" -eq "$before" ] && record "$model" "$surface" "$mode" PASS "$used" \
                              || record "$model" "$surface" "$mode" FAIL "$used"
}

# Per-cell verdicts, keyed model|surface|mode, for the table at the end. The
# running log says what went wrong; the table says how much of the fleet works.
CELLS=()
record() { CELLS+=("$1|$2|$3|$4|${5:-}"); }
# Verdict plus the tool_choice rung the cell settled on, abbreviated: a cell that
# had to fall back proves something weaker than one that did not.
verdict_of() {
    local key="$1|$2|$3" c v r
    for c in ${CELLS[@]+"${CELLS[@]}"}; do
        case "$c" in "$key|"*)
            v=$(printf '%s' "$c" | cut -d'|' -f4)
            r=$(printf '%s' "$c" | cut -d'|' -f5)
            case "$r" in named) r=n ;; any|required) r=r ;; auto) r=a ;; *) r='?' ;; esac
            printf '%s' "$v($r)"; return ;;
        esac
    done
    printf '%s' '-'
}

# The gateway under test does not always live in this script's clone -- a
# verification server runs from its own checkout against its own Postgres. Both
# overrides exist so the server-side assertions can be pointed at the clone that
# owns the running server, rather than silently reading this one's database and
# reporting a gap that is really two different databases.
DBURL="${SYSTEMPROMPT_DATABASE_URL:-$(python3 -c 'import json;print(json.load(open("'"$REPO_ROOT"'/.systemprompt/profiles/local/secrets.json"))["database_url"])' 2>/dev/null)}"
command -v psql >/dev/null 2>&1 || DBURL=""
RUN_START=$(dbq "select extract(epoch from now())")
build_provider_map

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
# Anchored to this script's own clone, never to the caller's cwd. Resolving
# ./target/... against cwd pairs whichever binary happens to be underfoot with
# the profile of a different checkout, which is how this assertion failed while
# every direct row count beside it was green.
CLI=""
for c in "${SYSTEMPROMPT_CLI:-}" "$REPO_ROOT/target/debug/systemprompt" "$REPO_ROOT/target/release/systemprompt" "$(command -v systemprompt 2>/dev/null)"; do
    [ -n "$c" ] && [ -x "$c" ] && { CLI="$c"; break; }
done
if [ -z "$CLI" ]; then
    fail "no systemprompt binary found (target/{debug,release} or PATH) — audit trail unverified"
else
    info "using CLI $CLI"
    # Keep the CLI's own words. "it failed" is not a diagnosis, and the reason is
    # usually a profile or database the message could simply have named.
    if CLI_ERR=$("$CLI" infra logs request list --limit 5 2>&1 >/dev/null); then
        ok "infra logs request list is answering"
    else
        fail "infra logs request list failed — the operational view over the audit spine is broken: $(printf '%s' "$CLI_ERR" | tr '\n' ' ' | head -c 300)"
    fi
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
        case "$c" in *"|PASS|"*) EXPECT=$(( EXPECT + 1 )) ;; esac
    done
    N=$(dbq "select count(*) from ai_request_tool_calls where tool_name = '$TOOL_NAME' and extract(epoch from created_at) > $RUN_START")
    if [ "${N:-0}" -ge "$EXPECT" ]; then
        ok "ai_request_tool_calls recorded $N '$TOOL_NAME' calls this run (expected >= $EXPECT, one per passing cell)"
    else
        fail "ai_request_tool_calls holds ${N:-0} '$TOOL_NAME' calls this run, expected >= $EXPECT — tool turns dispatched but not audited"
    fi
fi

say "per-model results"
info "tool_choice rung: (n) named this tool  (r) required/any  (a) auto — a fallback proves less"
printf '%-30s %-22s %-22s %-22s\n' MODEL messages chat responses
printf '%-30s %-22s %-22s %-22s\n' '---' 'buf/stream' 'buf/stream' 'buf/stream'
for model in "${MODEL_LIST[@]}"; do
    printf '%-30s %-22s %-22s %-22s\n' "$model" \
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
