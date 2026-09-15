# Skill optimization completion ledger

This ledger separates implemented behavior from observed validation. Reviewed reporting/privacy, supervisor and suggestion changes are integrated on `next`, including analytics 009 and evaluation 016. Current reruns have not completed. A concurrent raw-portfolio rewrite awaits user direction; the authorized default remains snapshot-backed serving, and this ledger does not claim that the current working tree is coherent.

Scope includes all 17 workstreams, guided independent holdout and retained blocked diagnostics. Organizational ownership, current consumer grants, immutable revisions and human publication approval remain required. Personal adoption, additional non-admin reviewer permissions, deployment, paid inference, automatic publication, skill-source repository commits and load testing are excluded. Throughput and p95 targets remain unmeasured.

## Current completion gates

| Area | Current status | Remaining evidence |
| --- | --- | --- |
| Feedback implementation | All 17 workstreams have retained implementations, including reporting/privacy reconciliation | Current integrated functional, migration, contract and static reruns |
| Consumer authorization | Current organization grants and device eligibility are enforced; bridge compaction resolves installation identity on the server before session binding | New installation requires a fresh native session; current integrated authorization and bridge regressions pending |
| Owner deletion and retention | Source-owner locks precede feedback barriers; analytics 009 fences committed pending evidence even before reporting initialization and preserves monotonic global cutoffs across owners | Current privacy, out-of-order compaction, deletion and concurrency reruns pending |
| Runtime startup and readiness | `12bcd27e3` retains preparation/start failures as blocked outcomes, preserves prior evidence/spend, fences cleanup by owner/worker/lease and labels, and exposes historical readiness separately from proof-backed admission | New startup, stale-fence, readiness and existing supervisor tests require completion of the corrected integrated batch |
| Analytics and dashboard | Snapshot-backed portfolio/SSE remains the default plan; privacy corrections, expiry and replay/rebuild fencing are implemented | Current privacy reruns pending; concurrent raw-portfolio rewrite awaits user direction |
| Native targets | Exporter builds passed; historical basic-probe image `95740b46…` and Docker cache were lost; image rebuilt as `6a49c8d3…`; automated targets remain disabled | Fresh immutable image identity and all 52 native cases with actual live gateway accounting; prior image proof cannot transfer |
| Migration compatibility | Permanent exact historical-checksum transition implemented; 69 selected migration tests passed in 1.727s, 419 excluded by filter | Latest analytics 009/evaluation 016 and full fresh/upgrade reruns pending |
| Local build and startup | Shared workspace build passed in 6m 01s; local startup completed in 13.6s with two MCP services; `/health` returned HTTP 200, healthy, version 0.52.0 | Local observation only; candidate privacy tests and delivery gates remain pending |
| Coverage | Remote baseline below required floor; new meaningful coverage work is in progress | Current coverage measurement and at least the unchanged 91.5% core floor; no lowered floor or exclusions |
| RustSec | All 11 lockfiles audited read-only; retained findings and existing exceptions listed below | Recheck any subsequently changed final lockfile and verify final supply-chain CI |
| Delivery | Final integration remains on `next`; concurrent unrelated changes preserved | Current core CI, final exact Astound core pin, both pushes and safe temporary branch/worktree cleanup pending |

The checksum transition recognizes the historical Rust string hash only when it matches the currently declared SQL exactly. It does not convert arbitrary drift, renamed slots or tombstones; a concurrent ledger change aborts the guarded transaction. Existing drift policy is unchanged. The successful local conversion/startup and the permanent regression suite are separate evidence.

## Workstream implementation and historical evidence

All results in this table describe the stated integrated batch, not an all-pass claim for the current snapshot.

| Branch | Implementation | Historical evidence / remaining native acceptance |
| --- | --- | --- |
| `feedback/contracts` | Integrated core `b386701cb`; shared models and identities, typed changes | Seven focused contract tests passed through Astound with online SQL validation (2218 unrelated tests filtered out) |
| `feedback/consumer-receipts` | Integrated `cfa9eb89c`; lock-query correction `3eaf1ca62` | 11 DB tests passed; full marketplace suite: 326 passed, 0 skipped |
| `feedback/git-verification` | Integrated `21c0b12a4`; Astound credential orchestration `9974e9e8` | 10 HTTPS/process/DB tests passed; full marketplace suite: 326 passed, 0 skipped |
| `feedback/analytics-facts` | Integrated `d9ebb3f95`; durable leased facts, replacement deltas, runtime and scheduler | Full analytics suite passed within a 720-test batch; all ten new DB cases passed, including late commits, fencing, backfill/recovery and shared-cost reference parity |
| `feedback/bridge-receipts` | Integrated `b284948f4`; active readback, scoped outbox and native session recovery; compaction binds server-resolved installation identity before a fresh native session for a new installation | Passed within the 1,154-test integrated marketplace/bridge/evaluator/scheduler batch |
| `feedback/inventory` | Integrated `77c79a311`; canonical catalog, history, explicit bindings and baseline preparation | Full marketplace suite passed: 338 tests, 0 skipped; includes ten new inventory fixtures |
| `feedback/astound-ingestion` | Integrated core `f60b5a8ac`, Astound `6797afe1`; authenticated native hooks, durable normalization/corrections and coordinated expiry | Authenticated hook, distinct consumer/publisher, late binding without duplication, concurrent claims, corrections, retention and restart tests passed in the 18-test cross-system batch |
| `feedback/evaluator-runtime` | Integrated `fcb94767e`; admission, supervisor, pin/cleanup contracts | New admission and probe-cleanup contracts passed in the 720-test batch; current integrated and full native validation pending |
| `feedback/adapter-claude-code` | Integrated `c30ef9b78`; isolated adapter and retained completion evidence | Eight adapter contracts passed in the 720-test batch; latest Linux basic pin/isolation/cleanup probe passed; full native and live gateway acceptance required |
| `feedback/adapter-opencode` | Integrated `96296e01b`; pinned configuration and bounded normalized evidence | Passed within the 1,154-test integrated marketplace/bridge/evaluator/scheduler batch |
| `feedback/adapter-codex` | Integrated `d805d3435`; pinned native adapter and bounded relay | Nine Node relay contracts and full scheduler suite (586 tests, 0 skipped) passed |
| `feedback/adapter-hermes` | Integrated core `f7fe34339`, Astound `a2185589`; pinned native namespace and bounded chat relay | All 15 Codex/Hermes Node relay contracts passed after isolated custom-provider authentication correction; pinned version/native isolation probe passed; full native/gateway acceptance pending |
| `feedback/analytics-snapshots` | Integrated core `9cd069f0b`, Astound `421cde18`; incremental daily/snapshot processing and all-owner atomic retention | Migration 007 applied; checkpoint-holding fixture corrected; full analytics/scheduler/bridge batch passed: 1,452 tests, 0 skipped, including snapshot correction, retention and functional freshness cases |
| `feedback/campaign-completion` | Integrated core `49890ae17`, Astound `179abf83`; independent holdout, retained diagnostics and inventory baseline setup; evaluation 016 and typed API retain atomic budget/suggestion operations | Combined evaluation/marketplace/API batch passed: 1,734 tests, 0 skipped; stateful migration acceptance passed in the 18-test cross-system batch |
| `feedback/dashboard-stream` | Integrated core `bad0abec2`, Astound `8deec1db`; snapshot portfolio, installation coverage and durable SSE generations | Four browser lifecycle tests passed; core installation coverage/SSE checks passed in the 1,734-test batch; raw-reference parity passed; production raw-query implementation removed and retained only as a test reference |
| `feedback/api-contracts` | Integrated `e532a7022`; typed OpenAPI, bounded collections, retained operations and consistent problems; transport/status corrections through `7b6ac501a` | Core operation/status and boundary tests passed in the 3,500-test batch; real HTTP issuance/failure tests passed in the corrected 3,814-test batch |
| `feedback/acceptance` | Supervisor recovery and bounded workspace fixtures; stateful fresh/upgrade parity; authenticated hook freshness; durable gateway failure journal; native fixture harness | Four supervisor recovery tests and the complete 18-test cross-system migration/ingestion/freshness batch passed; full native acceptance pending |

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


Migration ownership: marketplace 004 receipts, 005 dependency verification, 006 inventory, 007 installation coverage, 008 retained API operations; analytics 005 facts, 006 producer barrier, 007 snapshots/retention; evaluation 014 runtime, 015 campaigns; AI 024 durable accounting-failure projection; users 012 device eligibility interface, 013 owner-deletion/retention transaction barrier; Astound 059 authenticated evidence and coordinated retention. Contracts add no DDL. Reporting compatibility adds users 014, analytics 008, events 005, agent 012, AI 025, MCP 006, content 003 and logging 006 owner interfaces, plus Astound 060 raw expiry integration. Additive analytics 009 corrects preinitialization privacy and per-owner cutoff ordering; evaluation 016 retains atomic suggestion operation identities. Latest fresh/upgrade and regression reruns remain pending.

## Observed validation

| Evidence | Observed result | Scope and limits |
| --- | --- | --- |
| Shared workspace build | Passed; 6m 01s | Retained `feedback-work/artifacts/validation/start-workspace-build.log`; compilation does not establish current candidate test results |
| Local startup and health, 2026-09-15 07:39:38 UTC | Two MCP services started in 13.6s; `http://127.0.0.1:8080/health` returned HTTP 200 with `healthy`, version `0.52.0` | `feedback-work/artifacts/validation/local-start-verified.json`; local PostgreSQL after verified historical migration-ledger conversion, no deployment or paid inference |
| Native adapter and live gateway exporters | Both coordinated builds passed | Executables retained in `.target-feedback/debug/examples`; native execution remains pending |
| Selected migration suite | 69 passed; 1.727 seconds; 419 excluded by filter | Historical observed pass before latest analytics 009/evaluation 016; current rerun pending |
| Latest static retry | All 33 source gates plus formatter/cache/version checks passed | `feedback-work/artifacts/validation/preflight-static-current-retry.log`; before latest changes, current gates pending |
| Latest full core batch | 9,349 run: 9,318 passed, 31 failed | `feedback-work/artifacts/validation/core-integrated-complete-batch.log`; before integrated fixture/privacy/suggestion corrections; rerun pending |
| Latest cross-system acceptance | 21 run: 20 passed, one preinitialization privacy failure; 51.878 seconds | `feedback-work/artifacts/validation/astound-acceptance-constructor-retry.log`; before analytics 009 and corrected pending-evidence fixture; rerun pending |
| Latest functional hook freshness | Snapshot visible 497 ms after committed evidence; 714 ms total | One local authenticated hook flow with serial worker drains; not load, throughput or percentile evidence |
| Initial contract/receipt/Git/inventory batches | 7 focused shared-contract tests; 11 receipt DB tests; 10 Git tests; marketplace batches of 326 and later 338, zero skipped | Historical subsystem checkpoints listed above |
| Analytics/scheduler/bridge batch | 1,452 passed, zero skipped; 18.428 seconds test execution | Corrections, privacy retention, distinct identities, histograms and functional freshness; excludes compilation |
| Evaluation/marketplace/API batch | 1,734 passed, zero skipped; 8.794 seconds test execution | Holdout, installation coverage and stream recovery; predates final API additions |
| Seven-package core, `97c4d024b` | 3,500 passed, zero skipped; 19.286 seconds test execution | AI/evaluation/marketplace/API/scheduler/gateway/bridge |
| Corrected eight-package core, `76b5579e4` | 3,814 passed, zero skipped; 17.778 seconds test execution | Includes HTTPS Git, scoped credentials, real HTTP issuance/status and three supervisor terminal regressions; supersedes the earlier 3,811-pass/3-failure attempt |
| Post-merge core, `761c18abb` | 3,821 passed; 28.444 seconds test execution | Retained log `feedback-work/integrate-remote/systemprompt-astound/.build/logs/test-core-feedback-10e123f2c222.log`; historical pass before later grants/startup/retention changes |
| Corrected cross-system, core `76b5579e4` / Astound `90ec7ad2` | All 18 passed; 32.200 seconds test execution | Fresh/upgrade parity, stateful privacy expiry, authenticated ingestion, late attribution, restart and additive raw-reference parity; retained log `feedback-work/analytics-facts/systemprompt-astound/.build/logs/feedback-acceptance-integration-fd91245c8fb1.log`; supersedes the earlier 16-pass/2-failure attempt |
| Functional committed-evidence freshness in that cross-system run | Snapshot visible 221 ms after commit; 292 ms total | One authenticated hook and identical retry, one consumer, one configured owner, serial local worker drains, PostgreSQL 18.6; not throughput, a percentile, or sustained-load evidence |
| Supervisor recovery | Four tests passed | Historical recovery checkpoint; current cleanup changes require rerun |
| Astound static and browser gates | 33 source gates passed; 17 browser lifecycle tests passed | Historical integrated checkpoints; current-tree static/Clippy validation remains pending |
| Combined Node runner contracts | 25 passed; 4.267 seconds | Bounded process-group cleanup after leader exit, relay limits and complete sanitized-environment validation; supersedes earlier 15- and 22-test batches |
| Linux basic native probes | Claude Code 2.1.270, OpenCode 1.18.29, Codex 0.154.0 and Hermes 0.21.3 passed version/isolation/cleanup probes | Exact image and retained reports below; no inference or gateway metering established |
| Historical incomplete nine-package core, `039cf0857` | 3,206 passed; one failure; 1,344 unrun | `repository_assignments::assignment_requires_a_live_owned_worker_lease` fixture teardown reported PostgreSQL deadlock 40P01 while analytics retention ran concurrently. The lock-order fix is implemented; later batch results and current rerun status are recorded above; retained log `feedback-work/artifacts/validation/core-integrated-retry.log`; no current all-pass claim |

The successful cross-system parity permitted Astound `02b2d34c` to remove the production raw query and retain its nullable-identity implementation only as a test reference. Later merged-tree migration parity still requires its own run.

Validation uses PostgreSQL 18.6 and `/usr/bin/docker` 26.0.0 on Linux. Disposable databases `feedback_marketplace_test_20260914` and `feedback_snapshots_test_20260914` support functional tests; their existence alone does not establish schema parity. The current dedicated build directory is `.target-feedback`.

## Native acceptance limits

The historical basic-probe image was `sha256:95740b46ee115a6613cbb68aaa4bbd379eafce226256404609c3714cdc62f342`. That image and Docker build cache are now absent; the coordinated evaluator image rebuild passed with identity `sha256:6a49c8d3dde15a2f9830989e8c491296df745203b0347bf6a20dbb21810c3231`. Its resulting immutable identity requires fresh acceptance, because retained proof applies only to the original image. Retained basic reports are:

| Client | Report |
| --- | --- |
| Claude Code | `.build/native-claude/20260914T225358Z-2659364/report.json` |
| OpenCode | `.build/native-opencode/20260914T225419Z-2661179/report.json` |
| Codex | `.build/native-codex/20260914T225443Z-2662985/report.json` |
| Hermes | `.build/native-hermes/20260914T225503Z-2664368/report.json` |

These are basic version/isolation/cleanup probes only. Complete corrected execution requires all 13 native scenarios for each client and the actual live private-socket gateway fixture exercising capability admission, budget reservation/accounting and cleanup. Native wire replay and fixture-only results cannot enable a target. `LastVerified` readiness is historical evidence, not a current health check or a substitute for approved native proofs.

Earlier partial native execution exposed client-specific failures that led to the OpenCode request/descendant-cleanup fixes, Codex scoped MCP approval and complete sanitized-environment validation, and Hermes auxiliary/MCP corrections. External removal of shared build artifacts interrupted that run during Codex work. Missing exporters and unrun cases are infrastructure interruptions, not passing or failing client acceptance. Those partial results do not satisfy the current 13-scenario matrix.

Windows/macOS native bridge acceptance is unavailable in this Linux environment. Paid inference and deployment are excluded. No load, burst, million-invocation, 100-reader, throughput or p95 result is claimed.

## RustSec and coverage

The 2026-09-15 refresh audited all 11 tracked lockfiles read-only using cargo-audit 0.22.1 and advisory DB `e2e640471715167f73e22eaf761f2e547adafeec`. The findings remain the same as the earlier audit below. Retained refreshed evidence: `feedback-work/artifacts/rustsec/current-next-20260915/summary.json` and per-lock reports. Any subsequent final lock changes still require inspection.

The finding abbreviations below are: **RSA** = rsa 0.9.10, RUSTSEC-2023-0071; **XML** = quick-xml 0.39.4, RUSTSEC-2026-0194 and 0195; **warnings** = unmaintained proc-macro-error 1.0.4 (2024-0370), unmaintained ttf-parser 0.25.1 (2026-0192), and unsound glib 0.18.5 (2024-0429).

| Repository / lockfile | Raw exit | Findings and configured exceptions |
| --- | --- | --- |
| Astound `Cargo.lock` | 1 | RSA; existing exception |
| Astound `bridge/Cargo.lock` | 0 | Warnings; no Astound exceptions for these warnings |
| Astound `tests/Cargo.lock` | 1 | RSA with existing exception; warnings without Astound exceptions |
| Core `Cargo.lock` | 1 | RSA; existing exception |
| Core `bin/bridge/Cargo.lock` | 0 | Warnings; existing core exceptions |
| Core `crates/tests/Cargo.lock` | 1 | RSA, XML and warnings; existing core exceptions |
| Core `crates/tests/bench/Cargo.lock` | 1 | RSA; existing exception |
| Core `crates/tests/fuzz/Cargo.lock` | 1 | RSA; existing exception |
| Core `crates/tests/loadtest/Cargo.lock` | 0 | None; lock inspection did not run load tests |
| Core `crates/tests/mock-inference/Cargo.lock` | 0 | None |
| Core `scripts/rust-contracts/Cargo.lock` | 0 | None |

Exit 1 records retained vulnerability findings, not an audit execution failure; exit 0 can still include warnings. Incoming bridge dependency cleanup removed rkyv 0.7.46 and its previously reported vulnerability. The audit changed no lockfile or exception and does not establish target feature reachability. It is not an all-clean supply-chain claim or final CI result.

[Remote coverage run 34908562717](https://github.com/systempromptio/systemprompt-core/actions/runs/34908562717) measured **150,905 / 170,110 lines (88.710246%)** at `bf0c53209442dd4cc3a43d50d214e81aa4e82a5b`. This is below the 91.5% floor. At the same denominator, at least **4,746 additional existing lines** must become covered; code growth can increase that requirement. The retained artifact identifies real uncovered lines, not an instrumentation failure. New tests do not establish an increased percentage until coverage is rerun. The floor and exclusion policy remain unchanged.

## Remaining delivery gate

The shared workspace build and local startup have passed. Latest static, Clippy, unit, integration, contract, schema-upgrade, offline and coverage reruns still require recorded results, along with all available native acceptance and core CI. Only after those results are reviewed can the final exact core pin, both `next` pushes and safe temporary branch/worktree removal be marked complete. Unrelated concurrent files and SQL metadata remain preserved. The raw-portfolio rewrite awaits user direction; snapshot-backed serving remains the default authorized plan, with no claim of a coherent current working tree.
