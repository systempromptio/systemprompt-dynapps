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
| `anthropic` | customer | Anthropic API key | the image's gateway routes `claude-*` to `anthropic`; keys map by the `api_key_secret` name in `services/ai/providers.yaml`. |

## Optional

| key | effect when set |
|---|---|
| `knowledge_database_url` | keeps the knowledge index on a separate pgvector database instead of the instance database. Same URL format as `database_url`; that database needs `CREATE EXTENSION vector`. |
| `bedrock_api_key` | AWS Bedrock long-term API key, bearer, with model access in `eu-central-1`. Read **only** when `services/knowledge-bank/config.yaml` sets `model.provider: bedrock`; with that setting the knowledge-bank server refuses to boot without it, which fails the whole node. The shipped `local` provider ignores it (`docs/install/knowledge-model-provider.md`). |
| `database_write_url` | read/write split: reads use `database_url`, writes use this. Phase 2 only; cross-region replica lag breaks read-after-write for sessions. Leave unset at launch. |
| `vertex` | Vertex AI express-mode API key for the `vertex` provider (`gemini-*` routes, sent as `x-goog-api-key`). A route whose provider has no key is unusable, not fatal. |
| `vertex_maas` | bearer credential for the `vertex-maas` provider (`qwen.*`, `zai.*`, `moonshotai.*`, `deepseek.*`, `openai.gpt-oss-*`). Vertex Model-as-a-Service accepts a Google OAuth access token here, not an API key — see `docs/install/vertex.md`. |
| `smtp_host`, `smtp_port`, `smtp_username`, `smtp_password`, `smtp_from` | outbound e-mail (magic links, invitations). Without them invitations are printed to the log. |
| `slack_signing_secret`, `slack_bot_token` | inbound `/systemprompt` slash command and outbound alerts (`services/slack/astound.yaml`). |

## Injection

The platform reads the file at boot only (`secrets.source: file`). Render it
on each node from your vault at deploy time, before `docker compose up`:

```bash
install -d -m 0750 -o 1000 -g 1000 /etc/astound/profile
vault-cli read astound/prod/secrets.json > /etc/astound/profile/secrets.json   # OCI Vault, HashiCorp, SOPS — your choice
chown 1000:1000 /etc/astound/profile/secrets.json && chmod 0600 /etc/astound/profile/secrets.json
```

The container runs as uid 1000 and mounts the directory read-only. A change
to the file takes effect on the next `docker compose restart app`.

## Rotation

| key | procedure |
|---|---|
| provider keys, `bedrock_api_key`, SMTP, Slack | update the file on all nodes, rolling restart. No user impact. |
| `database_url` password | create the new role or password, update the file, rolling restart, drop the old. |
| `signing_key_pem` | update all nodes, restart **all** nodes within the same window; every session and PAT is invalidated, bridge clients re-link with a new code. |
| `oauth_at_rest_pepper`, `manifest_signing_secret_seed` | treat as non-rotatable in practice: rotating the pepper destroys stored OAuth tokens, rotating the seed forces every client re-sync. |
