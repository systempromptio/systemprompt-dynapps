# Vertex AI credentials for the gateway

The gateway serves two Google Vertex AI providers beside Anthropic. Both are
declared in `services/ai/providers.yaml`; each names the secret it reads its
credential from, and the secret store supplies the value per environment
(`systemprompt admin config secret set <name> <value>`). This page is for the
team issuing those credentials from the Google Cloud project.

| secret | provider | Google endpoint | what the gateway sends |
|---|---|---|---|
| `gemini` | `gemini` (Gemini 2.5 Pro, Flash, Flash-Lite) | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` | the value on the `x-goog-api-key` header |
| `vertex_maas` | `vertex-maas` (Qwen3, GLM-5, Kimi K2, DeepSeek V3.2, GPT-OSS) | `https://aiplatform.googleapis.com/v1beta1/projects/enterprise-ai-505512/locations/global/endpoints/openapi/chat/completions` | the value as `Authorization: Bearer` |

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

### Why not Vertex AI

An earlier revision of this page asked for a "Vertex AI API key in express
mode" against
`aiplatform.googleapis.com/v1beta1/publishers/google/models/{model}:generateContent`.
**That endpoint does not accept API-key authentication at all.** Probed
2026-09-05 with a valid Google API key:

```
HTTP 401
"API keys are not supported by this API. Expected OAuth2 access token or other
 authentication credentials that assert a principal."   (CREDENTIALS_MISSING)
```

This is a property of the endpoint, not of the key, so no API key of any
provenance can satisfy it. Serving Gemini from Vertex needs the same
service-account credential `vertex_maas` needs — the gateway can now mint and
refresh those, so it is a credential you do not yet have rather than a feature
you do not yet have. To switch, point `endpoint` at
`https://aiplatform.googleapis.com/v1beta1/publishers/google` and put the
service-account JSON in the secret named by `api_key_secret`.

### Why there is only one Gemini entry

The obvious shape — one catalog entry per credential type, both carrying the
Gemini models, pick whichever has a secret — does not load. `ProviderRegistry::
validate` hoists its `seen_models` set outside the per-provider loop
(`crates/shared/models/src/services/providers/mod.rs`), so a model id must be
unique across the *whole* catalog, not merely within one provider. Two entries
both listing `gemini-2.5-pro` fail at boot with `DuplicateModel`.

So the endpoint follows the credential inside a single entry, and switching a
deployment from an API key to a service account means editing `endpoint` and
`api_key_secret` on that one entry rather than enabling a second one.

## `vertex_maas` — the open models

The Model-as-a-Service endpoint is OpenAI-compatible, but it authenticates with
a **Google OAuth 2.0 access token** on the bearer header, not an API key. An
access token lasts about an hour, so a pasted token is not a credential — it
has to be minted and refreshed.

**What to ask the project for: a service-account JSON key** with
`roles/aiplatform.user` on `enterprise-ai-505512`. Store the whole JSON
document as the `vertex_maas` secret:

```bash
systemprompt admin config secret set vertex_maas "$(cat service-account.json)"
```

The gateway recognises a service-account key by the `type: service_account`
field the document declares about itself, signs an RS256 assertion with it,
exchanges that for an access token (RFC 7523), and caches the token until
shortly before it expires. No flag to set: a secret that is not such a document
is still sent verbatim as an API key.

Ask for the same credential for Gemini if you want it served from Vertex rather
than from `generativelanguage.googleapis.com` — it is one credential for all
twelve models, and it is the only thing the Vertex endpoints accept.

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
