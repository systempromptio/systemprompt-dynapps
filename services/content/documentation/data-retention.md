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
updated_at: "2026-10-08"
after_reading_this:
  - "Know how long requests, prompts, governance decisions and logs are kept before deletion"
  - "Know which records are kept indefinitely"
  - "Find and download the weekly and monthly archives from /admin/lifecycle"
  - "Read the live retention windows and the last cleanup run on /admin/configuration"
  - "Understand what finite retention means for audit and history claims"
  - "Know what the audit guard refuses, how the hash chain works and how to verify it"
  - "Archive, restore and legal-hold a user account"
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
| Archived user accounts and every row keyed on them | 90 days after archiving; never while under legal hold | profile `retention.archived_users_days` |
| Audit chain links (`audit_chain_links`) | 100 days | `audit_chain_verify` |

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

## Tamper resistance

The audit tables are append-only by mechanism, not only by convention. A set of database triggers checks every write, whichever path it comes from: core, this extension, the data console or a `psql` session.

**What the guard checks**

| Table | UPDATE | DELETE | TRUNCATE |
|---|---|---|---|
| `ai_requests` | Constrained (see below) | Only rows older than 90 days | Refused |
| `ai_safety_findings` | Refused | Only rows older than 90 days | Refused |
| `ai_request_attributions` | Refused | Only with the request it belongs to | Refused |
| `governance_decisions` | Refused (core's own trigger) | Only rows older than 30 days | Refused |
| `mcp_tool_executions` | Allowed (results land after the call) | Only rows older than 30 days | Refused |
| `plugin_usage_events` | Allowed (execution links, account merges) | Only rows older than 90 days | Refused |
| `secret_audit_log` | Refused | Only rows older than 365 days | Refused |
| `platform_data_clear_audit` (the data console's deletion receipts) | Refused | Never | Refused |
| `audit_chain_links`, `audit_guard_violations` | Refused | Only rows past their floor | Refused |

Each floor is at or below the retention window that deletes the table, so the scheduled retention jobs pass the guard. A delete that arrives by foreign-key cascade is judged by its parent: when a request expires, its findings and attributions go with it.

A gateway request is settled after it is written, so `ai_requests` cannot be fully frozen. Instead:

- `id`, `request_id`, `created_at`, `actor_kind`, `actor_id` and `api_key_id` never change.
- Once a request is `completed` or `rejected`, its routing and usage are frozen. That covers `provider`, `served_provider`, `model`, `requested_model`, the token counts, `cost_microdollars` and `finish_reason`.
- A completed request may only move to `failed`, which is what an accounting failure does.
- Everything else stays writable: the owner during an account merge, session and context links, the message count, error and accounting fields, and timings.

**Modes.** The guard has three modes, stored in `audit_guard_settings`:

- `off`: nothing is checked.
- `warn` (the default): the write goes through and a row lands in `audit_guard_violations`.
- `enforce`: the write is refused with `restrict_violation`.

`warn` follows the measurement-window posture the rest of this installation runs in. Move to `enforce` once `audit_guard_violations` has stayed empty under real traffic:

```sql
SELECT set_audit_guard('enforce', TRUE);   -- mode, hash chain on
```

A deliberate backfill, or a legal erasure inside a floor, runs in its own transaction with `SET LOCAL systemprompt.audit_maintenance = 'on'`. That makes it a visible, reviewable act rather than an ordinary statement.

**The hash chain.** The `audit_chain_seal` job runs every 10 seconds and appends a row to `audit_chain_links` for each new `ai_requests` and `ai_safety_findings` row. The row holds a sequence number, the previous link's hash, and `sha256(previous hash + the row's canonical fields)`:

- For a request, the canonical fields are its immutable columns.
- For a safety finding, they are every column.

`verify_audit_chain(table, from, to)` recomputes the chain and reports five kinds of break:

- missing links (a gap in the sequence)
- a link whose previous hash does not match
- a row whose hash no longer matches (altered)
- a row deleted inside the 90-day floor
- a row with no link at all that the sealer has already passed (written while the chain was off)

The `audit_chain_verify` job runs nightly at 04:45 UTC. It checks the last two days and writes one `audit_chain_verifications` row per table, including the anchor hash it started from and the head hash it reached. The run fails if it finds any break, so the failure appears in the job history.

The seal is eventually consistent: a new row is linked within one seal interval (10 seconds), and rows still waiting are counted as `unsealed_rows` on each verification and in the job's run message. Gateway inserts no longer take the chain's row lock; one replica seals at a time under an advisory lock. Tamper refusal is not deferred: the UPDATE, DELETE and TRUNCATE guards still run inside the writing statement. `SELECT set_audit_guard('warn', FALSE)` turns the chain off and the sealer stops.

**What an auditor can and cannot rely on**

- *Can:* the application, the console and an ordinary SQL session cannot silently rewrite or remove recent audit rows under `enforce`. Under `warn`, they cannot do it without leaving a violation row. A row changed or removed out-of-band shows up as a chain break on the next nightly run.
- *Can:* verify the chain on demand:

  ```sql
  SELECT * FROM verify_audit_chain('ai_requests', now() - interval '30 days', now());
  SELECT run_at, table_name, links_checked, problems, anchor_hash, head_hash
  FROM audit_chain_verifications ORDER BY run_at DESC LIMIT 10;
  ```

- *Cannot:* rely on any of this against the database owner. The application role owns these tables, and an owner can disable a trigger or recompute the whole chain. The defence is to keep the recorded head hashes off the instance: the weekly archives, the OTLP stream, or a copy of `audit_chain_verifications`. A rewritten chain then disagrees with heads recorded earlier. Separating the owning role from the runtime role, and write-once storage, are open decisions (DEC-007).
- *Cannot:* rely on sign-in history or access-rule edits to be covered. `user_sessions` and the access-control tables are mutable operational state and are not guarded.

## Archived users

Deleting a user archives the account (NFR-5.2). Archiving does four things:

- The account moves to status `deleted`, so it can no longer sign in.
- It leaves the user listings and rosters.
- Its sessions, API keys and device certificates are revoked in the same transaction.
- Who archived it, when and why are recorded on the `users` row.

Its history stays. The account can be restored within `retention.archived_users_days` (90 days by default). After that, `database_cleanup` purges it with every row keyed on it, unless it is under legal hold. A restored account signs in again, but credentials revoked at archive time stay revoked.

```bash
systemprompt admin users delete <user> --yes --reason "left the company"   # archive
systemprompt admin users delete <user> --yes --legal-hold                   # archive under hold
systemprompt admin users restore <user>
systemprompt admin users legal-hold <user> [--release]
systemprompt admin users delete <user> --yes --purge                        # physical, archived and not held only
```

The console's user page has the same **Archive** and **Restore** actions. API keys are never deleted on their own: revoking one sets `revoked_at` and keeps the row.

## Known limits

- **No write-once storage.** The guard and the hash chain make tampering refused or visible, but rows and archives still sit on the instance. See [Tamper resistance](#tamper-resistance) for what that does and does not prove.
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
  systemprompt infra jobs run audit_chain_verify
  ```

- Read the guard's findings while it runs in `warn`:

  ```sql
  SELECT table_name, operation, rule, count(*) FROM audit_guard_violations
  WHERE created_at > now() - interval '7 days' GROUP BY 1, 2, 3 ORDER BY 4 DESC;
  ```
