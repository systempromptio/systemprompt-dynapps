#!/usr/bin/env bash
# One-shot end-to-end verification of the Linux onboarding flow.
#
#     scripts/clean-client-install.sh sp-live-...
#
# Takes a PAT and proves the whole claim: a machine that has never seen Dynapps
# ends up running Claude Code against the gateway with the managed MCP servers
# already authenticated, and OpenCode enrolled with a provider block, an API
# key, and a loopback proxy that answers it. Everything between those two points is automated —
# repackage the tarball, boot a config-free container, run the published
# installer exactly as a user would, then assert the result.
#
# Get the PAT from the running server first (both steps are manual by design —
# the code is a credential and is deliberately not automatable):
#
#     systemprompt admin bridge issue-code --user-id <email|uuid|name>
#     curl -sS -X POST "$GATEWAY/v1/auth/bridge/session-pat" \
#       -H 'content-type: application/json' \
#       -d '{"code":"<CODE>","device_name":"clean-client"}' | jq -r .pat
#
# The code is one-shot and expires in minutes; the PAT it returns is durable.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="dynapps-clean-client:local"
CONTAINER="dynapps-clean-client-install"

# The gateway is reached from inside the container, so it cannot be localhost.
GATEWAY="${GATEWAY:-http://host.docker.internal:8080}"
SKIP_PACKAGE="${SKIP_PACKAGE:-0}"

say()  { printf '\033[1m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[33mwarn:\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

PAT="${1:-${DYNAPPS_BRIDGE_PAT:-}}"
case "$PAT" in
    sp-live-*) ;;
    "") fail "usage: $0 sp-live-...
       Mint one with 'systemprompt admin bridge issue-code --user-id <you>',
       then POST the code to \$GATEWAY/v1/auth/bridge/session-pat (see header)." ;;
    *)  fail "that does not look like a PAT (expected an sp-live-... value)." ;;
esac

command -v docker >/dev/null 2>&1 || fail "docker is required"

# ── Repackage ─────────────────────────────────────────────────────────────────
# The installer downloads the tarball the gateway serves, so a stale artefact
# silently tests yesterday's binary. package-bridge-linux.sh publishes into
# storage/files/downloads, which the gateway serves at /files/downloads.
if [ "$SKIP_PACKAGE" = "1" ]; then
    warn "SKIP_PACKAGE=1 — testing whatever tarball is already published."
else
    say "building and publishing the Linux bridge tarball"
    ( cd "$REPO_ROOT/bridge" && cargo build --release ) \
        || fail "bridge release build failed"
    "$REPO_ROOT/scripts/package-bridge-linux.sh" \
        || fail "packaging failed"
fi

# ── Gateway reachability ──────────────────────────────────────────────────────
# Checked from the host against the equivalent loopback URL: a failure here is
# almost always "the server is not running", and finding that out before the
# container starts saves a confusing download error inside it.
PROBE="${GATEWAY/host.docker.internal/localhost}"
if ! curl -fsS -o /dev/null --max-time 5 "$PROBE/files/downloads/install.sh"; then
    fail "the gateway is not serving $PROBE/files/downloads/install.sh
       Start it with 'just start' and make sure 'just publish' has run."
fi

# ── Image ─────────────────────────────────────────────────────────────────────
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    say "building the clean-client image"
    docker build -f "$REPO_ROOT/deploy/clean-client/Dockerfile" \
        -t "$IMAGE" "$REPO_ROOT/deploy/clean-client" \
        || fail "image build failed"
fi

docker rm -f "$CONTAINER" >/dev/null 2>&1 || true

# ── Run ───────────────────────────────────────────────────────────────────────
# No repo mount, no bridge bind-mount, no env file: the container must get the
# binary from the gateway exactly as a real user does. The PAT arrives as an
# argument rather than an env var so it is not inherited by anything the
# session later spawns.
say "running the published installer in a config-free container"
docker run --rm --name "$CONTAINER" \
    --hostname clean-client \
    --add-host host.docker.internal:host-gateway \
    -e DYNAPPS_BRIDGE_GATEWAY_URL="$GATEWAY" \
    -v "$REPO_ROOT/deploy/clean-client/gateway-forward.sh:/usr/local/bin/gateway-forward:ro" \
    --entrypoint bash \
    "$IMAGE" -lc '
set -euo pipefail
GATEWAY="$1"; PAT="$2"

# ── Feedback plane ────────────────────────────────────────────────────────────
# `install --pat` runs `bridge login`, and login now self-enrols a device
# credential — but only over https or loopback http, because the credential
# is a bearer token. host.docker.internal is neither, so relay the port through
# 127.0.0.1 and give the installer a localhost URL; the host gateway still
# answers. Anything other than that alias is used as given. The download goes
# through the relay too, so a broken relay fails here rather than at the
# device assertion below.
case "$GATEWAY" in
    http://host.docker.internal:*|http://host.docker.internal|http://host.docker.internal/*)
        FWD_PORT="${GATEWAY#http://host.docker.internal}"
        FWD_PORT="${FWD_PORT#:}"; FWD_PORT="${FWD_PORT%%/*}"
        gateway-forward "${FWD_PORT:-80}" host.docker.internal
        GATEWAY="http://localhost:${FWD_PORT:-80}"
        ;;
esac

# Stand in for MDM: delegate the enterprise policy directory to the user who
# runs the installer. Without this the bridge cannot write managed-mcp.json and
# correctly degrades to per-plugin provisioning — which is a valid outcome, but
# not the one this run is trying to prove.
sudo install -d -o tester -g tester -m 0755 /etc/claude-code
# Same stand-in for the OpenCode admin tier. Left unwritable the enrolment still
# succeeds by falling back to the user tier, which is a valid outcome but not
# the governed one this run is trying to prove.
sudo install -d -o tester -g tester -m 0755 /etc/opencode

curl -fsSL "$GATEWAY/files/downloads/install.sh" \
  | sh -s -- --download-base "$GATEWAY/files/downloads" \
             --gateway "$GATEWAY" \
             --pat "$PAT" \
             --host claude-code,opencode

echo
echo "── verification ──────────────────────────────────────────────────────"

rc=0

# The bridge deliberately does NOT write managed-mcp.json, and removes any it
# wrote before. Exclusive mode makes plugin and user servers vanish, including
# the built-in workspace server, so the sandbox bash tool and every skill that
# needs it stop working. Servers reach the CLI through per-plugin .mcp.json
# files instead. See core bin/bridge/src/install/managed_mcp/mod.rs, whose
# clear_policy() runs on every enrolment. Assert that contract, not its inverse.
if [ -f /etc/claude-code/managed-mcp.json ]; then
    echo "[FAIL] managed-mcp.json present — exclusive mode breaks plugin servers"
    cat /etc/claude-code/managed-mcp.json
    rc=1
else
    echo "[OK  ] no managed-mcp.json — plugin servers stay reachable"
fi

if grep -sq "allowManagedMcpServersOnly" /etc/claude-code/managed-settings.json; then
    echo "[FAIL] managed-settings.json carries the exclusive-mode allowlist"; rc=1
else
    echo "[OK  ] managed-settings.json carries no exclusive-mode allowlist"
fi

# claude mcp list reports what the CLI will actually load, after policy
# evaluation. The gateway-backed server must be among them.
echo
echo "claude mcp list:"
if claude mcp list > /tmp/mcp-list.txt 2>&1; then
    cat /tmp/mcp-list.txt
    if grep -q "systemprompt" /tmp/mcp-list.txt; then
        echo "[OK  ] the gateway MCP server is loaded"
    else
        echo "[FAIL] no systemprompt MCP server in the loaded set"; rc=1
    fi
else
    cat /tmp/mcp-list.txt; rc=1
fi

# Governance lives at the gateway, not in a local policy file. A user adding
# their own server is expected to keep working; asserting a refusal here would
# re-assert the exclusive mode the bridge deliberately removes.
echo
if claude mcp add --transport http probe https://example.com/mcp > /dev/null 2>&1; then
    echo "[OK  ] a user can still add their own MCP server"
    claude mcp remove probe > /dev/null 2>&1 || true
else
    echo "[FAIL] user-added MCP servers are refused — exclusive mode is on"; rc=1
fi

echo
echo "claude plugin list:"
claude plugin list || true

# ── Feedback plane ────────────────────────────────────────────────────────────
# Three things must be true before anything a session reports can be attributed
# to a device: the bridge enrolled one at login, every Claude Code hook names
# the host it runs in, and the OpenCode plugin stamps its chat requests. None of
# them is a step the user runs; all three fall out of the installer.
echo
echo "── feedback plane ────────────────────────────────────────────────────"

# Written by `bridge login` (which `install --pat` runs) under the bridge
# working dir — ~/.local/state/dynapps-bridge on Linux, XDG_STATE_HOME aside.
DEVICE_JSON="${XDG_STATE_HOME:-$HOME/.local/state}/dynapps-bridge/metadata/feedback/device.json"
if [ -f "$DEVICE_JSON" ]; then
    echo "[OK  ] device credential enrolled at login ($DEVICE_JSON)"
    [ "$(stat -c %a "$DEVICE_JSON")" = "600" ] \
        || { echo "[FAIL] $DEVICE_JSON is not 0600"; rc=1; }
else
    echo "[FAIL] no device credential at $DEVICE_JSON — login did not self-enrol (gateway must be https or loopback)"; rc=1
fi

# The plugin mirror Claude Code actually loads hooks from. Every http hook in
# it must carry x-systemprompt-host, or the proxy cannot tell which client a
# hook event came from. The glob is what Claude Code lays out:
# cache/<marketplace>/<plugin>/current/hooks/hooks.json.
HOOKS_SEEN=0; HOOKS_BAD=0
for hooks in "$HOME"/.claude/plugins/cache/*/*/current/hooks/hooks.json; do
    [ -f "$hooks" ] || continue
    HOOKS_SEEN=$((HOOKS_SEEN + 1))
    if grep -q "x-systemprompt-host" "$hooks"; then
        echo "[OK  ] $hooks names its host"
    else
        echo "[FAIL] $hooks carries no x-systemprompt-host header"; HOOKS_BAD=1
    fi
done
if [ "$HOOKS_SEEN" = "0" ]; then
    echo "[FAIL] no cached Claude Code hooks.json under ~/.claude/plugins/cache — no plugin with hooks was mirrored"; rc=1
elif [ "$HOOKS_BAD" = "1" ]; then
    rc=1
fi

# ── OpenCode enrolment ────────────────────────────────────────────────────────
# The regression this guards: `sync` writes the OpenCode MCP connectors into the
# user config, and before `--host` nothing ever wrote the provider block or the
# API key. The client then reached the proxy with no credential and every
# prompt came back "403 no loopback credential presented" — with the MCP
# servers visibly present, which is why it read as a gateway fault.
echo
echo "── opencode ──────────────────────────────────────────────────────────"

OC_MANAGED=/etc/opencode/opencode.json
OC_USER="$HOME/.config/opencode/opencode.json"
OC_AUTH="$HOME/.local/share/opencode/auth.json"

# "Admin tier" here is about PATH SELECTION, not enforcement. The file is mode
# 0644 and this script chowns /etc/opencode to the unprivileged test user, so
# the write succeeds through the ordinary unprivileged path. What this proves is
# that the installer chose the admin tier over the user-tier fallback in core
# integration/opencode/install/mod.rs — nothing more. There is no allowlist and
# no lockdown for OpenCode on any platform; governance is at the gateway.
OC_FOUND=""
if grep -sq systemprompt "$OC_MANAGED"; then
    echo "[OK  ] provider block in the admin tier ($OC_MANAGED) — path selection, not enforcement"
    OC_FOUND="$OC_MANAGED"
elif grep -sq systemprompt "$OC_USER"; then
    echo "[WARN] provider block fell back to the user tier ($OC_USER)"
    OC_FOUND="$OC_USER"
else
    echo "[FAIL] no systemprompt provider block in either tier — OpenCode has no route"; rc=1
fi

# Placement is not enough: the block must offer models the gateway can actually
# serve. A host that filters the catalogue by each provider native wire family
# hides working models, because surface governs advertisement only and the
# gateway translates any inbound wire to any provider wire. Gemini and Claude
# are the canaries — both are served over the OpenAI wire OpenCode speaks.
if [ -n "$OC_FOUND" ]; then
    # The trailing digit requirement keeps the npm package name
    # (@ai-sdk/openai-compatible) out of what is meant to be a model list.
    OC_MODELS=$(grep -oE "\"(claude|gemini|qwen|zai|openai|deepseek|moonshotai)[A-Za-z0-9._-]*[0-9][A-Za-z0-9._-]*\"" "$OC_FOUND" \
        | tr -d "\"" | sort -u | tr "\n" " ")
    echo "[INFO] models offered to OpenCode: ${OC_MODELS:-(none)}"
    case " $OC_MODELS " in
        *gemini-*) echo "[OK  ] gemini models are offered to OpenCode" ;;
        *) echo "[FAIL] no gemini-* model offered — the host surface filter is too narrow"; rc=1 ;;
    esac
    case " $OC_MODELS " in
        *claude-*) echo "[OK  ] claude models are offered to OpenCode" ;;
        *) echo "[FAIL] no claude-* model offered — the host surface filter is too narrow"; rc=1 ;;
    esac
fi

# Skill use from OpenCode reaches the ledger through a bridge-written plugin
# that carries a per-plugin hook token. The raw loopback key must never be in
# it: the proxy accepts only the scoped token on the hook route.
OC_PLUGIN="$HOME/.config/opencode/plugin/systemprompt-hooks.js"
if [ -f "$OC_PLUGIN" ]; then
    if grep -q "/api/public/hooks/track?plugin_id=" "$OC_PLUGIN"; then
        echo "[OK  ] OpenCode hook plugin points at the loopback track route"
    else
        echo "[FAIL] OpenCode hook plugin has no track route"; rc=1
    fi
    # The chat.headers hook is what links each inference request to the
    # OpenCode session, so a conversation and the skills it invoked share one
    # attribution. Without it the proxy sees anonymous chat requests.
    if grep -q "chat.headers" "$OC_PLUGIN"; then
        echo "[OK  ] OpenCode hook plugin stamps chat requests (chat.headers)"
    else
        echo "[FAIL] OpenCode hook plugin has no chat.headers hook — requests are not session-linked"; rc=1
    fi
    LOOPBACK_KEY_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/dynapps/bridge-loopback.key"
    if [ -r "$LOOPBACK_KEY_FILE" ] && grep -qF "$(cat "$LOOPBACK_KEY_FILE")" "$OC_PLUGIN"; then
        echo "[FAIL] OpenCode hook plugin embeds the raw loopback secret"; rc=1
    else
        echo "[OK  ] OpenCode hook plugin carries a scoped token, not the loopback secret"
    fi
else
    echo "[FAIL] no OpenCode hook plugin at $OC_PLUGIN — skill use from OpenCode is not recorded"; rc=1
fi

if grep -sq systemprompt "$OC_AUTH"; then
    echo "[OK  ] api key written to $OC_AUTH"
    [ "$(stat -c %a "$OC_AUTH")" = "600" ] \
        || { echo "[FAIL] $OC_AUTH is not 0600"; rc=1; }
else
    echo "[FAIL] no systemprompt entry in $OC_AUTH — this is the 403 case"; rc=1
fi

# The baseURL OpenCode was given must actually answer. Reading the key from the
# file rather than from the config proves the two halves agree.
KEY_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/dynapps/bridge-loopback.key"
if [ -r "$KEY_FILE" ]; then
    CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
        -H "Authorization: Bearer $(cat "$KEY_FILE")" \
        http://127.0.0.1:48217/v1/models)
    if [ "$CODE" = "200" ]; then
        echo "[OK  ] /v1/models through the loopback proxy returned 200"
    else
        echo "[FAIL] /v1/models through the loopback proxy returned $CODE"; rc=1
    fi
    # And the negative: no credential must still be refused, or the proxy is
    # open to anything that can reach the loopback interface.
    BARE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
        http://127.0.0.1:48217/v1/models)
    if [ "$BARE" = "403" ]; then
        echo "[OK  ] an uncredentialled request is still refused with 403"
    else
        echo "[FAIL] an uncredentialled request returned $BARE, expected 403"; rc=1
    fi

    # The same negative on the inference path OpenCode actually posts to. A
    # proxy that guards discovery but not inference guards nothing.
    BARE_CHAT=$(curl -s -o /dev/null -w "%{http_code}" --max-time 15 \
        -X POST -H "content-type: application/json" \
        -d "{\"model\":\"claude-haiku-4-5\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}" \
        http://127.0.0.1:48217/v1/chat/completions)
    if [ "$BARE_CHAT" = "403" ]; then
        echo "[OK  ] uncredentialled /v1/chat/completions is refused with 403"
    else
        echo "[FAIL] uncredentialled /v1/chat/completions returned $BARE_CHAT, expected 403"; rc=1
    fi

    # Behaviour, not placement: the catalogue handed to the client must match
    # what the provider block advertises, and one real completion must come back
    # over the wire OpenCode speaks.
    PROXY_MODELS=$(curl -s --max-time 15 -H "Authorization: Bearer $(cat "$KEY_FILE")" \
        http://127.0.0.1:48217/v1/models \
        | grep -oE "\"id\": *\"[^\"]+\"" | sed -e "s/.*: *\"//" -e "s/\"//" | sort -u | tr "\n" " ")
    echo "[INFO] proxy /v1/models: ${PROXY_MODELS:-(none)}"
    case " $PROXY_MODELS " in
        *gemini-*) echo "[OK  ] gemini models discoverable through the proxy" ;;
        *) echo "[FAIL] no gemini-* model through the proxy"; rc=1 ;;
    esac

    CHAT_CODE=$(curl -s -o /tmp/chat.json -w "%{http_code}" --max-time 60 \
        -X POST -H "Authorization: Bearer $(cat "$KEY_FILE")" \
        -H "content-type: application/json" \
        -d "{\"model\":\"claude-haiku-4-5\",\"max_tokens\":16,\"messages\":[{\"role\":\"user\",\"content\":\"Reply with exactly: pong\"}]}" \
        http://127.0.0.1:48217/v1/chat/completions)
    if [ "$CHAT_CODE" = "200" ]; then
        echo "[OK  ] a real completion came back over /v1/chat/completions"
    else
        echo "[FAIL] /v1/chat/completions returned $CHAT_CODE"; head -c 300 /tmp/chat.json; echo; rc=1
    fi
else
    echo "[FAIL] no loopback key at $KEY_FILE — the proxy never started"; rc=1
fi

exit $rc
' _ "$GATEWAY" "$PAT"

say "done"
