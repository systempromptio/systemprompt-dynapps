---
title: "Data Retention & Archives"
description: "How long each kind of record is kept, what is deleted and when, what survives indefinitely, and where the weekly and monthly archives land."
author: "Astound Digital"
slug: "data-retention"
keywords: "retention, data lifecycle, archive, deletion, audit trail, ai_requests, governance decisions, compliance, gdpr"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin", "compliance"]
published_at: "2026-09-29"
updated_at: "2026-09-29"
after_reading_this:
  - "Know how long requests, prompts, governance decisions and logs are kept before deletion"
  - "Know which records are kept indefinitely"
  - "Find and download the weekly and monthly archives from /admin/lifecycle"
  - "Read the live retention windows and the last cleanup run on /admin/configuration"
  - "Understand what finite retention means for audit and history claims"
related_docs:
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Conversation History & Search"
    url: "/documentation/enterprise-conversation-history"
  - title: "Observability Export (OTLP)"
    url: "/documentation/observability-export"
---

# Data Retention & Archives

**TL;DR:** Nothing on this instance is kept forever by accident. Raw gateway requests are deleted after **90 days**. Stored prompt and completion text goes after **30 days**, and raw request bodies are released after **7**. Governance decisions are kept for **180 days**. What survives indefinitely is the rolled-up record: daily usage rollups, per-conversation facts and detected anomalies. Before raw rows expire, a weekly job archives them to gzipped JSON Lines, which you can download from `/admin/lifecycle`.

## Why this matters

Retention is itself a compliance fact. Any claim about the audit trail or conversation history is bounded by it: a question about a request from four months ago can only be answered from the archive, not from the console. If your policy requires a longer or shorter window, the windows are profile settings, and changing them is a deployment decision.

## What is deleted, and when

Two jobs delete data. Both run nightly.

- **`database_cleanup`** (core, 04:00 UTC) enforces the windows in the profile's `retention:` block. Rows are deleted in batches of 5,000. The job only deletes because it is scheduled with `enforce: true`; without that it would just report what it would remove.
- **`plugin_usage_retention`** (web extension) runs `expire_raw_evidence` with a 90-day cutoff. That removes gateway requests and everything attached to them, hook events, and ended browser sessions, all in one transaction. The database function refuses any cutoff younger than 90 days, so this window cannot be shortened by accident.

These are the windows this installation runs with. The profile sets no `retention:` overrides, so the first group are core's defaults:

| Record | Kept for | Set by |
|---|---|---|
| Gateway requests (`ai_requests`) and the tool calls, scopes and safety findings attached to them | 90 days | `plugin_usage_retention` |
| Hook and plugin usage events | 90 days | `plugin_usage_retention` |
| Stored prompt and completion text (`ai_request_messages`) | 30 days | `ai.history.retention_days` in `services/ai/config.yaml` |
| Raw request and response bodies (`ai_request_payloads`) | 7 days, then set to NULL. Excerpts, hashes and sizes stay until the request itself expires. | profile default |
| Governance decisions | 180 days | profile default |
| MCP tool execution records | 365 days | profile default |
| Server log lines (`logs`) | 30 days | profile default |
| Traffic and behaviour events | 90 days | profile default |
| Processed durable events (`event_outbox`) | 7 days | profile default |
| Spent quota windows | 62 days | fixed (longest quota window plus carry-forward) |

`/admin/configuration` is the authority for a running instance. It reads the windows from the live profile and shows how many rows the last `database_cleanup` run deleted per table, and when the next run is due. If a deployment's profile overrides a window, that page shows the override, not the table above.

## What is kept indefinitely

These records are never expired by age:

- **Daily usage rollups** (`admin_usage_daily_rollups`, `plugin_usage_daily`). The analytics dashboards read these, so long-range usage and cost trends survive after the raw requests are gone.
- **Conversation facts** (`conversation_facts`, `conversation_skill_facts`). The deterministic per-conversation and per-skill record that the Analysis pages read.
- **Detected anomalies** (`usage_anomalies`).
- **The retention ledger itself** (`retention_runs`).

Once the raw rows expire, the platform can still tell you that a conversation happened, who had it, what it cost and which skills it used. It can no longer show you the prompt text or the individual requests. Those are in the archive.

## Archives

Three jobs keep an off-database copy before anything is deleted.

| Job | When | What it writes |
|---|---|---|
| `retention_daily_report` | daily, 04:30 UTC | Size and oldest row per managed table into `retention_runs`, so growth is a series rather than a guess. |
| `retention_export_weekly` | Sundays, 02:00 UTC | The raw request tables for the previous ISO week, one gzipped JSON Lines file per table plus a manifest, under `storage/exports/weekly/<yyyy>-W<ww>/`. It keeps 26 weeks, and prunes a week only once the monthly archive covering it exists. |
| `retention_export_monthly` | 1st of the month, 02:30 UTC | The rollup tables under `storage/exports/monthly/<yyyy>-<mm>/`, kept indefinitely. It then vacuums the managed tables and runs the health check. |

The weekly archive is the only copy of a request once it has passed its window.

Archives are written to the instance's own storage volume. They are not a separate or write-once store: anyone with access to the host filesystem can change or delete them. If you need tamper-evident retention, copy them off the host (see [Known limits](#known-limits)).

## The Data lifecycle page

`/admin/lifecycle` (console roles only) shows what the retention jobs measured, archived and found:

- **Measurements:** the latest size and row count per managed table, with week-on-week growth.
- **Archives:** every weekly and monthly archive, with its digest and a download link per table file and manifest.
- **Health report:** the last monthly check's ranked findings.
  - P1 means a queue is not draining, or a table's oldest row is a week past its expected window, which means retention is not being applied.
  - P2 means growth that will hurt within a year.
  - P3 is hygiene.

## Known limits

- **No write-once storage.** Rows and archives are protected by database and filesystem permissions, not by WORM storage or hash-chaining. The [roadmap](/documentation/enterprise-roadmap) records the hardening options.
- **Archives stay on the instance.** Nothing copies them off the host automatically. The [OTLP export](/documentation/observability-export) streams requests, decisions and logs to an external collector as they happen, which can serve as the off-host copy if your collector keeps them.
- **The health check expects shorter windows than some deletions use.** It judges `logs` against 14 days, `analytics_events` against 30 and `mcp_tool_executions` against 180. The profile defaults keep those tables for 30, 90 and 365 days, so until the two agree, expect the health report to list those tables as P1 findings.

## Check it yourself

- `/admin/configuration`: the live retention windows, rows deleted per table on the last run, and the next due time.
- `/admin/lifecycle`: measurements, archive downloads and the health report.
- Run a job out of turn:

  ```bash
  systemprompt infra jobs run retention_daily_report
  systemprompt infra jobs run database_cleanup
  systemprompt infra jobs history --job database_cleanup
  ```
