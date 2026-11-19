# Identity-backed external MCP servers

An external MCP server that another team hosts can trust **who is calling**
without running its own login. The gateway proxies every call. On each request
it signs a short-lived JWT for the calling user and sends it as the bearer.
The server checks that JWT against this instance's public keys.

The first such server is `sfcore_dev` (`services/mcp/sfcore_dev.yaml`).

## Flow

```
Claude Code / Cowork
  │  (bridge, signed in through Dynapps SSO (Microsoft Entra ID))
  ▼
Gateway  /api/v1/mcp/<server>/mcp
  │  1. authenticate the user (systemprompt JWT, live session)
  │  2. governance chain on tools/call: scope → secret scan → blocklist → rate limit
  │  3. GET /api/public/identity/<server>/token   (broker secret + user JWT)
  │       → rules.yaml entitlement for mcp_server/<server>
  │       → sign RS256 JWT {iss, aud=endpoint, sub, email, name, groups, iat, exp, jti}
  │  4. forward the MCP request with  Authorization: Bearer <that JWT>
  │     (the systemprompt JWT is never forwarded; the endpoint is never shown to clients)
  ▼
External server (e.g. Cloud Run)
  │  verify signature via /.well-known/jwks.json, check iss/aud/exp, read email
  ▼
  response → gateway → audit/trace row → client
```

A token is signed for every request and never cached. It expires after 300
seconds, so a revoked user or a withdrawn entitlement takes effect on the very
next call.

## Token contract

| part | value |
|---|---|
| header `alg` | `RS256` |
| header `kid` | the instance's active signing key id (published in JWKS) |
| header `typ` | `JWT` |
| `iss` | the instance issuer: `https://dynapps.systemprompt.io` in production |
| `aud` | the server's configured `endpoint`, exactly as written in its YAML |
| `sub` | the systemprompt user id, stable per user |
| `email` | the user's email |
| `name` | display name, or the username if there is none |
| `groups` | the ids of the groups the user belongs to right now, sorted; a user in no group carries `["unassigned"]` |
| `iat` / `exp` | issued-at and expiry; `exp = iat + 300` |
| `jti` | unique id per token |

The claim set is a contract. Adding a claim is compatible. Renaming or removing
one is not.

`groups` holds group **ids** as declared in `services/web/config/groups.yaml`
and managed in the console (`/admin/groups`), read from the same membership
the access rules use (`user_groups`), so an expired or revoked membership is
gone from the very next token. The user's
roles (`users.roles`) are not in the token: a server that needs to tell people
apart by what they may do there reads `groups`, and an administrator decides
membership in the console. A server that maps groups to its own roles should
treat a group it does not know as no role at all.

## Registering a server

1. Create `services/mcp/<id>.yaml` with `type: external`, the `endpoint`, a
   `tool_policy`, `oauth.required: false` and:
   ```yaml
   external_auth:
     token_endpoint: /api/public/identity/<id>/token
     header: Authorization
     scheme: Bearer
   ```
   The accessor signs only for a server whose `token_endpoint` names its own
   route. Any other id returns 404.
2. Add the file to `services/config/config.yaml` `includes:`.
3. Add an `mcp_server/<id>` entry with a `why` to
   `services/access-control/rules.yaml`, then apply it through `/admin/sync`
   (insert-only). This band is the entire access decision.
4. Add `<id>` to `IDENTITY_SERVERS` in
   `tests/unit/web/src/downstream_credential_configs.rs`.

Code: `extensions/web/admin/platform/src/services/identity_token.rs` (claims and
signing), `extensions/web/admin/platform/src/handlers/connector_auth/identity.rs`
(accessor). Contract tests:
`tests/contract/admin/src/identity_token_contract.rs`.
