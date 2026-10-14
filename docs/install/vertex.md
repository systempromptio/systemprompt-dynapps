# Vertex AI credentials for the gateway

This page is for the team issuing Google credentials from the Cloud project.
For the full set of secrets an operator must provide across every provider, and
how to set and verify them, see [required-secrets.md](required-secrets.md).

The gateway serves five Google providers beside Anthropic. All are declared in
`services/ai/providers.yaml`; each names the secret it reads its credential
from, and the secret store supplies the value per environment
(`systemprompt admin config secret set <name> <value>`).

| secret | provider | Google endpoint | what the gateway sends |
|---|---|---|---|
| `gemini` | `gemini` (`gemini-3.8-flash`, used by the classifier job) | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` | the value on the `x-goog-api-key` header |
| `vertex` | `vertex` (`vertex-gemini-3.8-flash`, plus whatever discovery adds) | `https://aiplatform.googleapis.com/v1/projects/{project}/locations/global/publishers/google/models/{model}:generateContent` | a minted OAuth token as `Authorization: Bearer` |
| `vertex` | `vertex-anthropic` (the `claude-*` models) | `https://aiplatform.googleapis.com/v1/projects/{project}/locations/global/publishers/anthropic/models/{model}:rawPredict` | a minted OAuth token as `Authorization: Bearer` |
| `vertex` | `vertex-maas` (GLM 5.2, DeepSeek V3.2, Kimi K2 Thinking, Qwen3 Coder, gpt-oss and the other open models) | `https://aiplatform.googleapis.com/v1beta1/projects/{project}/locations/global/endpoints/openapi/chat/completions` | a minted OAuth token as `Authorization: Bearer` |
| `vertex` | `vertex-maas-us-central1` (DeepSeek R1) | `https://us-central1-aiplatform.googleapis.com/v1beta1/projects/{project}/locations/us-central1/endpoints/openapi/chat/completions` | a minted OAuth token as `Authorization: Bearer` |

Every Vertex provider reads the **one** `vertex` secret: a single
service-account JSON in a single GCP project serves Gemini, Claude and the open
models alike. There is no separate `vertex_maas` secret any more.

**The `vertex` endpoint must be the `global` location.** Every Gemini 3.x
model is served from `locations/global` only; a regional endpoint such as
`us-central1-aiplatform.googleapis.com/.../locations/us-central1` returns `404`
for all of them, which took the whole provider down until 2026-09-11. The MaaS
endpoint was already global, except DeepSeek R1, which only us-central1 serves.

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
  https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash:generateContent \
  -d '{"contents":[{"role":"user","parts":[{"text":"ping"}]}]}'
```

A `200` with a `candidates` array means the key is good.

### Vertex authentication

Every Vertex entry uses the `vertex` service-account credential to obtain
OAuth access tokens, and fills its project-scoped endpoint from the same key. Use the separate `gemini` entry for API-key authentication.

### Why the Vertex Gemini models carry a prefix

The obvious shape — two catalog entries, one per credential type, both carrying
the bare Gemini model ids — does not load. `ProviderRegistry::validate` hoists
its `seen_models` set outside the per-provider loop
(`crates/shared/models/src/services/providers/mod.rs`), so a model id must be
unique across the *whole* catalog, not merely within one provider. Two entries
both listing `gemini-3.8-flash` fail at boot with `DuplicateModel`.

So the `vertex` entry prefixes its ids (`vertex-gemini-3.8-flash`) and carries the
real name in `upstream_model`. A client asking for `vertex-gemini-3.8-flash`
reaches Google's `gemini-3.8-flash`, and the two routes to the same model stay
independently selectable and independently audited.

## `vertex` — the service account

Vertex AI — including the OpenAI-compatible Model-as-a-Service endpoint —
authenticates with a **Google OAuth 2.0 access token** on the bearer header, not an API key. An
access token lasts about an hour, so a pasted token is not a credential — it
has to be minted and refreshed.

**What to ask the project for: a service-account JSON key** with
`roles/aiplatform.user` on the project named in the endpoint. Store the whole
JSON document as the `vertex` secret:

```bash
systemprompt admin config secret set vertex "$(cat service-account.json)"
```

The gateway recognises a service-account key by the `type: service_account`
field the document declares about itself, signs an RS256 assertion with it,
exchanges that for an access token (RFC 7523), and caches the token until
shortly before it expires. No flag to set: a secret that is not such a document
is still sent verbatim as an API key.

Every Vertex provider (`vertex`, `vertex-anthropic`, `vertex-maas`,
`vertex-maas-us-central1`) names this one secret, so one service-account key in
one project covers every Vertex model and the gateway mints and caches a single
token for all of them.

The open models the credential must be entitled to include, by their Vertex
names:

```
zai-org/glm-5.2-maas
deepseek-ai/deepseek-v3.2-maas
moonshotai/kimi-k2-thinking-maas
qwen/qwen3-coder-480b-a35b-instruct-maas
openai/gpt-oss-120b-maas
```

`services/ai/providers.yaml` is the full list. Every Qwen, GLM 4.7/5, Kimi,
DeepSeek and MiniMax endpoint and `gpt-oss-20b` were deprecated on 2026-07-21
and retire on 2026-10-21; they stay declared until then (see the lifecycle
rule below). Only `gpt-oss-120b` and `glm-5.2` are not retiring.

Each must be enabled in Model Garden for the project; a model that is not
enabled returns `404` to every user who picks it.

## How a developer reaches them

Nothing changes on the developer's machine. The bridge install points Claude
Code at the gateway and turns on gateway model discovery, so every model above
appears in Claude Code's `/model` picker beside the Claude models the moment
the catalog ships. A request names the model id from the catalog
(`qwen.qwen3-coder-480b`, `vertex-gemini-3.8-flash`, ...), the gateway routes it to the
provider, rewrites it to the Vertex name, and records the call with the user,
the tokens and the cost in the same audit trail as every Anthropic call.

## Which Vertex models are served

The list above is what this deployment authored by hand. It is not the whole
story, because Vertex publishes and retires MaaS models on its own schedule and
nobody edits `services/ai/providers.yaml` when it does. So at boot the gateway
asks Vertex what it currently publishes and reconciles that against the rate
card compiled into the core release. Four outcomes:

| Vertex publishes it | The rate card prices it | Documentation still supports it | Result |
|---|---|---|---|
| yes | yes | yes | **served** — added to the catalog and callable |
| yes | no | — | **reported only** — a model with no price cannot be billed, so it is not installed |
| no | yes | — | **reported only** — priced ahead of availability |
| yes | yes | no | **retiring** — withheld; an explicit declaration is kept and warned about |

### The lifecycle rule: docs decide, never a probe

Vertex keeps listing a model right up to the day it is switched off, so the
listing cannot say whether a model is *currently supported*. Google's model
pages can, and every rate-card entry in the core release records what its
page says: `launch_stage`, `released`, `retires_on`, `price_until` and the
`docs` URL it was read from. A model is served when its stage is GA (or the
entry opts into preview) **and** its retirement is more than 30 days away.
Nothing is ever called to find this out — the publisher listing is the only
network traffic discovery makes.

The daily job also reads the calendar: 60 days before any served model's
`retires_on` or `price_until` (Gemini 3.8/3.7/3.6 Flash are on introductory
pricing to 2026-12-31) it warns in the log with the model and the date.

An entry written by hand in `services/ai/providers.yaml` always wins: discovery
never overwrites or re-prices an explicitly declared model.

Discovery runs **once, at boot**. The provider registry is fixed for the life
of the process, so a model Vertex publishes at lunchtime is served after the
next restart, not before. A daily job re-probes and warns in the log when
something newly published is already priced — that warning is the signal that a
restart would widen the catalog.

The listing Vertex returns is global, not per-project: a model can appear as
published and still return `404` to this project because it is not enabled in
Model Garden. Entitlement shows up at call time, never at discovery time.

To see what the last pass found:

```bash
systemprompt admin config catalog discovery
```

Each row is a model id, its state — `served`, `unpriced`,
`priced-not-published`, `explicit` (declared by hand), or `retiring` — and the
documented retirement date where the rate card records one. The footer names any
publisher whose listing call failed, which makes the report partial rather than
wrong.

### Adding a model

Two ways, depending on who owns the decision:

- **Operator** — add a priced entry to `services/ai/providers.yaml`, the same
  as any other model. This is the right move for a one-deployment need.
- **Core** — add it to the Vertex rate card in the core release, so every
  deployment picks it up on upgrade.

Either way, check `services/ai/gateway.yaml` in the same change: a model with no
matching route is priced but unreachable, and nothing fails loudly to tell you.

### Turning it off

```bash
SYSTEMPROMPT_VERTEX_DISCOVERY=0
```

Set in the server's environment, this skips the probe entirely and the catalog
is exactly what the YAML declares. Discovery is also skipped on its own when no
provider endpoint is on `aiplatform.googleapis.com`, or when the credential for
one is not a service-account key — an air-gapped or credential-less boot never
reaches for the network.
