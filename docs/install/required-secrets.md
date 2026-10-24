# Required secrets

Everything an Dynapps operator must provide for Claude Code and OpenCode to
work end to end.

**Claude on this installation is served only by the customer's Anthropic key
(customer requirement).** The platform model secrets are `anthropic` and
`deepseek`; OpenCode Go uses each member's own key, which is not a secret the
operator provides. The `vertex` and `gemini` credentials are not declared and
their secrets are removed; the rest of this page describes them for a restore
(declare the credential, provider and endpoint, set its secret, add the
deployments).

Each platform credential in
[`services/ai/credentials.yaml`](../../services/ai/credentials.yaml) names the
secret it reads its key from, and each provider in `services/ai/providers.yaml`
names its credential; the profile secret store supplies the value per
environment. The catalog is shipped in the image and is identical in
every environment — only the secrets differ.

Credential issuing for the Google project is covered in more detail in
[vertex.md](vertex.md).

## The secrets

| secret | credential | providers | unlocks |
|---|---|---|---|
| `anthropic` | `anthropic` (platform) | `anthropic` | every `claude-*` model (`claude-opus-5-5`, `claude-sonnet-5-5`, `claude-haiku-5-5`) |
| `deepseek` | `deepseek` (platform) | `deepseek` | `deepseek-v4-pro` (first deployment) and `deepseek-flash` |
| none (per member) | `opencode-go` (personal) | `opencode-go` | the `go-*` models, and `deepseek-v4-pro`'s second deployment; see [OpenCode Go](../../services/content/documentation/opencode-go-subscription.md) |
| `vertex` (not declared) | — | — | Claude, Gemini and the open models on Vertex AI, for a restore |
| `gemini` (not declared) | — | — | `gemini-3.8-flash` on the public Gemini API, for a restore |

One secret is the minimum: `anthropic` alone gives Claude Code a working
default, since `gateway.yaml` sets `default_model: claude-sonnet-5-5`. Claude
Code's 1M-context form (`claude-sonnet-5-5[1m]`) is derived from the model's
`limits.context_window` and is never written in config. Every other secret
adds models to the picker.

A model whose credential has no key is still advertised on `/v1/models` — that
endpoint filters by the model's `clients:`, not by credential — and fails at
dispatch naming the missing secret. A missing key is health state, never a
change to the catalog. Advertise only what you have credentialed, or expect
that error in the audit trail.

### Gemini: API key or Vertex, not both interchangeably

`gemini` and `vertex` are two credentials for the same Google models, each
with its own provider and endpoint.

- The **public endpoint** (`generativelanguage.googleapis.com`) takes an API key
  on `x-goog-api-key`. That is the `gemini` secret. Nothing else is needed.
- **Vertex AI** rejects API-key authentication as a class, with `401`
  `CREDENTIALS_MISSING`. It wants an OAuth access token asserting a principal.
  That is the `vertex` secret.

A model is declared once in `models.yaml`, so `gemini-3.8-flash` served both
ways is one model with two deployments (`gemini/api` and a `vertex` endpoint),
in failover order.

## The Google service account

Every Vertex endpoint uses the one `vertex` credential: a service-account JSON key
with `roles/aiplatform.user` on the Google project. Store the whole document
verbatim as the secret value. There is no separate `vertex_maas` secret.

```bash
systemprompt admin config secret set vertex "$(cat service-account.json)"
```

The gateway recognises a service-account key by the `type` field the document
declares about itself — `"service_account"` — which is Google's own
self-description rather than a guess about shape
(core `crates/entry/api/src/services/gateway/service/credentials/google.rs`). It signs an RS256 assertion
with the key, exchanges it for an access token under RFC 7523 on the
`https://www.googleapis.com/auth/cloud-platform` scope, and caches the token
until shortly before it expires. There is no flag to set. A secret that is not
such a document is sent verbatim as an API key, exactly as before.

### The project id comes from the service account, never the catalog

Every Vertex endpoint URL in `providers.yaml` is project-scoped and written with a
`{project}` segment. The gateway fills it from the `project_id` field of the
service-account key in the secret, so swapping the secret for another
customer's re-targets the endpoint with it and nothing in `services/` names a
project. A project id is a tenant identifier that Vertex echoes in every IAM
error the gateway relays, and this file ships in every image — the boot
validator refuses a catalog that names one literally
(`ProviderRegistryError::LiteralProjectInEndpoint`). An endpoint with
`{project}` paired with a plain API key is refused at dispatch: an API key
names no project and the gateway will not guess one.

Each MaaS model must also be enabled in Model Garden for that project. One that
is not enabled returns `404` to every user who picks it, regardless of the
credential.

## Setting a secret

```bash
systemprompt admin config secret set <name> <value>
```

`set` is the only subcommand. It writes into the secrets file of the **active
profile**, so switch profiles first when targeting a deployed environment
(`systemprompt admin session switch production`). Infrastructure secrets —
database URLs, the at-rest pepper, the signing seed — are refused by this
command and are provisioned out of band.

The secret store is loaded once at process start
(`SecretsBootstrap::get`), so a newly set secret takes effect on the next server
restart, not immediately.

## Verifying

**The catalog, unauthenticated.** `/v1/models` carries no auth layer, so this
needs no credential and proves only that the catalog loaded and the
`clients:` filter works:

```bash
curl -sS localhost:8080/v1/models | jq -r '.data[].id'
```

**A real dispatch, which is what actually proves a secret.** `/v1/messages`
needs a PAT *and* a minted session; an arbitrary `x-session-id` is rejected
rather than created on demand.

```bash
PAT=$(systemprompt admin users api-key issue --user <user-id> --name scratch \
        | grep -o 'sp-live-[^ ]*')
SID=$(curl -sS -X POST localhost:8080/api/public/gateway/sessions \
        -H "Authorization: Bearer $PAT" -H 'content-type: application/json' \
        -d '{}' | jq -r .session_id)

curl -sS -X POST localhost:8080/v1/messages \
  -H "Authorization: Bearer $PAT" -H "x-session-id: $SID" \
  -H 'content-type: application/json' -d '{
    "model":"claude-haiku-5-5","max_tokens":64,
    "messages":[{"role":"user","content":"ping"}]}' | jq '{stop:.stop_reason, usage}'
```

Substitute the model id for whichever secret you are checking. A credential
problem surfaces as a `PreAudit` dispatch error naming the secret; a project
mismatch surfaces as the Google `403` relayed through.

Every call above lands a row with the user, the model, the tokens and the cost:

```bash
systemprompt infra logs request list --limit 5
```

## How a developer reaches these models

Nothing changes on the developer's machine. The bridge install points Claude
Code at the gateway and enables gateway model discovery, so every credentialed
model appears in the `/model` picker beside the Claude models. OpenCode reads
the same catalog. A request names the catalog id (`claude-sonnet-5-5`, `deepseek-v4-pro`,
`go-kimi-k3`), the gateway sends it down that model's deployments in
[`services/ai/deployments.yaml`](../../services/ai/deployments.yaml),
rewrites it to the upstream name where `upstream_model` differs, and audits the
call.
