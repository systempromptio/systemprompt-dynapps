# Vertex AI credentials for the gateway

The gateway serves two Google Vertex AI providers beside Anthropic. Both are
declared in `services/ai/providers.yaml`; each names the secret it reads its
credential from, and the secret store supplies the value per environment
(`systemprompt admin config secret set <name> <value>`). This page is for the
team issuing those credentials from the Google Cloud project.

| secret | provider | Google endpoint | what the gateway sends |
|---|---|---|---|
| `vertex` | `vertex` (Gemini 2.5 Pro, Flash, Flash-Lite) | `https://aiplatform.googleapis.com/v1beta1/publishers/google/models/{model}:generateContent` | the value on the `x-goog-api-key` header |
| `vertex_maas` | `vertex-maas` (Qwen3, GLM-5, Kimi K2, DeepSeek V3.2, GPT-OSS) | `https://aiplatform.googleapis.com/v1beta1/projects/enterprise-ai-505512/locations/global/endpoints/openapi/chat/completions` | the value as `Authorization: Bearer` |

## `vertex` — a Vertex AI API key

Gemini on Vertex AI accepts an API key in express mode, and the key travels on
the `x-goog-api-key` header, which is what the gateway's Gemini adapter already
uses. Issue a **Vertex AI API key** in the project, restricted to the Vertex AI
API, and hand over the key string. No service account, no OAuth exchange.

Check it before handing it over:

```bash
curl -sS -H "x-goog-api-key: $KEY" \
  -H 'content-type: application/json' \
  https://aiplatform.googleapis.com/v1beta1/publishers/google/models/gemini-2.5-flash-lite:generateContent \
  -d '{"contents":[{"role":"user","parts":[{"text":"ping"}]}]}'
```

## `vertex_maas` — the open models

The Model-as-a-Service endpoint is OpenAI-compatible, but it authenticates with
a **Google OAuth 2.0 access token** on the bearer header, not an API key. The
proxy this replaces minted that token on every request from Application
Default Credentials. The gateway reads its secret once at boot and presents it
verbatim, and an access token expires after one hour, so a pasted token is not
a working credential.

What we need from the project, in order of preference:

1. **A service-account JSON key** with `roles/aiplatform.user` on
   `enterprise-ai-505512`. The gateway will exchange it for access tokens and
   refresh them itself. This needs a small change on our side before the nine
   models go live, and it is the option we are building for.
2. **Confirmation that a Vertex AI API key is accepted on this endpoint.** If
   the project has API-key access enabled for Model Garden partner models, the
   same kind of key as `vertex` works and no service account is needed. Test
   it with:

```bash
curl -sS -H "x-goog-api-key: $KEY" -H 'content-type: application/json' \
  https://aiplatform.googleapis.com/v1beta1/projects/enterprise-ai-505512/locations/global/endpoints/openapi/chat/completions \
  -d '{"model":"openai/gpt-oss-20b-maas","messages":[{"role":"user","content":"ping"}]}'
```

A `200` means option 2 is open; a `401` naming an OAuth token means option 1.

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
