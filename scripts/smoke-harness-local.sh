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
#   CAPTURE_DIR  when set, each client goes through scripts/harness-capture-proxy.py
#                and every inference request body lands there as JSON
#
# Afterwards: systemprompt infra logs request list --since 30m
set -uo pipefail

url="${GATEWAY_URL:-http://localhost:8080}"
user="${SMOKE_USER:-ed@systemprompt.io}"
models="${MODELS:-claude-sonnet-5-5 deepseek-flash go-kimi-k3 go-minimax-m3}"
clients="${CLIENTS:-claude opencode codex}"
cli="${SYSTEMPROMPT_CLI:-target/debug/systemprompt}"
opencode_bin="${OPENCODE_BIN:-$HOME/.opencode/bin/opencode}"
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

code="$("$cli" --json admin bridge issue-code --user-id "$user" 2>/dev/null \
    | python3 -c 'import json,sys; print(next(s["content"] for s in json.load(sys.stdin)["sections"] if s["heading"] == "code"))')"
[ -n "$code" ] || { echo "could not issue a bridge code for $user" >&2; exit 1; }
pat="$(curl -sS -X POST "$url/v1/auth/bridge/session-pat" -H 'content-type: application/json' \
    -d "{\"code\":\"$code\",\"device_name\":\"smoke-harness-local\"}" \
    | python3 -c 'import json,sys; print(json.load(sys.stdin).get("pat",""))')"
[ -n "$pat" ] || { echo "could not exchange the bridge code for a PAT" >&2; exit 1; }

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
            XDG_DATA_HOME="$work/home-$client/.local/share" XDG_STATE_HOME="$work/home-$client/.local/state"
        case "$client" in
        claude)
            export ANTHROPIC_BASE_URL="$client_url" ANTHROPIC_AUTH_TOKEN="$pat" DISABLE_AUTOUPDATER=1
            cont=(); [ "$turn" -gt 0 ] && cont=(--continue)
            with_timeout "$claude_bin" -p "${cont[@]}" --model "$model" \
                --allowedTools Read "$prompt" ;;
        opencode)
            cont=(); [ "$turn" -gt 0 ] && cont=(--continue)
            with_timeout "$opencode_bin" run "${cont[@]}" -m "dynapps/$model" "$prompt" ;;
        codex)
            export CODEX_HOME="$HOME/.codex" DYNAPPS_PAT="$pat"
            if [ "$turn" -gt 0 ]; then
                with_timeout "$codex_bin" exec --skip-git-repo-check -m "$model" resume --last "$prompt"
            else
                with_timeout "$codex_bin" exec --skip-git-repo-check -m "$model" "$prompt"
            fi ;;
        esac
    ) > "$log" 2>&1
    status=$?
    if [ "$status" -eq 0 ] && grep -qiE 'alpha|bravo|b\.txt|a\.txt' "$log"; then
        record "$client" "$model" ok
    else
        record "$client" "$model" "FAIL (exit $status): $(tail -n 3 "$log" | tr '\n' ' ' | cut -c1-300)"
    fi
}

write_configs() {
    local home="$work/home-$1"
    mkdir -p "$home/.config/opencode" "$home/.codex"
    python3 - "$home" "$client_url" "$pat" "$models" <<'PY'
import json, sys
home, url, pat, models = sys.argv[1:5]
ids = models.split()
json.dump({
    "$schema": "https://opencode.ai/config.json",
    "provider": {"dynapps": {
        "npm": "@ai-sdk/openai-compatible", "name": "Dynapps",
        "options": {"baseURL": url + "/v1", "apiKey": pat},
        "models": {m: {"name": m} for m in ids}}},
}, open(f"{home}/.config/opencode/opencode.json", "w"), indent=2)
open(f"{home}/.codex/config.toml", "w").write(
    'model_provider = "dynapps"\n\n[model_providers.dynapps]\nname = "Dynapps"\n'
    f'base_url = "{url}/v1"\nenv_key = "DYNAPPS_PAT"\nwire_api = "responses"\n')
PY
}

port=18210
proxies=()
for client in $clients; do
    client_url="$url"
    if [ -n "${CAPTURE_DIR:-}" ]; then
        port=$((port + 1))
        python3 scripts/harness-capture-proxy.py "$port" "$url" "$CAPTURE_DIR" "$client" &
        proxies+=("$!")
        sleep 1
        client_url="http://127.0.0.1:$port"
    fi
    write_configs "$client"
    turn=0
    for model in $models; do
        run_step "$client" "$model" "$turn"
        turn=$((turn + 1))
    done
done

if [ "${#proxies[@]}" -gt 0 ]; then kill "${proxies[@]}" 2> /dev/null; fi
echo
echo "client    model              status"
cat "$results"
echo "client logs: $work   gateway view: $cli infra logs request list --since 30m"
if grep -q FAIL "$results"; then exit 1; fi
rm -rf "$work"
