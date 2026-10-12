---
title: "Model Gateway, Routing & Data Residency"
description: "Route all AI traffic through one provider-neutral gateway with ACL-enforced model access, quotas, latency SLOs, private endpoints, and declared residency requirements."
author: "Dynapps"
slug: "enterprise-model-routing"
keywords: "gateway, routing, models, providers, vertex ai, failover, acl, quotas, residency, no-retain, shadow ai, slo, chat completions, retirement"
kind: "guide"
public: true
tags: ["enterprise", "gateway", "admin"]
published_at: "2026-08-25"
updated_at: "2026-10-08"
after_reading_this:
  - "Route any Anthropic- or OpenAI-compatible client through the provider-neutral gateway"
  - "Know which models this instance serves, from which provider, and which retire soon"
  - "Understand how model access is granted, and that every route is open to every role today"
  - "Know which residency and no-retain requirements are declared (none today) and how to add one"
  - "Read latency against an SLO threshold on the analytics dashboard"
  - "Add a private or self-hosted endpoint as a configuration entry"
  - "Name a fallback provider on a route and read a failover back off the audit row"
related_docs:
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Model Gateway, Routing & Data Residency

**TL;DR:** Configured clients send AI traffic through one provider-neutral
gateway, on `/v1/messages` (Anthropic Messages), `/v1/chat/completions`
(OpenAI Chat) and `/v1/responses` (OpenAI Responses). Routes map a requested
model pattern onto a provider. **Claude is served from Google Vertex AI and
fails over to Anthropic's own API**, and every request records which provider
actually answered. A model that no route or catalog entry names is refused.
Every route is currently open to every signed-in role. The residency and
no-retain requirements are a working mechanism, but no route declares one
today.

## One gateway, any provider

The gateway accepts three client wire formats and sends to each provider in
that provider's own format:

| Endpoint | Client format | Typical client |
|---|---|---|
| `POST /v1/messages` | Anthropic Messages | Claude Code, the Anthropic SDKs |
| `POST /v1/chat/completions` | OpenAI Chat Completions | OpenCode, OpenAI-compatible tools |
| `POST /v1/responses` | OpenAI Responses | Responses-API clients |

The providers this instance declares speak three upstream formats:
`anthropic` (Anthropic and Claude on Vertex), `gemini` (Gemini by API key and
on Vertex) and `openai-chat` (Vertex Model-as-a-Service). A route matches on
model pattern, and **first match wins**. A route can rewrite the model with
`upstream_model`, so the backend can change without any change to the
client. Request and wire details are in the [Gateway API](/documentation/gateway-api).

## What this instance serves

The catalog (`services/ai/providers.yaml`) and the routes
(`services/ai/gateway.yaml`) ship inside the image, so every environment
boots the same set.

| Requested id | Provider | Models |
|---|---|---|
| `claude-*` | `vertex-anthropic` (Vertex AI, `global`), failing over to `anthropic` | Opus 5.5, Opus 5, Sonnet 5, Fable 5.1, Fable 5, Haiku 4.5 |
| `anthropic-claude-*` | `anthropic` (Anthropic's API) | the same Claude models, hidden from every model list |
| `vertex-gemini-*` | `vertex` (Vertex AI, `global`) | Gemini 3.8 Flash, 3.5 Flash, 3.5 Flash-Lite |
| `gemini-*` | `gemini` (public API, by key) | Gemini 3.8 Flash; also used by the classifier job |
| `qwen.*`, `zai.*`, `moonshotai.*`, `deepseek.*`, `minimax.*`, `openai.gpt-oss-*` | `vertex-maas` (Vertex Model-as-a-Service) | the open models |
| `deepseek.r1-*` | `vertex-maas-us-central1` | DeepSeek R1, served only from `us-central1` |

The Claude models, at Vertex's global rates:

| Model | Input / output per million | Context | Max output |
|---|---|---|---|
| `claude-opus-5-5` | $4.00 / $20.00 | 1M | 128k |
| `claude-opus-5` | $5.00 / $25.00 | 1M | 128k |
| `claude-sonnet-5` | $2.00 / $10.00 | 1M | 128k |
| `claude-fable-5-1` | $10.00 / $50.00 | 1M | 128k |
| `claude-fable-5` | $10.00 / $50.00 | 1M | 128k |
| `claude-haiku-4-5` | $1.00 / $5.00 | 200k | 64k |

`gateway.default_model` is `claude-sonnet-5[1m]`. The `[1m]` suffix is Claude
Code's 1M-context marker: the bridge seeds it into Claude Code, and the
gateway strips it before routing, so the request reaches Vertex as
`claude-sonnet-5`. Without the marker, Claude Code budgets 200k for every
gateway model. `claude-haiku-4-5` also answers to its dated id,
`claude-haiku-4-5-20251001`. The Claude 4.x Opus and Sonnet models were
removed on 2026-09-24. With `allow_unlisted_models: false`, a request for a
retired id is refused rather than forwarded.

### Retirement dates

| Date | What stops |
|---|---|
| **2026-10-15** | Vertex guarantees `claude-haiku-4-5` only until this date |
| **2026-10-21** | Google turns off every Qwen, Kimi, DeepSeek and MiniMax Model-as-a-Service endpoint, plus `zai.glm-5`, `zai.glm-4.7` and `openai.gpt-oss-20b`. After this date, core hides the retired entries from every model list |

`openai.gpt-oss-120b` and `zai.glm-5.2` are not retiring. `zai.glm-5.2` is a
preview, guaranteed until at least 2026-10-08.

## Claude on Vertex AI

The `vertex-anthropic` provider uses the same Google service account (the
`vertex` secret) as every other Vertex provider:

```yaml
providers:
- name: vertex-anthropic
  wire: anthropic
  surface: anthropic
  endpoint: https://aiplatform.googleapis.com/v1/projects/{project}/locations/global/publishers/anthropic
  api_key_secret: vertex
```

Clients call it exactly as they would call Anthropic. The gateway sends the
Anthropic Messages format unchanged; only the envelope around it differs:

- The gateway mints an OAuth token from the service account and fills
  `{project}` from its key.
- Requests go to the model's own `:rawPredict` or `:streamRawPredict` path.
- The API version travels in the body as `anthropic_version`, not in a header.
- Only the `anthropic-beta` flags listed in the provider's `accepted_betas`
  are forwarded, and a body field a dropped flag gates is removed with it
  (`context-management-*` gates `context_management`), so Vertex never sees the
  field without its flag. A flag or field Vertex still refuses is learned at
  runtime and dropped, and the request is re-sent once. Clients need no setting
  for this: `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS` is not required, and
  setting it turns those betas off on every route, including the ones that
  serve them.

Whether a provider is hosted on Vertex is read from the endpoint host, so the
catalog cannot claim one platform while pointing at another.

## Provider failover

A route may list **`fallbacks`**: deployments tried in order after the
primary, each a `provider` and optionally an `upstream_model` when that
deployment's model id differs. The shipped `claude-*` route fails over from
Vertex AI's `global` endpoint to a second Vertex deployment,
`vertex-anthropic-eu`, on the `europe-west1` regional endpoint under the same
`vertex` service account. Traffic never leaves Vertex:

```yaml
gateway:
  routes:
  - id: claude-star-4203d1
    name: Claude (Vertex AI)
    model_pattern: claude-*
    provider: vertex-anthropic
    fallbacks:
    - provider: vertex-anthropic-eu
```

The older `fallback_provider` / `fallback_upstream_model` keys are refused at
boot; move them into a one-entry `fallbacks` list.

**Anthropic's API is in no failover chain.** A provider serves a route's
traffic only when the route (or one of its chains) names it, and no `claude-*`
chain names `anthropic`. The regional deployment serves each Claude model under
a hidden `vertex-eu-claude-*` alias, priced at Vertex's 10% regional premium.
The project's Claude quota in `europe-west1` must be requested from Google in
advance, or the fallback answers 429 too. The separate `anthropic-claude-*` and
`claude-native-tools` routes name Anthropic as their primary and are unaffected.

**Per-project chains.** `by_scope` gives requests attributed to a value in
one scope dimension their own chain, for example a project's own Vertex
project or region. A value the route does not list is refused unless
`unmapped: shared` sends it down the route's own chain:

```yaml
    by_scope:
      dimension: project
      unmapped: shared
      chains:
        storefront:
          provider: vertex-anthropic
          fallbacks:
          - provider: vertex-anthropic-eu
```

The project is the one the gateway attributed the request to: the
`x-systemprompt-scope-project` header, the API key's bound project, or the
person's primary project. The audit row's route match records
`scope:project=<id>`.

**When it triggers.** The primary keeps its bounded retry budget for
transient answers (429, 503). Failover happens when that budget is spent,
when the primary answers any other **5xx**, or when the **connection fails**
(DNS, TLS, timeout). The same governed request is then re-sent to the
next deployment in the chain, under that deployment's own retry budget, and so
on down the list. A 4xx other than 429 never fails
over, because the fault is in the request, not the provider. A per-provider
**circuit breaker** skips a primary that keeps failing and tries the fallback
first; when every deployment is tripped, all are tried anyway, so a request is
never left unsent.

**What is re-sent.** The request that governance and the safety scanners
already judged is rebuilt in the fallback's wire format. Neither plane runs
again, because the content has not changed.

**What the audit shows.** The `ai_requests` row keeps `provider` as the
route's primary and records the provider that answered in
**`served_provider`**. The request is repriced at that provider's rate, and
the route-match descriptor gains a `failover:<primary>-><fallback>` segment.
`systemprompt infra logs audit <request-id>` prints both.

| `route_match` contains | Meaning |
|---|---|
| `failover:vertex-anthropic->vertex-anthropic-eu` | the primary failed and the next deployment answered (`a->b->c` for a longer chain) |
| `scope:project=<id>` | the request was attributed to that project and served by its `by_scope` chain (or the shared chain under `unmapped: shared`) |
 The Prometheus
counter **`gateway_upstream_failovers_total{from,to,reason}`** (`reason` is
`status_<code>`, `transport` or `circuit_open`) is on `/metrics`.

**What boot checks.** The fallback must exist, differ from the primary, serve
a priced model, and satisfy the route's `requires:` block. If a fallback
cannot be used at request time, the client gets the primary's own error, as
it would with no fallback declared.

### Selection strategy

A route, and each `by_scope` chain, can set `strategy` to choose which
deployment a request tries first. Failover after that first attempt works
as described above.

| `strategy` | First attempt |
|---|---|
| `ordered` (default) | The chain order: `provider`, then `fallbacks`. |
| `weighted` | Drawn by each deployment's `weight` (default 1) among deployments whose circuit breaker is closed. The rest follow in descending weight. |
| `least_busy` | The healthy deployment with the fewest requests in flight on this server process. |

`weight` sits on the route for the primary and on each `fallbacks` entry.
`weight: 0` is refused at load. A weight under `ordered` is ignored. A
non-ordered strategy adds `strategy:<name>` to the request's `route_match`,
and `gateway_deployment_selected_total{route,provider,strategy}` counts every
selection. This instance's routes keep the default `ordered`.

### Context-window check

Before any upstream call, the gateway estimates the request's input tokens
from its text and compares the estimate with the selected model's
`context_window` in the provider catalog. Images and tool inputs are not
counted, so the estimate errs low and never refuses a request that fits.

- **It fits.** The request is sent as normal. Failover deployments whose
  declared window is too small are skipped.
- **It does not fit, and the route lists `context_fallbacks`.** The request
  goes to the first context fallback whose window fits. `route_match`
  records `context_window:a->b`.
- **It does not fit, and no fallback fits.** The request is refused with a
  400 in the caller's API format, with `error_key: context_window_exceeded`
  naming the estimate, the limit and the model. Nothing is sent upstream, so
  nothing is billed.

A model with no declared window is never refused. An oversized `max_tokens`
is clamped to the model's output ceiling rather than refused.

## Model access control

Model access is an access-control entity, `gateway_route/*`, declared in
`services/access-control/rules.yaml` and administered at `/admin/access-control`.
The mechanism grants or denies routes per role, group or project, and a
request for a route the caller is not entitled to gets **HTTP 403**
(`authz denied [<policy>]: <reason>`), and the denial is audited.

**As configured, every route is open to every signed-in role.** Usage is
shaped by governance and quota windows, not by entitlement. To restrict a
model, declare a rule for its route with an allow band and a reason. Route
ids are generated (`slug + fnv1a6`), so the console names each route by the
`name:` its declaration carries.

`/admin/gateway` has four tabs: **Overview** (dispatch order and provider
health), **Providers** (each upstream with its models and routes),
**Routes** (the ordered dispatch table) and **Settings**. **Policies** is a
fifth tab that opens its own page. The route editor there writes back to
`gateway.yaml`, and a saved route is dispatched after the next restart.

## Shadow-AI posture

The gateway runs with **`allow_unlisted_models: false`**, so a model that is
not configured cannot be called by anyone. Traffic that never reaches the
gateway, such as someone calling a provider directly from a laptop, is a
network-control matter for corporate IT. The usual control is an egress
allow-list that lets only the gateway reach provider APIs.

## Latency SLOs

The analytics **Overview** tab reports **p50 and p95** latency and the share
of requests over an SLO threshold, per model. The threshold is a view setting
(`?slo_ms=`, 5 s by default, clamped to 500 ms–60 s) and is not stored
anywhere. There is no SLO alerting. Per-use-case SLO targets and error-budget
alerts are on the [roadmap](/documentation/enterprise-roadmap).

## Quota windows

Quota windows are warning thresholds on this instance (`quota_mode: warn`).
They record breaches and never refuse. See
[Cost Management](/documentation/enterprise-cost-management) and
[Governance Modes](/documentation/governance-modes).

## Private and self-hosted endpoints

A private endpoint that speaks the OpenAI or Anthropic format is **a
configuration entry, not a code change**. A provider declared with the
`backend` surface serves routes without appearing in any model list, and
every route is validated at boot. **None is configured on this instance.**

## Data classification & residency

Each provider, and optionally each model, carries a **`governance:`** block
with two booleans: `european` (data stays in the EU) and `no_retain` (the
provider contractually does not retain prompts or completions). A route
states what it requires with a **`requires:`** block. The following is an
**illustration**, not the shipped configuration:

```yaml
gateway:
  routes:
  - model_pattern: claude-*
    provider: vertex-anthropic
    requires:
      no_retain: true
```

Enforcement happens twice:

- **At boot:** the server refuses to start a route whose provider or model
  does not satisfy its `requires:` block.
- **At dispatch:** a request that would reach a provider that does not
  satisfy the block is denied and audited, with the failed condition named in
  the route-match descriptor.

**What is declared today:**

- Every provider is `no_retain: true` and `european: false`.
- Every Vertex endpoint uses the `global` location (or `us-central1` for
  DeepSeek R1).
- **No route declares `requires:`**, so neither flag is enforced at dispatch,
  and there is **no EU route**.

To enforce no-retain, add `requires: {no_retain: true}` to a route. An EU
guarantee needs an EU-located provider entry first. The flags are
declarations, so keep them in step with the provider agreements; they do not
prove physical residency on their own. Classifying the *data* itself, rather
than the route, is on the [roadmap](/documentation/enterprise-roadmap).

## Check it yourself

- `/admin/gateway?tab=routes`: the dispatch order, with each route's provider
  and fallback.
- `/admin/gateway?tab=providers`: each provider's endpoint, wire and models.
- `/admin/analytics`: the Overview tab's latency and SLO breach share, and
  the Models tab.
- `/admin/requests`: open a request to see `provider` and `served_provider`.
- `systemprompt infra logs request list --since 1h`
- `systemprompt admin config catalog discovery` shows what the last Vertex
  discovery pass found.
