# Centralized MCP connections

The Dynapps portal marketplaces expose the portal's hosted servers, Odoo
(`dynapps-odoo`) and DynBrain (`dynbrain`), through the Systemprompt MCP
gateway. Each employee links their own portal.dynapps.es account on
`/admin/connectors`. The bridge displays the same server-owned
connection model in its Profile view; **Manage on server** opens that page.
Provider credentials are never placed in a marketplace bundle or desktop config.

## Local development

Edit core in the persistent sibling `../systemprompt-core` checkout. Both server
Cargo workspaces and the bridge resolve through that path directly. Build and
test from this repo. `bridge/CORE_REF` pins the committed
core revision for CI. The bridge's `.cargo/config.toml` includes the `bridge/web`
account UI overlay.

## Provisioning

Connector enablement uses `mcp_servers.<provider>.enabled` in
`services/mcp/dynapps-odoo.yaml` and `services/mcp/dynbrain.yaml`. Both are
`true` and included by `services/config/config.yaml`. The Account page, bridge
manifest, and credential accessor read the same loaded services configuration.
Restart after changes. There is no separate enablement flag in secrets or
environment variables.

Store the application settings below in the active server profile's secret store
or equivalent uppercase environment variables. Never put credentials in service
YAML. Reuse the instance's existing `encryption_master_key`; do not rotate it for
this integration. `oauth_at_rest_pepper` hashes OAuth identifiers and cannot
replace the encryption key used to recover provider tokens.

| Setting | Value |
| --- | --- |
| `encryption_master_key` | Persistent 32-byte encryption key encoded as 64 hexadecimal characters; required to store OAuth grants, preserve it across restarts |
| `mcp_credential_broker_secret` | Random server-only secret of at least 32 bytes; shared by core and the account accessor in the same instance |

Set `server.api_external_url` to the HTTPS origin of the deployment and provision
its credentials independently. Local user grants are not copied into production.
Both servers use the generic OAuth connector (`connector.adapter: generic`): the
portal publishes protected-resource metadata and supports dynamic client
registration, so no client secret is stored. Callbacks are registered under that
origin:

- `https://YOUR-SERVER/api/public/connectors/dynapps-odoo/callback`
- `https://YOUR-SERVER/api/public/connectors/dynbrain/callback`

The registry definitions stay enabled because enabled marketplace plugins must
resolve their server references. The manifest filter independently requires
provider configuration, marketplace entitlement, and a verified connection before
publishing any connector to the employee. A registered server is not proof of a
working provider connection.

## Portal account check

A portal.dynapps.es login is required; the portal refuses consent for anyone
without one. For the local profile, use `http://localhost:8080/admin/connectors`.
The callback is `http://localhost:8080/api/public/connectors/<server>/callback`.
Use `localhost` consistently for login and consent so the session cookie is
available when the portal redirects back. If the server runs on a remote development
machine, forward port 8080 to the browser's machine before starting consent.
After consent, use **Test** on the row to list the tools the account can call.

## Acceptance checks

1. Use two entitled users with different provider permissions and one non-entitled user.
2. Link accounts on the server. Check the same identities and states in the bridge.
3. Sync a clean Claude Code installation. Verify the connectors with `/mcp` and
   make a read-only request against each provider, without provider CLI login.
4. Enroll another machine as the same employee. Sync and repeat without consent.
5. Disconnect an account. Requests must fail immediately; both account views
   refresh within 15 seconds while visible, with a 30-second bridge background
   refresh. Manifest sync follows availability changes.
6. Exercise expired grants, provider outages, callback replay/account switches,
   concurrent refresh, and non-entitled denial. An outage preserves the grant.
7. Check manifests and local config for absence of provider tokens. Check audit
   identity and trace IDs for tool calls and governance denials.

There is no fallback to a personal API token, no automatic switch to shared
application identity and no direct desktop access.

## Verification and deployment

Failed account verification preserves encrypted grants for retry while blocking
connector use until verification succeeds. Tenant identity must match the
provider's authenticated resource list.

Validate bridge synchronization, ordinary-user access, multiple devices,
refresh, disconnect and provider outages for the deployment being released.
Verify MCP session continuity across replicas before using multiple application
instances. A single successful account check does not establish these properties.

See [configured connectors](../CONFIGURED-CONNECTORS.md) for OAuth configuration.
