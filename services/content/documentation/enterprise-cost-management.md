---
title: "Cost Management & FinOps"
description: "Attribute every AI request's cost to a user, group, project, model and provider; read the Cost tab; watch spend against quota-window warning thresholds; and export cost reports."
author: "Astound Digital"
slug: "enterprise-cost-management"
keywords: "cost, finops, spend, quota windows, warning thresholds, chargeback, csv, cost tab, provider cost"
kind: "guide"
public: true
tags: ["enterprise", "finops", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-29"
after_reading_this:
  - "Read the Cost tab: spend by day, by provider and by model, and consumption by group or project"
  - "Attribute any request's cost to its user, group, project, model, and provider"
  - "Read and edit the quota windows, and know that they warn rather than refuse"
  - "Know what is not built: budgets that stop spend, forecasts, digests, alert delivery"
  - "Export cost reports from the web UI or the CLI"
related_docs:
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Cost Management & FinOps

**TL;DR:** Every AI request carries its own cost, attributed to a user, their primary group and project, a model and a provider. The **Cost** tab on `/admin/analytics` shows spend by day, provider and model, and consumption by group or project. Every table there exports.

Spend is watched by the gateway's **quota windows**: $20 and 600 requests per user per hour, and $200 per day for the whole instance. Since 2026-09-04 those windows only **warn**. A crossing is recorded and reported, and the request still runs, so nothing on this instance stops spend.

There are no forecasts, no scheduled digests and no alert delivery.

## Per-request attribution

Cost is not a rollup bolted on afterwards. **Each request row carries user, model, provider, tokens and cost.** Cost is stored as microdollar integers end to end and priced from the model catalogue at the rate of the provider that actually served the request, so a failover is repriced at the fallback's rate.

A person can belong to several groups and projects. Their cost is attributed to the **primary** group and project from their scope defaults, so container totals add up to the instance total rather than double-counting. Chargeback to a project is a filter, not a reconciliation exercise.

```bash
systemprompt analytics costs summary
systemprompt analytics costs breakdown --by user --since 30d      # spend, requests, tokens, conversations per user
systemprompt analytics costs breakdown --by provider --since 30d
systemprompt infra logs request list --limit 20 --user <user-id>
```

**Cost is computed, not billed.** It comes from the price catalogue applied to observed token counts. That makes it an accurate estimate, not an invoice. Reconcile it against the provider's bill during acceptance.

## The Cost tab

`/admin/analytics?tab=cost` shows, for the selected window and scope (everyone, one group, one project, or one person):

- **Cost KPIs and a cost-by-day chart.**
- **Providers:** requests, share, tokens and cost per supplier.
- **Models:** the same spend, split by the model that earned it.
- **Consumption by container:** users, sessions, requests and share per group or per project (toggle **Group by**). This table reads the daily rollups, which keep the group and project a person held on the day the row was written.

Every provider and model row drills into `/admin/requests` with that dimension applied.

The tab has two audiences:

- The internal view (the default) is the supplier bill.
- `?audience=customer` shows consumption only. The customer view **carries no cost column at all**: its query selects none, so a report shared outside the platform team cannot leak spend.

For one project on one printable page, including its members, usage and configuration, use `/admin/projects/{id}/report`.

## Quota windows: warning thresholds

Spend thresholds are the gateway's quota windows, declared in `services/gateway/policies.yaml`:

| Window | Subject | Threshold |
|---|---|---|
| Hourly | each user | 600 requests or $20 |
| Daily | the whole instance (`organization`) | $200 |

The per-user window is sized to flag a runaway agent within the hour. The instance window flags an unusually expensive day.

The quota plane runs in **warn mode** (`quota_mode: warn`, since 2026-09-04). A window past its threshold is recorded as a `warn` decision under policy `quota`. It appears in `systemprompt infra logs governance report` and on `/admin/governance`, and the request is never refused. [Governance Modes](/documentation/governance-modes) explains the measurement window and how a plane goes back to enforce. Under enforce, a spent window answers HTTP 429 with `retry-after`.

Costs are attributed one request late by design: a request's cost is only known after its response, so the warning lands on the request after the crossing.

### Subjects and periods

A window names a **subject** (whose usage it counts) and a **period**.

The subjects this instance resolves are:
- `user` (core's own);
- `group`, `project` and `role`, whose providers are in `extensions/web/admin/identity/src/authz/`, and `salesforce` (the linked account), in `extensions/web/admin/platform/src/authz/`;
- `organization` (the whole installation, subject id `default`), whose provider is in `extensions/web/shared/admin-core/src/authz/organization.rs`.

A person on several groups or projects is counted against their primary one. `quota_fault_mode: closed` means a window whose subject cannot be resolved refuses the request rather than passing it uncounted.

**Only the two windows above are declared.** A monthly window per project, group or role is supported but not configured.

Periods are an hour, a day, a week, any number of seconds, or a **calendar month**. Core's windows are fixed-length and aligned to the Unix epoch, so the calendar month is an interim provided by the admin extension:

- The window is declared as `window_seconds: 2678400` (31 days).
- The daily `quota_month_window` job rewrites the live row so it ends at the month's end.
- The same job carries yesterday's usage bucket into today's, so the bucket holds month-to-date usage and the first of the month starts from zero.
- The one gap is the first minute after UTC midnight, while core's policy cache is stale.

### Two console pages

Both pages read and write the same table, `ai_gateway_policies`:

- **`/admin/gateway/policies`** edits the windows, the safety scanners and block lists, and the warn/enforce switch of each plane.
  - Core re-reads the table on every request (with a sixty-second cache), so a save takes effect without a restart or a file edit.
  - `/admin/sync` then shows the edit as drift on the `gateway_policies` plane, until it is exported to `services/gateway/policies.yaml` and committed, or overwritten from code.
- **`/admin/governance/quotas`** shows usage against each window's threshold, per subject, for the current period, with a meter for the threshold closest to being reached. Under warn mode, a subject past 100% was warned about, never refused.

## What is not built

These controls are **not** available. The roadmap records each one:

- **Budgets that stop spend.** The quota windows can enforce, but they are switched to warn by decision, and no per-project or per-group monthly window is declared.
- **Forecasting or burndown.** Nothing projects month-end spend from the run rate.
- **Scheduled digests.** No job sends a periodic cost summary.
- **Alert delivery.** A threshold crossing is a `warn` row and a log line; no email, chat or webhook message is sent. With the [OTLP export](/documentation/observability-export) turned on, those log lines reach your own alerting stack.

## Self-service reporting

- **Web:** every table on the analytics pages, including the Cost tab, has an **Export** action. It downloads the rows on screen, with the same scope and window, as CSV, JSON, JSON Lines or Markdown (`/admin/export/{dataset}`).
- **CLI:** cost reports accept `--since` / `--until` and `--export <file.csv>`. `analytics costs breakdown --by model|provider|user|agent` slices a window, and `analytics costs trends` gives the series. `systemprompt analytics --help` lists the rest.

## Provider and model comparison

The Cost tab's Providers and Models tables, and `analytics costs breakdown --by provider`, show what each supplier and model cost for the same traffic. This lets you estimate what a route change would save.

Quality-normalised comparison ("is the cheaper model good enough?") needs an agreed quality baseline for your workloads and is not built. See the [roadmap](/documentation/enterprise-roadmap#coming).

## Check it yourself

- `/admin/analytics?tab=cost`: the supplier bill for the window. Switch **Group by** between group and project.
- `/admin/analytics?tab=cost&audience=customer`: the same consumption with no cost column.
- `/admin/governance/quotas`: usage against each window's threshold.
- `/admin/gateway/policies`: the live quota windows and `quota_mode`.
- From the CLI:

  ```bash
  systemprompt analytics costs summary --since 7d
  systemprompt analytics costs breakdown --by provider --since 30d
  systemprompt infra logs governance report --since 7d
  ```
