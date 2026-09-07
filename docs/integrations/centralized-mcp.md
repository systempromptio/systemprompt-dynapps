# Centralized MCP pilot

The India Development marketplace exposes Atlassian, GitHub and Salesforce
through the Systemprompt MCP gateway. Each employee links their own account on
`/admin/profile#connected-accounts`. The bridge displays the same server-owned
connection model in its Profile view; **Manage on server** opens that page.
Provider credentials are never placed in a marketplace bundle or desktop config.

## Local development

Both Cargo workspaces resolve core from the persistent sibling
`../systemprompt-core` checkout. The bridge already depends on that checkout.
Build and test from this repo. Do not copy edits into `.vendor` for local builds.
The bridge's `.cargo/config.toml` includes the `bridge/web` account UI overlay.
Container release builds require published core dependencies or an explicit
vendoring step; sibling paths are deliberately for local development.

## Provisioning

Store values in the active server profile's secret store or the equivalent
uppercase environment variables. Never put credentials in service YAML.

| Setting | Value |
| --- | --- |
| `mcp_credential_broker_secret` | Random server-only secret of at least 32 bytes; shared by core and the account accessor in the same instance |
| `atlassian_mcp_enabled` | `true` when ready to test Atlassian authorization |
| `github_mcp_enabled` | `true` when ready to test GitHub authorization |
| `github_mcp_client_id`, `github_mcp_client_secret` | Registered GitHub App OAuth credentials, with expiring user tokens enabled |
| `salesforce_mcp_enabled` | `true` when ready to test Salesforce authorization |
| `salesforce_mcp_domain` | HTTPS My Domain origin, for example `https://example.my.salesforce.com` |
| `salesforce_mcp_client_id`, `salesforce_mcp_client_secret` | Salesforce External Client App credentials |
| `salesforce_private_key` | Existing JWT-bearer signing key when using admin pre-authorization |

Register browser callbacks as:
`<server origin>/api/public/connectors/<provider>/callback`.
Atlassian uses its hosted MCP authorization-server metadata and dynamic client
registration. Its organization must allow the server callback domain and,
where IP allowlisting is enabled, the server's outbound IP.

Salesforce Hosted MCP requires an **External Client App**, `mcp_api`,
`refresh_token`, and named-user JWT access tokens. This integration also uses
`openid` to verify the account and organization. For administrative provisioning,
map the user's Salesforce username through the existing admin identity endpoint
and configure pre-authorization and the application's signing certificate.
The Account page's **Test connection** verifies that path. Browser OAuth is the
alternative. Disconnect prevents silent re-provisioning until the user reconnects.

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
