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
    --config-file "$CORE/crates/tests/.config/nextest.toml" --no-tests fail)
for package in "${packages[@]}"; do args+=(-p "$package"); done
args+=(-E "$filter")
cd "$ROOT"
exec "$ROOT/scripts/build-coordinator.sh" run test-core-feedback "$package_list:$filter" -- cargo "${args[@]}"
