#!/usr/bin/env bash
# Records version, executable bytes, CLI flags and Linux container isolation without inference.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image="${1:-systemprompt-evaluator:local}"
expected_version=2.1.270
docker_cli="${EVAL_DOCKER_BIN:-/usr/bin/docker}"
[[ "$docker_cli" = /* && -x "$docker_cli" ]] || { echo 'Set EVAL_DOCKER_BIN to an absolute Docker executable' >&2; exit 64; }
[[ "$image" != -* && "$image" != *$'\n'* && ${#image} -le 255 ]] || exit 64
command -v timeout >/dev/null || { echo 'Linux timeout is required for bounded native probes' >&2; exit 64; }
configuration=$(mktemp -d)
report_dir="$repo_root/.build/native-claude/$(date -u +%Y%m%dT%H%M%SZ)-$$"
mkdir -p "$report_dir"
docker_env=(env -i "PATH=$PATH" "HOME=$configuration" "DOCKER_CONFIG=$configuration")
if [[ -n "${DOCKER_HOST:-}" ]]; then docker_env+=("DOCKER_HOST=$DOCKER_HOST"); fi
docker=("${docker_env[@]}" "$docker_cli")
container="eval-claude-native-$$"
phase=inspect
cleanup_confirmed=false
image_id=unknown
cleanup() {
    result=$?
    trap - EXIT
    if timeout 15 "${docker[@]}" rm --force "$container" >/dev/null 2>&1; then cleanup_confirmed=true; fi
    if remaining=$(timeout 15 "${docker[@]}" ps -aq --filter "name=^/${container}$" 2>/dev/null); then
        if [[ -z "$remaining" ]]; then cleanup_confirmed=true; else cleanup_confirmed=false; fi
    else
        cleanup_confirmed=false
    fi
    if [[ "$cleanup_confirmed" != true ]]; then result=1; fi
    python3 - "$report_dir/report.json" "$result" "$phase" "$image_id" "$cleanup_confirmed" <<'PY'
import json, sys
path, result, phase, image, cleanup = sys.argv[1:]
with open(path, 'w') as output:
    json.dump({'client': 'claude-code', 'expected_version': '2.1.270', 'image_id': image,
               'status': 'passed' if result == '0' else 'failed', 'last_phase': phase,
               'cleanup_confirmed': cleanup == 'true', 'paid_inference': False,
               'gateway_metering_verified': False, 'automated_target_enabled': False}, output, indent=2)
PY
    rm -rf "$configuration"
    echo "Claude native probe report: $report_dir/report.json"
    exit "$result"
}
trap cleanup EXIT
image_id=$(timeout 20 "${docker[@]}" image inspect --format '{{.Id}}' "$image")
[[ "$image_id" =~ ^sha256:[a-f0-9]{64}$ ]] || { echo 'Docker image has no exact local image ID' >&2; exit 1; }
run() {
    local label="$1"; shift
    phase="$label"
    (ulimit -f 1024
    timeout 25 "${docker[@]}" run --pull=never --name "$container" \
        --network none --read-only --cap-drop ALL --security-opt no-new-privileges \
        --pids-limit 64 --memory 512m --cpus 1 --user 1001:1001 \
        --tmpfs /tmp:rw,nosuid,nodev,size=32m,mode=1777 \
        --tmpfs /home/tester/.claude:rw,nosuid,nodev,size=16m,uid=1001,gid=1001,mode=0700 \
        --tmpfs /home/tester/.config:rw,nosuid,nodev,size=16m,uid=1001,gid=1001,mode=0700 \
        --tmpfs /home/tester/.cache:rw,nosuid,nodev,size=16m,uid=1001,gid=1001,mode=0700 \
        --entrypoint /usr/bin/env "$image_id" -i HOME=/home/tester \
        PATH=/usr/local/bin:/usr/bin:/bin CLAUDE_CONFIG_DIR=/home/tester/.claude \
        DISABLE_AUTOUPDATER=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
        "$@" >"$report_dir/$label.stdout" 2>"$report_dir/$label.stderr")
    timeout 15 "${docker[@]}" rm --force "$container" >/dev/null
}
run version /usr/local/bin/claude --version
[[ "$(cat "$report_dir/version.stdout")" = "$expected_version (Claude Code)" ]] || { echo 'Native executable version does not match the pinned adapter image' >&2; exit 1; }
run executable /usr/bin/sha256sum /usr/local/bin/claude
run flags /usr/local/bin/claude --bare --restricted --strict-mcp-config --setting-sources '' \
    --permission-mode dontAsk --permission-prompts none --no-session-persistence --no-chrome \
    --max-turns 1 --tools Read,Glob,Grep --allowedTools Read,Glob,Grep \
    --disallowedTools Bash,Agent,Task,WebSearch,WebFetch,Write,Edit,Skill \
    --settings '{"disableAllHooks":true,"permissions":{"defaultMode":"dontAsk","disableBypassPermissionsMode":"disable"}}' \
    --output-format stream-json --verbose --version
run isolation /usr/local/bin/node -e "$(cat "$repo_root/scripts/evaluator/probe.js")"
phase=complete
