#!/usr/bin/env bash
# Create the Systemprompt External Client App in the source org from source,
# let the platform mint its consumer key and secret, then read them back.
# Usage: scripts/bootstrap-source-org.sh <org-alias>
# Prereq: sf org login web --alias <org-alias> --instance-url https://<my domain>
set -euo pipefail
cd "$(dirname "$0")/.."
org=${1:?org alias (e.g. dev, us)}

# Why: the app must exist before its OAuth settings, and OAuth before the
# policy, so this is three deploys, not one.
echo "→ app + global OAuth settings (the platform mints the consumer key) + OAuth settings"
sf project deploy start \
  --source-dir force-app/main/default/externalClientApps \
  --source-dir bootstrap \
  --source-dir force-app/main/default/extlClntAppOauthSettings \
  --target-org "$org" --wait 10
echo "→ admin policy + permission set"
sf project deploy start --source-dir policies --source-dir force-app/main/default/permissionsets \
  --target-org "$org" --wait 10

# Why: retrieve lands in the project's own source dirs (an --output-dir is
# refused outside the project and silently empty inside .forceignore'd paths).
# The global settings come back WITH the consumer key, so they are moved to the
# gitignored .retrieved/ straight away.
echo "→ retrieving the platform's copies (oauthLink, consumer key)"
sf project retrieve start \
  --metadata "ExternalClientApplication:Systemprompt" \
  --metadata "ExtlClntAppOauthSettings:Systemprompt_oauth" \
  --metadata "ExtlClntAppGlobalOauthSettings:Systemprompt_glblOauth" \
  --target-org "$org" --wait 10
glbl=bootstrap/main/default/extlClntAppGlobalOauthSets/Systemprompt_glblOauth.ecaGlblOauth-meta.xml
mkdir -p .retrieved && cp "$glbl" .retrieved/
key=$(sed -n 's:.*<consumerKey>\(.*\)</consumerKey>.*:\1:p' "$glbl")
# The bootstrap copy is the deploy request and must stay key-free.
sed -i '/<consumerKey>/d' "$glbl"
[ -n "$key" ] || { echo "no consumerKey in the retrieved global settings" >&2; exit 1; }

echo "→ force-app OAuth settings now carry the org's oauthLink (commit force-app/):"
grep -E "oauthLink|OauthScopes" force-app/main/default/extlClntAppOauthSettings/Systemprompt_oauth.ecaOauth-meta.xml
echo
echo "Consumer key retrieved (in .retrieved/, gitignored). The consumer SECRET is not"
echo "exposed by the Metadata API: read it once in Setup → External Client App Manager →"
echo "Systemprompt → Settings → OAuth Settings → Consumer Key and Secret. Then:"
echo "  systemprompt admin config secret set salesforce_mcp_client_id     '$key'"
echo "  systemprompt admin config secret set salesforce_mcp_client_secret '<consumer secret>'"
