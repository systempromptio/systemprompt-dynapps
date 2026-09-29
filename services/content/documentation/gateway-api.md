---
title: "Gateway API"
description: "The governed inference gateway: the /v1/messages, /v1/chat/completions and /v1/responses contracts, the required x-session-id header, the model list, status codes, and the three profile API URLs."
author: "Astound Digital"
slug: "gateway-api"
keywords: "gateway, /v1/messages, /v1/chat/completions, /v1/responses, /v1/models, x-session-id, inference, model allow-list, status codes, api url, governance"
kind: "guide"
public: true
tags: ["gateway", "api", "governance"]
published_at: "2026-05-19"
updated_at: "2026-09-29"
after_reading_this:
  - "Call the gateway in the Anthropic or OpenAI format"
  - "Supply a credential and an attested x-session-id so a request is not rejected"
  - "Know which model ids this instance serves and what an unlisted one gets back"
  - "Read the status codes the gateway returns before any upstream call"
  - "Pick the right profile api_*_url for in-container vs host callers"
related_docs:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
---

# Gateway API

**TL;DR:** The gateway serves three inference endpoints under `/v1`, one for
each client wire format. Each one authenticates, authorises, governs and
audits the call before it is forwarded upstream. Every request needs a
credential (a bridge JWT or a personal access token) **and** an
`x-session-id` header that matches a session the gateway issued. The model
must be one the catalog or a route names, or the request is refused before
any upstream call. Clients normally never see any of this: the bridge
configures Claude Code and OpenCode for you.

## Endpoints

| Method and path | Format | Used by |
|---|---|---|
| `POST /v1/messages` | Anthropic Messages (`model`, `max_tokens`, `messages[]`) | Claude Code, the Anthropic SDKs |
| `POST /v1/chat/completions` | OpenAI Chat Completions | OpenCode and other OpenAI-compatible tools |
| `POST /v1/responses` | OpenAI Responses | Responses-API clients |
| `GET /v1/models` | model list | model pickers; lists every advertised model |

All three inference endpoints share one pipeline: authentication, session
check, route and entitlement, governance chain, safety scanners, quota
windows, dispatch, audit. The format a client speaks is independent of the
provider that serves it. An OpenCode request on `/v1/chat/completions` for
`claude-sonnet-5` is served by Claude on Vertex, exactly like the same model
requested on `/v1/messages`.

## Request contract

| Header | Purpose | If missing or wrong |
|--------|---------|-----------|
| `Authorization: Bearer <jwt or PAT>` *or* `x-api-key: <PAT>` | Caller identity | `401` |
| `x-session-id: <session id>` | Binds the call to a session for audit and conversation continuity | missing: `400` (`missing required x-session-id header`); not the caller's session: `401` |
| `x-gateway-conversation-id: <id>` *(optional)* | Pins the conversation; otherwise it is derived from the messages | — |

The session must be one the gateway issued:

- **Bridge JWT.** The session is minted with the token. `x-session-id` must
  equal the JWT's `session_id` claim.
- **Personal access token.** First mint a session with
  `POST /api/public/gateway/sessions`, using the same PAT, then send the
  returned `session_id`. An unknown or revoked session gets `401` with
  `unknown or revoked session; mint one at POST /api/public/gateway/sessions`.

A CLI admin session token is **not** a gateway credential. It fails with
`kid does not match any known signing key`.

## Models

The catalog (`services/ai/providers.yaml`) and the routes
(`services/ai/gateway.yaml`) ship in the image. A request is dispatched only
if its `model` matches a route pattern or is declared in the catalog. The
gateway runs with `allow_unlisted_models: false`, so nothing else is
forwarded, whether or not a `default_provider` is set.

| Family | Ids a client can request |
|---|---|
| Claude (served from Vertex AI, failing over to Anthropic) | `claude-opus-5-5`, `claude-opus-5`, `claude-sonnet-5`, `claude-fable-5-1`, `claude-fable-5`, `claude-haiku-4-5` (also `claude-haiku-4-5-20251001`) |
| Claude on Anthropic's API directly | `anthropic-claude-*`: routable, but hidden from `/v1/models` |
| Gemini on Vertex AI | `vertex-gemini-3.8-flash`, `vertex-gemini-3.5-flash`, `vertex-gemini-3.5-flash-lite` |
| Gemini by API key | `gemini-3.8-flash` |
| Open models on Vertex (retiring 2026-10-21 unless noted) | `qwen.*`, `zai.glm-*` (`zai.glm-5.2` stays), `moonshotai.*`, `deepseek.*`, `minimax.*`, `openai.gpt-oss-*` (`openai.gpt-oss-120b` stays) |

`GET /v1/models` is the authoritative list for a running instance. The
Claude 4.x Opus and Sonnet ids and other retired dated ids are no longer
declared and are refused.

`gateway.default_model` (`claude-sonnet-5[1m]`) is the default the bridge
hands to Claude Code. A request is not rerouted to it. The `[1m]` suffix is
Claude Code's 1M-context marker, which the gateway accepts on any model and
strips before routing.

## Status codes before the upstream call

| Status | Cause |
|---|---|
| `400` | missing `x-session-id`, a malformed body, or a model not permitted by gateway policy (`model '<id>' is not permitted by gateway policy`) |
| `401` | missing, invalid, expired or revoked credential, or a session that does not belong to the caller |
| `403` | the caller is not entitled to the matched route (`authz denied [<policy>]: <reason>`) |
| `429` | a quota window is exhausted. **Not returned on this instance**, where quotas run in warn mode |

Governance and safety denials also answer before the upstream call. Both
planes run in warn mode on this instance and do not deny: see
[Governance Modes](/documentation/governance-modes).

## Governance requirements on routes

Providers and models carry a `governance:` block (`european`, `no_retain`),
and a route can require either with `requires:`. A route whose target does
not satisfy its requirement fails boot, and dispatch checks the resolved
target again. No route on this instance declares a requirement today. The
walkthrough is in
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
