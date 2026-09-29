---
title: "Enterprise Roadmap & Known Limitations"
description: "What has shipped since the first enterprise release, what is not built yet, what was removed, and one consolidated table of known limitations."
author: "Astound Digital"
slug: "enterprise-roadmap"
keywords: "roadmap, limitations, semantic caching, ab testing, redaction, phi, data classification, alerts, budgets, scim, removed features"
kind: "guide"
public: true
tags: ["enterprise", "roadmap", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-29"
after_reading_this:
  - "Find the walkthrough for every enterprise capability that has shipped"
  - "Know which capabilities are not built yet, and what each one depends on"
  - "Know which earlier features were removed, and what replaced them"
  - "Check the consolidated known-limitations table before relying on a feature"
related_docs:
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
---

# Enterprise Roadmap & Known Limitations

**TL;DR:** This page is the single place that lists what is *not* available. It has four parts:

1. What has shipped since the first enterprise release, each with a link to its walkthrough.
2. What is not built yet, and what each item depends on.
3. What was deliberately removed.
4. A consolidated table of known limitations.

Roadmap entries describe gaps and dependencies, not release commitments. Before relying on a feature, check its own guide and the deployment's configuration.

One fact affects everything below. Since 2026-09-04 the governance chain, the safety scanners and the quota windows all run in **warn mode**: they detect and record, and they refuse nothing. See [Governance Modes](/documentation/governance-modes).

## Shipped

| Capability | What it does | Walkthrough |
|---|---|---|
| **Provider failover** | `claude-*` requests are served from Claude on Vertex AI. When Vertex is out of capacity or unreachable, the same governed request is re-sent to Anthropic's API. The audit row records which provider served it, and the request is repriced at that provider's rate. | [Model Gateway: Provider failover](/documentation/enterprise-model-routing#provider-failover) |
| **AD FS single sign-on** | SAML 2.0 against Astound's AD FS is the only way to sign in. AD groups map to roles, groups and projects, and being in a mapped group is what admits a person. | [AD FS SSO Setup](/documentation/adfs-sso-setup) |
| **OpenAI-compatible IDE endpoint** | The gateway serves `/v1/chat/completions` (and `/v1/responses`) beside `/v1/messages`. The bridge registers it as an OpenCode provider, so IDE traffic is governed and audited like any other. | [Connect OpenCode](/documentation/connect-opencode) |
| **Observability export** | The audit trail (requests, tool calls, governance decisions, server logs) is pushed to any OTLP/HTTP collector: Datadog, Grafana/LGTM, or Splunk behind a collector. | [Observability Export (OTLP)](/documentation/observability-export) |
| **Retention and archives** | Every high-volume table has a deletion window. Raw requests are archived weekly before they expire, and rollups are archived monthly. | [Data Retention & Archives](/documentation/data-retention) |
| **Time-bound access** | Group and project membership, manual roles, access rules and device certificates can carry an expiry. An hourly sweep revokes what has lapsed. Personal access tokens are capped at 90 days. | [User & Access Management](/documentation/enterprise-user-access) |
| **Conversation history** | Every user can list and search their own conversations. Admins can search everyone's, with export. | [Conversation History & Search](/documentation/enterprise-conversation-history) |
| **Versioning and rollback for prompt content** | Skills are the unit of prompt distribution. Each marketplace version is content-hashed, and versions can be compared. Rollback re-pins a previous digest. Every invocation records which published version served it. | [Versions](/documentation/analysis-versions), [Update a Skill](/documentation/update-a-skill), [Code ↔ Instance Sync](/documentation/services-sync) |
| **Group and project drill-down** | Analytics, cost and quota windows can be scoped to a group, a project or a person. Group and project pages have their own usage tabs. | [Analytics](/documentation/enterprise-analytics#groups-projects-and-people) |
| **Knowledge through connectors** | Google Workspace (Drive, Gmail and Calendar, through Agent Search), Atlassian (Jira and Confluence) and Salesforce are reachable as governed MCP servers, each filtered to what the signed-in person can already open. | [Access Control](/documentation/access-control) |

## Coming

None of these is built. Each lists what it depends on.

### Tenant-isolated semantic caching

This would serve semantically similar requests from a cache instead of paying the provider again, with strict isolation so that one project's cached answers can never reach another.

Today no semantic cache exists, and every request goes to the provider. Provider-side prompt caching still applies where the provider offers it. Building a semantic cache needs embedding infrastructure, similarity thresholds, and a proof of tenant isolation. It is a project in its own right, and only worthwhile if real traffic repeats itself enough.

### A/B model testing

This would split live traffic on a route between models by percentage, with stable assignment and the variant stamped on the audit row.

Today route matching is strictly first-match. Comparisons are done by switching a route for one cohort, or offline: [Evaluate Plugins](/documentation/analysis-evaluate-plugins) runs a fixed prompt suite against two versions of a skill set.

### In-flight redaction

This would rewrite a detected value (a credit-card number, a national id) out of the request before it is forwarded, instead of blocking or only recording it.

Today scanners detect and record, and transcripts are masked at the display layer. The provider still receives the original text. Redaction is a design item, not a toggle.

### PHI categories

Health-identifier detection would join the PII set. Today the scanners cover email, card number, US SSN and phone. PHI is only worth adding if regulated health data actually passes through the gateway.

### Data classification beyond two flags

Routes can require a provider to be `european` or `no_retain`, and dispatch refuses a provider that lacks the flag. A richer classification scheme (labels on requests, routed by class) needs Astound's data-classification scheme first. Today every provider is declared `european: false`, and no route requires either flag. See [Data classification & residency](/documentation/enterprise-model-routing#data-classification-residency).

### Latency objectives per use case

The analytics Overview tab splits requests either side of a latency threshold you choose on the page. Per-workload objectives, error budgets and alerts on them need target latencies per workload class. The raw figures (total and upstream latency per request) are recorded and exported.

### Quality-normalised model comparison

"Is the cheaper model good enough?" needs an agreed evaluation set of representative tasks, with expected outcomes. Cost and latency per provider are already measured. Quality has no source until that set exists.

### Tool schema meta-validation

This would validate every MCP tool's JSON Schema against the JSON-Schema meta-schema at load time and in CI, and fail fast on a malformed one. Today a tool whose schema a provider rejects surfaces as that provider's error, and the request page shows the client and provider schemas side by side.

### Alert delivery

This would send budget-threshold warnings, usage anomalies and failed jobs to a named channel (email, chat or webhook). Today they are rows and log lines only. The [OTLP export](/documentation/observability-export) can carry the log lines into your own alerting stack once a collector is named.

### Manager view of history

This would let a line or project manager see their team's conversations. The history scope resolver already accepts a managed-members list, but nothing supplies one yet. See [Conversation History](/documentation/enterprise-conversation-history#who-sees-whose-history).

### Outcome metrics

This would join AI usage to results in a system of record: Jira transitions, merged pull requests, Salesforce records. It needs a decision on which outcome counts, a sync of that system's records, and an identity mapping. See [From usage to outcomes](/documentation/enterprise-analytics#from-usage-to-outcomes).

### Enterprise knowledge platform

Governed retrieval over a corpus the platform owns would need its own ingestion, index and access-filtered retrieval, together with an accuracy benchmark, a knowledge graph and a curated use-case library. None of this is built. Today knowledge is reached live through the connectors listed under [Shipped](#shipped), under each person's own permissions.

## Removed

These were built and then taken out on purpose. They are listed so that an older document or screenshot does not mislead.

| Removed | When | What replaced it |
|---|---|---|
| Per-organization budgets, run-rate early warning, burndown forecasts, scheduled cost digests | 2026-09-01, with organizations and departments | Quota windows as warning thresholds ([Cost Management](/documentation/enterprise-cost-management)). Groups and projects replace organizations and departments. |
| Slack alerting | 2026-09-10 | Nothing pushes an alert; see [Alert delivery](#alert-delivery). |
| Code-impact metrics (AI-authored lines, applied edits, commit lines) | 2026-09-16 | Nothing. They were proxies too weak to report ([Analytics](/documentation/enterprise-analytics#what-is-not-measured)). |
| The in-house retrieval server | September 2026 | The Google Workspace, Atlassian and Salesforce connectors. |
| Passkey and invitation sign-in, Salesforce sign-in | 2026-08-29 to 08-31 | AD FS single sign-on. |

## Known limitations

| Limitation | Current state | Where it is discussed |
|---|---|---|
| **Nothing is enforced** | Governance, safety and quota planes run in warn mode since 2026-09-04. Only authentication refuses. | [Governance Modes](/documentation/governance-modes) |
| **No spend ceiling** | Quota windows warn. No per-project or per-group monthly window is declared, although the subjects and calendar-month periods are supported. | [Cost Management](/documentation/enterprise-cost-management) |
| **SCIM provisioning** | Not offered. AD FS pushes no SCIM, so an endpoint would have no caller. The directory group decides access at each sign-in, bounded by the session lifetime. Revisit if Okta or Entra fronts the instance. | [User & Access Management](/documentation/enterprise-user-access) |
| **Audit immutability** | Rows are append-only by convention, not by mechanism: no WORM storage or hash-chaining. Archives sit on the instance's own disk. Hardening options include revoking UPDATE/DELETE grants, hash-chaining, and exporting to write-once storage. | [Data Retention](/documentation/data-retention) |
| **Finite history** | Raw requests are kept for 90 days, prompt text for 30 and raw bodies for 7. Older history lives in the weekly archives. | [Data Retention](/documentation/data-retention) |
| **Per-key budgets** | An API key or personal access token inherits its owner's scope. It is not its own quota subject, so there is no per-key budget, rate or model policy. One account per application gives per-application limits today. | [User & Access Management](/documentation/enterprise-user-access) |
| **Model access is open** | The access rule for gateway routes is open to every role. Models are shaped by governance and quotas, not restricted per role. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **No EU route** | Every provider is declared `european: false`, and Vertex endpoints use the global location. | [Model Gateway & Routing](/documentation/enterprise-model-routing#data-classification-residency) |
| **Shadow AI outside the gateway** | Unlisted models are refused at the gateway. Traffic that never reaches the gateway is invisible to it, and blocking it is a network egress control for Astound IT. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **Tab-acceptance and commit metrics** | Not measurable. Claude Code emits no accept/reject signal, and commits outside Claude Code are not visible. | [Analytics](/documentation/enterprise-analytics#what-is-not-measured) |
| **Metrics endpoint** | The Prometheus `/metrics` listener only starts when the profile sets `server.metrics_port`, and neither shipped profile does. | [Observability Export](/documentation/observability-export) |
| **VS Code Copilot** | No configuration is published. OpenCode is the supported IDE client of the OpenAI-compatible endpoint. | [Connect OpenCode](/documentation/connect-opencode) |

## Check it yourself

- `/admin/gateway/policies`: the warn/enforce state of each plane.
- `/admin/gateway`: the routes, including the Vertex route's fallback provider.
- `/admin/system/observability`: whether a collector is configured.
- `/admin/configuration` and `/admin/lifecycle`: the retention windows and archives.
- `systemprompt infra logs governance report --since 7d`: what warn mode recorded in the last week.
