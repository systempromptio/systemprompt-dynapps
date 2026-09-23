# Systemprompt connector — Salesforce package

One External Client App (ECA), built in **ONTRQ**
(`ontrq.my.salesforce.com`), is how Salesforce authorises the Systemprompt
gateway. The gateway holds one consumer key and secret; each org-and-surface is
a row in `services/config/salesforce-orgs.yaml` and a Connect button on the
profile page. The app is authored `Packaged`, so a second org can install it as
a 2GP package later without rebuilding it. Decision and tradeoffs: `docs/decisions/salesforce-connector.md`.

```
deploy/salesforce/
  sfdx-project.json          force-app = the managed package; policies = per-org source
  .forceignore               keeps the global OAuth settings (the secret) out of git
  force-app/main/default/
    externalClientApps/Systemprompt.eca-meta.xml             the app, distributionState Packaged
    extlClntAppOauthSettings/Systemprompt_oauth.ecaOauth-meta.xml  callback, scopes, PKCE, oauthLink
    permissionsets/Systemprompt_MCP_Access.permissionset-meta.xml  API access marker (optional)
  policies/main/default/
    extlClntAppOauthPolicies/…                               admin policy, deployed per org
  bootstrap/                 global OAuth settings, deployed once so the platform mints the key
  scripts/bootstrap-source-org.sh  deploys everything to the source org and reads the key/secret back
  scripts/retrieve-eca.sh    re-pulls the packaged files after a change in Setup
  scripts/package-version.sh builds a package version on the Dev Hub
  scripts/install.sh         installs a version + deploys the policy into an org
```

Prerequisites: Salesforce CLI (`sf`), an authenticated alias per org
(`sf org login web --alias ontrq`), Dev Hub enabled on the source org
(Setup → Dev Hub), and — for a managed package — a namespace registered on
a Developer Edition org and linked to the Dev Hub (Setup → Namespace
Registries). Put the namespace in `sfdx-project.json` once it exists.

## Order of work

This instance fronts one org, `ontrq.my.salesforce.com`, through one connector
(`salesforce`, on the Headless 360 endpoint). Steps 1–2 and 5 are the whole
setup for it. Steps 3–4 build and install the package and are
needed only to add a *second* org later — the app is authored `Packaged` from the
start so that stays possible without rebuilding it.

To rehearse against a throwaway org first, run steps 1–2 on a Developer Edition
org and point a temporary registry entry at it.

### 1. Create the app in the source org (CLI, once)

```
sf org login web --alias ontrq --instance-url https://ontrq.my.salesforce.com
scripts/bootstrap-source-org.sh ontrq
```

The script deploys the app, the global OAuth settings (authored without a
key — the platform mints the consumer key and secret on deploy), the OAuth
settings (scopes `MCP, RefreshToken, OpenID`; the platform generates
`oauthLink`), the admin policy and the permission set; then retrieves the
platform's copies so the packaged OAuth settings file carries the real
`oauthLink`, keeps the consumer key in `.retrieved/` (gitignored) and prints
the two `secret set` commands. The consumer **secret** is not exposed by the
Metadata API: read it once in Setup → External Client App Manager →
Systemprompt → Settings → OAuth Settings. Both are the same for every org that
installs the package. Commit `force-app/`.

The policy is `AllSelfAuthorized`: a login in the org is the entitlement.
Pre-authorising a permission set is a Setup-only action (no metadata), so an
org that wants an explicit allow-list switches the policy there.

Then, still in the source org: Setup → **API Catalog → MCP Servers** →
activate **SObject All** *and* **Headless 360**. One per registry entry; a
surface left inactive makes its Connect row consent cleanly and then fail every
tool call.

Setup-UI alternative: External Client App Manager → New (`Systemprompt`,
Packaged), enable OAuth with the callbacks in `bootstrap/…`, scopes mcp_api /
refresh_token / openid, PKCE, secret required for refresh flow, named-user JWT
tokens; then `scripts/retrieve-eca.sh ontrq`.

### 2. Re-retrieve after any change made in Setup

```
scripts/retrieve-eca.sh ontrq
```

Review the diff: `distributionState` stays `Packaged`, `oauthLink` is
unchanged. Commit `force-app/`. `.retrieved/` holds the secret and is
gitignored; delete it when done.

### 3. Build the package (Dev Hub) — only to add a second org

```
sf package create --name systemprompt-connector --package-type Managed \
  --path force-app --target-dev-hub ontrq        # once; writes packageAliases
scripts/package-version.sh ontrq                 # every release; prints the 04t… id
sf package version promote --package <04t…> --target-dev-hub ontrq   # when proven
```

While testing, an `Unlocked` package behaves the same for install and OAuth and
needs no namespace; switch to `Managed` for a real second org.

### 4. Install into a second org — only to add one

```
scripts/install.sh <alias> <04t…>
```

Then in that org: Setup → API Catalog → MCP Servers → activate the surfaces it
needs. The installed app references ONTRQ's OAuth settings, so the consumer key
stored in step 1 works there too; nothing is typed into the second org. A user
without a login there is refused by Salesforce at consent — that is the
entitlement. Assign `Systemprompt_MCP_Access` only to users whose profile
lacks API access.

### 5. Gateway configuration

`services/config/salesforce-orgs.yaml` already declares both entries against
`https://ontrq.my.salesforce.com` with `environment: production`, each with its
`services/mcp/<id>.yaml` and an `mcp_server/<id>` block in
`services/access-control/rules.yaml`. Store the consumer key and secret from step
1 as `salesforce_mcp_client_id` / `salesforce_mcp_client_secret` on the target
profile and deploy; until then both rows read "Not provisioned".

Users then open **Account → Connected accounts**, press **Connect** on a row,
sign in to *that* org (log out of other Salesforce orgs first — Salesforce's
login cannot pick an org for a multi-org user), consent, **Test connection**,
and sync the bridge.

## Adding another org later

Install the package there (step 4), then add an `orgs.yaml` entry, a
`services/mcp/salesforce-<slug>.yaml` listed in `services/config/config.yaml`,
and an `mcp_server/salesforce-<slug>` rule. No new secrets, no callback edit:
every org shares `/api/public/connectors/salesforce/callback` and the gateway
recovers the org from the OAuth `state`.

## Sandbox refresh

A refresh recreates the org without the installed package. Reinstall (step 4),
reactivate SObject All, then
`POST /api/public/admin/connectors/salesforce-<slug>/reprovision` on the
gateway so every user is told to reconnect.

## Open checks

- Whether a callback URL added to the app on ONTRQ reaches a subscriber org
  without a new package version (it should — subscribers reference the source
  settings — but test it against a scratch org before relying on it). Only
  matters once a second org exists.
