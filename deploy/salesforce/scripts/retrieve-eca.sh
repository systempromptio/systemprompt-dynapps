#!/usr/bin/env bash
# Pull the External Client App and its packaged OAuth settings from the source
# org into force-app/, filling in the platform-generated `oauthLink`.
# Usage: scripts/retrieve-eca.sh <source-org-alias>
set -euo pipefail
cd "$(dirname "$0")/.."
org=${1:?source org alias (e.g. dev, us)}
sf project retrieve start \
  --metadata "ExternalClientApplication:Systemprompt" \
  --metadata "ExtlClntAppOauthSettings:Systemprompt_oauth" \
  --target-org "$org"
grep -q "<oauthLink>" force-app/main/default/extlClntAppOauthSettings/Systemprompt_oauth.ecaOauth-meta.xml \
  || { echo "no oauthLink came back: OAuth is not enabled on the app yet (run bootstrap-source-org.sh)" >&2; exit 1; }
echo "Retrieved. Review the diff, keep distributionState=Packaged, commit force-app/."
