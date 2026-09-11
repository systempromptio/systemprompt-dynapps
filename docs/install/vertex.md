# Vertex AI credentials for the gateway

This page is for the team issuing Google credentials from the Cloud project.
For the full set of secrets an operator must provide across every provider, and
how to set and verify them, see [required-secrets.md](required-secrets.md).

The gateway serves three Google providers beside Anthropic. All are declared in
`services/ai/providers.yaml`; each names the secret it reads its credential
from, and the secret store supplies the value per environment
(`systemprompt admin config secret set <name> <value>`).

| secret | provider | Google endpoint | what the gateway sends |
|---|---|---|---|
| `gemini` | `gemini` (Gemini 2.5 Pro, Flash, Flash-Lite) | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` | the value on the `x-goog-api-key` header |
| `vertex` | `vertex` (`vertex-gemini-2.5-pro`, `vertex-gemini-2.5-flash`) | `https://us-central1-aiplatform.googleapis.com/v1/projects/{project}/locations/us-central1/publishers/google/models/{model}:generateContent` | a minted OAuth token as `Authorization: Bearer` |
| `vertex_maas` | `vertex-maas` (Qwen3, GLM-5, Kimi K2, DeepSeek V3.2, GPT-OSS) | `https://aiplatform.googleapis.com/v1beta1/projects/{project}/locations/global/endpoints/openapi/chat/completions` | a minted OAuth token as `Authorization: Bearer` |

**`{project}` is filled by the gateway** from the `project_id` of the
service-account key stored in the secret. It is never written literally in
`providers.yaml`: the catalog ships in every image, Vertex echoes the project
id in every IAM error, and the gateway relays those errors to the caller. The
boot validator refuses a literal `projects/<id>` on a Vertex host.

## `gemini` — a Gemini API key (what the gateway uses today)

Issue a **Gemini API key** and hand over the key string. The gateway sends it on
`x-goog-api-key` against `generativelanguage.googleapis.com`. No service
account, no OAuth exchange.

```bash
curl -sS -H "x-goog-api-key: $KEY" \
  -H 'content-type: application/json' \
  https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent \
  -d '{"contents":[{"role":"user","parts":[{"text":"ping"}]}]}'
```

A `200` with a `candidates` array means the key is good.

### Vertex authentication

The `vertex` and `vertex_maas` entries use service-account credentials to obtain
OAuth access tokens. Configure their project-scoped endpoints and credentials
together. Use the separate `gemini` entry for API-key authentication.

### Why the Vertex Gemini models carry a prefix

The obvious shape — two catalog entries, one per credential type, both carrying
the bare Gemini model ids — does not load. `ProviderRegistry::validate` hoists
its `seen_models` set outside the per-provider loop
(`crates/shared/models/src/services/providers/mod.rs`), so a model id must be
unique across the *whole* catalog, not merely within one provider. Two entries
both listing `gemini-2.5-pro` fail at boot with `DuplicateModel`.

So the `vertex` entry prefixes its ids (`vertex-gemini-2.5-pro`) and carries the
real name in `upstream_model`. A client asking for `vertex-gemini-2.5-flash`
reaches Google's `gemini-2.5-flash`, and the two routes to the same model stay
independently selectable and independently audited.

## `vertex_maas` — the open models

The Model-as-a-Service endpoint is OpenAI-compatible, but it authenticates with
a **Google OAuth 2.0 access token** on the bearer header, not an API key. An
access token lasts about an hour, so a pasted token is not a credential — it
has to be minted and refreshed.

**What to ask the project for: a service-account JSON key** with
`roles/aiplatform.user` on the project named in the endpoint. Store the whole
JSON document as the `vertex_maas` secret:

```bash
systemprompt admin config secret set vertex_maas "$(cat service-account.json)"
```

The gateway recognises a service-account key by the `type: service_account`
field the document declares about itself, signs an RS256 assertion with it,
exchanges that for an access token (RFC 7523), and caches the token until
shortly before it expires. No flag to set: a secret that is not such a document
is still sent verbatim as an API key.

The same credential serves the `vertex` entry, so one service-account key
covers all eleven Vertex models. Set it under both secret names — the gateway
resolves and caches a token per secret name, not per document:

```bash
systemprompt admin config secret set vertex "$(cat service-account.json)"
```

The models the credential must be entitled to, by their Vertex names:

```
qwen/qwen3-next-80b-a3b-instruct-maas
qwen/qwen3-next-80b-a3b-thinking-maas
qwen/qwen3-coder-480b-a35b-instruct-maas
qwen/qwen3-235b-a22b-instruct-2507-maas
zai-org/glm-5-maas
moonshotai/kimi-k2-thinking-maas
deepseek-ai/deepseek-v3.2-maas
openai/gpt-oss-120b-maas
openai/gpt-oss-20b-maas
```

Each must be enabled in Model Garden for the project; a model that is not
enabled returns `404` to every user who picks it.

## How a developer reaches them

Nothing changes on the developer's machine. The bridge install points Claude
Code at the gateway and turns on gateway model discovery, so every model above
appears in Claude Code's `/model` picker beside the Claude models the moment
the catalog ships. A request names the model id from the catalog
(`qwen.qwen3-coder-480b`, `gemini-2.5-pro`, ...), the gateway routes it to the
provider, rewrites it to the Vertex name, and records the call with the user,
the tokens and the cost in the same audit trail as every Anthropic call.
