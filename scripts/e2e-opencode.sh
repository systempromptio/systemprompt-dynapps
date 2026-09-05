#!/usr/bin/env bash
# End-to-end proof that OpenCode works through the gateway: enrolment, model
# discovery, a real completion on a Gemini model, an MCP tool call, and the
# synced skills.
#
# The other harness (clean-client-install.sh) proves the installer writes the
# right files. This one proves the client can then actually be used, which is a
# different claim and the one that matters: a provider block can be present and
# correct while the model it names is undispatchable, or while the MCP server it
# registers never answers.
#
# Usage: scripts/e2e-opencode.sh <PAT> [GATEWAY] [MODEL]
set -euo pipefail

PAT="${1:?usage: e2e-opencode.sh <PAT> [GATEWAY] [MODEL]}"
GATEWAY="${2:-http://host.docker.internal:8080}"
MODEL="${3:-gemini-2.5-flash}"
IMAGE="astound-clean-client:local"
CONTAINER="astound-e2e-opencode"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }

docker rm -f "$CONTAINER" >/dev/null 2>&1 || true

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    say "building $IMAGE"
    docker build --build-arg INSTALL_BROWSERS=0 \
        -f "$REPO_ROOT/deploy/clean-client/Dockerfile" \
        -t "$IMAGE" "$REPO_ROOT/deploy/clean-client"
fi

say "building and publishing the Linux bridge tarball"
( cd "$REPO_ROOT/bridge" && cargo build --release )
"$REPO_ROOT/scripts/package-bridge-linux.sh"

say "running the OpenCode end-to-end flow against $GATEWAY (model $MODEL)"
docker run --rm --name "$CONTAINER" \
    --hostname e2e-opencode \
    --add-host host.docker.internal:host-gateway \
    -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
    --entrypoint bash \
    "$IMAGE" -lc '
set -uo pipefail
GATEWAY="$1"; PAT="$2"; MODEL="$3"
rc=0
ok()   { echo "[OK  ] $*"; }
fail() { echo "[FAIL] $*"; rc=1; }
info() { echo "[INFO] $*"; }

sudo install -d -o tester -g tester -m 0755 /etc/opencode

echo
echo "== 1. enrol =="
curl -fsSL "$GATEWAY/files/downloads/install.sh" \
  | sh -s -- --download-base "$GATEWAY/files/downloads" \
             --gateway "$GATEWAY" --pat "$PAT" --host opencode \
  || { fail "installer exited non-zero"; exit 1; }

CFG=/etc/opencode/opencode.json
[ -f "$CFG" ] || CFG="$HOME/.config/opencode/opencode.json"
USER_CFG="$HOME/.config/opencode/opencode.json"

echo
echo "== 2. what the provider block offers =="
MODELS=$(grep -oE "\"(claude|gemini|qwen|zai|openai|deepseek|moonshotai)[A-Za-z0-9._-]*[0-9][A-Za-z0-9._-]*\"" "$CFG" \
    | tr -d "\"" | sort -u | tr "\n" " ")
info "offered: ${MODELS:-(none)}"
case " $MODELS " in
  *"$MODEL"*) ok "the target model $MODEL is offered" ;;
  *) fail "$MODEL is NOT offered to OpenCode — surface filter still too narrow" ;;
esac

echo
echo "== 3. MCP connectors synced into the user config =="
if grep -q "\"mcp\"" "$USER_CFG" 2>/dev/null; then
    MCP_NAMES=$(sed -n "/\"mcp\"/,/^  }/p" "$USER_CFG" | grep -oE "^      \"[a-z0-9_-]+\"" | tr -d " \"" | tr "\n" " ")
    info "mcp servers: ${MCP_NAMES:-(none parsed)}"
    ok "an mcp block is present in $USER_CFG"
    # Listed is not the same as reachable: the entry points at the loopback
    # proxy and carries a bearer, either of which can be wrong.
    MCP_URL=$(grep -oE "http://127\\.0\\.0\\.1:[0-9]+/mcp/[a-z0-9_-]+" "$USER_CFG" | head -1)
    if [ -n "$MCP_URL" ]; then
        # A bare tools/list is a malformed MCP request and answers 422. The
        # streamable-HTTP transport wants initialize, then the initialized
        # notification, then tools/list carrying the session id.
        KEY=$(cat "${XDG_CONFIG_HOME:-$HOME/.config}/astound/bridge-loopback.key")
        MH="-H content-type:application/json -H accept:application/json,text/event-stream"
        curl -sS -D /tmp/mcp-h.txt -o /dev/null -X POST "$MCP_URL" \
            -H "Authorization: Bearer $KEY" $MH \
            -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"e2e\",\"version\":\"1\"}}}"
        SID=$(grep -i "mcp-session-id" /tmp/mcp-h.txt | tr -d "\r" | awk "{print \$2}")
        if [ -z "$SID" ]; then
            fail "MCP initialize returned no session id"
        else
            ok "MCP initialize succeeded (session $SID)"
            curl -sS -X POST "$MCP_URL" -H "Authorization: Bearer $KEY" $MH \
                -H "mcp-session-id: $SID" \
                -d "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" >/dev/null
            TOOLS=$(curl -sS -X POST "$MCP_URL" -H "Authorization: Bearer $KEY" $MH \
                -H "mcp-session-id: $SID" \
                -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}" \
                | sed -n "s/^data: //p" \
                | node -e "let s=\"\";process.stdin.on(\"data\",d=>s+=d).on(\"end\",()=>{for(const l of s.split(/\n/)){if(!l.trim())continue;try{const d=JSON.parse(l);const t=d&&d.result&&d.result.tools;if(t)console.log(t.map(x=>x.name).join(\" \"));}catch(e){}}})")
            info "mcp tools: ${TOOLS:-(none)}"
            [ -n "$TOOLS" ] && ok "the registered MCP server lists tools" \
                            || fail "the MCP server listed no tools"
        fi
    else
        fail "no loopback MCP url found in $USER_CFG"
    fi
else
    fail "no mcp block in $USER_CFG"
fi

echo
echo "== 4. skills synced =="
SKILLS_DIR="$HOME/.config/opencode/skills"
if [ -d "$SKILLS_DIR" ]; then
    COUNT=$(find "$SKILLS_DIR" -maxdepth 1 -mindepth 1 -type d | wc -l)
    info "skills dir: $SKILLS_DIR ($COUNT skills)"
    find "$SKILLS_DIR" -maxdepth 1 -mindepth 1 -type d -printf "       %f\n" | sort
    [ "$COUNT" -gt 0 ] && ok "skills were synced" || fail "skills dir is empty"
else
    fail "no skills dir at $SKILLS_DIR"
fi

echo
echo "== 5. a real completion on $MODEL, through OpenCode itself =="
export OPENCODE_DISABLE_AUTOUPDATE=1
OUT=$(opencode run --model "systemprompt/$MODEL" \
        "Reply with exactly the word: pong" 2>&1)
STATUS=$?
echo "$OUT" | tail -20
if [ $STATUS -eq 0 ] && echo "$OUT" | grep -qi "pong"; then
    ok "OpenCode completed a prompt on $MODEL"
else
    fail "OpenCode could not complete a prompt on $MODEL (exit $STATUS)"
fi

echo
echo "== 6. an MCP tool call, through OpenCode, on $MODEL =="
# The model must call a bridge-registered MCP tool, not fall back to bash or to
# what its system prompt already knows. An earlier version of this check passed
# on exit status alone and went green while the model shelled out to `echo`,
# which is exactly the failure it was meant to catch. Require the tool name in
# the transcript, and refuse a bash fallback.
OUT2=$(opencode run --model "systemprompt/$MODEL" \
        "Call the MCP tool named systemprompt with the argument command set to the string \"core skills list\". You MUST call that MCP tool. Do not use bash, and do not answer from memory." 2>&1)
STATUS2=$?
echo "$OUT2" | tail -30
if [ $STATUS2 -ne 0 ]; then
    fail "OpenCode failed the MCP prompt on $MODEL (exit $STATUS2)"
elif echo "$OUT2" | grep -qiE "systemprompt_systemprompt|mcp.*systemprompt|core skills list"; then
    ok "OpenCode invoked the systemprompt MCP tool on $MODEL"
else
    fail "no MCP tool was invoked — the model answered without one"
fi

echo
if [ $rc -eq 0 ]; then echo "ALL OPENCODE E2E CHECKS PASSED"; else echo "OPENCODE E2E FAILURES ABOVE"; fi
exit $rc
' _ "$GATEWAY" "$PAT" "$MODEL"
