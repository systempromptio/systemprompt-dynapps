#!/usr/bin/env bash
# Core's lint-unbounded-bodies gate over this install's sources; the rule lives in
# ../systemprompt-core/scripts/lint-unbounded-bodies.sh at bridge/CORE_REF (see core-gate.sh).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$root/scripts/core-gate.sh" lint-unbounded-bodies.sh extensions src bridge/src
