# Configured personal MCP accounts

Profile authentication is independent of marketplace membership. Active users
can authorize configured personal connectors; resource rules still govern use.
Enabled servers are listed even when `display_in_web` is false. Servers without
personal authorization show “No authorization required”. Saved accounts remain
disconnectable after a server is removed or disabled.

## Generic OAuth

Add the server YAML to the service aggregator's includes:

```yaml
mcp_servers:
  example-tools:
    type: external
    binary: ''
    package: null
    port: 5050
    endpoint: https://mcp.example.com/mcp
    enabled: true
    display_in_web: false
    oauth:
      required: false
      scopes: [user]
      audience: mcp
      client_id: null
    connector:
      adapter: generic
      scopes: [tools:read]
      authorization_origins:
        - https://login.example.com
      # Omit both for dynamic client registration:
      # client_id_secret: example_mcp_client_id
      # client_secret: example_mcp_client_secret
```

The `oauth` block governs inbound access; `connector` configures outbound OAuth.
Core supplies the credential-broker endpoint automatically when `connector` is
set and `external_auth` is absent. Preserve the existing broker secret.

The resource must publish protected-resource metadata identifying its OAuth
authorization server, whose metadata must advertise PKCE S256. The resource
origin is trusted. Explicitly list any additional discovery, registration,
authorization or token origin. HTTPS is required, including for internal
services with certificates trusted by the host. HTTP redirects are not followed.

Without dynamic registration, configure secret references for a registered
client. Its callback is
`https://YOUR-SERVER/api/public/connectors/example-tools/callback`.
Client secrets never belong in YAML or browser responses.

Generic verification initializes MCP and lists accessible tools. It does not
invent an account identity when no provider identity endpoint exists. Atlassian,
GitHub and Salesforce retain their specific adapters and verification.

Changing a generic resource or OAuth settings requires reconnection. Refresh
also checks the original issuer and token endpoint. Disconnect removes local
credential access for every enrolled device.

Three optional `connector` keys shape the flow for issuers that need them:

| Key | Effect |
| --- | --- |
| `display_name` | Label on the Connectors row instead of the server id |
| `authorization_params` | Extra query parameters on the authorization request. Only `access_type`, `prompt`, `login_hint` and `hd` are accepted; every parameter the flow itself sets is refused at load |
| `identity: userinfo` | After consent, read `sub` and `email` from the issuer's OIDC `userinfo_endpoint` and show the address on the row. Requires the `openid` scope |

Issuer identifiers are compared as URLs, with a single trailing slash ignored,
so an issuer advertised as `https://idp.example/` in protected-resource metadata
matches `https://idp.example` in its own metadata.

## Google Workspace (Agent Search, `google_drive`)

`services/mcp/google_drive.yaml` connects Google Agent Search's hosted MCP
server (Discovery Engine, ex-Vertex AI Search) over **federated** Google
Drive, Gmail and Calendar data stores: nothing is copied, all are searched
live, and Google filters every result to what the signed-in user can already
open — their Drive permissions, their own mailbox and calendar. Tools: `search`,
`conversational_search`, `list_engines`. Scope:
`https://www.googleapis.com/auth/discoveryengine.readwrite`. The endpoint is
**regional** — `eu-discoveryengine.googleapis.com/mcp` for an `eu` engine; the
global host answers `Incorrect API endpoint used` for anything else.

**No licence.** Discovery Engine is usage-billed (10k queries/month free);
Gemini Enterprise is the licensed end-user app on the same engine and is not
needed. Proven 17 Sep 2026 in a project with nothing but the API enabled. The
OAuth scope is Discovery Engine only — no `gmail.*` scope; Google reads the
mailbox server-side under the user's Workspace identity. (RAG Engine, the
corpus-copy alternative, was dropped the same day: with per-user Drive ACLs
available live there is no reason to hold a copy.)

The row uses the generic adapter: Google publishes protected-resource metadata
for the endpoint and OpenID metadata at `accounts.google.com`. Two Google
specifics are carried by the keys above: `authorization_params: {access_type:
offline, prompt: consent}` — Google issues a refresh token only when offline
access is requested, and without one the connection expires with the hour-long
access token — and `identity: userinfo` so the row shows the Google address.

The engine's serving config is a **tool argument**, and nothing model-visible
in a server config reaches the model, so the `google_drive_search` skill's
`## Instance` block holds that one resource name. **On handover** two things
change and nothing else: the `servingConfig` in
`services/skills/google_drive_search/SKILL.md` and the OAuth client id/secret.

On the Google side, in one GCP project:

1. Enable `discoveryengine.googleapis.com`; attach billing and a budget alert.
2. AI Applications → Settings → Identity provider → Google Identity (before
   the data store; the wizard refuses otherwise).
3. Create federated data stores for Drive, Gmail and Calendar (location
   `eu`) and one Enterprise-tier search app over **all of them** — an app
   created with a single store can never gain a second (`Engines with a
   single data store cannot add or remove data stores`); the app's
   `…/servingConfigs/default_search` is the `servingConfig`. Scriptable against
   the Discovery Engine REST API with an `x-goog-user-project` header:
   `dataStores.create` (v1alpha) with **all three** of `contentConfig:
   GOOGLE_WORKSPACE`, `aclEnabled: true` and `workspaceConfig: {type:
   GOOGLE_DRIVE | GOOGLE_MAIL | GOOGLE_CALENDAR, dasherCustomerId: <Workspace
   customer id, e.g. C01hjjuoz from `gcloud organizations list`>}`, then
   `engines.create` with every `dataStoreId`. A store created without those
   three fields is accepted, silently unbound from the Workspace, and returns
   nothing forever — `workspaceConfig` is immutable, so delete and recreate.
   `GOOGLE_PEOPLE` (directory) is not servable by the search backend in `eu`
   (`No mapping of workspace config type found`); Chat, Groups, Sites and Keep
   were left out deliberately. The console wizard sets all of this for you;
   the API does not.
4. Create an OAuth **Web application** client with an **Internal** consent
   screen, the scope above and the redirect
   `https://YOUR-SERVER/api/public/connectors/google_drive/callback`. Store
   the pair as `google_drive_mcp_client_id` / `google_drive_mcp_client_secret`.
5. Grant the readership group `roles/mcp.toolUser` and
   `roles/discoveryengine.viewer` on the project. IAM decides who may call;
   Drive decides what they see. The Astound rule for `mcp_server/google_drive`
   decides who sees the row.
6. Workspace Admin → Security → API controls → Manage third-party app access:
   mark Google's Agent Search client as **Trusted**, and turn on "Smart
   features and personalization in other Google products" (Gmail user
   settings, org-wide or per user). Without both, searches succeed and return
   nothing; propagation can take up to 24 h.
7. Turn on Data Access audit logs for Discovery Engine.

Acceptance: connect with an account inside the group — the row reads
"Google Workspace · Connected" with the address, Test lists `search`, and a call
an hour later still succeeds (refresh path). Two users with different Drive
access to one file get different results for the same query; an account
outside the group is refused by Google, not answered.

## Migration and failures

Migration 051 replaces the three-provider database restrictions with server-ID
validation. Existing accounts, encrypted grants, revisions and generations are
preserved. Apply migrations with the new binary before adding another provider.
Older binaries cannot manage generic accounts; do not roll back after creating
them without first disconnecting/removing those accounts.

Authorization database failures stop inference with HTTP 503 and a retry hint.
Governance hooks return an explicit deny envelope. Look for
`authorization_unavailable` in logs; the next request evaluates fresh policy
after database access is restored.

## Gating skills on a connection

A live connection is an authorization subject. `services/access-control/rules.yaml`
accepts a `connector:` band (precedence 160, between `group` and the Salesforce
`salesforce` link band) whose values are server ids; a person matches while
`mcp_connector_accounts` holds a `connected` row for that server
(`extensions/web/admin/src/authz/connector.rs`). The UK sales building-block
skills use it — `skill/google_workspace_search` allows `connector: [google_drive]`
— so a skill that can only fail without its connector is not offered to a
person who has not connected it, and appears the moment they do.

## Tool permissions in the clients

Every tool on a managed server is allowed by default in Claude Code and
Claude Desktop/Cowork — the governance chain already judges each call, so the
client's own "Claude wants to use …" prompt adds no control. The bridge writes
`permissions.allow` rules (`mcp__<server>`, plus `mcp__plugin_<plugin>_<server>`
for each plugin that mirrors the server) into Claude Code's managed settings
file when it is writable and `~/.claude/settings.json` otherwise, and a per-tool
`toolPolicy` map into the desktop `managedMcpServers` policy from the tool list
the server reported to the bridge's auth probe (`metadata/mcp-tools.json`).

Every enabled server must declare `tool_policy` on its deployment — `allow`
(the value every server here carries), `prompt` or `deny`. A server without it
is withheld from the signed manifest and rejected at boot validation:

```yaml
mcp_servers:
  example-tools:
    tool_policy: prompt   # allow | prompt | deny; required
```

The decision travels in the signed manifest (`ManagedMcpServer.tool_policy`
under the `*` key), so a change here reaches every bridge on its next sync.
