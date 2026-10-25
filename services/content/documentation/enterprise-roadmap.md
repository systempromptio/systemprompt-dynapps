---
title: "Enterprise Roadmap & Known Limitations"
description: "What has shipped since the first enterprise release, what is not built yet, what was removed, and one consolidated table of known limitations."
author: "Dynapps"
slug: "enterprise-roadmap"
keywords: "roadmap, limitations, semantic caching, ab testing, redaction, phi, data classification, alerts, budgets, api keys, failover, load shedding, scim, removed features"
kind: "guide"
public: true
tags: ["enterprise", "roadmap", "admin"]
published_at: "2026-08-25"
updated_at: "2026-10-08"
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

Each entry distinguishes what is **built** (in core or this repository) from what is **enabled on this instance**. Several 0.63 capabilities are built but deliberately left off here.

One fact affects everything below. Since 2026-09-04 the governance chain, the safety scanners and the quota windows all run in **warn mode**: they detect and record, and they refuse nothing. See [Governance Modes](/documentation/governance-modes).

## Shipped

| Capability | What it does | Walkthrough |
|---|---|---|
| **Failover chains and per-project routing** (0.63) | A model lists its deployments in failover order, each behind its own circuit breaker, and a route can pin or narrow that chain. **Available, not used here:** every `claude-*` id is served from Anthropic's own API with no failover, and DeepSeek is reachable only by an explicit `deepseek-*` id; the Claude models each have a single deployment. The audit row records `served_provider` and a `route_match` of `failover:a->b`; `gateway_upstream_failovers_total` counts hops. | [Model Gateway: Provider failover](/documentation/enterprise-model-routing#provider-failover) |
| **Per-key budgets and limits** (0.63) | An API key can carry a model allowlist, a spend budget, a request ceiling and scope bindings, and is its own quota subject. **Built, available on issuance:** set with `admin users api-key issue` or `POST /api/v1/admin/api-keys`; budgets follow `quota_mode`, so they warn here. | [User & Access Management](/documentation/enterprise-user-access#per-key-limits-and-scope-bindings) |
| **Scope attribution and chargeback keys** (0.63) | Every gateway request is attributed to a project, cost centre and practice (header, key binding or default), stored in `ai_request_attributions` and exported as `systemprompt.scope.*` span attributes. Quotas reserve at admission and settle at completion; a quota `429` is machine-readable. **Enabled.** | [Cost Management](/documentation/enterprise-cost-management#scope-attribution-project-cost-centre-practice) |
| **In-flight redaction** (0.63) | Request findings in `safety.redact_categories` are rewritten as `[REDACTED:<category>]` before forwarding; content that cannot be redacted is refused, in either mode. **Built, not enabled:** no category is listed, because the fail-closed refusal would apply under warn mode. Switched on with the enforce flip. | [Content Safety: In-flight request redaction](/documentation/enterprise-safety-guardrails#in-flight-request-redaction) |
| **Alert signal** (0.63) | The hourly `usage_anomaly_scan` flags per-user spend and request-rate spikes as `systemprompt_usage_anomaly_total` and a `usage_anomaly` event; quota breaches count in `systemprompt_quota_denials_total`. **Enabled (scheduled).** Delivery to a person is the operator's monitoring stack, through OTLP or a metrics scrape. | [Audit Trail: Anomaly detection](/documentation/enterprise-audit-observability#anomaly-detection) |
| **Multi-node operation** (0.63) | Replicas boot concurrently (each claims its instance id), migrations run once (`database.migrate_on_boot: false` / `--skip-migrate`), nodes split by `server.role: all \| gateway \| admin`, and `server.max_in_flight` sheds load with a `503` and a saturated `/readyz`. **Built; the self-host template sets them, the production profile sets none.** | [Deploy: Growing to more nodes](/documentation/deploy#growing-to-more-nodes) |
| **Tool schema meta-validation** | Every MCP tool's input and output schema is checked against the JSON Schema meta-schema (Draft 2020-12, or the draft its `$schema` names) plus the shapes providers refuse: top-level `type: object`, `required` keys `properties` declares, no external `$ref`. **Enabled:** the bundled `systemprompt` server refuses to start on a violation, CI runs the same checks, and the request detail page badges a client schema that fails. | [MCP, Tool Governance & Distribution: Tool schema validation](/documentation/enterprise-tool-governance#tool-schema-validation) |
| **Microsoft Entra ID single sign-on** | OpenID Connect against Dynapps' Entra ID tenant is the only way to sign in. Entra groups map to roles, groups and projects, and being in a mapped group is what admits a person. | [Entra ID SSO Setup](/documentation/entra-id-sso-setup) |
| **OpenAI-compatible IDE endpoint** | The gateway serves `/v1/chat/completions` (and `/v1/responses`) beside `/v1/messages`. The bridge registers it as an OpenCode provider, so IDE traffic is governed and audited like any other. | [Connect OpenCode](/documentation/connect-opencode) |
| **Observability export** | The audit trail (requests, tool calls, governance decisions, server logs) is pushed to any OTLP/HTTP collector: Datadog, Grafana/LGTM, or Splunk behind a collector. | [Observability Export (OTLP)](/documentation/observability-export) |
| **Retention and archives** | Every high-volume table has a deletion window. Raw requests are archived weekly before they expire, and rollups are archived monthly. | [Data Retention & Archives](/documentation/data-retention) |
| **Time-bound access** | Group and project membership, manual roles, access rules and device certificates can carry an expiry. An hourly sweep revokes what has lapsed. Personal access tokens are capped at 90 days. | [User & Access Management](/documentation/enterprise-user-access) |
| **Conversation history** | Every user can list and search their own conversations. Admins can search everyone's, with export. | [Conversation History & Search](/documentation/enterprise-conversation-history) |
| **Versioning and rollback for prompt content** | Skills are the unit of prompt distribution. Each marketplace version is content-hashed, and versions can be compared. Rollback re-pins a previous digest. Every invocation records which published version served it. | [Versions](/documentation/analysis-versions), [Update a Skill](/documentation/update-a-skill), [Code ↔ Instance Sync](/documentation/services-sync) |
| **Group and project drill-down** | Analytics, cost and quota windows can be scoped to a group, a project or a person. Group and project pages have their own usage tabs. | [Analytics](/documentation/enterprise-analytics#groups-projects-and-people) |
| **Knowledge through connectors** | The Dynapps portal's Odoo (`dynapps-odoo`) and DynBrain (`dynbrain`) servers are reachable as governed MCP servers, each filtered to what the signed-in person can already open. | [Access Control](/documentation/access-control) |

## Coming

None of these is built. Each lists what it depends on.

### Tenant-isolated semantic caching

This would serve semantically similar requests from a cache instead of paying the provider again, with strict isolation so that one project's cached answers can never reach another.

Today no semantic cache exists, and every request goes to the provider. Provider-side prompt caching still applies where the provider offers it. Building a semantic cache needs embedding infrastructure, similarity thresholds, and a proof of tenant isolation. It is a project in its own right, and only worthwhile if real traffic repeats itself enough.

### A/B model testing

This would split live traffic on a route between models by percentage, with stable assignment and the variant stamped on the audit row.

Today route matching is strictly first-match. Comparisons are done by switching a route for one cohort, or offline: [Evaluate Plugins](/documentation/analysis-evaluate-plugins) runs a fixed prompt suite against two versions of a skill set.

### PHI categories

Health-identifier detection would join the PII set. Today the scanners cover email, card number, US SSN and phone. PHI is only worth adding if regulated health data actually passes through the gateway.

### Data classification beyond two flags

Routes can require a provider to be `european` or `no_retain`, and dispatch refuses a provider that lacks the flag. A richer classification scheme (labels on requests, routed by class) needs Dynapps' data-classification scheme first. Today every provider is declared `european: false`, and no route requires either flag. See [Data classification & residency](/documentation/enterprise-model-routing#data-classification-residency).

### Latency objectives per use case

The analytics Overview tab splits requests either side of a latency threshold you choose on the page. Per-workload objectives, error budgets and alerts on them need target latencies per workload class. The raw figures (total and upstream latency per request) are recorded and exported.

### Quality-normalised model comparison

"Is the cheaper model good enough?" needs an agreed evaluation set of representative tasks, with expected outcomes. Cost and latency per provider are already measured. Quality has no source until that set exists.

### Alert delivery

The signal shipped in 0.63 (see [Shipped](#shipped)); built-in delivery to a named channel (email, chat or webhook) did not, and is not planned in the platform. Route the metrics or the [OTLP export](/documentation/observability-export) into your own alerting stack.

### External guardrail providers (Model Armor)

Core 0.63 lets an extension scanner receive its own configuration (`scanner_settings.<name>.config`) with per-scanner `fail_mode` and `timeout_ms`, which is the seam a vendor guardrail service plugs into. No external provider is wired. A Google Model Armor scanner was assessed on 2026-10-08 and placed out of scope for the gateway: Model Armor is applied by the platform team on the GCP side (Gemini floor settings, Gemini Enterprise and the query service), and the gateway keeps its built-in scanners.

### Outcome metrics

This would join AI usage to results in a system of record: Odoo records, merged pull requests, closed tasks. It needs a decision on which outcome counts, a sync of that system's records, and an identity mapping. See [From usage to outcomes](/documentation/enterprise-analytics#from-usage-to-outcomes).

### Enterprise knowledge platform

Governed retrieval over a corpus the platform owns would need its own ingestion, index and access-filtered retrieval, together with an accuracy benchmark, a knowledge graph and a curated use-case library. None of this is built. Today knowledge is reached live through the connectors listed under [Shipped](#shipped), under each person's own permissions.

## Removed

These were built and then taken out on purpose. They are listed so that an older document or screenshot does not mislead.

| Removed | When | What replaced it |
|---|---|---|
| Per-organization budgets, run-rate early warning, burndown forecasts, scheduled cost digests | 2026-09-01, with organizations and departments | Quota windows as warning thresholds ([Cost Management](/documentation/enterprise-cost-management)). Groups and projects replace organizations and departments. |
| Slack alerting | 2026-09-10 | Metrics and events for the operator's monitoring stack; see [Alert delivery](#alert-delivery). |
| Code-impact metrics (AI-authored lines, applied edits, commit lines) | 2026-09-16 | Nothing. They were proxies too weak to report ([Analytics](/documentation/enterprise-analytics#what-is-not-measured)). |
| The in-house retrieval server | September 2026 | The Dynapps portal connectors (Odoo and DynBrain). |
| Passkey and invitation sign-in | 2026-08-29 to 08-31 | Single sign-on, now Microsoft Entra ID over OpenID Connect. |

## Known limitations

| Limitation | Current state | Where it is discussed |
|---|---|---|
| **Nothing is enforced** | Governance, safety and quota planes run in warn mode since 2026-09-04. Only authentication refuses. | [Governance Modes](/documentation/governance-modes) |
| **No spend ceiling** | Quota windows and per-key budgets warn. A daily per-project window ($100) is declared; no monthly window is, although calendar-month periods are supported. | [Cost Management](/documentation/enterprise-cost-management) |
| **Redaction off** | In-flight redaction is built but no category is listed, so detected values still reach the provider; transcripts are masked at the display layer only. | [Governance Modes](/documentation/governance-modes) |
| **SCIM provisioning** | Not offered. Entra ID can push SCIM, but the platform has no SCIM endpoint yet. The directory group decides access at each sign-in, bounded by the session lifetime. | [User & Access Management](/documentation/enterprise-user-access) |
| **Audit immutability** | Audit tables are guarded by database triggers: settled rows cannot be rewritten, recent rows cannot be deleted, and `ai_requests` and safety findings are hash-chained and verified nightly. The guard ships in `warn` mode until enforcement is switched on. The application role still owns the tables, and archives sit on the instance's own disk, so a database owner can bypass the guard. Separating the owning role and adding write-once storage remain open (DEC-007). | [Data Retention](/documentation/data-retention#tamper-resistance) |
| **Finite history** | Raw requests are kept for 90 days, prompt text for 30 and raw bodies for 7. Older history lives in the weekly archives. | [Data Retention](/documentation/data-retention) |
| **Model access is open** | The access rule for gateway routes is open to every role. Models are shaped by governance and quotas, not restricted per role. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **No EU route** | Every provider is declared `european: false`, and Vertex endpoints use the global location. | [Model Gateway & Routing](/documentation/enterprise-model-routing#data-classification-residency) |
| **Shadow AI outside the gateway** | Unlisted models are refused at the gateway. Traffic that never reaches the gateway is invisible to it, and blocking it is a network egress control for Dynapps IT. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **Tab-acceptance and commit metrics** | Not measurable. Claude Code emits no accept/reject signal, and commits outside Claude Code are not visible. | [Analytics](/documentation/enterprise-analytics#what-is-not-measured) |
| **Metrics endpoint** | The Prometheus `/metrics` listener only starts when the profile sets `server.metrics_port`, and neither shipped profile does. | [Observability Export](/documentation/observability-export) |
| **VS Code Copilot** | A Custom Endpoint configuration is documented but not validated by Dynapps. OpenCode is the supported IDE client of the OpenAI-compatible endpoint. | [Connect OpenCode: VS Code GitHub Copilot](/documentation/connect-opencode#vs-code-github-copilot) |
| **No external guardrail provider** | The scanners running are the built-in `heuristic`, `secrets` and `pii_extended`. A Model Armor scanner in the gateway is out of scope (decision 2026-10-08); Model Armor applies on the GCP side. | [Content Safety](/documentation/enterprise-safety-guardrails) |
| **No push alerts** | The platform emits metrics and events; it sends nothing to people. | [Audit Trail](/documentation/enterprise-audit-observability#anomaly-detection) |

## Check it yourself

- `/admin/gateway/policies`: the warn/enforce state of each plane.
- `/admin/gateway`: the routes, including the Vertex route's `fallbacks` and per-project chains.
- `systemprompt admin users api-key list --user <user>`: a key's limits and scope bindings.
- `/admin/system/observability`: whether a collector is configured.
- `/admin/configuration` and `/admin/lifecycle`: the retention windows and archives.
- `systemprompt infra logs governance report --since 7d`: what warn mode recorded in the last week.
