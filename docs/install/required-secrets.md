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
| `anthropic` | `anthropic` | Anthropic API key | the eleven `claude-*` models, including `claude-opus-5`, `claude-sonnet-5` and `claude-fable-5-1` |
| `gemini` | `gemini` | Gemini API key | `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite` |
| `vertex` | `vertex` | Google service-account JSON | `vertex-gemini-2.5-pro`, `vertex-gemini-2.5-flash` |
| `vertex_maas` | `vertex-maas` | Google service-account JSON | the nine open models: four Qwen3, `zai.glm-5`, `moonshotai.kimi-k2-thinking`, `deepseek.v3.2`, `openai.gpt-oss-120b`, `openai.gpt-oss-20b` |
| `openai` | `openai` and `openai-responses` | OpenAI API key | `gpt-4.1`, `gpt-4.1-mini`, and `o4-mini` on the `/responses` wire |
| `cerebras` | `cerebras` | Cerebras API key | `gpt-oss-120b` |

One secret is the minimum: `anthropic` alone gives Claude Code and OpenCode a
working default, since `gateway.yaml` sets `default_provider: anthropic` and
`default_model: claude-sonnet-5`. Every other secret adds models to the picker.

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

The model ids are prefixed on the Vertex entry (`vertex-gemini-2.5-pro`) because
a model id must be unique across the whole catalog: `ProviderRegistry::validate`
hoists its `seen_models` set outside the per-provider loop, so two entries both
claiming `gemini-2.5-pro` fail at boot with `DuplicateModel`. Credential and
endpoint therefore travel together per entry.

## The Google service account

`vertex` and `vertex_maas` take the **same kind** of credential and can take the
same one: a service-account JSON key with `roles/aiplatform.user` on the Google
project. Store the whole document verbatim as the secret value.

```bash
systemprompt admin config secret set vertex "$(cat service-account.json)"
systemprompt admin config secret set vertex_maas "$(cat service-account.json)"
```

The gateway recognises a service-account key by the `type` field the document
declares about itself — `"service_account"` — which is Google's own
self-description rather than a guess about shape
(core `crates/entry/api/src/services/gateway/service/credentials/google.rs`). It signs an RS256 assertion
with the key, exchanges it for an access token under RFC 7523 on the
`https://www.googleapis.com/auth/cloud-platform` scope, and caches the token
until shortly before it expires. There is no flag to set. A secret that is not
such a document is sent verbatim as an API key, exactly as before.

### The project id in the endpoint must match the service account

Both Vertex endpoints in `providers.yaml` are project-scoped. A token minted for
one project is refused by another with `403 IAM_PERMISSION_DENIED`, so the
endpoint and the credential are replaced **together** when serving a different
customer. This is the failure fixed in `5bd1b9dc`, where `vertex_maas` pointed
at a project the service account did not belong to.

`${VAR}` does not help here: interpolation is not applied to the provider
catalog, and an un-expanded placeholder reaches Google verbatim as a project
name.

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
    "model":"vertex-gemini-2.5-flash","max_tokens":64,
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
`vertex-gemini-2.5-pro`, `gemini-2.5-flash`), the gateway routes it by the
patterns in [`services/ai/gateway.yaml`](../../services/ai/gateway.yaml),
rewrites it to the upstream name where `upstream_model` differs, and audits the
call.
