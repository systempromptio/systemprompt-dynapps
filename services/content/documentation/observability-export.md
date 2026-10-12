---
title: "Observability Export (OTLP)"
description: "Ship the audit trail — every AI request with its tool calls and governance decisions, plus the server log — to Datadog, Grafana/LGTM, Splunk or any OpenTelemetry collector."
author: "Dynapps"
slug: "observability-export"
keywords: "otlp, opentelemetry, datadog, splunk, grafana, lgtm, google cloud, cloud logging, cloud trace, bigquery, chargeback, siem, export, telemetry, observability, upstream latency"
kind: "guide"
public: true
tags: ["enterprise", "audit", "observability", "admin"]
published_at: "2026-09-29"
updated_at: "2026-10-08"
after_reading_this:
  - "Configure the observability.otlp profile block to point at your collector"
  - "Know exactly which records are exported as spans and log records, and which attributes they carry"
  - "Test the connection and force an export from /admin/system/observability"
  - "Forward the export to Cloud Logging and Cloud Trace through an OpenTelemetry Collector"
  - "Know what is not exported: history before the first export, metrics, and archives"
related_docs:
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Data Retention & Archives"
    url: "/documentation/data-retention"
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
---

# Observability Export (OTLP)

**TL;DR:** The `otlp_export` job pushes the audit trail to any OpenTelemetry collector over OTLP/HTTP. Every completed AI request becomes a span, with its tool calls and governance decisions as child spans. Every server log line becomes a log record. To turn it on, add an `observability.otlp` block to the profile and restart; a profile without the block exports nothing. **Neither profile shipped with this repository declares the block**, so no telemetry leaves the instance until a deployment names a collector.

## What is exported

| Signal | Source | Shape |
|---|---|---|
| `traces` | `ai_requests` (completed rows), `tool_call_ledger`, `governance_decisions` | One span per AI request. Each tool call belonging to it, and each governance decision made under its trace, is a child span. |
| `logs` | the `logs` table | One OTLP log record per row. |

A request span carries the OpenTelemetry GenAI attributes plus the platform's own:

- **GenAI:** `gen_ai.system`, `gen_ai.request.model`, `gen_ai.response.model`, `gen_ai.response.finish_reason`, and the input, output, cache-read and cache-creation token counts.
- **Identity:** `enduser.id`, `session.id`, `systemprompt.actor.id`, `systemprompt.actor.kind`, `systemprompt.client.kind`.
- **Routing and cost:** `systemprompt.route`, `systemprompt.served_provider` (the provider that actually answered, which differs from the route's provider after a failover), `systemprompt.wire_protocol`, `systemprompt.cost.usd` and `systemprompt.cost.microdollars`.
- **Timing:** `systemprompt.latency_ms` (total) and `systemprompt.upstream_latency_ms` (time spent waiting on the provider). The difference between the two is the gateway's own overhead, so you can chart it in your own stack. Some older rows have no upstream figure.
- **Attribution:** `systemprompt.scope.<dimension>` and `systemprompt.scope.<dimension>.source` (`header`, `api_key` or `default`) for every scope dimension the request was attributed to (here `project`, `cost_centre` and `practice` among them), and `systemprompt.api_key.id` for the key that authenticated it. These are the chargeback keys.
- **Correlation:** `systemprompt.request.id`, `systemprompt.trace.id`, `systemprompt.context.id`, `systemprompt.instance.id`, `systemprompt.request.kind`, `systemprompt.request.status`.

Prompt and completion bodies are **not** exported. The spans carry the accounting and the chain, not the conversation.

## Configure it

The block lives at the top level of the deployment's `profile.yaml`:

```yaml
observability:
  otlp:
    endpoint: https://otel-collector.example.internal:4318
    protocol: http            # the only supported value; see below
    headers:
      Authorization: "Bearer <collector token>"
    signals: [traces, logs]   # default: both
    batch_seconds: 15         # 1–3600; minimum spacing between two exports of a signal
```

- **`endpoint`** is the collector's base URL. The exporter appends `/v1/traces` and `/v1/logs`, and uses an endpoint that already ends in one of those paths as given. Profile validation requires an `http://` or `https://` URL with a host.
- **`protocol`** must be `http`. The exporter speaks OTLP/HTTP with protobuf bodies. `grpc` is refused at profile validation, so point the exporter at your collector's HTTP receiver, conventionally port 4318.
- **`headers`** are sent verbatim on every export request. Put your vendor's API key or bearer token here. The production profile is not committed to git, so a token does not land in the repository.
- The job runs every 15 seconds. `batch_seconds` sets a floor on how often each signal ships.

The profile is read once at boot, so adding or changing the block needs a restart. Without the block, the job is not scheduled at all.

### Choosing a target

- **Datadog:** send OTLP/HTTP straight to the Datadog Agent's or OpenTelemetry Collector's OTLP receiver.
- **Grafana / LGTM:** send to Grafana Alloy or an OpenTelemetry Collector that forwards traces to Tempo and logs to Loki.
- **Splunk:** put an OpenTelemetry Collector in front, with Splunk HEC as its exporter. The platform does not speak HEC directly.

### Google Cloud

> **Example, untested on GCP.** This configuration follows the OpenTelemetry Collector
> Contrib `googlecloud` exporter's documented options. Dynapps has not run it against a
> Google Cloud project; validate it in a non-production project first.

The gateway speaks OTLP/HTTP only, and Google Cloud's services do not take that push
directly from it, so run an OpenTelemetry Collector (the `otelcol-contrib` image, or
Google's build of it) beside the instance, for example as a sidecar or a small
Deployment on GKE. Its service account needs `roles/cloudtrace.agent` and
`roles/logging.logWriter`.

```yaml
# otel-collector.yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

processors:
  memory_limiter:
    check_interval: 1s
    limit_percentage: 75
  batch: {}
  resourcedetection:
    detectors: [gcp]

exporters:
  googlecloud:
    project: <gcp-project-id>   # omit to use the detected project
    log:
      default_log_name: systemprompt

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, resourcedetection, batch]
      exporters: [googlecloud]   # -> Cloud Trace
    logs:
      receivers: [otlp]
      processors: [memory_limiter, resourcedetection, batch]
      exporters: [googlecloud]   # -> Cloud Logging
```

Then point the profile at the collector's HTTP receiver:

```yaml
observability:
  otlp:
    endpoint: http://otel-collector:4318
    protocol: http
    signals: [traces, logs]
```

- **Cloud Trace** receives one trace per AI request, with tool calls and governance
  decisions as child spans. Filter on `systemprompt.scope.project` or
  `systemprompt.served_provider`.
- **Cloud Logging** receives the server log under the `systemprompt` log name.
- **BigQuery.** A Cloud Logging sink with a BigQuery dataset as destination
  (`gcloud logging sinks create <name> bigquery.googleapis.com/projects/<project>/datasets/<dataset> --log-filter='logName:"systemprompt"'`)
  lands the log records as tables. Spans go to Cloud Trace, not Cloud Logging, so the
  sink does not carry them; a chargeback table needs the span attributes
  (`systemprompt.scope.project`, `systemprompt.scope.cost_centre`,
  `systemprompt.scope.practice`, `systemprompt.api_key.id`,
  `systemprompt.cost.microdollars`, `gen_ai.request.model`) routed to the warehouse by
  an additional collector pipeline, which this example does not include.

## Delivery guarantees

- **At least once.** Each signal has a watermark in `otlp_export_state`. The watermark only advances after the collector acknowledges a batch.
  - A batch that fails is retried three times within the run.
  - If it still fails, it keeps its cursor and is sent again at the next tick. Nothing is skipped.
- **No duplicates on retry.** Span and record ids are digests of the row keys, so a re-sent batch overwrites rather than duplicates.
- **Retryable responses** are the ones OTLP/HTTP names (429, 502, 503, 504) and connection failures. Any other 4xx is not retried within the run. The watermark still does not move, so the signal stalls until the cause (usually a wrong token or path) is fixed. The console page shows it as the last error, with growing lag.
- **No backfill.** A signal's watermark starts at "now" the first time it exports. History from before you configured the block is not sent. Use the [archives](/documentation/data-retention#archives) for that.

## The console page

`/admin/system/observability` (console roles) shows:

- **The configured block:** endpoint, protocol, signals, batch spacing.
- **The export ledger:** for each signal, the current watermark, lag (now minus watermark), last success, last error, and batch counters.
- **Test connection:** posts an empty OTLP/HTTP envelope with the exact URL, headers and content type the exporter uses. A 2xx means a real batch will be accepted. Nothing is exported and no watermark moves.
- **Export now:** runs one export immediately, without waiting for `batch_seconds`.

## What is not exported

- **Metrics.** The Prometheus `/metrics` listener carries series such as:
  - `gateway_upstream_failovers_total{from,to,reason}` (every failover hop);
  - `systemprompt_quota_denials_total{subject_kind,dimension,mode}` and `systemprompt_usage_anomaly_total{kind,subject_kind}` (the alert signals);
  - `gateway_overhead_seconds` and `gateway_upstream_duration_seconds` (latency histograms);
  - `otlp_export_batches_total{signal,status}`.

  Metrics are not part of the OTLP push. The listener only starts when the profile sets `server.metrics_port`, and neither shipped profile sets it, so scrape it only after a deployment enables it.
- **Alerts.** The platform sends no email, chat or webhook alerts. Budget-threshold warnings and usage anomalies are recorded as log lines, database rows, metrics and (for `usage_anomaly_scan`) a `usage_anomaly` event. Once the export is on, those log lines reach your collector, and alert rules belong there.
- **Archives and rollups.** Archives stay on the instance's storage. See [Data Retention & Archives](/documentation/data-retention).

## Check it yourself

- `/admin/system/observability`: use **Test connection** first, then **Export now**, then check that lag stays near the batch spacing.
- In your collector, filter for spans whose `systemprompt.instance.id` matches this instance, and compare a span's `systemprompt.request.id` with `/admin/requests/{id}`.
- Confirm the job is scheduled (it only appears once the block is present), and that requests are arriving to export:

  ```bash
  systemprompt infra jobs show otlp_export
  systemprompt infra logs request list --since 1h
  ```
