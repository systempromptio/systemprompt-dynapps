# Centralized MCP connections

The India Development marketplace exposes Atlassian, GitHub and Salesforce
through the Systemprompt MCP gateway. Each employee links their own account on
`/admin/profile#connected-accounts`. The bridge displays the same server-owned
connection model in its Profile view; **Manage on server** opens that page.
Provider credentials are never placed in a marketplace bundle or desktop config.

## Local development

Edit core in the persistent sibling `../systemprompt-core` checkout, then run
`just vendor-core` to refresh the build copy. Both server Cargo workspaces resolve
through `.vendor/systemprompt-core` so container builds include the same source.
The bridge uses the sibling checkout directly. Build and test from this repo;
never make persistent edits in `.vendor`. `bridge/CORE_REF` pins the committed
core revision for CI. The bridge's `.cargo/config.toml` includes the `bridge/web`
account UI overlay.

## Provisioning

Connector enablement uses `mcp_servers.<provider>.enabled` in
`services/mcp/atlassian.yaml`, `services/mcp/github.yaml`, and
`services/mcp/salesforce.yaml`. All three are `true` and included by
`services/config/config.yaml`. The Account page, bridge manifest, and credential
accessor read the same loaded services configuration. Restart after changes.
There is no separate enablement flag in secrets or environment variables.

Store the application settings below in the active server profile's secret store
or equivalent uppercase environment variables. Never put credentials in service
YAML. Reuse the instance's existing `encryption_master_key`; do not rotate it for
this integration. `oauth_at_rest_pepper` hashes OAuth identifiers and cannot
replace the encryption key used to recover provider tokens.

| Setting | Value |
| --- | --- |
| `encryption_master_key` | Persistent 32-byte encryption key encoded as 64 hexadecimal characters; required to store OAuth grants, preserve it across restarts |
| `mcp_credential_broker_secret` | Random server-only secret of at least 32 bytes; shared by core and the account accessor in the same instance |
| `github_mcp_client_id`, `github_mcp_client_secret` | Registered GitHub App OAuth credentials, with expiring user tokens enabled |
| `salesforce_mcp_domain` | HTTPS My Domain origin, for example `https://example.my.salesforce.com` |
| `salesforce_mcp_client_id`, `salesforce_mcp_client_secret` | Salesforce External Client App credentials |
| `salesforce_private_key` | Existing JWT-bearer signing key when using admin pre-authorization |

Set `server.api_external_url` to the HTTPS origin of the deployment and provision
its credentials independently. Local user grants are not copied into production.
Register callbacks under that origin:

- `https://YOUR-SERVER/api/public/connectors/atlassian/callback`
- `https://YOUR-SERVER/api/public/connectors/github/callback`
- `https://YOUR-SERVER/api/public/connectors/salesforce/callback`

Atlassian uses its hosted MCP authorization-server metadata and dynamic client
registration. Its organization must allow the server callback domain and,
where IP allowlisting is enabled, the server's outbound IP.

Salesforce's first test uses browser OAuth. An org administrator activates
`sobject-all` in Setup → MCP Servers → Salesforce
Servers, then creates an **External Client App** with the callback URLs above,
PKCE, `mcp_api` and `refresh_token` scopes, and JWT-based access tokens for named
users. This integration also requests `openid` to verify account identity. Set
`salesforce_mcp_domain`, `salesforce_mcp_client_id`, and
`salesforce_mcp_client_secret` on the server and authorize a test user with the
required object and field access. No signing certificate or private key is needed
for browser OAuth. New app settings can take up to 30 minutes to become available.
See [Salesforce's app setup guide](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/create-external-client-app.html).

The current service and verification endpoints target a production Salesforce
org. Sandbox orgs require `/platform/mcp/v1/sandbox/platform/sobject-all`; both
endpoints must be updated before sandbox testing. Local Systemprompt can connect
to either org type once the endpoint is correct. See
[Salesforce's endpoint documentation](https://developer.salesforce.com/docs/platform/hosted-mcp-servers/guide/postman.html).
The `sobject-all` connector covers record operations and queries, not every
Salesforce development or deployment workflow.

Optional administrative provisioning instead maps the user's Salesforce username
through the existing admin identity endpoint and uses pre-authorization plus the
application's signing certificate. **Test connection** verifies that path.
Disconnect prevents silent re-provisioning until the user reconnects.

The registry definitions stay enabled because enabled marketplace plugins must
resolve their server references. The manifest filter independently requires
provider configuration, India entitlement, and a verified connection before
publishing any connector to the employee. A registered server is not proof of a
working provider connection.

## Atlassian account check

An Atlassian login alone is not proof of Jira/Confluence access. Open Jira from
Atlassian Home and confirm an existing project loads. The supplied pilot site is
`https://astounddigital.atlassian.net`, project `EAICOMM`.

Once this build is running, open Account → Connected accounts → Atlassian →
Connect. Enter `https://astounddigital.atlassian.net` in the site address field
and approve the site in Atlassian's consent screen. The
server checks that it belongs to the authorized resource list. Then use
**Test connection** and sync the bridge.

For the local profile, use `http://localhost:8080/admin/profile#connected-accounts`.
The callback is `http://localhost:8080/api/public/connectors/atlassian/callback`.
Use `localhost` consistently for login and consent so the session cookie is
available when Atlassian redirects back. If the server runs on a remote development
machine, forward port 8080 to the browser's machine before starting consent.

A denied OAuth redirect suggests the application's domain is not allowed.
Consent followed by denied tool access can indicate an IP restriction or missing
product permissions. An organization administrator can inspect Atlassian
Administration → Rovo → Rovo MCP server. Do not create a new organization merely
to connect an existing employee account.

## Acceptance checks

1. Use two India users with different provider permissions and one non-India user.
2. Link accounts on the server. Check the same identities and states in the bridge.
3. Sync a clean Claude Code installation. Verify the connectors with `/mcp` and
   make a read-only request against each provider, without provider CLI login.
4. Enroll another machine as the same employee. Sync and repeat without consent.
5. Disconnect an account. Requests must fail immediately; both account views
   refresh within 15 seconds while visible, with a 30-second bridge background
   refresh. Manifest sync follows availability changes.
6. Exercise expired grants, provider outages, callback replay/account switches,
   concurrent refresh, and non-India denial. An outage preserves the grant.
7. Check manifests and local config for absence of provider tokens. Check audit
   identity and trace IDs for tool calls and governance denials.

GitHub PAT and Atlassian personal API tokens can be submitted through the Account
page as explicit fallbacks. They are verified and encrypted on the server. There
is no automatic switch to shared application identity or direct desktop access.

## Verification and deployment

Failed account verification preserves encrypted grants for retry while blocking
connector use until verification succeeds. Tenant identity must match the
provider's authenticated resource list.

Validate bridge synchronization, ordinary-user access, multiple devices,
refresh, disconnect and provider outages for the deployment being released.
Verify MCP session continuity across replicas before using multiple application
instances. A single successful account check does not establish these properties.

See [configured connectors](../CONFIGURED-CONNECTORS.md) for OAuth configuration.
