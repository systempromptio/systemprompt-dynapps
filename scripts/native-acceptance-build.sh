#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE="$ROOT/../systemprompt-core"
mode="${1:-adapter}"
case "$mode" in
    adapter) package=systemprompt-scheduler-tests; example=native_adapter_fixture ;;
    gateway) package=systemprompt-api-integration-tests; example=native_gateway_replay ;;
    *) echo 'Expected adapter or gateway exporter' >&2; exit 64 ;;
esac
fail() { echo "native-acceptance-build: $*" >&2; exit 64; }
[[ -f "$CORE/crates/tests/Cargo.toml" ]] || fail 'sibling core test workspace missing'
rg -q '^\[patch\.crates-io\]' "$ROOT/Cargo.toml" || fail 'local core patch required for coordinator fingerprint'
cd "$ROOT"
if [ -z "${DATABASE_URL:-}" ] && [ -f "$ROOT/.systemprompt/profiles/local/secrets.json" ]; then
    DATABASE_URL=$(python3 - "$ROOT/.systemprompt/profiles/local/secrets.json" <<'PYDB'
import json, sys
try:
    value = json.load(open(sys.argv[1]))["database_url"]
    if not isinstance(value, str) or not value:
        raise ValueError("missing URL")
    print(value)
except (OSError, KeyError, ValueError):
    sys.exit("test-core-feedback: local profile has no usable database_url")
PYDB
    )
    export DATABASE_URL
fi
if [ -n "${DATABASE_URL:-}" ] && [ "${SQLX_OFFLINE:-auto}" != true ]; then
    export SQLX_OFFLINE=false
    compile_mode="online"
else
    export SQLX_OFFLINE=true
    export SQLX_OFFLINE_DIR="${SQLX_OFFLINE_DIR:-$CORE/.sqlx}"
    [ -d "$SQLX_OFFLINE_DIR" ] || fail "offline query cache missing; provide DATABASE_URL or a prepared SQLX_OFFLINE_DIR"
    compile_mode="offline"
    echo "test-core-feedback: offline compilation requires prepared core and test query metadata" >&2
fi
execution_fingerprint=$(python3 - <<'PYENV'
import hashlib, os
values = [os.environ.get(key, "") for key in ("DATABASE_URL", "SQLX_OFFLINE_DIR", "SQLX_OFFLINE", "SYSTEMPROMPT_TEST_DATABASE_URL", "TEST_DATABASE_URL")]
print(hashlib.sha256("\0".join(values).encode()).hexdigest())
PYENV
)
exec "$ROOT/scripts/build-coordinator.sh" run native-acceptance-build "$mode:$compile_mode:$execution_fingerprint" -- cargo build --manifest-path "$CORE/crates/tests/Cargo.toml" -p "$package" --example "$example"
