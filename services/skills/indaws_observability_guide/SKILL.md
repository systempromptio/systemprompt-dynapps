---
name: indaws-observability-guide
description: >-
  Guide to Dyncloud observability — the in-cluster white-box stack (metrics, logs, dashboards) and the
  external black-box status page. Use when the user asks about metrics, logs, dashboards, alerts,
  uptime, "is X up", where a Grafana dashboard lives, or how alerting reaches Teams/email. Read-only;
  every product, dashboard, exporter, metric name and host is read live from the documentation MCP,
  never from memory. Triggers: "grafana", "prometheus", "loki", "dashboard", "metrics", "logs",
  "alerta", "status page", "uptime", "observabilidad".
version: 0.2.2
effort: low
model: sonnet
allowed-tools: [Read, Bash, Grep, Glob]
---

# Observability guide

Observability has **two planes**: a **white-box** one inside the clusters (metrics, logs,
dashboards) and a **black-box** one outside them (uptime, seen from the internet). Everything
is **read-only** from here; changes — dashboards, alert rules, exporters — go via **GitOps**
(use `gitops-change`). Which products, dashboards and exporters make up each plane is read
**live from the documentation MCP**, never from memory, and never invented.

## ⚠️ Safety rules

Detail in `references/shared/danger-gates.md`. Here: **kubectl READ-ONLY**; human access to
Grafana depends on the **Azure AD group** (see `references/shared/access-model.md`); **never**
dump secret values (datasource tokens, apiKeys); **never invent** a metric, a dashboard or a
service name — look it up, or say you cannot confirm it.

## What it is / when

Use me to understand **what is observed and where to look**: metrics, logs, dashboards and
uptime. To chase a specific incident end-to-end, combine me with `incident-runbooks`.

## Platform detail: ask the documentation MCP

The observability inventory — which products make up each plane, which dashboards exist and in
which folder, the exporters and the metric names they publish, the datasources and the cluster
selector, how alerts are routed, the hosts and the repos behind them — is **documentation, read
at the moment it is needed**. A copy in this file is a copy that goes stale.

- **Identify the server by its tool family, never by its name**: it is the one exposing
  `mcp__*__docs_*`. The middle segment is whatever the user called their server — something
  you read at runtime, never something you write down. Several matches or none → **ask which
  one before calling any tool**.
- **Preflight before the first lookup**, not after it fails. Then **scope by name**: discover
  the platform documentation with `docs_list_spaces` and match it **by its name**; never by a
  numeric id.
- **Ask in natural language and cite the source** document for every fact you state.
- The contract (identification, preflight, degradation) is
  **`references/shared/docs-access.md`**. **How to query well is `common-skills:dynbrain`** —
  read that skill and use it; don't reimplement it here.
- **MCP unavailable** → say it plainly, ask the user to connect it (the OAuth handshake is
  theirs to complete, you cannot do it for them), still answer with this skill's core (safety
  rules, the flow below, gotchas), and **mark explicitly what stays unconfirmed**. Never invent a
  version, a host or a path.

## Flow

1. **Step 1** — metrics/logs (in-cluster stack) or uptime (status page)? Choose the plane; the
   two have **separate failure domains**, so "green" in one says nothing about the other.
2. **Step 2** — resolve the concrete names you need (dashboard, datasource, exporter, metric,
   service) by asking the documentation MCP. Never type one from memory.
3. **Step 3** — confirm what the user is allowed to reach: access to Grafana/Prometheus depends
   on the **Azure AD group** (`references/shared/access-model.md`).
4. **Step 4** — a change (dashboard, rule, exporter) is never applied by hand → `gitops-change`.

## GOTCHAS

1. **The real service names of the metrics and log backends are not the ones you would guess** from the product names, and they are not identical in every cluster (one carries a per-cluster prefix). A query against a made-up service name fails silently or returns nothing. **Confirm the actual names before querying.**
2. **The metric family that reports DB backup status is not the one you would expect** from the database operator's own collector — that one reads 0 with the backup plugin in place and produces **false alarms**. Confirm the real metric name before writing a query, a panel or an alert rule.
3. KSM does not emit the `dyncloud.io/*` labels without `metricLabelsAllowlist` → dashboard filters don't bind.
4. Grafana **without render plugin**: panels are not validated by automatic capture (query API or human).
5. The **status page runs outside the cluster**; uptime alerting is delivered from there (Teams/email), not by Grafana. Looking for an uptime alert in the metrics stack is looking in the wrong plane.
6. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `gitops-change` | any change to dashboards, rules or exporters |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `cluster-inspector` | metrics/logs when inspecting |
| `incident-runbooks` | signals for diagnosis |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
