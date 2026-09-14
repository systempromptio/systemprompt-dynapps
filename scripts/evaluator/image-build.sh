#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image="${1:-systemprompt-evaluator:local}"
docker_cli="${EVAL_DOCKER_BIN:-/usr/bin/docker}"
[[ "$docker_cli" = /* && -x "$docker_cli" ]] || { echo 'EVAL_DOCKER_BIN must be an absolute Docker executable' >&2; exit 64; }
[[ "$image" =~ ^[a-zA-Z0-9][a-zA-Z0-9._/:~-]{0,254}$ ]] || { echo 'Invalid evaluator image tag' >&2; exit 64; }
configuration=$(mktemp -d)
trap 'rm -rf "$configuration"' EXIT
docker_env=(env -i "PATH=$PATH" "HOME=$configuration" "DOCKER_CONFIG=$configuration")
if [[ -n "${DOCKER_HOST:-}" ]]; then docker_env+=("DOCKER_HOST=$DOCKER_HOST"); fi
mkdir -p "$repo_root/.build/evaluator-image"
report="$repo_root/.build/evaluator-image/$(date -u +%Y%m%dT%H%M%SZ)-$$"
# Builder requests remain under the shared coordinator lock and have a finite wall budget.
timeout --signal=TERM --kill-after=30s 45m "${docker_env[@]}" "$docker_cli" build \
    --progress=plain --tag "$image" --iidfile "$report.image-id" \
    --file "$repo_root/deploy/evaluator/Dockerfile" "$repo_root/deploy/evaluator" \
    > >(tee "$report.stdout") 2> >(tee "$report.stderr" >&2)
echo "Evaluator image identity: $report.image-id"
