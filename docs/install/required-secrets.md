# Required secrets

Everything an Astound operator must provide for Claude Code, OpenCode and
Gemini to work end to end. Each provider entry in
[`services/ai/providers.yaml`](../../services/ai/providers.yaml) names the
secret it reads its credential from; the profile secret store supplies the
value per environment. The catalog is shipped in the image and is identical in
every environment — only the secrets differ.

Credential issuing for the Google project is covered in more detail in
[vertex.md](vertex.md).

## The secrets

| secret | provider entry | credential | unlocks |
|---|---|---|---|
| `vertex` | `vertex`, `vertex-anthropic`, `vertex-maas`, `vertex-maas-us-central1` | Google service-account JSON | the six `claude-*` models (`claude-opus-5-5`, `claude-opus-5`, `claude-sonnet-5`, `claude-fable-5-1`, `claude-fable-5`, `claude-haiku-4-5`), the `vertex-gemini-*` models, and the open models (`zai.glm-5.2`, `deepseek.v3.2`, `moonshotai.kimi-k2-thinking`, `qwen.qwen3-coder-480b`, `openai.gpt-oss-120b` and the rest of `vertex-maas`) |
| `anthropic` | `anthropic` | Anthropic API key | the `anthropic-claude-*` twins, and failover for `claude-*` when Vertex is out of capacity |
| `gemini` | `gemini` | Gemini API key | `gemini-3.8-flash` (the classifier job) |

One secret is the minimum: `vertex` alone gives Claude Code and OpenCode a
working default, since `gateway.yaml` sets `default_provider: vertex-anthropic` and
`default_model: "claude-sonnet-5[1m]"` (Claude Code's 1M-context form of
`claude-sonnet-5`; other hosts get the bare id). Every other secret adds models to the picker.

A model whose provider has no secret is still advertised on `/v1/models` — that
endpoint filters by API surface, not by credential — and fails at dispatch with
`Gateway API key secret '<name>' not configured`. Advertise only what you have
credentialed, or expect that error in the audit trail.

### Gemini: API key or Vertex, not both interchangeably

`gemini` and `vertex` are two entries for the same Google models, differing in
credential and endpoint.

- The **public endpoint** (`generativelanguage.googleapis.com`) takes an API key
  on `x-goog-api-key`. That is the `gemini` secret. Nothing else is needed.
- **Vertex AI** rejects API-key authentication as a class, with `401`
  `CREDENTIALS_MISSING`. It wants an OAuth access token asserting a principal.
  That is the `vertex` secret.

The model ids are prefixed on the Vertex entry (`vertex-gemini-3.8-flash`) because
a model id must be unique across the whole catalog: `ProviderRegistry::validate`
hoists its `seen_models` set outside the per-provider loop, so two entries both
claiming `gemini-3.8-flash` fail at boot with `DuplicateModel`. Credential and
endpoint therefore travel together per entry.

## The Google service account

Every Vertex provider reads the one `vertex` secret: a service-account JSON key
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

Every Vertex endpoint in `providers.yaml` is project-scoped and written with a
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
needs no credential and proves only that the catalog loaded and the surface
filter works:

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
    "model":"vertex-gemini-3.8-flash","max_tokens":64,
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
the same catalog. A request names the catalog id (`qwen.qwen3-coder-480b`,
`vertex-gemini-3.8-flash`, `gemini-3.8-flash`), the gateway routes it by the
patterns in [`services/ai/gateway.yaml`](../../services/ai/gateway.yaml),
rewrites it to the upstream name where `upstream_model` differs, and audits the
call.
