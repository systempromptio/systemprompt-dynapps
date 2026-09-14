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
| `feedback/bridge-receipts` | Integrated `b284948f4`; active readback, scoped outbox and native session recovery | Passed within the 1,154-test integrated marketplace/bridge/evaluator/scheduler batch |
| `feedback/inventory` | Integrated `77c79a311`; canonical catalog, history, explicit bindings and baseline preparation | Full marketplace suite passed: 338 tests, 0 skipped; includes ten new inventory fixtures |
| `feedback/astound-ingestion` | Integrated core `f60b5a8ac`, Astound `6797afe1`; authenticated native hooks, durable normalization/corrections and coordinated expiry | Migration 059 applied successfully to disposable DB; behavioral tests pending |
| `feedback/evaluator-runtime` | Integrated `fcb94767e`; admission, supervisor, pin/cleanup contracts | New admission and probe-cleanup contracts passed in the 720-test batch; full adapter/native validation remains pending |
| `feedback/adapter-claude-code` | Integrated `c30ef9b78`; isolated adapter and retained completion evidence | Eight adapter contracts passed in the 720-test batch; native probe rejected stale local executable at version check and confirmed cleanup |
| `feedback/adapter-opencode` | Integrated `96296e01b`; pinned configuration and bounded normalized evidence | Passed within the 1,154-test integrated marketplace/bridge/evaluator/scheduler batch |
| `feedback/adapter-codex` | Integrated `d805d3435`; pinned native adapter and bounded relay | Nine Node relay contracts and full scheduler suite (586 tests, 0 skipped) passed |
| `feedback/adapter-hermes` | Integrated core `f7fe34339`, Astound `a2185589`; pinned native namespace and bounded chat relay | All 15 Codex/Hermes Node relay contracts passed after isolated custom-provider authentication correction; pinned version/native isolation probe passed; full native/gateway acceptance pending |
| `feedback/analytics-snapshots` | Integrated core `9cd069f0b`, Astound `421cde18`; incremental daily/snapshot processing and all-owner atomic retention | Migration 007 applied; checkpoint-holding fixture corrected; full analytics/scheduler/bridge batch passed: 1,452 tests, 0 skipped, including snapshot correction, retention and functional freshness cases |
| `feedback/campaign-completion` | Integrated core `49890ae17`, Astound `179abf83`; independent holdout, retained diagnostics and inventory baseline setup | Combined evaluation/marketplace/API batch passed: 1,734 tests, 0 skipped; migration acceptance pending |
| `feedback/dashboard-stream` | Integrated core `bad0abec2`, Astound `8deec1db`; snapshot portfolio, installation coverage and durable SSE generations | Four browser lifecycle tests passed; core installation coverage/SSE checks passed in the 1,734-test batch; raw-reference parity pending |
| `feedback/api-contracts` | Integrated `e532a7022`; typed OpenAPI, bounded collections, retained operations and consistent problems; transport/status corrections through `7b6ac501a` | First compile found a SQL alias and owned-input fixture issue; corrections integrated; behavioral gate pending |
| `feedback/acceptance` | Supervisor recovery and bounded workspace fixtures; stateful fresh/upgrade parity; authenticated hook freshness; durable gateway failure journal; native fixture harness | Four supervisor recovery tests passed; migration, ingestion, accounting and full native acceptance pending |

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

Migration ownership: marketplace 004 receipts, 005 dependency verification, 006 inventory, 007 installation coverage, 008 retained API operations; analytics 005 facts, 006 producer barrier, 007 snapshots/retention; evaluation 014 runtime, 015 campaigns; AI 024 durable accounting-failure projection; Astound 059 authenticated evidence and coordinated retention. Contracts do not add DDL.

## Observed integrated validation

- 1,452 analytics/scheduler/bridge tests passed with zero skipped (18.428 seconds test execution, excluding compilation). Includes correction, privacy retention, distinct/histogram and functional snapshot freshness fixtures.
- 1,734 evaluation/marketplace/API tests passed with zero skipped (8.794 seconds test execution, excluding compilation). Includes independent holdout, installation coverage and stream recovery. This run predates the final API operation/status additions.
- Four supervisor recovery tests passed. The newer bounded-workspace, durable accounting journal, stateful migration and authenticated hook freshness cases still require execution.
- All 33 Astound source gates passed after integration through the Hermes runtime authentication correction. All 17 browser lifecycle tests passed after adding reconnect backoff, custom-range polling, failed-refresh recovery and DOM teardown coverage to the normal lifecycle gate.
- All 22 Codex/Hermes/OpenCode Node relay contracts passed, including local credential rotation, rejection of stale/placeholder tokens, host credential isolation, hard forwarding limits and cleanup.
- The seven-package AI/evaluation/marketplace/API/scheduler/gateway/bridge batch passed: 3,500 tests, zero skipped, 19.286 seconds test execution (core `97c4d024b`). Includes retained API operations/status, publication races, independent dependency credentials, bridge upgrades/rollbacks, bounded workspaces and durable accounting-failure recovery. Later scoped-CA, HTTP issuance and OpenCode v2 tests still require execution.
- Astound integration now compiles and creates a fresh schema after moving receipt columns into declarative CREATE TABLE definitions. The suite found two legacy publication fixtures missing newly required complete source attestation; correction and a complete rerun remain required. No migration or ingestion parity pass is claimed yet.

The current Linux evaluator image is `sha256:34fbd863b8c721abf2a56a6e8bd5f7190b7ddfba9976bf5fd81072f0f586c24a`. Claude Code 2.1.270, OpenCode 1.18.29, Codex 0.154.0 and Hermes 0.21.3 all passed exact version, native isolation and cleanup probes on that image. These probes use no provider inference and do not establish gateway metering. Automated native targets remain disabled.

Earlier deterministic-provider execution exposed client-specific defects now corrected: Codex sandbox mount targets and MCP names, OpenCode advisory request limits, Hermes auxiliary title requests and MCP toolset selection, and bounded failure deadlines. OpenCode now enforces the request ceiling in its parent relay; native steps alone are insufficient. Codex denial tests now require boundary-specific evidence so sandbox setup failure cannot count as successful isolation. A complete run of all corrected clients remains required. Retained native wire replay alone does not prove execution-capability admission or budget reservations; the implemented live private-socket gateway fixture still requires compilation and execution.

Validation uses PostgreSQL 18.6 and `/usr/bin/docker` 26.0.0 on Linux. Disposable schema-only fixtures `feedback_marketplace_test_20260914` and `feedback_snapshots_test_20260914` support functional database tests; they are not proof of fresh/upgrade parity. Windows/macOS native bridge acceptance is unavailable in this Linux environment. No paid inference, load, throughput or p95 measurements have been run.

Incoming core `cc4f65942` passed Quality and Supply Chain. Its CI bridge timeout is addressed by `82c7f50d6`; baseline coverage was 88.95%, below the 91.5% floor. Coverage deficiencies are real missing tests, not an instrumentation failure. Final coverage and CI verification remain outstanding; no floor or exclusion policy has been weakened.

Astound's dependency manifests match core's `rmcp =3.3.0`, `rustls 0.23.45` and `jsonwebtoken 11` requirements. The earlier three-lockfile RustSec audit passed under the existing advisory policy, with unsound/unmaintained dependency warnings; the final resolved-lockfile audit remains required.

Final static/Clippy/unit/integration/contract/coverage gates, core CI, exact core pin, both remote pushes and safe temporary branch cleanup remain outstanding. Unrelated concurrent files and deleted SQL metadata in the main core checkout are preserved and excluded from task commits.
