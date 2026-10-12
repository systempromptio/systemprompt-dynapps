---
title: "Dashboard"
description: "The admin overview: ten live headline figures against the previous window, five charts over time, spend by project and group, the most used models, and what is waiting on a person."
author: "Dynapps"
slug: "dashboard"
keywords: "dashboard, overview, admin home, kpi, spend, requests, latency, error rate, models, projects, groups, approvals, anomalies, live"
kind: "guide"
public: true
tags: ["dashboard", "analytics", "admin"]
published_at: "2026-02-18"
updated_at: "2026-09-29"
after_reading_this:
  - "Read every headline figure on the overview and know where each one drills down to"
  - "Switch the window between 24 hours, 7 days and 30 days and read the comparison"
  - "Find which projects, groups and models carry the spend"
  - "See the approvals and usage anomalies waiting on a person"
related_docs:
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Cost Management"
    url: "/documentation/enterprise-cost-management"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
---

# Dashboard

**TL;DR:** `/admin` is the overview — one screen for the whole instance over
the last 24 hours, 7 days or 30 days. Ten headline tiles compare the window
with the one before it, five charts show it over time, two leaderboards rank
projects and groups by spend, one table ranks models, and a queue shows the
approvals and anomalies waiting on a person. The page refreshes itself every
half minute while it is open. Every figure links one click deeper.

## Who sees it

The overview is for the console roles, `admin` and `platform_admin`. Anyone
else who opens `/admin` is sent to their profile, or — if they participate in
a marketplace — to the scoped pages that participation opens. See
[Create & Manage Users → Roles](/documentation/admin-user-management#roles).

## The window

The **24h / 7d / 30d** buttons in the page header choose the window (24 hours
is the default; the choice is in the URL as `?preset=`, so it can be
bookmarked). Changing it redraws the page without a reload.

Each tile's note compares the window with the one immediately before it
("vs previous 7d"). When the previous window carried too few requests to
compare against, a note says so and no tile shows a percentage.

## Live refresh

While the tab is visible the page fetches its own figures every 30 seconds
and updates them in place — tiles count to their new value, charts keep your
hidden series and focus, changed table cells flash. The **Live** indicator in
the header shows how old the figures are; **Pause** stops the refresh, and it
stops by itself when your session ends or the fetch keeps failing. Motion is
switched off under your system's reduced-motion setting, and the page works
without JavaScript — it just does not refresh.

The live figures come from `GET /admin/api/overview`, which renders the same
markup as the page from the same queries, so a refreshed number can never
disagree with the one on first load.

## Headline figures

| Tile | What it counts | Opens |
|---|---|---|
| **Requests** | Gateway requests in the window | Request log |
| **Spend** | Gateway cost in the window | Analytics → Cost |
| **Active people** | Distinct accounts that made a request | Users |
| **Conversations** | Distinct gateway conversations | Analysis → Conversations |
| **Tokens** | Input and output tokens; the note gives the share read from cache | Analytics → Models |
| **Tool calls** | Tool calls across every client | Analytics → Tools |
| **p50 latency** | Half of routed requests finished faster than this | Traces |
| **p95 latency** | 95% of routed requests finished faster than this | Traces |
| **Error rate** | Failed requests over all requests | Request log, failed only |
| **Denied** | Requests recorded as `rejected` — refused before routing | Request log, rejected only |

While governance, safety and quotas run in warn mode, **Denied** counts only
what is still refused — for example a model the gateway does not route — not
what warn mode would have blocked. Those are recorded as warnings; see
[Governance Modes](/documentation/governance-modes).

## Over time

Five charts, bucketed to the window:

- **Requests, conversations and people**
- **Tokens and cache reads**
- **Tool calls, errors and denials** — with rejected requests as their own series
- **Latency per bucket** — p50 and p95, captioned with the worst bucket of each
- **Spend by model** and **Requests by client** — the top series, the rest as *Other*; requests for a model the gateway does not route appear as *Unrouted*

## Where the spend goes

**Top projects by spend** and **Top groups by spend** sit side by side and
share one layout: requests, spend and share of the window's spend. Attribution
is exclusive — each person counts once, under their primary project or group
— so each board adds up to the whole. People with no primary project or group
appear as *Unattributed*. **All** opens the full list; a row opens that
project's or group's page with its own usage tab.

## Most used models

One row per model the gateway served in the window, ordered by requests, with
share of traffic, spend and median latency. A model the gateway refused is
marked **REJECTED**. **All models** opens the Analytics Models tab, which
reads the same rows.

## Waiting on a person

- **Pending approvals** — tool calls held by a `require_approval` policy.
  Opens the approvals queue at `/admin/governance/approvals`.
- **Open anomalies** — usage the anomaly detector flagged in the last 24
  hours, listed below the tile with metric, window, observed value, baseline
  and severity. Anomalies are shown here and written to the server log; no
  notification is sent anywhere else. See
  [Audit & Observability](/documentation/enterprise-audit-observability).

## Getting around

The header buttons open the **Request log** and **Analytics**. The sidebar
groups the console:

| Group | Pages |
|---|---|
| Activity | Overview, Analytics, By person |
| People | Users, Groups, Projects, Roles & permissions, Access control, Devices & enrolments |
| Analysis | Conversations, Skills, Reports, Versions |
| Governance | Governance, Approvals, Secrets, Quotas |
| Platform | Configuration, Lifecycle, Code sync, Marketplaces, Plugins, Skills, MCP servers, Gateway |
| Account | Profile, Connect, Connectors, My conversations |
| Developer | Requests, Sessions, Conversations, Traces, Tools & artifacts |

The same figures are available from the CLI for scripting and export:

```bash
systemprompt analytics overview --since 7d
systemprompt analytics overview --export overview.csv
systemprompt analytics costs breakdown --by model --since 30d
```

## Check it yourself

1. Open `/admin`, choose **7d**, and note **Requests**.
2. Click the tile. The request log opens on the same window; its count matches.
3. Run `systemprompt analytics overview --since 7d` and compare the request
   total. A small difference is requests that landed between the two reads.
