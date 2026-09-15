---
title: "Cost Management, Budgets & FinOps"
description: "Attribute gateway usage to users, projects, and models; inspect spend warnings and export cost reports."
author: "Astound Digital"
slug: "enterprise-cost-management"
keywords: "cost, budget, finops, spend, caps, alerts, forecasting, csv, digests, chargeback"
kind: "guide"
public: true
tags: ["enterprise", "finops", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-11"
after_reading_this:
  - "Read the Spend tab: totals, trends, and model distribution"
  - "Read the spend warning thresholds and why nothing is refused on cost"
  - "Attribute any request's cost to its user, project, model, and provider"
  - "Export cost reports as CSV from the web UI or CLI"
related_docs:
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Cost Management, Budgets & FinOps

**TL;DR:** Every AI request lands with its cost attributed to a user, project, model, and provider. The gateway's quota windows watch spend per user and per day for the whole instance and, since 2026-09-04, only warn: a window past its threshold lands in the governance warnings report and the request still runs. The Spend tab shows totals, trends and the cost-by-model split, filterable by project; reports export as CSV from the web and CLI.

## Per-request attribution

Cost tracking is not a rollup bolted on afterwards: **each request row carries user, model, provider, tokens, and cost** (stored as microdollar integers end to end), and users can belong to multiple projects through directory or dashboard membership. Dashboards and reports slice on all of these dimensions in near real time, so chargeback to a project is a filter, not a reconciliation project.

```bash
systemprompt analytics costs summary
systemprompt analytics costs breakdown --by user --since 30d      # spend, requests, tokens, conversations per user
systemprompt infra logs request list --limit 20 --user <user-id>
```

## The Spend tab

`/admin/analytics` includes a Spend tab showing:

- **Total spend and daily cost trend** for the selected period, with cost per request and token counters.
- **Model distribution and cost-by-model series** — where the money actually goes.
- **Project selector** — every figure on the tab narrows to Commerce or Core.

## Spend warning thresholds

Spend thresholds are the gateway's **quota windows** (`services/gateway/policies.yaml`): a per-user hourly window sized to flag a runaway agent, and an instance-wide daily window sized to flag an unusually expensive day. The quota plane runs in **warn mode** (`quota_mode: warn`, set on 2026-09-04): a window past its threshold is recorded as a `warn` decision under policy `quota`, shows up on the governance warnings report at `/admin/governance/warnings` and in `systemprompt infra logs governance report`, and the request is never refused. There is no spend clamp on this instance. Costs are attributed one request late by design (a request's cost is known after its response), so the warning lands on the request after the crossing.

Switching a window back to enforcement (`quota_mode: enforce`, HTTP 429 with `retry-after` once a window is spent) is a review-visible change: a unit test pins the mode and the thresholds.

Per-project budgets are not yet available — see the [roadmap](/documentation/enterprise-roadmap).

## Self-service reporting

- **CLI** — cost reports accept `--since` / `--until` and export CSV. Start from `systemprompt analytics costs summary`; `analytics costs breakdown --by user|model|provider|agent` slices the same window, and `systemprompt analytics --help` lists the rest.
- **Web** — the admin UI exposes CSV export endpoints for usage and request reports, narrowed to the project you select.

## Provider and model cost comparison

An internal report (also reachable from the CLI) breaks cost and margin down **by provider and by model**, so you can see what a switch of route would save. Quality-normalized comparison ("is the cheaper model good enough?") requires a quality baseline for your workloads — see the [roadmap](/documentation/enterprise-roadmap).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured. Screenshots below are illustrative and may show an earlier configuration.

![The Spend tab with total spend, trend, and cost per request](/files/images/evidence/req-004-spend.png)
*The Spend tab: total spend, daily cost trend, and cost per request.*

![Cost distribution across models](/files/images/evidence/req-004-model-mix.png)
*The model mix: where spend actually lands, by model.*
