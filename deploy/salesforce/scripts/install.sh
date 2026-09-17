#!/usr/bin/env bash
# Install the packaged app into a subscriber org and deploy the admin policy.
# Usage: scripts/install.sh <org-alias> <04t package version id>
set -euo pipefail
cd "$(dirname "$0")/.."
org=${1:?target org alias (e.g. uk)}
version=${2:?package version id (04t…)}
sf package install --package "$version" --target-org "$org" --wait 20 --no-prompt
sf project deploy start \
  --source-dir policies \
  --target-org "$org"
echo "Now activate SObject All in Setup → API Catalog → MCP Servers. Users whose profile lacks API access: sf org assign permset --name Systemprompt_MCP_Access --target-org $org --on-behalf-of <username>"
