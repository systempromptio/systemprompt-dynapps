#!/usr/bin/env bash
# Core's lint-serde-aliases gate over this install's sources; the rule lives in
# ../systemprompt-core/scripts/lint-serde-aliases.sh at bridge/CORE_REF (see core-gate.sh).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$root/scripts/core-gate.sh" lint-serde-aliases.sh --allowlist scripts/serde-aliases-allowlist.txt extensions src bridge/src
