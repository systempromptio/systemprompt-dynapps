# Gateway routes: providers, CLI configuration, and access control

`POST /v1/messages` at the Anthropic wire format. Every inference request flows through the same governance pipeline as every tool call, on infrastructure you operate.

- **SDK- and Claude-Desktop-compatible.** Authenticated with a systemprompt JWT in `x-api-key` (falls back to `Authorization: Bearer`). No new credential type — existing user JWTs serve as the gateway credential.
- **Routes by declared model.** A requested model resolves to its deployments, each a `provider/endpoint`, with no pattern matching. Endpoint wires: `anthropic`, `openai-chat`, `openai-responses`, `gemini`. An `anthropic` endpoint is a transparent byte proxy (extended thinking, cache-control headers, SSE events preserved verbatim); other wires get full request/response/SSE conversion. Platform keys resolve from the secrets file by name; personal keys are stored per member.
- **Zero overhead when disabled.** The `/v1` router mounts only if `gateway.enabled: true` in `services/ai/gateway.yaml`.

## Services YAML

The AI catalog is services configuration, shipped in the image beside the agents and MCP servers, as five files under `services/ai/`, all listed in `services/config/config.yaml` `includes:`. They are not profile sections: a profile that carries `providers:` or `gateway:` fails to load with a message naming these files. Nothing in them varies per environment; the platform secrets they name live in each environment's secret store.

```yaml
# credentials.yaml: who authenticates and who pays
credentials:
  anthropic:   { kind: platform, secret: anthropic }
  deepseek:    { kind: platform, secret: deepseek }
  opencode-go: { kind: personal, label: OpenCode Go }

# providers.yaml: the vendor account and its endpoints (one wire + one URL each)
providers:
  anthropic:
    credential: anthropic
    endpoints:
      api: { wire: anthropic, url: https://api.anthropic.com/v1 }
  opencode-go:
    credential: opencode-go
    endpoints:
      chat:     { wire: openai-chat, url: https://opencode.ai/inference/go/openai/v1 }
      messages: { wire: anthropic,   url: https://opencode.ai/inference/go/anthropic/v1 }

# models.yaml: each model once; `clients` replaces any per-surface listing
models:
  claude-sonnet-5-5:
    limits: { context_window: 1000000, max_output_tokens: 128000 }
    clients: [claude]
  deepseek-v4-pro:
    clients: [openai]

# deployments.yaml: model -> ordered endpoints (routing and failover)
deployments:
  claude-sonnet-5-5: [anthropic/api]
  deepseek-v4-pro:   [deepseek/api]
  go-deepseek-v4-pro: [opencode-go/chat]

# gateway.yaml: one default model; routes are policy only
gateway:
  enabled: true
  default_model: claude-sonnet-5-5
  routes:
    - id: claude-eu
      models: [claude-*]
      deployments: [anthropic/api]
```

A credential is `platform` (a profile secret) or `personal` (each member stores their own key once, however many endpoints it serves). A model's id is unique across the catalog; `upstream_model` names it upstream when that differs. A deployment entry is `provider/endpoint` or an object with `endpoint:` plus per-deployment `upstream_model` and `pricing` overrides. Failover never crosses from a platform credential to a personal one.

A route carries a declared, stable `id` and only does policy: an `alias` (a requested id resolves to a catalog model), a pin (`deployments:` narrows the chain), or a narrowing by `models:` glob and `when:` scope. `default_model` is the bare catalog id; Claude Code's `[1m]` marker is derived from `limits.context_window` and stripped by the gateway, never written in config.

**Hosting is read from the endpoint URL.** An endpoint on `aiplatform.googleapis.com` is Vertex AI. For those, the credential's secret is a Google service account, which is exchanged for an OAuth token and fills `{project}` in the URL. The request envelope follows the platform: Claude on Vertex (`wire: anthropic`, `…/publishers/anthropic`) is posted to `/models/<id>:rawPredict` or `:streamRawPredict`, with `anthropic_version` in the body. The gateway and the in-process AI service share this resolution. This installation serves `claude-*` from Anthropic's API only.

## Configuring the catalog from the CLI

Worked example: serve a new model from a new provider. Instead of hand-editing the services files, use `admin config`, which edits them and validates against the merged catalog:

```bash
# 1. Store the platform key and declare the credential, provider and endpoint
systemprompt admin config secret set gemini <GEMINI_API_KEY>
systemprompt admin config credential add --id gemini --kind platform --secret gemini
systemprompt admin config provider add --id gemini --credential gemini \
  --endpoint api --wire gemini --url https://generativelanguage.googleapis.com/v1beta

# 2. Declare the model with its deployment
systemprompt admin config model add --id gemini-3.8-flash --deployment gemini/api

# 3. Optionally alias a requested id to it
systemprompt admin config gateway route add --id flash-alias \
  --alias fast=gemini-3.8-flash
```

`admin config deployment set --model <id> --endpoint <provider/endpoint> …` replaces a model's chain in failover order, and `admin config gateway default-model set --model <id>` sets the default. Then run `systemprompt admin config apply --reason "<why>"` so access control and gateway policies are written to the database.

## Client attribution

Every `/v1/messages` row names the client that sent it (`ai_requests.client_kind`) and how
strongly that is evidenced (`ai_requests.client_attestation`). Hosts the bridge provisions —
`claude-code`, `claude-desktop`, `codex` (desktop app), `codex-cli`, `opencode` (desktop app), `opencode-cli` and `hermes` — are attested by the bridge's per-host
token with nothing to configure; a direct client (Pi, a raw SDK, curl) declares itself with
`x-systemprompt-client: <kind>` from the closed vocabulary, or is recorded as `other` with its
User-Agent and SDK headers kept as evidence. The full tier table, the header names, the 400
behaviour for a malformed declaration, and copy-paste SDK examples are on the hosted page:
[Gateway → Client attribution](https://systemprompt.io/documentation/services/gateway#client-attribution).
On this instance the tier is the tone of the client chip on every conversation page, and
`systemprompt infra logs request list` shows it as the `attestation` column.

## Routes are access-controlled

Each route is gated by an `access_control_entities` row keyed on its declared id. Renaming a route's id is therefore a new entity, denied (`unknown to access control`) until the access-control plane is applied. Boot never seeds or reconciles any plane; it happens in one place:

- **`systemprompt admin config apply --reason <why>`** — reconciles the catalog from the loaded services config, then writes `services/access-control/rules.yaml`, governance and gateway policies to the database. Run it after `admin config migrate` and the migrations, and after any route edit. `/admin/sync` reports drift between code and the database.

Routes are granted by `entity_match`, never by a literal `entity_id`:

```yaml
- entity_type: gateway_route
  entity_match: "*"
  access: allow
  default_included: true
  roles: [user]
```

A route that needs a narrower grant gets a glob over its declared id (`entity_match: "go-*"`).

## Extensible provider registry

`GatewayRoute.provider` is a free-form string resolved at dispatch time against a startup-built registry. Extension crates register new upstreams with:

```rust
inventory::submit! {
    systemprompt_api::services::gateway::GatewayUpstreamRegistration {
        tag: "my-provider",
        factory: || std::sync::Arc::new(MyUpstream),
    }
}
```

The `GatewayUpstream` trait (`async fn proxy(&self, ctx: UpstreamCtx<'_>)`) is the single integration seam. Built-in tags seeded automatically; extension tags may shadow built-ins (logged as a warning). Full detail: [`core/CHANGELOG.md`](https://github.com/systempromptio/systemprompt-core/blob/main/CHANGELOG.md#030---2026-04-22).

## Bridge self-update feed

The desktop bridge updates itself through this gateway. Release assets live in a
private GitHub repository that the bridge holds no credential for, so the
gateway resolves the newest `bridge-v*` release and proxies the bytes:

| Route | Purpose |
|-------|---------|
| `GET /v1/bridge/latest?platform=<slug>` | Version, SHA-256, size, and release-notes URL for the newest published build |
| `GET /v1/bridge/download/{platform}` | Streams that platform's asset |

Both are authenticated exactly like the other `/v1/bridge/*` routes. Platform
slugs are `macos`, `windows`, `linux-x86_64`, and `linux-aarch64`.

```yaml
gateway:
  enabled: true
  bridge_releases:
    repo: systempromptio/systemprompt-internal
    # Named, not inlined, so the token never lands in a config file or a
    # profile dump. Needs `contents: read` on the repo.
    token_secret: SYSTEMPROMPT_BRIDGE_RELEASES_TOKEN
    tag_prefix: bridge-v
    assets:
      macos: systemprompt-internal-bridge-macos.zip
      windows: systemprompt-internal-bridge-windows.exe
      linux-x86_64: systemprompt-internal-bridge-linux-x86_64.tar.gz
      linux-aarch64: systemprompt-internal-bridge-linux-aarch64.tar.gz
```

macOS points at the `.zip`, not the `.dmg`: the updater unpacks it with `ditto`
and verifies the bundle's signature before swapping it into `/Applications`.
The `.dmg` remains what the admin Bridge Setup page hands to humans. Asset names
must match `.github/workflows/bridge-release.yml` exactly.

The advertised `sha256` is read from the release's `SHA256SUMS` — generated and
cosign-signed by the release workflow — rather than computed here, so the digest
the updater enforces is the one that was signed at publish time. A download that
does not match it is discarded and never executed.

**Omitting `bridge_releases` disables updates rather than breaking them.** The
endpoints answer `404`, and the bridge treats a failed check as a debug log:
the button stays as it is and no error reaches the user.

**Staged rollouts and pinning** are config, not a client release. Set
`pinned_version: 0.1.6` to hold a fleet on one build; remove it to resume
tracking the newest release.
