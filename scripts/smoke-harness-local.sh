#!/usr/bin/env bash
# Real-client end-to-end smoke for the cross-harness matrix, against the LOCAL
# gateway (`just start`). Each installed client — Claude Code (anthropic wire),
# OpenCode (openai-chat wire), Codex CLI (responses wire) — runs a prompt that
# forces two parallel file reads, then continues the SAME session on every
# model in MODELS, so one history crosses every provider wire.
#
# Isolation: every client gets a throwaway HOME / XDG_CONFIG_HOME / CODEX_HOME
# under a temp dir; the user's own ~/.claude, ~/.config and ~/.codex are never
# read or written. The PAT is minted for SMOKE_USER through a one-time bridge
# code and never printed.
#
#   GATEWAY_URL  default http://localhost:8080
#   SMOKE_USER   default ed@systemprompt.io (needs a saved Go key for go-*)
#   MODELS       default "claude-sonnet-5-5 deepseek-flash go-kimi-k3 go-minimax-m3"
#   CLIENTS      default "claude opencode codex"
#   CREDENTIAL   bridge (default: through a real dynapps-bridge proxy, BRIDGE_BIN),
#                jwt (bridge JWT straight at the gateway) or pat (raw PAT + session)
#   OPENCODE_HOME   where current opencode is npm-installed (default ~/.cache/dynapps-smoke/opencode)
#   OPENCODE_LEGACY 1 = the old ~/.opencode build with its provider package pinned
#   CAPTURE_DIR  when set, each client goes through scripts/harness-capture-proxy.py
#                and every inference request body lands there as JSON
#
# Afterwards: systemprompt infra logs request list --since 30m
#
# With CREDENTIAL=pat, a PAT principal reaches authz with no roles (core pins this in
# gateway_messages_auth.rs: only JWT principals carry the user's roles), so a
# route is reachable over a raw PAT only while its access_control entity is
# default-open. A route closed in the database 403s here even for an admin;
# compare it with rules.yaml on /admin/sync before blaming a wire.
set -uo pipefail

url="${GATEWAY_URL:-http://localhost:8080}"
user="${SMOKE_USER:-ed@systemprompt.io}"
models="${MODELS:-claude-sonnet-5-5 deepseek-flash go-kimi-k3 go-minimax-m3}"
clients="${CLIENTS:-claude opencode codex}"
credential="${CREDENTIAL:-bridge}"
bridge_bin="${BRIDGE_BIN:-bridge/target/release/dynapps-bridge}"
cli="${SYSTEMPROMPT_CLI:-target/debug/systemprompt}"
# OpenCode: the current release by default, kept in OPENCODE_HOME and
# installed there with npm on first use (never the user's own ~/.opencode).
# OPENCODE_LEGACY=1 runs the old ~/.opencode build instead, with its provider
# package pinned (see seed_opencode_cache); OPENCODE_BIN overrides either.
opencode_home="${OPENCODE_HOME:-$HOME/.cache/dynapps-smoke/opencode}"
if [ "${OPENCODE_LEGACY:-0}" = 1 ]; then
    opencode_bin="${OPENCODE_BIN:-$HOME/.opencode/bin/opencode}"
    opencode_pin="$opencode_pin"
else
    opencode_bin="${OPENCODE_BIN:-$opencode_home/node_modules/.bin/opencode}"
    opencode_pin="${OPENCODE_PROVIDER_PIN-}"
fi
case " ${CLIENTS:-claude opencode codex} " in *" opencode "*)
    if [ ! -x "$opencode_bin" ] && [ -z "${OPENCODE_BIN:-}" ] && [ "${OPENCODE_LEGACY:-0}" != 1 ]; then
        command -v npm > /dev/null || { echo "npm is needed to install current opencode into $opencode_home (or set OPENCODE_BIN)" >&2; exit 1; }
        npm install --silent --prefix "$opencode_home" opencode-ai@latest > /dev/null 2>&1 \
            || { echo "could not install opencode-ai@latest into $opencode_home" >&2; exit 1; }
    fi
    [ -x "$opencode_bin" ] || { echo "opencode not found at $opencode_bin (set OPENCODE_BIN or OPENCODE_LEGACY=1)" >&2; exit 1; }
    ;;
esac
codex_bin="${CODEX_BIN:-$HOME/.local/bin/codex}"
claude_bin="${CLAUDE_BIN:-$HOME/.local/bin/claude}"
step_timeout="${STEP_TIMEOUT:-180}"

work="$(mktemp -d)"
with_timeout() {
    if command -v timeout > /dev/null; then timeout "$step_timeout" "$@"; else "$@"; fi
}
mkdir -p "$work/project"
printf 'alpha: the first file\n' > "$work/project/a.txt"
printf 'bravo: the second file\n' > "$work/project/b.txt"
first_prompt="Read a.txt and b.txt using two tool calls issued in parallel in the same turn, then quote both lines."
next_prompt="Read a.txt and b.txt again with two parallel tool calls and tell me which file is longer."

# Codes, PATs and session ids live only in files under $work (mode 700) and
# are never echoed. The EXIT trap revokes this run's PAT (found by its unique
# device name) and ends every gateway session it minted, pass or fail.
chmod 700 "$work"
device="smoke-harness-local-$$-$(date +%s)"
sessions="$work/sessions"
: > "$sessions"
proxies=()
drop_work=0
cleanup() {
    if [ "${#proxies[@]}" -gt 0 ]; then kill "${proxies[@]}" 2> /dev/null; fi
    while read -r sid; do
        [ -n "$sid" ] || continue
        "$cli" admin users session end "$sid" -y > /dev/null 2>&1
    done < "$sessions"
    [ -s "$work/user_id" ] || return 0
    "$cli" --json admin users api-key list --user "$(cat "$work/user_id")" 2> /dev/null \
        | python3 -c 'import json,sys
for k in json.load(sys.stdin)["sections"][0]["content"]:
    if k["name"] == sys.argv[1] and not k.get("revoked_at"): print(k["id"])' "$device" \
        | while read -r id; do
            "$cli" admin users api-key revoke --user "$(cat "$work/user_id")" --id "$id" > /dev/null 2>&1 \
                && echo "revoked smoke PAT $id" || echo "could not revoke smoke PAT $id" >&2
        done
    if [ "$drop_work" -eq 1 ]; then rm -rf "$work"; fi
}
trap cleanup EXIT

"$cli" --json admin bridge issue-code --user-id "$user" 2> /dev/null | python3 -c '
import json, sys
sections = {s["heading"]: s["content"] for s in json.load(sys.stdin)["sections"]}
open(sys.argv[1], "w").write(sections["code"])
open(sys.argv[2], "w").write(sections["user_id"])' "$work/code" "$work/user_id" \
    || { echo "could not issue a bridge code for $user" >&2; exit 1; }
pat="$(python3 -c 'import json,sys; print(json.dumps({"code": open(sys.argv[1]).read(), "device_name": sys.argv[2]}))' \
        "$work/code" "$device" \
    | curl -sS -X POST "$url/v1/auth/bridge/session-pat" -H 'content-type: application/json' -d @- \
    | python3 -c 'import json,sys; print(json.load(sys.stdin).get("pat",""))')"
rm -f "$work/code"
[ -n "$pat" ] || { echo "could not exchange the bridge code for a PAT" >&2; exit 1; }

# One credential per client, written to $cred as {token, headers}. `bridge`
# is the full member path: the PAT logged into a real dynapps-bridge in a
# throwaway home, every client pointed at its loopback proxy, which swaps the
# loopback secret for a JWT and sets the identity headers itself (current
# opencode overwrites x-session-id with its own session id, so a client
# configured straight at the gateway fails "X-Session-ID does not match"). `jwt` is
# what a real member's bridge sends: the PAT exchanged at /v1/auth/bridge/pat
# for a JWT carrying the user's roles plus the headers it binds (x-session-id
# among them). `pat` sends the raw PAT with a session minted at
# /api/public/gateway/sessions.
mint_credential() {
    case "$credential" in
    jwt)
        curl -sS -f -X POST "$url/v1/auth/bridge/pat" -H "authorization: Bearer $pat" > "$1" ;;
    pat)
        curl -sS -f -X POST "$url/api/public/gateway/sessions" -H "authorization: Bearer $pat" \
            | python3 -c 'import json,sys
sid = json.load(sys.stdin)["session_id"]
json.dump({"token": sys.argv[1], "headers": {"x-session-id": sid}}, open(sys.argv[2], "w"))' "$pat" "$1" ;;
    bridge)
        python3 -c 'import json,sys; json.dump({"token": open(sys.argv[1]).read().strip(), "headers": {}}, open(sys.argv[2], "w"))' \
            "$work/bridge-secret" "$1" ;;
    *) echo "CREDENTIAL must be bridge, jwt or pat" >&2; return 1 ;;
    esac
}

upstream="$url"
if [ "$credential" = bridge ]; then
    [ -x "$bridge_bin" ] || { echo "CREDENTIAL=bridge needs $bridge_bin (just bridge-build)" >&2; exit 1; }
    bhome="$work/home-bridge"
    mkdir -p "$bhome"
    benv=(env HOME="$bhome" XDG_CONFIG_HOME="$bhome/.config" XDG_DATA_HOME="$bhome/.local/share" XDG_CACHE_HOME="$bhome/.cache")
    printf '%s' "$pat" | "${benv[@]}" "$bridge_bin" login --stdin --gateway "$url" > "$work/bridge-login.log" 2>&1 \
        || { echo "dynapps-bridge login failed (see $work/bridge-login.log)" >&2; exit 1; }
    "${benv[@]}" setsid "$bridge_bin" proxy > "$work/bridge-proxy.log" 2>&1 &
    proxies+=("$!")
    for _ in $(seq 1 30); do grep -q 'ANTHROPIC_AUTH_TOKEN=' "$work/bridge-proxy.log" && break; sleep 0.5; done
    sed -n 's/.*ANTHROPIC_AUTH_TOKEN=\([^ ]*\).*/\1/p' "$work/bridge-proxy.log" | head -n 1 > "$work/bridge-secret"
    upstream="$(sed -n 's/.*ANTHROPIC_BASE_URL=\([^ ]*\).*/\1/p' "$work/bridge-proxy.log" | head -n 1)"
    [ -s "$work/bridge-secret" ] && [ -n "$upstream" ] \
        || { echo "dynapps-bridge proxy did not start (see $work/bridge-proxy.log)" >&2; exit 1; }
fi

results="$work/results"
: > "$results"
record() { printf '%-9s %-18s %s\n' "$1" "$2" "$3" | tee -a "$results"; }

run_step() {
    local client="$1" model="$2" turn="$3" log="$work/$1-$2.log" status
    local prompt="$first_prompt"
    [ "$turn" -gt 0 ] && prompt="$next_prompt"
    (
        cd "$work/project" || exit 1
        export HOME="$work/home-$client" XDG_CONFIG_HOME="$work/home-$client/.config" \
            XDG_DATA_HOME="$work/home-$client/.local/share" XDG_STATE_HOME="$work/home-$client/.local/state" \
            XDG_CACHE_HOME="$work/home-$client/.cache"
        case "$client" in
        claude)
            export ANTHROPIC_BASE_URL="$client_url" ANTHROPIC_AUTH_TOKEN="$token" DISABLE_AUTOUPDATER=1 \
                ANTHROPIC_CUSTOM_HEADERS="$(python3 -c 'import json,sys
for k, v in json.load(open(sys.argv[1]))["headers"].items(): print(f"{k}: {v}")' "$cred")"
            cont=(); [ "$turn" -gt 0 ] && cont=(--continue)
            with_timeout "$claude_bin" -p "${cont[@]}" --model "$model" --debug-file "$work/$client-$model.debug" \
                --allowedTools=Read <<< "$prompt" ;;
        opencode)
            cont=(); [ "$turn" -gt 0 ] && cont=(--continue)
            with_timeout "$opencode_bin" run "${cont[@]}" -m "dynapps/$model" "$prompt" < /dev/null ;;
        codex)
            export CODEX_HOME="$HOME/.codex" DYNAPPS_PAT="$token"
            if [ "$turn" -gt 0 ]; then
                with_timeout "$codex_bin" exec --skip-git-repo-check -m "$model" resume --last "$prompt"
            else
                with_timeout "$codex_bin" exec --skip-git-repo-check -m "$model" "$prompt"
            fi ;;
        esac
    ) > "$log" 2>&1
    status=$?
    if [ "$status" -eq 0 ] && ! grep -qE "^(Error|ERROR)|APICallError" "$log" && grep -qiE 'alpha|bravo|b\.txt|a\.txt' "$log"; then
        record "$client" "$model" ok
    else
        record "$client" "$model" "FAIL (exit $status): $(tail -n 3 "$log" | tr '\n' ' ' | cut -c1-300)"
    fi
}

# opencode installs its provider package as `@latest` on first use. 0.5.x is
# built on AI SDK v5, and @ai-sdk/openai-compatible 2.x+ reports usage as
# nested objects, so every finished step dies in opencode's cost maths with
# "[DecimalError] Invalid argument: [object Object]" whatever the gateway
# sends. Seed the cache with the last v5-compatible major and mark it as the
# `latest` opencode asked for so it is not replaced. Only the legacy lane
# pins (OPENCODE_LEGACY=1); OPENCODE_PROVIDER_PIN overrides, "" disables.
# turns this off for opencode builds on a newer SDK.
seed_opencode_cache() {
    local cache="$1/.cache/opencode" pin="$opencode_pin"
    [ -n "$pin" ] || return 0
    # Any opencode start first stamps its cache version, wiping a cache it did
    # not stamp, so let it do that before seeding.
    HOME="$1" XDG_CACHE_HOME="$1/.cache" "$opencode_bin" --version > /dev/null 2>&1
    mkdir -p "$cache"
    echo '{"dependencies":{}}' > "$cache/package.json"
    # opencode's own plugins are seeded too: installing one later re-resolves
    # every `latest` in the cache and would undo the pin.
    # One package per `add`, the way opencode installs them (a combined add
    # hoists opencode-anthropic-auth's own deps away).
    local pkg
    for pkg in opencode-copilot-auth@latest opencode-anthropic-auth@latest "@ai-sdk/openai-compatible@$pin"; do
        BUN_BE_BUN=1 "$opencode_bin" add --force --exact --cwd "$cache" "$pkg" > /dev/null 2>&1 \
            || { echo "could not seed $pkg for opencode" >&2; return 1; }
    done
    printf '{"dependencies":{"%s":"latest","%s":"latest","%s":"latest"}}\n' \
        @ai-sdk/openai-compatible opencode-copilot-auth opencode-anthropic-auth > "$cache/package.json"
}

write_configs() {
    local home="$work/home-$1"
    if [ "$1" = opencode ]; then seed_opencode_cache "$home" || exit 1; fi
    mkdir -p "$home/.config/opencode" "$home/.codex"
    python3 - "$home" "$client_url" "$models" "$cred" <<'PY'
import json, sys
home, url, models, cred = sys.argv[1:5]
cred = json.load(open(cred))
pat, headers = cred["token"], cred["headers"]
ids = models.split()
json.dump({
    "$schema": "https://opencode.ai/config.json",
    "provider": {"dynapps": {
        "npm": "@ai-sdk/openai-compatible", "name": "Dynapps",
        "options": {"baseURL": url + "/v1", "apiKey": pat, "includeUsage": True,
                    "headers": headers},
        "models": {m: {"name": m} for m in ids}}},
}, open(f"{home}/.config/opencode/opencode.json", "w"), indent=2)
open(f"{home}/.codex/config.toml", "w").write(
    'model_provider = "dynapps"\n\n[model_providers.dynapps]\nname = "Dynapps"\n'
    f'base_url = "{url}/v1"\nenv_key = "DYNAPPS_PAT"\nwire_api = "responses"\n'
    "http_headers = { " + ", ".join(f'"{k}" = "{v}"' for k, v in headers.items()) + " }\n")
PY
}

port=18210
for client in $clients; do
    client_url="$upstream"
    if [ -n "${CAPTURE_DIR:-}" ]; then
        port=$((port + 1))
        python3 scripts/harness-capture-proxy.py "$port" "$upstream" "$CAPTURE_DIR" "$client" &
        proxies+=("$!")
        sleep 1
        client_url="http://127.0.0.1:$port"
    fi
    # The gateway refuses a PAT without a minted x-session-id (the bridge
    # mints its own); one per client keeps its model switches in one session.
    cred="$work/cred-$client.json"
    mint_credential "$cred" || { echo "could not mint a $credential credential for $client" >&2; exit 1; }
    token="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["token"])' "$cred")"
    python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["headers"].get("x-session-id", ""))' "$cred" >> "$sessions"
    write_configs "$client"
    turn=0
    for model in $models; do
        run_step "$client" "$model" "$turn"
        turn=$((turn + 1))
    done
done

echo
echo "client    model              status"
cat "$results"
echo "client logs: $work   gateway view: $cli infra logs request list --since 30m"
if grep -q FAIL "$results"; then exit 1; fi
drop_work=1
