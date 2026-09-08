#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
exec bash "$root/../systemprompt-core/scripts/check-fail-open.sh" "$root/extensions" "$root/bridge/src"
