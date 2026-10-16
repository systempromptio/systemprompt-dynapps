# secrets.json — every key, who owns it, what breaks without it

`secrets.json` sits beside `profile.yaml` in the profile directory that is
bind-mounted into every node. It is one flat JSON object. **All nodes must
carry the same file byte for byte** — three of the keys below are the shared
cryptographic identity of the cluster, and a node with a different value mints
tokens the other two reject.

## Required

| key | owner | format | why |
|---|---|---|---|
| `database_url` | customer DBA | `postgres://user:pw@host:5432/systemprompt?sslmode=require` | the primary. Never a read replica: boot runs `pg_is_in_recovery()` and refuses a standby. |
| `oauth_at_rest_pepper` | generated once by `render-profile.sh` | ≥ 32 random chars | at-rest encryption of stored OAuth material. Changing it orphans every stored token. |
| `manifest_signing_secret_seed` | generated once | standard base64 of exactly 32 random bytes (44 chars) | signs the plugin manifests the bridge verifies. Changing it forces every client to re-sync. |
| `signing_key_pem` | generated once | standard base64 (one line) of an RSA-2048 PKCS#8 PEM file | signs every JWT (sessions, PATs, bridge tokens). Rotating it signs everyone out. |
| `anthropic` | customer | Anthropic API key | the `anthropic` provider, which the image's `claude-*` route fails over to from `vertex-anthropic`; keys map by the `api_key_secret` name in `services/ai/providers.yaml`. |

## Optional

| key | effect when set |
|---|---|
| `database_write_url` | read/write split: reads use `database_url`, writes use this. Phase 2 only; cross-region replica lag breaks read-after-write for sessions. Leave unset at launch. |
| `vertex` | Google service-account JSON (`roles/aiplatform.user`) for **every** Vertex provider: `vertex` (`vertex-gemini-*`), `vertex-anthropic` (the primary for `claude-*`), `vertex-maas` and `vertex-maas-us-central1` (`qwen.*`, `zai.*`, `moonshotai.*`, `deepseek.*`, `minimax.*`, `openai.gpt-oss-*`). One key, one GCP project; there is no separate `vertex_maas` secret — see `docs/install/vertex.md`. A route whose provider has no key is unusable, not fatal. |
| `gemini` | Gemini API key for the `gemini` provider (`gemini-*`, sent as `x-goog-api-key`); the classifier job dispatches here. |
| `dynapps_release_token` | the bridge update feed. `gateway.yaml` names it `token_secret: DYNAPPS_RELEASE_TOKEN`; core looks a custom secret up by that exact key and then by its lowercase form, so this lowercase key in `secrets.json` is what it finds (an uppercase key works too). `services/ai/gateway.yaml` points `bridge_releases` at the private `systempromptio/systemprompt-dynapps` repo, so the gateway needs a GitHub token to list releases, read `SHA256SUMS`, and proxy the installer bytes. **Dynapps issues this token** — a fine-grained PAT scoped to that one repository with `Contents: Read-only`, never a classic PAT. Without it `/v1/bridge/latest` answers 503 `bridge release token secret DYNAPPS_RELEASE_TOKEN is not configured`, and every bridge shows an update-check error every 6 h and on each wake-from-sleep; nobody can self-update. An **expired or revoked** token reads differently — 502 `github returned 401` — and the fix is always to rotate it: issue a new fine-grained PAT, replace `dynapps_release_token` on every node and restart (see Rotation). Record the renewal date. |

## Injection

The platform reads the file at boot only (`secrets.source: file`). Render it
on each node from your vault at deploy time, before `docker compose up`:

```bash
install -d -m 0750 -o 1000 -g 1000 /etc/dynapps/profile
vault-cli read dynapps/prod/secrets.json > /etc/dynapps/profile/secrets.json   # OCI Vault, HashiCorp, SOPS — your choice
chown 1000:1000 /etc/dynapps/profile/secrets.json && chmod 0600 /etc/dynapps/profile/secrets.json
```

The container runs as uid 1000 and mounts the directory read-only. A change
to the file takes effect on the next `docker compose restart app`.

## Rotation

| key | procedure |
|---|---|
| provider keys, `dynapps_release_token` | update the file on all nodes, rolling restart. No user impact. |
| `database_url` password | create the new role or password, update the file, rolling restart, drop the old. |
| `signing_key_pem` | update all nodes, restart **all** nodes within the same window; every session and PAT is invalidated, bridge clients re-link with a new code. |
| `oauth_at_rest_pepper`, `manifest_signing_secret_seed` | treat as non-rotatable in practice: rotating the pepper destroys stored OAuth tokens, rotating the seed forces every client re-sync. |

## 1Password

Instead of rendering `secrets.json`, the platform can read every key straight
from a 1Password vault at boot and re-read it on a schedule. The file above
is then not used at all; the key table still says what each key must hold.

**Service account (`backend: cli`, the default).** Create a 1Password service
account with read access to one vault only (`dynapps-prod` below), and give
the container its token as `OP_SERVICE_ACCOUNT_TOKEN`. The image ships the
pinned `op` CLI (`/usr/local/bin/op`, root `Dockerfile` stage `opcli`); the
platform spawns it with a cleared environment, passes the token through the
environment only — never on the command line — and reads each distinct item
once with `op item get --format json`, falling back to `op read op://…` per
reference when an item cannot be read whole.

```yaml
# profile.yaml
secrets:
  source: external
  external:
    kind: onepassword
    refresh_secs: 300          # re-read every 5 min; omit for boot-only (floor 60)
    backend: cli
    vault: dynapps-prod
    items:                     # secrets.json key -> op://<vault>/<item>/<field>
      anthropic: op://dynapps-prod/platform/anthropic
      entra_oidc_client_secret: op://dynapps-prod/identity/entra_oidc_client_secret
      encryption_master_key: op://dynapps-prod/identity/encryption_master_key
```

```bash
# docker-compose / systemd: the token is the only secret left on the node
OP_SERVICE_ACCOUNT_TOKEN=ops_...   # from your bootstrap secret store, mode 0600
```

Each `op://` reference names its own vault; `vault:` is the vault `item:`
mode reads and the one the source reports itself by. Instead of `items:`,
`item: <title>` maps every CONCEALED or STRING field of
that one item to the key named by the field's label. Fields resolve by label,
then id; an item title or field label that matches twice is an error, never a
guess — reference it by id.

**Connect server (`backend: connect`).** For a self-hosted 1Password Connect
server, set `backend: connect`, `connect_url: https://op-connect.internal:8443`,
and provide the Connect token as `OP_CONNECT_TOKEN` (or name another variable
with `token_env`). Requests time out after `timeout_secs` (default 10, max
120), retry up to `retries` times (default 3, max 10) on 429/5xx/connection
errors, never on 401/403/404, and never follow redirects.

A literal `token:` in the profile is refused at boot: the profile names the
variable (`token_env`), the environment holds the value.

**Refresh and rotation.** With `refresh_secs`, the `secrets_refresh` job
reloads on that cadence; `/admin/configuration` shows the source, generation,
last reload and last error with key names (never values), and an admin can
**Reload now**. A changed provider key takes effect on the next reload with no
restart. Keys that are pinned for the life of the process (`encryption_master_key`,
`oauth_at_rest_pepper`, `manifest_signing_secret_seed`, `database_url`,
`signing_key_pem`, `jwt_secret`) are
refused on reload (core's `PINNED_SECRET_KEYS`) — the old secrets stay live and the reload is reported as
failed — so those still follow the rotation table above, with a restart.
`entra_oidc_client_secret` is read at every OIDC code exchange, so a reload
reaches the next sign-in. Anything an MCP server uses still needs a restart,
since each server is handed its secrets when it is spawned. `admin config secret set` refuses an external profile; check
the source with `admin config secret check`. Runbook:
`services/content/documentation/secrets-management.md`
(`/documentation/secrets-management`).

**Kubernetes.** The Helm chart's `secrets.mode: onepassword` renders this
block from `secrets.onepassword.*` and injects the token from the Secret named
in `secrets.onepassword.tokenSecret`; no `secrets.json` is mounted.
