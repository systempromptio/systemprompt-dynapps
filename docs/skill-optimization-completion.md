# Skill optimization completion ledger

Scope: the feedback-loop implementation plan, including guided independent holdout and retained blocked diagnostics. No load testing, deployment, paid inference, automatic publication, source-repository commits, personal asset adoption, or new non-admin reviewer permissions.

Integration starts from Astound `8af04e69` and core `cc4f65942` (the three incoming dependency updates are integrated). An unrelated edit to `docs/research/gcp-managed-rag-as-mcp.md` is preserved and excluded from task commits.

Status records observed results only. A contract or route alone does not establish subsystem completion. Native combinations remain disabled until their isolation and metering acceptance passes. Throughput and p95 targets are unmeasured.

| Branch | Implementation | Tests and observed result |
| --- | --- | --- |
| `feedback/contracts` | Integrated core `b386701cb`; shared models and identities, typed changes | Seven focused contract tests passed through Astound with online SQL validation (2218 unrelated tests filtered out) |
| `feedback/consumer-receipts` | Integrated `cfa9eb89c`; lock-query correction `3eaf1ca62` | 11 DB tests passed; full marketplace suite: 326 passed, 0 skipped |
| `feedback/git-verification` | Integrated `21c0b12a4`; Astound credential orchestration `9974e9e8` | 10 HTTPS/process/DB tests passed; full marketplace suite: 326 passed, 0 skipped |
| `feedback/analytics-facts` | Integrated `d9ebb3f95`; durable leased facts, replacement deltas, runtime and scheduler | Full analytics suite passed within a 720-test batch; all ten new DB cases passed, including late commits, fencing, backfill/recovery and shared-cost reference parity |
| `feedback/bridge-receipts` | In progress | Not run |
| `feedback/inventory` | In progress | Not run |
| `feedback/astound-ingestion` | Pending | Not run |
| `feedback/evaluator-runtime` | Integrated `fcb94767e`; admission, supervisor, pin/cleanup contracts | New admission and probe-cleanup contracts passed in the 720-test batch; full adapter/native validation remains pending |
| `feedback/adapter-claude-code` | Integrated `c30ef9b78`; isolated adapter and retained completion evidence | Eight adapter contracts passed in the 720-test batch; native probe rejected stale local executable at version check and confirmed cleanup |
| `feedback/adapter-opencode` | In progress | Not run |
| `feedback/adapter-codex` | Pending | Not run |
| `feedback/adapter-hermes` | Pending | Not run |
| `feedback/analytics-snapshots` | Pending | Not run |
| `feedback/campaign-completion` | Pending | Not run |
| `feedback/dashboard-stream` | Pending | Not run |
| `feedback/api-contracts` | Pending | Not run |
| `feedback/acceptance` | Prerequisite proxy IPv4 fix `82c7f50d6`; coordinated core test recipe `38fb8a13`; full acceptance pending | Shell syntax/recipe parsing passed; four focused bridge tests passed (149 unrelated tests filtered out) |

Required acceptance groups: authenticated consumer/device/session evidence and retries; dependency credentials/provenance/bytes/modes; inventory reconciliation/history; admission before budget reservation and four adapters; normalized facts and fenced durable processing; corrections/deletions/privacy retention; snapshot parity/distinct/histograms; SSE recovery/teardown; REST pagination/idempotency/auth/origin/OpenAPI; fresh/upgrade migration parity; static, Clippy, unit, integration, contract and coverage gates; core CI and exact final pin; functional committed-evidence-to-snapshot freshness.

## Requirement ownership

| Branch | Required behavior and verification |
| --- | --- |
| contracts | Canonical identities; compatible client serialization; separate receipt/session contracts; dependency manifests; inventory inputs; normalized analytics changes; migration allocation. |
| consumer-receipts | Enrolled-device credentials; authenticated consumer and retained publisher; current grants/revocation; exact installation identity; idempotent/conflicting retries; separate authenticated session bindings; immutable evidence and correctable history; unknown historical identities. |
| git-verification | Application-resolved scoped secrets for import/sync/verification; isolated bounded HTTPS Git; credential forwarding/redaction/cleanup; every dependency source/commit/root/bytes/modes; cycles/provenance/submodules rejected; complete retained attestation manifest. |
| analytics-facts | Independently deduplicated invocation/request/assessment/association facts and timestamps; failed spend and unknown pricing; non-additive shared costs; fenced PostgreSQL leases, transactional checkpoints, corrections/tombstones, diagnostics and restart-safe backfill without sequence commit assumptions. |
| bridge-receipts | Readback bytes/modes for every implemented host; explicit unavailable verification; bounded durable outbox; identical acknowledgments/conflict rejection; offline unacknowledged installs; native-session binding; tampering/partial install/upgrade/rollback recovery. |
| inventory | Configured plus imported canonical inventory; explicit bindings and conflicts; unused/unpublished/unavailable/withdrawn entries; effective membership and unknown historical state; three-way sync preserving candidates/publications/evidence; general baseline capture. |
| astound-ingestion | Durable authenticated adapter; retryable attribution; publisher distinct from consumer; late receipts correct attribution without duplicate invocation; replace owner-equals-consumer trigger after parity. |
| evaluator-runtime | Shared capability registry and actual admission on every launch/follow-up before reservation; supervisor/gateway/leases/network/cancellation/restart/cleanup integration; matched baseline/candidate environments; desktop install-only. |
| adapter-claude-code | Implement pinned executable/image, isolated workspace/config/credentials, skill installation, bounded execution, normalized evidence and verified cleanup; native isolation/metering contracts govern enablement. |
| adapter-opencode | Same execution contracts; preserve existing serialized client compatibility; native isolation/metering contracts govern enablement. |
| adapter-codex | Same execution contracts; native isolation/metering contracts govern enablement. |
| adapter-hermes | Same execution contracts; native isolation/metering contracts govern enablement. |
| analytics-snapshots | Incremental daily additive totals; exact 90-day distinct identities; versioned geometric histograms; 1/7/30/90/365-day snapshots; asynchronous custom ranges; replace contributions on correction/deletion; privacy-safe coordinated retention and reviewed-evidence FK handling; non-identifying totals until day 365 with small-cohort suppression and unavailable older distinct metrics. |
| campaign-completion | Capability/version-aware setup/reports; inventory-wide baselines; guided independent holdout; enough independent benchmark cases without lowered thresholds; persistent actionable blocked diagnostics. |
| dashboard-stream | Snapshot-only portfolio serving; installation and usage coverage separated; watermarks/freshness/health/percentile coverage; core SSE limits and teardown; durable generation reconnect/resync and notification wake-up hints. |
| api-contracts | Consumer/admin separation; typed bounded /api/v1 resources; cursor collections; idempotency and resolvable operation status; consistent problems and cookie-origin protection; OpenAPI for all subsystem routes and full campaign lifecycle. |
| acceptance | Cross-system fixtures; raw-reference parity before old-path removal; fresh/upgrade schemas; migration suites; static/Clippy/unit/integration/contract/coverage gates; core CI; functional concurrency/fencing/restart/retention/freshness; native acceptance or precise limitations; current RustSec; final exact pin, both next pushes and safe temporary branch cleanup. |

Migration ownership: marketplace 004 receipts, 005 dependency verification, 006 inventory; analytics 005 facts, 006 snapshots/retention; evaluation 014 runtime, 015 campaigns. Contracts do not add DDL.

Incoming core CI observation: `cc4f65942` passed Quality and Supply Chain; CI failed the bridge sibling-proxy test on a 20-second timeout, and Coverage failed. These are unresolved baseline findings, assigned to acceptance.

Dependency synchronization: Astound root/test manifests now match incoming core's exact `rmcp =3.3.0`; all three lockfiles resolve `rustls 0.23.45`. `just audit` passed on 2026-09-14 under existing advisory ignores, with glib unsoundness and unmaintained proc-macro-error/ttf-parser warnings. No advisory policy was weakened. Initial static run failed on concurrently edited core events formatting outside this task; those changes are preserved.

Validation environment: `/usr/bin/docker` is available (26.0.0); the PATH Docker wrapper fails through WSL. Marketplace focused tests use `feedback_marketplace_test_20260914`, copied schema only via a PostgreSQL 18 client; this is functional test setup, not fresh/upgrade migration parity acceptance.

Claude native probe on 2026-09-14: existing local image `sha256:fe707807cbec2d19d6d160c6bb5472f1faa0d97414f1009b1ff1941422cb6003` failed the exact 2.1.270 version check. Container cleanup was confirmed. No inference was run and no native target was enabled. The final combined adapter image still requires rebuilding and acceptance.
