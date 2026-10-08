# Requirements reconciliation against 0.63.1 — 2026-10-08

Baseline: systemprompt-astound **0.63.1** on published core **0.63.1**, both
released 2026-10-08. Production (`astound.systemprompt.io`) runs 0.63.1
(`/api/v1/health` checked today). The client host `sp-dev.astound.digital`
still runs **0.62.0**, the version every "vendor-verified gap" in the
register was observed on.

Three inputs were reconciled:

| Input | Where | What changed today |
|---|---|---|
| IT registry (FR/NFR/AST/UC, as of 0.62.0) | `AI_Gateway_Requirements_Registry_EN.xlsx` | Four columns added to every requirements sheet: *Status 0.63.1*, *Disposition*, *Evidence 0.63.1*, *Change since 0.62.0*; a disposition column on both UC sheets; vendor status + evidence on *Enterprise mapping*; a 0.63.1 count table on *Summary*. |
| Enterprise register (REQ-001..062, DEC, OBJ) | `[Enterprise AI] systemprompt.io requirements register (7).xlsx` | Status, Ed's comments, Evidence and Delivered-on rewritten per row; a *State 0.63.1* column; Objectives and Open Questions (DEC) updated; two statuses added to Definitions. |
| Cloudfresh gap map + target architecture (06.10) | `Gateway — Cloudfresh Proposal and Gap Map_EN (1).docx`, `EAI — … TR-001–TR-022_EN (1).docx` | Each gap-map row has a disposition below; meeting prep in `cloudfresh-meeting-prep-2026-10-09.md`. |

Decisions on every remaining gap are in `integration.txt` (repo root). The
scripts that produced the workbooks are in `.build/requirements-refresh/`
(`recon_data.py` holds the per-row verdicts; `apply_xlsx.py` applies them to
the originals saved beside them).

## Headline

| Section | Total | Completed | Out of scope (decided) | In progress |
|---|---|---|---|---|
| Functional FR-1..14 | 78 | 40 | 38 | 0 |
| Non-functional NFR-1..8 | 36 | 31 | 5 | 0 |
| Already have AST-01..11 | 11 | 11 | 0 | 0 |
| Enterprise additions (incl. FR-15..18, NFR-1.6/2.5/5.6/6.4) | 31 | 18 | 11 | 0 (+2 decision-only: FR-13.8 targets, NFR-2.5 load test) |
| Enterprise register REQ-001..062 | 62 | 46 Delivered | 16 Out of scope | 0 In progress |

"Completed" includes items the Cloudfresh gap map assigns to a GCP service
(Secret Manager CSI, Cloud SQL Auth Proxy, Cloud Monitoring, BigQuery); the
disposition cell says so each time. "Out of scope" is an explicit decision
with a reason, never a missing feature left unexplained.

## What core 0.63.0 closed (the 7 Oct list)

Every item of `core-requirements.txt` except D1 (Model Armor, now out of
scope by decision) shipped in core 0.63.0 and is on production:

| Plan item | Core commit | Requirement ids |
|---|---|---|
| A1–A5 skill frontmatter, hook timeouts, listing budget, dependency pinning, dev-file exclusion | 4b582d0a8, 0dbd4fa2b | FR-16.1–16.5, REQ-055–058, 060 |
| A6 Vertex field stripping | 4f353fc2 (+ astound 6574fd31) | FR-1.9, REQ-059 |
| B1–B4 attribution, cost centre, quota reservation, per-key limits | 15523f567, 7cfc2c69e, 118dbcef1, 073401f7b | FR-3.1, 5.2, 5.3, 5.7, REQ-006/009/010/013/022/033 |
| C1–C2 per-scope routing, ordered failover | a25654542 | FR-2.1, 2.4, REQ-032, UC-22 |
| D1 scanner settings, D2 redaction | 512b394ec, eab3b5c4c | FR-7.1, 7.7, REQ-036 |
| E1–E5 concurrent boot, GCS, roles, shedding, skip-migrate | fdb630b7c, 575a53550, 967b9e3f9, 56e638d61, 4c6869000 | NFR-2.1–2.4, 3.5, UC-19/44/45 |
| F1–F4 anomaly signal, writable roots, overhead histograms, SECURITY.md | 3164acfa6, 5f9004cea, d1f07e885, f3c924c24 | FR-13.2/13.3/13.7, NFR-1.1, 4.4, 4.7, 4.8 |

Bridge 0.63.0 also enrols Claude Code on first run (REQ-061) and the Linux
installer refuses macOS (REQ-062, astound 6574fd31).

## Gap plan (owners as of today)

In scope — all built on 2026-10-08 (integration.txt §A). Validation: `just prepare`, `just clippy` over all three workspaces against the patched core, and `just preflight-static` (39 gates) are green; no test suites were run.

| # | Item | Owner | State |
|---|---|---|---|
| A1 | Helm chart + GKE runbook (`deploy/helm/systemprompt-astound`, `docs/install/gke.md`, doc page `deploy-gke`) | agent helm-gke | **done** (helm lint clean; not yet run on GKE) |
| A2 | Docs refresh to 0.63.1 + repo `SECURITY.md` | agent docs | **done** |
| A3 | Nightly gateway-overhead p50/p95/p99 on `/admin` + `metrics_port` in the self-host template (migration 117, job `gateway_overhead_rollup`) | agent overhead | **done** |
| A4 | Manager-scoped history for `project_manager` (+ `history_access` event) | agent history | **done** |
| A5 | Weighted / least-busy deployment selection (`strategy`, `weight`) on core `next` | agent core-routing | **done** (core next ef05c3928, unpushed; astound migration 119) |
| A6 | Context-window pre-check + context fallback (`context_fallbacks`, 400 `context_window_exceeded`) on core `next` | agent core-routing | **done** (core next ef05c3928) |
| A7 | Tool-schema meta-validation at registration + CI gate | agent schema-vertex | **done** (external-server registration refusal needs a core hook) |
| A8 | Vertex→Vertex fallback chain in `services/ai/gateway.yaml`; Anthropic removed from chains | agent schema-vertex | **done** (vertex-anthropic-eu, europe-west1; request EU quota) |
| A9 | Audit hardening (REVOKE, hash chain, explicit retention) + archive-instead-of-delete with restore and legal hold | agent audit | **done** (migration 118 ships in warn; core next ec334c86e) |

Out of scope by decision (integration.txt §B): Model Armor in the gateway,
per-subject TPM/concurrency, in-product alert channels, time windows / in-app
IP allowlist / break-glass, second IdP / SCIM / invitations / multi-org,
caches / auto-selection / A-B / compression, embeddings / batch /
pass-through / OpenAPI→MCP / tool filter, RAG / knowledge / memory /
workflows / sandbox, FOCUS / Terraform provider / Vault SDK / in-process
Cloud SQL IAM / probes / console catalog / per-team logging, the removed
forecasting and productivity features, provider breadth beyond Vertex.

Owed by Astound / Cloudfresh, not gateway work: decisions 3 (enforce timing,
with DEC-003 categories and DEC-004 sizes), 4 (identity), 5 (business
users), 6 (who operates on GKE), 7 (transcript retention), DEC-007 (audit
regime), per-use-case latency targets (FR-13.8), the load test (NFR-2.5)
once GKE exists, and the sp-dev upgrade to 0.63.1.

## Cloudfresh gap map, row by row

| Gap-map row | Gateway side (their ask) | State on 0.63.1 |
|---|---|---|
| Budgets only warn | enforce by user/group/project | Mechanism built (reservation, 429 with reset); per-project $100/day window declared; flip = decision 3 |
| IDE request has no project | determine + verify project per request | Built: header → key binding → primary project; 403 if not a member |
| Fallback goes to Anthropic | fallback to another Vertex region | Done: `vertex-anthropic → vertex-anthropic-eu` (europe-west1) in every chain; Anthropic in no chain |
| No cross-region balancing | route non-EU to global endpoint | Done: `strategy: weighted|least_busy` + `weight` per deployment (core next) |
| No TPM / concurrency | route by project to its GCP project | `by_scope` built; per-project GCP projects = provider entry + one line |
| No external guardrails | call Model Armor | **Out of scope in the gateway** (B1); applied on GCP side |
| Basic scanners | use SDP templates | Same as above; redaction + per-scanner fail mode built |
| Secrets in file/env | mount from Secret Manager | No code change; Helm SecretProviderClass (A1) |
| Postgres password | Auth Proxy | No code change; sidecar in Helm (A1) |
| No Helm / Terraform | Helm chart + pipeline | Done: `deploy/helm/systemprompt-astound` + `docs/install/gke.md` |
| Files on node disk | Cloud Storage | Built (`storage.backend: gcs`) |
| Single DB | reports to read replica | Supported in core; kept off (replica lag); HA by Cloudfresh |
| Inference + admin in one process | two deployments | Built (`server.role`) |
| No export to standard systems | OTLP endpoint | No code change; collector example documented |
| Alerts not delivered | emit signals | Built (`systemprompt_usage_anomaly_total` + event) |
| No chargeback | cost centre on ai_requests | Built (attribution rows + OTLP attributes; cost_centre and practice dimensions) |
| Perimeter / hardening | non-root, read-only FS, Artifact Registry | Built in core + image; Helm restricted profile (A1) |
| No RAG | register query service as MCP | Supported today (remote MCP registration) |
| No embeddings / batch | no change | Out of scope (bypass) |
| No memory / workflows / sandbox | pass agent calls with run_id | Supported (scope headers / attribution) |
| AD FS only, no SCIM | decision 4 | Waiting on decision 4 |
| Prompt registry | marketplaces from git | Built (skill versions, rollback) |
| Response cache / auto-selection | not now | Out of scope |

## Corrections to the IT registry's 0.62.0 reading

- FR-5.6 / REQ-014 and REQ-011/016: forecasting, run-rate warnings and
  digests were **removed** on 2026-09-01; the register's "Delivered" was
  stale. Now recorded as out of scope with the GCP replacement.
- FR-7.8 / REQ-024: shadow-AI "Delivered" means the gateway side only;
  off-gateway traffic is a VPC-SC / egress control.
- FR-15.x / REQ-007/008: code-impact metrics were removed 2026-09-16; out of
  scope pending DEC-002.
- FR-17.3: `/openai/v1` is not a route and never was; the OpenAI base URL is
  `https://<host>/v1`. Documented today.
- REQ-053 / REQ-023: AD FS SAML is live and the only sign-in; the register's
  "passkey-only" comment predates 2026-08-29.
