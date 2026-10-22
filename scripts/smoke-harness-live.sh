#!/usr/bin/env bash
# Opt-in live smoke for the cross-harness matrix (tests/integration/harness):
# one tool-call round trip per provider through a running gateway, in the
# exact shape that broke production request ba6f9208 — an OpenCode history
# with two parallel tool calls answered by two separate role:"tool" messages.
# Every model must answer 200; a strict upstream that sees the turns unmerged
# answers 400 "tool_use ids were found without tool_result blocks".
#
# Never part of CI or `just test`: it spends real tokens, and a personal
# provider (opencode-go*) needs the PAT owner's own key saved on the instance.
#
#   GATEWAY_URL   gateway base (default http://localhost:8080)
#   GATEWAY_PAT   a personal access token for that instance (required)
#   MODELS        space-separated ids (default: one per catalogue provider)
#
# Portable across macOS and Linux: curl, sed and bash only.
set -euo pipefail

url="${GATEWAY_URL:-http://localhost:8080}"
pat="${GATEWAY_PAT:?set GATEWAY_PAT to a personal access token for $url}"
models="${MODELS:-claude-haiku-5-5 deepseek-flash go-kimi-k3 go-minimax-m3}"
session="smoke-harness-$(date +%s)"

body() {
    cat <<JSON
{"model":"$1","stream":false,"max_tokens":256,"messages":[
 {"role":"system","content":"Answer in one short sentence."},
 {"role":"user","content":"Read a.rs and b.rs, then tell me what they define."},
 {"role":"assistant","content":"","tool_calls":[
  {"id":"call_smoke_1","type":"function","function":{"name":"read","arguments":"{\"filePath\":\"a.rs\"}"}},
  {"id":"call_smoke_2","type":"function","function":{"name":"read","arguments":"{\"filePath\":\"b.rs\"}"}}]},
 {"role":"tool","tool_call_id":"call_smoke_1","content":"fn a() {}"},
 {"role":"tool","tool_call_id":"call_smoke_2","content":"fn b() {}"}],
 "tools":[{"type":"function","function":{"name":"read","description":"Read a file",
  "parameters":{"type":"object","properties":{"filePath":{"type":"string"}},"required":["filePath"]}}}]}
JSON
}

failed=0
for model in $models; do
    out="$(mktemp)"
    status="$(curl -sS -o "$out" -w '%{http_code}' "$url/v1/chat/completions" \
        -H "authorization: Bearer $pat" -H "content-type: application/json" \
        -H "x-session-id: $session" -H "x-opencode-session: $session" \
        --data "$(body "$model")" || echo 000)"
    if [ "$status" = "200" ]; then
        echo "ok    $model"
    else
        echo "FAIL  $model -> HTTP $status: $(head -c 400 "$out")"
        failed=1
    fi
    rm -f "$out"
done
exit "$failed"
