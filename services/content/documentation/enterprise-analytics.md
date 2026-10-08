---
title: "Usage, Adoption & Productivity Analytics"
description: "Track AI usage and adoption in /admin/analytics — requests, spend, weekly active users, models, skills, tools and sessions — scoped to the whole instance, one group, one project or one person."
author: "Astound Digital"
slug: "enterprise-analytics"
keywords: "analytics, usage, adoption, wau, groups, projects, inactive users, skills, tools, models, dashboards"
kind: "guide"
public: true
tags: ["enterprise", "analytics", "admin"]
published_at: "2026-08-25"
updated_at: "2026-10-08"
after_reading_this:
  - "Navigate the Overview, Models, Skills, Tools, Sessions and Cost tabs"
  - "Scope any view to a group, a project or one person, and pick a window from 15 minutes to 90 days"
  - "Drill from a group or project page into its usage"
  - "Find people who were provisioned but have not used the platform in 30 days"
  - "Know what is not measured: code output, commits, and business outcomes"
related_docs:
  - title: "Cost Management & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Conversation History & Search"
    url: "/documentation/enterprise-conversation-history"
  - title: "Data Retention & Archives"
    url: "/documentation/data-retention"
  - title: "Dashboard"
    url: "/documentation/dashboard"
---

# Usage, Adoption & Productivity Analytics

**TL;DR:** `/admin/analytics` answers "what did the AI do?" across six tabs: **Overview, Models, Skills, Tools, Sessions** and **Cost**.

- **Window:** presets from 15 minutes to 90 days, or a custom range of up to 30 days.
- **Scope:** everyone, one group, one project or one person.
- **Drill-down:** group and project pages carry their own usage tabs, and `/admin/users` has a filter for people who have not used the platform in 30 days.
- **Not measured:** code output and commits (those metrics were withdrawn), and business outcomes (never measured).

The analytics pages are for console roles. A signed-in user without console access sees only their own conversations, on [Conversation History](/documentation/enterprise-conversation-history).

## Window, scope and attribution

Every tab shares the same controls, carried in the query string, so a view can be bookmarked:

- **Window:** presets of 15 minutes, 1 hour, 24 hours, 7 days, 30 days and 90 days, plus a custom range capped at 30 days for query cost. Raw requests are kept for 90 days ([Data Retention](/documentation/data-retention)), so the 90-day preset is also the longest reach into raw history.
- **Scope:** everyone, one **group** (`?group=`), one **project** (`?project=`), or one **person** (`?user_id=`). Groups and projects come from Active Directory at sign-in or from console membership. See [User & Access Management](/documentation/enterprise-user-access).
- **Attribution:** **Exclusive** by default. Each person's usage counts once, against their primary group or project, so container totals add up to the instance total. **Member** (`?attr=member`) counts a person in every group they belong to, which answers "who uses what". The page says on screen that those totals overlap.

## The tabs

| Tab | What it answers |
|---|---|
| **Overview** | KPIs: requests, total spend, weekly active users, tokens. Request and cost trends, the model mix, a **top-users** leaderboard (requests, share, requests per day, tokens, last active, groups), and the split of requests either side of a latency SLO threshold you set on the page (`?slo_ms=`, default 5 s). |
| **Models** | Per model: requests, share, prompt and completion tokens, median and p95 latency, errors, and **redirects** (requests where the model asked for differs from the model that served). Unrouted requests have their own bucket. |
| **Skills** | Skill adoption. Per skill: invocations, split into typed `/plugin:skill` commands and ones the model dispatched itself, plus users, conversations reached, requests and conversation spend, and how many invocations resolved to a published version. |
| **Tools** | MCP health. Per server and per tool: executions, share, success, failed, pending, and users. |
| **Sessions** | Conversation cost: per conversation, the model, requests, tokens and last activity. |
| **Cost** | The supplier bill and consumption by group or project. See [Cost Management](/documentation/enterprise-cost-management). |

Every row drills into `/admin/requests` with the dimension it names, plus the scope and window already on screen. Every table exports as CSV, JSON, JSON Lines or Markdown.

For the analysis of individual conversations and skill runs (outcomes, judge scores, versions), see [Analysis](/documentation/analysis).

```bash
systemprompt analytics overview
systemprompt analytics requests stats
systemprompt analytics costs breakdown --by user --since 7d
systemprompt analytics conversations list --since 7d --source gateway
systemprompt analytics tools list
```

## Groups, projects and people

The org drill-down uses two flat containers, not a hierarchy:

- A **group** is people and entitlement.
- A **project** is work attribution and an access subject.

There is no layer above them (no hub or division).

- **`/admin/groups`** lists every group. **`/admin/groups/{id}`** has the tabs **Usage**, **Members**, **Access**, **Projects** and **Mappings** (the directory groups that feed it). The **Unassigned** bucket collects everyone with no group, so gaps in directory mapping are visible.
- **`/admin/projects`** lists projects with their spend and tooling health. **`/admin/projects/{id}`** has **Members**, **Usage** and **Settings** tabs, and **`/admin/projects/{id}/report`** lays all of it out on one printable page.
- **`/admin/users/{id}?tab=usage`** is one person's usage. The older `/admin/analytics/users/{id}` link redirects there.

## Adoption

- **Weekly active users** on the Overview tab, with the change against the previous period.
- **Requests per user per day** on the top-users leaderboard: is usage broad and shallow, or concentrated in a few people?
- **Provisioned but not using it:** `/admin/users?filter=inactive-30d` lists everyone with no gateway request, no browser session activity and no recorded activity in the last 30 days. A person who has never been active has no last-active date at all, so "never used it" is distinguishable from "used it on day one".

## What is not measured

- **Code output.** AI-authored lines, applied edits, permission-grant rate and commit lines were withdrawn in September 2026, and their tables were dropped. Those numbers were proxies too weak to report.
- **Commits made outside Claude Code** have never been visible. Measuring them needs an authoritative source-control host and an identity mapping into it, which is still awaiting a decision.
- **Tab-acceptance rate.** Claude Code emits no accept/reject signal for completions, so an acceptance metric would need an IDE-level integration that does not exist.

These gaps are also listed on the [Enterprise Roadmap](/documentation/enterprise-roadmap#known-limitations).

## Gateway overhead

Gateway overhead is the time the gateway adds around the provider call: authentication, the governance chain, the safety scanners, quota checks, routing and the audit write. It is measured on real traffic, never on a synthetic benchmark.

- **Formula.** For every completed, non-synthetic gateway request that recorded both timings, overhead = `latency_ms - upstream_latency_ms` from its `ai_requests` row, floored at zero. `latency_ms` is the total the gateway recorded for the request; `upstream_latency_ms` is the provider call alone.
- **Nightly rollup.** The `gateway_overhead_rollup` job runs at 00:10 UTC (`services/scheduler/config.yaml`) and writes the previous UTC day into `gateway_overhead_daily`: request count, p50/p95/p99 of overhead and p50/p95/p99 of upstream latency, in milliseconds, computed with Postgres `percentile_cont`. There is one row per inbound wire dialect (`wire_protocol`) and one `all` row. The table keeps 400 days. Re-running the job overwrites the day's rows, so it is safe to run again by hand: `systemprompt infra jobs run gateway_overhead_rollup`.
- **Where to read it.** The **Gateway overhead** block on the console landing page (`/admin`) shows last night's p50, p95 and p99, each with a 30-night sparkline, and a table of the last 30 nights with upstream p95 beside them. The block reads the nightly rows, so it does not move with the 24h/7d/30d control. A tile turns amber at a p95 of 250 ms and red at 1 s.
- **Live source.** When `server.metrics_port` is set in the profile, the server also exports Prometheus histograms on that port: `gateway_overhead_seconds` and `gateway_upstream_duration_seconds`. They are the live view for alerting and short windows; the nightly table is the published record. The self-host profile template sets `metrics_port: 9090` and the compose files expose that port on the compose network only. Production does not set it today.

## Check it yourself

- `/admin/analytics?preset=7d`: the Overview for the last week.
- `/admin/analytics?tab=skills&preset=30d`: skill adoption for the month.
- `/admin/groups` and `/admin/projects`: open one and read its **Usage** tab.
- `/admin/users?filter=inactive-30d`: who has not used the platform in 30 days.
- `/admin`: the **Gateway overhead** block, last night's p50/p95/p99.
- From the CLI:

  ```bash
  systemprompt analytics overview
  systemprompt analytics requests stats
  ```

## From usage to outcomes

Everything above measures **consumption**. Mapping consumption to **outcomes** is a separate question, and it needs a decision before it needs engineering.

### What is measured today

Every request through the gateway records:
- input, output, cache-read and cache-creation tokens;
- cost and latency;
- model and provider;
- the user, session and trace it belongs to.

Those rows roll up per user per day. The dashboards and each person's Usage tab read those rollups.

**Cost is computed, not billed.** It comes from a model price catalogue applied to observed token counts, so it is an accurate estimate rather than an invoice. The customer view of the Cost tab (`?audience=customer`) deliberately carries no cost at all, so a report shared with a team shows usage without exposing spend.

### The limit worth stating

The only outcome-shaped data captured today is **a model's judgement**. The `conversation_judge` job reads each finished conversation and records:
- a title, a summary and an intent;
- an outcome;
- a 0–100 completion score for whether the assistant delivered what was originally asked.

This is a useful coaching signal and a reasonable way to spot conversations that went badly; see [Analysis](/documentation/analysis). It is not a business metric and should not be presented as one: it is a model's opinion of the work, not an observed result in a system of record.

Nothing today joins AI usage to a ticket closing, a deal advancing, an incident being resolved, or a release shipping.

### The decision required

To map usage to outcomes, we first have to agree what an outcome *is*. The candidates, roughly in order of how directly they can be observed:

- **Jira issue transitions:** cycle time from in-progress to done, reopen rate, throughput per person per sprint. This is the most concrete option for engineering work, and the easiest to game if it becomes a target.
- **Source control:** pull requests merged, review turnaround, change failure rate. This is closest to delivery, but only meaningful once commits made outside Claude Code are visible.
- **Salesforce records:** opportunities advanced, cases resolved. This is the right measure for commercial rather than engineering usage.
- **Ratified judgement:** keep the judge's outcome, but have a human confirm or reject it, which turns an opinion into a label.

Whichever is chosen, two dependencies follow, and neither is optional:

1. **A sync of that system's records into the platform.** The Atlassian and Salesforce connectors let each person *query* those systems under their own permissions. Nothing yet copies their records into analytics, where they could be joined with usage.
2. **An identity-mapping rule** from the external account to a platform user: a Jira account id, a Salesforce username, a commit-author email. This is the harder half, and the equivalent source-control question is still open.

Until an outcome is defined and its source system connected, the platform can report what AI cost and what it was used for. It should not claim more than that.
