#!/usr/bin/env bash
# rustfmt --check over one workspace's own members.
#
#   scripts/fmt-check.sh [path/to/Cargo.toml]
#
# `cargo fmt --all` also formats every local path dependency, which for the
# tests and bridge workspaces is the core checkout (../systemprompt-core or
# .vendor/systemprompt-core). Core is formatted by core's toolchain and
# config, so this passes each workspace member explicitly with `-p`.
set -euo pipefail
manifest="${1:-Cargo.toml}"
mapfile -t packages < <(
    cargo metadata --manifest-path "$manifest" --no-deps --format-version 1 --offline \
        | python3 -c 'import json, sys; [print("-p\n" + p["name"]) for p in json.load(sys.stdin)["packages"]]'
)
if [ "${#packages[@]}" -eq 0 ]; then
    echo "fmt-check: no workspace members found in $manifest" >&2
    exit 1
fi
cargo fmt --manifest-path "$manifest" "${packages[@]}" -- --check
