#!/usr/bin/env bash
# The bridge's browser-side lifecycle (profile connections, tabs, portfolio
# live/page) must pass its node:test suites under tests/bridge/. These modules
# ship as plain JS with no compiler, so the tests are the only thing that
# catches a broken connect/disconnect or tab-state transition before release.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
node --experimental-vm-modules --test "$root/tests/bridge/profile-connections.test.mjs" "$root/tests/bridge/sp-tabs.test.mjs" "$root/tests/bridge/portfolio-live.test.mjs" "$root/tests/bridge/portfolio-page.test.mjs"
