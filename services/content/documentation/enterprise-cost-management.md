---
title: "Cost Management & FinOps"
description: "Attribute every AI request's cost to a user, group, project, model and provider; read the Cost tab; watch spend against quota-window warning thresholds; and export cost reports."
author: "Dynapps"
slug: "enterprise-cost-management"
keywords: "cost, finops, spend, quota windows, warning thresholds, chargeback, csv, cost tab, provider cost, cost centre, practice, api key budget, otlp, bigquery"
kind: "guide"
public: true
tags: ["enterprise", "finops", "admin"]
published_at: "2026-08-25"
updated_at: "2026-10-08"
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

Spend is watched by the gateway's **quota windows**: $20 and 600 requests per user per hour, $100 per project per day, and $200 per day for the whole instance. An API key can carry its own budget on top. Since 2026-09-04 those windows only **warn**. A crossing is recorded and reported, and the request still runs, so nothing on this instance stops spend.

There are no forecasts, no scheduled digests and no alert delivery.

## Per-request attribution

Cost is not a rollup bolted on afterwards. **Each request row carries user, model, provider, tokens and cost.** Cost is stored as microdollar integers end to end and priced from the model catalogue at the rate of the provider that actually served the request, so a failover is repriced at the fallback's rate.

A person can belong to several groups and projects. Their cost is attributed to the **primary** group and project from their scope defaults, so container totals add up to the instance total rather than double-counting. Chargeback to a project is a filter, not a reconciliation exercise.

### Scope attribution: project, cost centre, practice

Every gateway request is also attributed to one value in each **scope dimension** the instance registers, and each value is stored in `ai_request_attributions` with its source:

- `header`: the client sent `x-systemprompt-scope-<dimension>` (see [Gateway API](/documentation/gateway-api#scope-headers));
- `api_key`: the API key is bound to that value;
- `default`: the caller's primary value.

A caller who works on two projects can therefore charge each request to the right one by header, instead of everything landing on their primary project. A header or bound value the caller does not hold is refused with `403`.

On this instance three dimensions are registered for chargeback:

| Dimension | Values a person holds | Default |
|---|---|---|
| `project` | the projects they are a live member of | their primary project |
| `cost_centre` | the cost centres of those projects (a project's optional **Cost centre** field) | their primary project's cost centre |
| `practice` | the practices of those projects (a project's optional **Practice** field) | their primary project's practice |

Each dimension is resolved on its own, by the same header → API key binding → default order. A caller on several cost centres picks one with `x-systemprompt-scope-cost_centre`, and likewise for `practice`. None of the three changes an access decision; they exist for attribution and for quota windows keyed on them.

The request row also records `api_key_id`, the key that authenticated it, so spend per key is available alongside spend per user.

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
| Daily | each project (`project`) | $100 |
| Daily | the whole instance (`organization`) | $200 |

The per-user window is sized to flag a runaway agent within the hour. The per-project window flags a project's unusually expensive day, and the instance window the whole installation's. Because `quota_fault_mode` is `closed`, a person on no project is recorded as a quota warning at the project window; give everyone a project before switching quotas to enforce.

The quota plane runs in **warn mode** (`quota_mode: warn`, since 2026-09-04). A window past its threshold is recorded as a `warn` decision under policy `quota`. `enforce` and `warn` are independent switches for the quota and safety planes. It appears in `systemprompt infra logs governance report` and on `/admin/governance`, and the request is never refused. [Governance Modes](/documentation/governance-modes) explains the measurement window and how a plane goes back to enforce. Under enforce, a spent window answers HTTP 429 with a `Retry-After` equal to the time until the window resets, and an `error.quota` body `{window_seconds, subject, dimension, limit, used, resets_at, retry_after_seconds}` (see [Gateway API](/documentation/gateway-api#status-codes-before-the-upstream-call)). Every breach, in either mode, increments `systemprompt_quota_denials_total{subject_kind, dimension, mode}`.

### Reservation, settlement and release

A request's real cost is only known after its response, so admission works on an **estimate** and trues it up afterwards:

1. **Reserve.** Before the provider is called, the gateway estimates the request's tokens (body length / 4 for input, `max_tokens` clamped to the model ceiling for output) and prices them at the pinned rate card. The estimate is reserved in every window the request falls under. In-flight spend therefore counts against the ceiling, and under enforce a request whose estimate alone breaches a cost ceiling is refused before the provider is called.
2. **Settle.** On completion the bucket is corrected to the audited usage, in the window the request was admitted to.
3. **Release.** A failed or abandoned request releases its reserved tokens and cost. The request count stays.

### Subjects and periods

A window names a **subject** (whose usage it counts) and a **period**.

The subjects this instance resolves are:
- `user` (core's own);
- `group`, `project` and `role`, whose providers are in `extensions/web/admin/identity/src/authz/`;
- `organization` (the whole installation, subject id `default`), whose provider is in `extensions/web/shared/admin-core/src/authz/organization.rs`.

A window can also key on `api_key` or on any scope dimension, in which case it counts the request's attributed value. A person on several groups or projects is otherwise counted against their primary one. `quota_fault_mode: closed` means a window whose subject cannot be resolved refuses the request rather than passing it uncounted.

**Only the three windows above are declared.** A monthly window per project, group or role, or a window keyed on `cost_centre` or `practice`, is supported but not configured.

Periods are an hour, a day, a week, any number of seconds (`window_seconds`), or a UTC calendar period: `calendar: day`, `calendar: week` (from Monday) or `calendar: month`. A calendar window resets on the calendar boundary, so a monthly ceiling covers exactly the current month and the first of the month starts from zero.

### Per-key limits

An API key can carry its own limits, set at issuance:

```bash
systemprompt admin users api-key issue --user <user-id> --name ci-runner \
  --model claude-sonnet-5-5 --model claude-haiku-5-5 \
  --budget-microdollars 50000000 --max-requests 2000 --window-seconds 86400 \
  --scope project=<project-id>
```

- `--model` (repeatable) is an allowlist. A request for another model is refused with `403` before routing.
- `--budget-microdollars` and `--max-requests` are ceilings over `--window-seconds`, which they require. They run as an extra `api_key` quota window, so they reserve, settle and answer `429` like the policy windows, and follow the same `quota_mode` (warn on this instance).
- `--scope <dimension>=<value>` (repeatable) binds the key to one value per dimension. The key's requests are attributed to it (source `api_key`) unless a header names that dimension. The binding is checked against the owner's memberships at issuance and on every request.

With none of these flags, a key behaves as before. `api-key list` prints each key's limits and scopes.

### Two console pages

Both pages read and write the same table, `ai_gateway_policies`:

- **`/admin/gateway/policies`** edits the windows, the safety scanners and block lists, and the warn/enforce switch of each plane.
  - Core re-reads the table on every request (with a sixty-second cache), so a save takes effect without a restart or a file edit.
  - `/admin/sync` then shows the edit as drift on the `gateway_policies` plane, until it is exported to `services/gateway/policies.yaml` and committed, or overwritten from code.
- **`/admin/governance/quotas`** shows usage against each window's threshold, per subject, for the current period, with a meter for the threshold closest to being reached. Under warn mode, a subject past 100% was warned about, never refused.

## Chargeback outside the platform

The console and CLI answer "who spent what" inside the instance. To charge spend back in a finance system, export it:

- **OTLP spans.** With [Observability Export](/documentation/observability-export) turned on, every exported `ai_request` span carries `systemprompt.scope.<dimension>` and `systemprompt.scope.<dimension>.source` for each attributed dimension (so `systemprompt.scope.project`, `systemprompt.scope.cost_centre` and `systemprompt.scope.practice` here), plus `systemprompt.api_key.id`, beside the request's tokens and cost. A collector can forward them to a warehouse; on Google Cloud the path is OTLP → OpenTelemetry Collector → Cloud Logging → a log sink into BigQuery (see [Observability Export: Google Cloud](/documentation/observability-export#google-cloud)).
- **Exports.** The Cost tab and `analytics costs` exports (below) give the same figures as CSV or JSON.

The export is not in the FinOps Open Cost and Usage Specification (FOCUS) format. Mapping the span attributes onto FOCUS columns is the collector's or warehouse's job.

## What is not built

These controls are **not** available. The roadmap records each one:

- **Budgets that stop spend.** The quota windows and per-key budgets can enforce, but quotas are switched to warn by decision, and no per-project or per-group monthly window is declared.
- **Forecasting or burndown.** Nothing projects month-end spend from the run rate.
- **Scheduled digests.** No job sends a periodic cost summary.
- **Alert delivery.** A threshold crossing is a `warn` row, a log line and an increment of `systemprompt_quota_denials_total`; the hourly `usage_anomaly_scan` adds `systemprompt_usage_anomaly_total` and a `usage_anomaly` event for unusual per-user spend. No email, chat or webhook message is sent: delivery is your monitoring stack's job, through the [OTLP export](/documentation/observability-export) or a metrics scrape.
- **FOCUS export.** Cost data is not emitted in the FOCUS format.

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
