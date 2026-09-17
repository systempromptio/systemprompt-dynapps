#!/usr/bin/env bash
# Build a new managed package version from the source org (Dev Hub).
# First time only: sf package create --name systemprompt-connector \
#   --package-type Managed --path force-app --target-dev-hub <alias>
# Usage: scripts/package-version.sh <devhub-alias>
set -euo pipefail
cd "$(dirname "$0")/.."
hub=${1:?dev hub alias (e.g. dev, us)}
sf package version create \
  --package systemprompt-connector \
  --installation-key-bypass \
  --code-coverage \
  --wait 20 \
  --target-dev-hub "$hub"
echo "Promote when proven: sf package version promote --package <04t…> --target-dev-hub $hub"
