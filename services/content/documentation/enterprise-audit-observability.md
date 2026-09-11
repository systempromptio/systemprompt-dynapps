---
title: "Audit Trail, Traceability & Observability"
description: "Audit every AI call with actor, cost, and latency; follow one trace id from request to tool to cost; ingest OTLP; and catch cost or error spikes automatically."
author: "Astound Digital"
slug: "enterprise-audit-observability"
keywords: "audit, trace, observability, otlp, sse, anomaly detection, requests, traceability"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-11"
after_reading_this:
  - "Inspect any AI request's actor, model, tokens, cost, and latency at /admin/entities/requests"
  - "Follow a single trace id through policy decision, model call, tool execution, and cost"
  - "Ingest OTLP telemetry and watch the live audit stream over SSE"
  - "Inspect persisted cost, volume, and error anomalies"
related_docs:
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Audit Trail, Traceability & Observability

**TL;DR:** Every model call writes an audit row — actor, model, provider, tokens, cost, latency, trace ids — browsable at `/admin/entities/requests`. One trace id links the whole chain (request → policy decision → model call → tool execution → cost) and resolves at `/admin/entities/traces`. The platform ingests OTLP, streams live audit events over SSE, and a persisted anomaly job records cost, volume, and error spikes.

## Per-call audit: /admin/entities/requests

Every request through the gateway lands a row with **DB-locked actor attribution**: who made the call, which model and provider served it, token counts in and out, cost, latency, status, and the trace ids that link it onward. `/admin/entities/requests` lists and filters these rows; the CLI reaches the same data:

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic
systemprompt infra logs audit <request-id>
```

`audit <request-id>` reconstructs the full context for one call — identity, policy evaluations, prompt, response, and cost.

## End-to-end traceability: /admin/entities/traces

One **trace id** correlates the entire chain: the inbound request, the governance policy decision, the model call, any MCP or tool executions it triggered, and the resulting cost. `/admin/entities/traces` is the universal chain resolver — paste a trace id and read the whole story, whichever link you started from.

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>
```

## Observability ingest and live stream

- **OTLP ingest** — the platform accepts OpenTelemetry traces, logs, and metrics, so instrumented clients and services report into the same place.
- **Live audit stream** — audit events stream as JSON over **Server-Sent Events**, useful for watching an incident unfold or feeding a live wallboard.

## Anomaly detection

A **persisted anomaly job** watches for cost spikes, volume spikes, and error spikes against recent baselines. Detections are persisted to `usage_anomalies`, logged on first detection per window, and shown on the dashboard. This installation has no configured Slack delivery. The current detectors are threshold-based rather than learned baselines; tuning iterates with real traffic.

## Known caveats

Two limitations are stated here once and tracked on the [Enterprise Roadmap](/documentation/enterprise-roadmap):

- **Append-only by convention, not mechanism.** There is no WORM storage or hash-chaining. Database privileges and retention/cleanup operations determine whether rows can be changed or deleted.
- **No push egress.** Telemetry flows *in* (OTLP ingest, SSE out to connected clients), but there is no built-in exporter pushing audit data to Datadog, Splunk, or an external OTEL collector.

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured. Screenshots below are illustrative and may show an earlier configuration.

### Screenshots

![A request audit record showing actor, model, policy chain, cost, and trace ids](/files/images/evidence/req-026-requests-audit.png)

![One trace id resolving the full chain: session, identity, and spans](/files/images/evidence/req-027-trace-chain.png)

![Latency reported against a configured SLO threshold on the spend dashboard](/files/images/evidence/req-029-latency-slo.png)

![The spend dashboard's budget warnings and anomalies sections](/files/images/evidence/req-009-spend-meters.png)
