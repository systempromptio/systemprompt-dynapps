#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
node --experimental-vm-modules --test "$root/tests/bridge/profile-connections.test.mjs" "$root/tests/bridge/sp-tabs.test.mjs"
