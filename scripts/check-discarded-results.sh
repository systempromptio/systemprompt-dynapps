#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$root/../systemprompt-core/scripts/check-discarded-results.sh" "$root/extensions" "$root/bridge/src"
