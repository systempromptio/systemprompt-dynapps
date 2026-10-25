#!/usr/bin/env bash
# Core's lint-owned-tasks gate over this install's sources; the rule lives in
# ../systemprompt-core/scripts/lint-owned-tasks.sh at bridge/CORE_REF (see core-gate.sh).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$root/scripts/core-gate.sh" lint-owned-tasks.sh extensions src bridge/src
