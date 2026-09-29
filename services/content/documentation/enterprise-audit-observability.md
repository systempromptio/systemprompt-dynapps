---
title: "Audit Trail, Traceability & Observability"
description: "Audit every AI call with actor, cost, and latency; follow one trace id from request to tool to cost; export the trail to your own stack over OTLP; and catch cost or error spikes automatically."
author: "Astound Digital"
slug: "enterprise-audit-observability"
keywords: "audit, trace, observability, otlp, export, anomaly detection, requests, traceability, retention"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-29"
after_reading_this:
  - "Inspect any AI request's actor, model, tokens, cost, and latency at /admin/requests"
  - "Follow a single trace id through policy decision, model call, tool execution, and cost at /admin/traces"
  - "Know that the trail is exported to an OTLP collector only once a deployment names one"
  - "Know how long audit records are kept and where the archives are"
  - "Find persisted cost, volume, and error anomalies, and know that nothing pushes an alert"
related_docs:
  - title: "Observability Export (OTLP)"
    url: "/documentation/observability-export"
  - title: "Data Retention & Archives"
    url: "/documentation/data-retention"
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Audit Trail, Traceability & Observability

**TL;DR:** Every model call writes an audit row with the actor, model, the provider that actually served it, tokens, cost, total and upstream latency, and trace ids. You can browse these rows at `/admin/requests`. One trace id links the whole chain (request → policy decision → model call → tool execution → cost), and `/admin/traces` resolves it.

The trail can be pushed to Datadog, Grafana or Splunk through an OTLP collector, but only after a deployment configures one. Raw requests are kept for 90 days, and weekly archives are kept beyond that. An hourly job records cost, volume and error spikes. Nothing pushes an alert: anomalies and warnings go to the log, the console and, once export is on, your collector.

## Per-call audit: /admin/requests

Every request through the gateway lands a row with **DB-locked actor attribution**. The row records:

- who made the call;
- which model was asked for, which route matched, and which provider served it (after a failover, `served_provider` names the fallback);
- token counts in and out, and cost;
- total latency, and the upstream latency spent waiting on the provider;
- status;
- the trace ids that link it onward.

`/admin/requests` lists and filters these rows. `/admin/requests/{id}` opens one, with the governance decisions and safety findings made about it. The serving provider and upstream latency are stored on the row and carried by the [OTLP export](/documentation/observability-export), but the request page does not display them yet. The CLI reaches the same data:

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic
systemprompt infra logs request list --since 2026-09-08 --until 2026-09-12 --user <user-id>
systemprompt infra logs request list --before <cursor>      # the last row's `cursor`: strictly older rows
systemprompt infra logs audit <request-id>                  # identity, model, tokens, cost, counts
systemprompt infra logs audit <request-id> --messages --limit 20 --max-content 400
```

`audit <request-id>` reconstructs the context for one call: identity, policy evaluations, tokens and cost, with `message_count` and `tool_call_count`.

The transcript is opt-in and paged:
- `--messages` and `--tools` include the rows;
- `--offset` and `--limit` walk through them;
- `--max-content` bounds each body.

This means a long session never has to come back as one document. The admin MCP server's `request_log` and `conversation_audit` tools call the same surface.

## End-to-end traceability: /admin/traces

One **trace id** links the entire chain:

1. the inbound request;
2. the governance policy decision;
3. the model call;
4. any MCP or tool executions it triggered;
5. the resulting cost.

`/admin/traces/{id}` resolves any trace id, whichever link you started from, and `/admin/api/chain/{id}` returns the same chain as JSON.

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>
```

## Governance decisions in the trail

Every governance and safety decision is recorded, whether or not it refused anything. The instance currently runs every plane in **warn mode**: stages and scanners run and record `warn` decisions, but nothing is refused (see [Governance Modes](/documentation/governance-modes)). `/admin/governance` shows the decisions and safety findings side by side, and `systemprompt infra logs governance report --since 7d` rolls them up.

## Telemetry in and out

- **In:** Claude Code's own OpenTelemetry output reaches the gateway's `/otel` endpoint through the bridge, which authenticates it. The gateway stores it beside the audit trail.
- **Out:** the `otlp_export` job ships every completed request as a span, with its tool calls and governance decisions as child spans, plus the server log as log records, to the collector named in the profile's `observability.otlp` block.
  - Datadog and Grafana/LGTM take it directly.
  - Splunk takes it through an OpenTelemetry Collector.
  - Neither profile in this repository declares a collector, so nothing is exported until a deployment adds one.
  - Setup, attributes and delivery guarantees are on [Observability Export (OTLP)](/documentation/observability-export).
  - `/admin/system/observability` shows the export's lag and last error, and has **Test connection** and **Export now** buttons.

## Retention

The trail is finite by design.

| Record | Kept for |
|---|---|
| Raw gateway requests | 90 days |
| Stored prompt and completion text | 30 days |
| Raw request bodies | 7 days |
| Governance decisions | 180 days |

After those windows, the daily rollups and per-conversation facts are kept indefinitely. Before raw rows expire, a weekly job archives them to gzipped JSON Lines, which you can download from `/admin/lifecycle`. The windows, the archive jobs and what they mean for audit claims are on [Data Retention & Archives](/documentation/data-retention).

## Anomaly detection

The hourly `usage_anomaly` job compares the last complete hour's requests, cost and errors with the trailing week's hourly average. It flags a metric that is both:

- past its multiplier: 3× for requests and cost, 5× for errors; and
- past an absolute floor: 50 requests, $1, or 10 errors.

The floor stops a quiet instance from flagging 3 requests where there was 1.

Detections are persisted to `usage_anomalies`, so a restart cannot forget them. They are written once to the server log as a warning on first detection, and shown on the console overview (`/admin`) for 24 hours.

**There is no push alerting.** The platform sends no chat, email or webhook alert. The earlier Slack delivery was removed from the codebase. To be paged, turn on the [OTLP export](/documentation/observability-export) and alert on the warning log line in your own stack. The thresholds are first-pass constants and will need tuning against real traffic.

## Known caveats

These limitations are tracked on the [Enterprise Roadmap](/documentation/enterprise-roadmap):

- **Append-only by convention, not by mechanism.** There is no WORM storage or hash-chaining. Whether rows can be changed or deleted depends on database privileges and the retention jobs. Archives sit on the instance's own disk.
- **No alert channel.** See above.
- **Some older rows have no upstream latency or serving provider.** Those columns were not populated before late September 2026, so overhead and failover analysis is reliable only from then on.

## Check it yourself

- `/admin/requests`: open any request and check the actor, model, tokens, cost, latency, policy decisions and safety findings.
- `/admin/traces`: open a trace and follow it to its tool calls and cost.
- `/admin/governance`: the decisions log and safety findings for the same window.
- `/admin/system/observability`: whether a collector is configured, and the export lag.
- `/admin/lifecycle` and `/admin/configuration`: archives, retention windows and the last cleanup run.
- From the CLI:

  ```bash
  systemprompt infra logs request list --since 1h
  systemprompt infra logs trace list --limit 5
  systemprompt infra logs governance report --since 7d
  ```
