#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE="$ROOT/../systemprompt-core"

fail() { echo "test-core-feedback: $*" >&2; exit 64; }

[ "$#" -ge 1 ] && [ "$#" -le 2 ] || fail "usage: $0 package[,package...] [nextest-filter]"
package_list="$1"
filter="${2:-all()}"
[ "${#package_list}" -le 2048 ] || fail "package list exceeds 2048 characters"
[ "${#filter}" -le 2048 ] || fail "filter exceeds 2048 characters"
[[ "$filter" != *$'\n'* && "$filter" != *$'\r'* ]] || fail "filter must be one line"
[[ "$package_list" =~ ^systemprompt-[a-z0-9-]+-tests(,systemprompt-[a-z0-9-]+-tests)*$ ]] || fail "supply comma-separated systemprompt-*-tests package names"
IFS=',' read -r -a packages <<< "$package_list"
[ "${#packages[@]}" -le 16 ] || fail "select at most 16 test packages per batch"
[ -f "$CORE/crates/tests/Cargo.toml" ] || fail "sibling core test workspace is missing"
[ -f "$CORE/crates/tests/.config/nextest.toml" ] || fail "core nextest configuration is missing"
# The coordinator includes the sibling source tree only while this patch is active.
grep -qE '^\[patch\.crates-io\]' "$ROOT/Cargo.toml" || fail "enable the local core patch so the coordinator fingerprints core source"

args=(nextest run --manifest-path "$CORE/crates/tests/Cargo.toml"
    --config-file "$CORE/crates/tests/.config/nextest.toml" --no-tests fail --no-fail-fast)
for package in "${packages[@]}"; do args+=(-p "$package"); done
args+=(-E "$filter")
cd "$ROOT"
# Feedback acceptance mutates its database. Require an explicit fixture URL;
# never inherit the running application's local profile database.
[ -n "${DATABASE_URL:-}" ] || fail "set DATABASE_URL explicitly to a dedicated test database"
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
exec "$ROOT/scripts/build-coordinator.sh" run test-core-feedback "$package_list:$filter:$compile_mode:$execution_fingerprint" -- \
    bash "$ROOT/scripts/test-core-feedback-run.sh" "$CORE/crates/tests/Cargo.toml" "${args[@]}"
