# Cloudfresh / Google reseller session — prep (Friday 2026-10-09)

Attendees per Boris: Cloudfresh (Google reseller), Kateryna, Vlad, Lisa, IT.
Pre-reads: "Gateway — Cloudfresh Proposal and Gap Map" and "EAI — Target
Architecture and Responses to TR-001–022" (06.10.2026), the IT registry, the
global-context diagram. Our position paper: `docs/status/requirements-reconciliation-2026-10-08.md`
and `integration.txt`.

## 1. Opening correction: their snapshot is 0.62.0, we are on 0.63.1

Their document says "if any of this is no longer accurate, we will correct it
at the start". It is no longer accurate. Say this first, in one minute:

| Their statement (29.09 snapshot) | Today (0.63.1, released 08.10, on production) |
|---|---|
| Only authentication and model ACL block; everything else warns | Still warn **by policy**, but the enforce machinery is complete: quota reservation at admission, machine-readable 429 with reset time, per-key budgets, in-flight redaction. Flipping is a config change per plane. |
| Backup provider is Anthropic API, traffic leaves GCP | Routes are ordered chains; Anthropic is used only when listed. The chain is being switched to Vertex→Vertex (second region) today. |
| No project tags on requests (release risk, D-33) | Every request is attributed to **project, cost centre and practice** (header → API-key binding → primary project), verified against membership (403), stored and exported on every OTLP span. |
| Docker Compose on 1–3 VMs, secrets in a file, nodes boot one at a time | Concurrent boot, `server.role gateway|admin`, load shedding, `--skip-migrate`, GCS file storage, read-only root, writable paths declared. A Helm chart with Secret Manager CSI and Cloud SQL Auth Proxy is landing. |
| Extensibility only by Rust rebuild | True, and by design. The seams they need exist: scanner settings (opaque config, fail mode, timeout), route selectors, upstream providers, subject-attribute providers. |
| 114 requirements: 45 / 39 / 28 | 0.63.1 reconciliation: of 156 registry rows, 93 completed, 54 out of scope by decision, 9 in progress. Priorities are now assigned (xlsx). |

## 2. Their seven decisions — our answers

1. **Providers.** Yes: everything through Vertex; fallback is a second Vertex
   region, not Anthropic. Config change, landing today. Ask Cloudfresh to
   request Claude-on-Vertex quota in the EU region **and** the global
   endpoint in advance; name the region they want (europe-west1 is where
   Claude is served; confirm current availability).
2. **Attribution format.** Already built: `x-systemprompt-scope-project`,
   `-cost_centre`, `-practice` headers; a virtual key can be bound to a
   project so a pipeline needs no header; a user's primary project is the
   default. Spans carry `systemprompt.scope.<dim>` and `.source`. Agree only
   the *values*: project ids, cost-centre codes, practice names, and who
   owns the mapping (PSA → gateway).
3. **Enforce.** Agree their sequence: two weeks of "would have blocked"
   reporting, then secrets + budgets first. We need DEC-003 (categories:
   block vs redact vs audit) and DEC-004 (sizes) signed off; the per-project
   $100/day window is our starting proposal. Note: redaction's fail-closed
   refusal applies even in warn mode, which is why it is not switched on yet.
4. **Identity.** Our recommendation: keep AD FS as the IdP for the gateway at
   launch; mirror AD groups into Google Groups (their attribute service) so
   the same groups drive GE and the query service. Contractors: Workforce
   Identity Federation on their side; on ours a second IdP, SCIM and
   invitations are out of scope until this decision lands.
5. **Business users.** Agree: Gemini Enterprise for business users, the
   gateway for developers and agents. One caveat to put on record: GE
   traffic is **not** audited by the gateway, so the "all traffic through the
   gateway" commitment (their commitment 2) needs rewording to "all developer
   and agent traffic".
6. **Who implements/operates.** Gateway code: systemprompt (us), as today;
   Astound configures. GKE operation: propose Cloudfresh runs the landing
   zone and cluster, Astound runs the Helm release through CI, we support.
   Ask for a named operator and an on-call path.
7. **Claude Code transcripts.** Today: raw requests 90 days, prompt text 30,
   bodies 7, weekly archives; manager and admin reads are now logged. Propose
   keeping that and letting DEC-007 (audit regime) set the final window.

## 3. Delineation: systemprompt vs Google functions

Keep this short and say it as a rule: **the gateway decides per call; GCP
provisions once.**

| Concern | systemprompt (gateway) | Google / Cloudfresh |
|---|---|---|
| Identity to the gateway | AD FS SAML, roles, groups, projects, device certs, PATs with per-key limits | Google Groups for GE / query service; Workforce Identity Federation |
| Model access + routing | Route ACLs, per-project chains to the right GCP project/region, failover, strategy | Model projects, quotas, provisioned throughput, regional endpoints |
| Cost | Token cost per request, reservation + ceilings, attribution keys on every row and span | Billing export, Cloud Billing budgets (monetary cap), BigQuery, Looker |
| Safety | Governance chain on tool calls; built-in scanners; redaction; per-scanner fail mode | Model Armor for Gemini/GE/query service; SDP at ingestion |
| Audit | Request → decision → model → tool → cost with one trace id; OTLP export; tamper-resistance landing | Cloud Logging / Trace retention, SIEM, Cloud Audit Logs |
| Knowledge | Registers the query service as a governed remote MCP server; passes the user's identity; no index of its own | Ingestion, Agent Search / AlloyDB, query service, eval harness |
| Agents | Governs their model calls (run_id, project) | ADK runtime on Cloud Run/GKE, Cloud Workflows, A2A registration in GE |
| Platform | One stateless image, schema + migrations, probes, metrics port | GKE, Cloud SQL HA, Secret Manager, GCS, Artifact Registry, Cloud Armor, VPC-SC |

## 4. Corrections to the TR document that touch us

- **TR-012 / TR-022.** Agree with their recommendation of one MCP server
  that takes the user's token and the project as a parameter. That is
  exactly how the gateway registers remote MCP servers today; the gateway
  passes the signed-in identity, so per-project deployments are unnecessary.
- **TR-021.** Claude Code / OpenCode conversations are *not* outside the
  system: the gateway holds them, searchable by the user and (new) their
  project manager, exportable. If a shared conversation knowledge base is
  wanted, it can ingest from our export rather than from clients.
- **Commitment 2 nuance:** "developer tools do not pass a project ID
  themselves" is solved on our side (bridge-injected / key-bound project);
  the "wrapper that adds headers" they propose is not needed.
- **"The gateway has no monetary cutoffs (G6)."** It has cost ceilings per
  user/org/project/key in microdollars; what it lacks is a *GCP-billing*
  reconciliation, which is theirs. Keep both: gateway ceilings stop a runaway
  within minutes, Cloud Billing budgets catch drift monthly.

## 5. Questions to send Boris before the session

1. Which GCP project and region host the gateway, and does the Vertex
   quota request for Claude (EU + global) go in this week?
2. Who runs the landing zone and GKE cluster day to day, and who holds the
   on-call path for the gateway deployment?
3. Will Secret Manager CSI and Cloud SQL Auth Proxy with IAM be allowed on
   the cluster (Autopilot or Standard)?
4. Attribution values: who owns the project-id / cost-centre / practice
   lists, and will the PSA push them?
5. Can sp-dev be upgraded to 0.63.1 before the client re-verifies
   REQ-055..062? Every one of those is fixed there.
6. Is the spend-cap feature available under the Cloudfresh billing account
   (their doc says reseller accounts may not have it)?
7. For GE business users: is it accepted that their traffic is outside the
   gateway's audit, and is that the final shape of commitment 2?

## 6. Things not to concede

- Model Armor inside the gateway: out of scope (decision 08.10). The seam
  exists if they fund it later.
- Replica reads for analytics: off by design; propose measuring with the
  load test before adding complexity.
- Separate Gemini API provider: fine to remove once Vertex Gemini quota is
  confirmed for the classifier job.

## 7. One-slide summary

Gateway on 0.63.1: attribution, enforce-ready budgets, per-project Vertex
routing, GKE-ready image. Landing this week: Helm chart, Vertex-only
fallback, selection strategies, context-window checks, audit hardening,
overhead figure. Needs from Astound/Cloudfresh: decisions 3, 4, 6, 7;
Vertex quota; GCP project; sp-dev upgrade.
