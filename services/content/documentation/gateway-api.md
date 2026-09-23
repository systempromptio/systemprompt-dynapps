---
title: "Gateway API (/v1/messages)"
description: "The governed inference gateway: the /v1/messages contract, the required x-session-id header, model allow-listing, and the three profile API URLs."
author: "Astound Digital"
slug: "gateway-api"
keywords: "gateway, /v1/messages, x-session-id, inference, model allow-list, api url, governance"
kind: "guide"
public: true
tags: ["gateway", "api", "governance"]
published_at: "2026-05-19"
updated_at: "2026-09-11"
after_reading_this:
  - "Call the governed inference gateway at POST /v1/messages"
  - "Supply the required x-session-id header so a request is not rejected with HTTP 400"
  - "Understand how the gateway model allow-list rejects un-listed models with HTTP 403"
  - "Pick the right profile api_*_url for in-container vs host callers"
related_docs:
  - title: "Authentication"
    url: "/documentation/authentication"
---

# Gateway API

**TL;DR:** the gateway exposes `POST /v1/messages` — an Anthropic-Messages-compatible
endpoint that authenticates, authorises, governs, and audits every inference call before
proxying it upstream. Every request **must** carry an `x-session-id` header and an
`Authorization` (or `x-api-key`) credential. The model named in the body must be
permitted by the gateway policy allow-list, or the gateway returns `403` before any
upstream call is made.

## Request contract

`POST /v1/messages`

Required headers:

| Header | Purpose | Missing → |
|--------|---------|-----------|
| `Authorization: Bearer <jwt>` *or* `x-api-key: <key>` | Caller identity | `401 Unauthorized` |
| `x-session-id: <session id>` | Binds the call to a session for audit and conversation continuity | `400 Bad Request` ("missing required x-session-id header") |
| `x-gateway-conversation-id: <id>` *(optional)* | Pins the conversation; otherwise it is derived from the message body | — |

Body: the Anthropic Messages shape — `model`, `max_tokens`, `messages[]`.

The `x-session-id` header is **mandatory**. A request without it is rejected with
`400`, not a clearer error — mint a gateway session with `POST /api/public/gateway/sessions` using the same PAT, then pass the returned `session_id`. The bridge handles this automatically. A CLI admin session token is not a gateway credential.

## Model allow-listing

The provider registry (`services/ai/providers.yaml`, shipped in the image)
declares every provider and the models it serves; those model ids are the
deployment's allow-list. A request whose `model` is neither declared in the
registry nor matched by a gateway route (or absorbed by
`gateway.default_provider`) is denied with `403` before any upstream call — this
is the egress control an air-gapped deployment relies on.

Both the dispatch gate (`GatewayConfig::is_model_exposed`) and the `/profile`
model list derive from the registry, so adding a model means editing the
`providers` block.

The Anthropic models this deployment serves are `claude-opus-5-5`,
`claude-opus-5`, `claude-sonnet-5`, `claude-fable-5-1`, `claude-fable-5` and
`claude-haiku-4-5`; the last also answers to its dated id
`claude-haiku-4-5-20251001`. A request that names no model, or one no route matches,
falls back to `gateway.default_model`, which is `claude-sonnet-5[1m]`. The `[1m]`
suffix selects Claude Code's 1M context budget; the gateway accepts it on any
model and strips it before routing and forwarding. Retired dated
ids such as `claude-opus-4-1-20250805`, and the Claude 4.x Opus and Sonnet models,
are no longer listed and are refused.

Gateway-route RBAC additionally keys on the route `id`. If the caller's role or
department is not assigned to the route (and the route is not `default_included`), the
gateway returns `403` with a message that names the route id, the model, and the
remedy.

## Governance requirements on routes

Providers and models can carry a `governance:` block (`european`, `no_retain`)
and a route can demand it with `requires:`. A route whose provider or model
cannot satisfy its requirement refuses to boot, and dispatch re-checks the
resolved target — a denied call is audited with the requirement named in the
route-match descriptor. The full walkthrough, with a worked YAML example, is in
[Model Gateway, Routing & Data Residency](/documentation/enterprise-model-routing).

## Profile API URLs

A profile declares three server URLs; pick the right one for the caller:

| Field | Use it for |
|-------|------------|
| `api_internal_url` | Service-to-service calls inside the deployment network |
| `api_server_url` | Server API base URL; the listening interface and port are configured separately through `host` and `port` |
| `api_external_url` | The public, host-facing address; used for OAuth/WebAuthn URL generation and consumed by `session login` |

When the host-published port differs from the in-container port (as in the air-gap
stack), an in-container caller must use the in-container address and host callers must
use the published one — a single value cannot satisfy both. The air-gap profile pins
all three to the in-container `:8080` and automated host callers pass the published
port explicitly.
