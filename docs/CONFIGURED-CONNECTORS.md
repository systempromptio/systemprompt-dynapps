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

## Google Cloud RAG Engine (`google_rag`)

`services/mcp/google_rag.yaml` connects Google's hosted Agent Platform MCP
retrieval server for one RAG corpus. Google publishes protected-resource
metadata for the regional endpoint and OpenID metadata at
`accounts.google.com`, so the generic adapter applies, with two Google
specifics carried by the keys above: `authorization_params: {access_type:
offline, prompt: consent}` — Google issues a refresh token only when offline
access is requested, and without one the connection expires with the hour-long
access token — and `identity: userinfo` so the row shows the Google address.

On the Google side, in the GCP project that owns the corpus:

1. Enable `aiplatform.googleapis.com`; create the RAG corpus in `europe-west3`
   and import the agreed Drive folders (shared with the project's RAG Data
   Service Agent). Drive sharing is not re-checked at query time — the corpus is
   a copy, so one project holds one corpus for one readership.
2. Create an OAuth **Web application** client with an **Internal** consent
   screen and the redirect
   `https://YOUR-SERVER/api/public/connectors/google_rag/callback`. Store the
   pair as `google_rag_mcp_client_id` / `google_rag_mcp_client_secret`.
3. Grant the readership group `mcp.tools.call` and
   `aiplatform.ragCorpora.query` on the project (Google documents
   `roles/mcp.toolUser` + `roles/aiplatform.user`; a custom role with just the
   two permissions is narrower). Every member needs a Google Workspace
   identity; the Astound rule for `mcp_server/google_rag` decides who sees the
   row, Google IAM decides whose sign-in can query.
4. Turn on Data Access audit logs for `aiplatform.googleapis.com`.

Acceptance: connect with an account inside the group — the row reads
"Google Drive knowledge · Connected" with the address, Test lists
`retrieve_contexts`, and a call an hour later still succeeds (refresh path);
an account outside the group is refused by Google, not answered.

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
