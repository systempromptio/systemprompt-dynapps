---
title: "Model Gateway, Routing & Data Residency"
description: "Route all AI traffic through one provider-neutral gateway with ACL-enforced model access, quotas, latency SLOs, private endpoints, and declared residency requirements."
author: "Astound Digital"
slug: "enterprise-model-routing"
keywords: "gateway, routing, models, providers, acl, quotas, residency, no-retain, shadow ai, slo"
kind: "guide"
public: true
tags: ["enterprise", "gateway", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-11"
after_reading_this:
  - "Route any Anthropic- or OpenAI-compatible client through the provider-neutral gateway"
  - "Grant and revoke model access per role or AD group via gateway_route ACLs"
  - "Enforce data-residency and no-retain requirements declaratively on routes"
  - "Set quota windows and latency SLO thresholds and read the breach reporting"
  - "Add a private or self-hosted endpoint as a configuration entry"
related_docs:
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Model Gateway, Routing & Data Residency

**TL;DR:** Configured clients send AI traffic through the provider-neutral gateway (`/v1/messages`, `/v1/responses`, four wire protocols) that rewrites models, enforces per-role access with 403s, refuses unlisted models, applies quota windows, tracks latency against an SLO, and — via governance metadata on providers and routes — refuses to send classified traffic to a provider that retains data.

## One gateway, any provider

The gateway speaks **four wire protocols** behind `/v1/messages` and `/v1/responses`, so any Anthropic-SDK or OpenAI-compatible client points at it unchanged. Routes match on model pattern and can rewrite via `upstream_model` — a client asking for `claude-*` can be transparently served by a different backend without any application change. Substitution across Cerebras, OpenAI, and Gemini is proven in live deployments.

## The shipped Anthropic lineup

The catalog (`services/ai/providers.yaml`) is implementation configuration
shipped in the image, so every environment boots the same model set. The
Anthropic entries, in listing order, are:

| Model | Input / output per million | Context | Max output |
|---|---|---|---|
| `claude-opus-5` | $5.00 / $25.00 | 1M | 128k |
| `claude-sonnet-5` | $2.00 / $10.00 | 1M | 128k |
| `claude-fable-5-1` | $10.00 / $50.00 | 1M | 128k |
| `claude-fable-5` | $10.00 / $50.00 | 1M | 128k |
| `claude-opus-4-8` | $5.00 / $25.00 | 1M | 128k |
| `claude-opus-4-7` | $5.00 / $25.00 | 1M | 128k |
| `claude-opus-4-6` | $5.00 / $25.00 | 1M | 128k |
| `claude-sonnet-4-6` | $3.00 / $15.00 | 1M | 128k |
| `claude-haiku-4-5` | $1.00 / $5.00 | 200k | 64k |
| `claude-opus-4-5` | $5.00 / $25.00 | 200k | 64k |
| `claude-sonnet-4-5` | $3.00 / $15.00 | 200k | 64k |

The last three carry their dated ids as aliases, so an existing client pinned to
`claude-haiku-4-5-20251001` keeps working. `gateway.default_model` is
`claude-sonnet-5`. Retired dated ids are no longer served: because
`allow_unlisted_models` is `false`, a request naming one is refused rather than
silently forwarded.

## Model access control

Access to models is a **`gateway_route` ACL matrix**, administered in the admin UI and granted per role or per AD group (which is how a project is named). A request for a route the caller is not entitled to is refused with **HTTP 403**, the denial is audited, and grants are revocable with immediate effect.

## Credentials

Personal access tokens carry an optional **expiry**, a **prefix** (identify a leaked token without exposing it), and immediate **revocation** from `/admin/devices/pats`. Note that a key is not its own governance subject — budgets and quotas bind to the owning user and the instance, not to the individual key (see the [roadmap](/documentation/enterprise-roadmap)).

## Shadow-AI posture

The gateway runs with **`allow_unlisted_models: false`**: a model that is not explicitly configured cannot be called, whoever asks. Combined with route entitlements and the full audit trail, everything that touches the gateway is governed. Traffic that never touches the gateway — someone calling a provider directly from their laptop — is a network-control matter for corporate IT, outside any application platform's reach.

## Latency SLOs

A **configurable latency SLO threshold** is tracked per the reporting views: breach percentage over the period, alongside **p50 and p95** latency and per-model latency data. Per-use-case SLO taxonomies and error-budget alerting are planned — see the [roadmap](/documentation/enterprise-roadmap).

## Quota windows

Quota windows are configured per user and instance-wide in `services/gateway/policies.yaml`. The shipped `quota_mode: warn` records breaches without refusing requests; it does not impose a hard spend cap. See [Cost Management](/documentation/enterprise-cost-management) for the active thresholds and enforcement option.

## Private and self-hosted endpoints

Any OpenAI- or Anthropic-compatible private endpoint is **a configuration entry, not a code change**. Declaring a provider with the `Backend` surface keeps it un-advertised — it serves routes without appearing in any public model listing — and every route is validated at boot.

## Data classification & residency

Provider and model YAML carries a **`governance:` block** with two booleans: `european` (data stays in the EU) and `no_retain` (the provider contractually does not retain prompts or completions). Routes declare what they demand via a **`requires:` block**:

```yaml
providers:
- name: anthropic
  governance:
    european: false
    no_retain: true
gateway:
  routes:
  - model_pattern: claude-*
    provider: anthropic
    requires:
      no_retain: true
```

Enforcement happens twice:

- **At boot** — the server refuses to start a route whose provider or model does not satisfy the route's `requires:` block. A misconfiguration is caught before any traffic flows.
- **At dispatch** — a request that would land on a non-satisfying provider is denied, with a policy audit row whose descriptor records the failed `requires:` condition, so the denial is explainable after the fact.

The gateway checks the declared metadata; operators must verify that endpoint configuration and provider terms justify those declarations. These booleans do not independently prove physical residency or contractual retention behavior. Classifying *data* (rather than routes) is planned — see the [roadmap](/documentation/enterprise-roadmap).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured. Screenshots below are illustrative and may show an earlier configuration.

### Screenshots

![The gateway routing configuration, including the requires/no_retain policy block](/files/images/evidence/req-037-gateway-routes.png)

![Per-model latency and spend reported for the same period](/files/images/evidence/req-018-models-latency.png)
