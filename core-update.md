# Core update: rip the experiment engine out of `systemprompt-core`

Decision (Ed, 2026-09-16): the product observes, compares and measures skill
versions. It does not optimise or experiment. The narrow waist is the
version-controlled marketplace bundle (`domain/marketplace/managed`). Everything
core carries for running skills against datasets — campaigns, experiments,
executions, budgets, workers, native evaluator targets, relay/gateway metering
for experiments, holdouts, suggestions, approvals — is to be removed, not
disabled.

Astound has already removed its side (Campaigns/Experiments tabs, relay crate,
evaluator deploy/scripts, fixture MCP role, `reviewed_production_failures`).
After that cut astound consumed exactly one thing from `systemprompt-evaluation`:
the `EvaluationExtension` value in test fixtures so its migrations register. It
went when the crate went. (A `prune` residue sweep was drafted the same day and
abandoned unlanded — dropping the tables *is* the cleanup.)

Line counts are from the sibling checkout on 2026-09-16 (`crates/…`).

## 1. Delete outright

| Area | Path | Lines | Notes |
|---|---|---|---|
| Evaluation domain crate | `domain/evaluation/` (all of `src/`, `schema/`, `.sqlx/`, `CHANGELOG.md`) | 7,122 | Keep nothing. `eval_cases` / `SamplerService` / `admin evals promote` (the "golden case" path) are also unused by astound and go with it. See §3 for the one table to consider keeping. |
| Evaluator supervisor | `app/scheduler/src/jobs/evaluation_supervisor.rs`, `app/scheduler/src/services/evaluator/**` (adapters for claude_code / opencode / codex / hermes, supervisor_*, client, container_verification, workspace, README) | 5,437 | Also the `evaluation_supervisor` job registration in `app/scheduler/src/jobs/mod.rs:16,31` and the `Evaluation` variant in `app/scheduler/src/error.rs`. |
| Optimisation orchestrator | `app/runtime/src/optimization/{candidate,capture,diagnostics,holdout,holdout_partition,inventory,iteration,mod}.rs` | ~1,100 | **Keep `git_sources.rs`** (`GitSourceOrchestrator`, managed-only despite the module name; astound's `sync_git` handler calls it). Re-home it under `app/runtime/src/managed/` or `domain/marketplace`, and drop the `OptimizationError::Evaluation` variant. |
| Evaluation API routes | `entry/api/src/routes/evaluation/**` (campaigns, campaign_completion, lifecycle, inventory, snapshots, publications, operations, optimization_resources, consumer, worker handlers) | 3,535 | Mount point `entry/api/src/services/server/routes/evaluation.rs`; the `evaluation_repositories()` read in `services/server/routes/mod.rs:54`. `consumer/*` (device enrollment, receipts, session-bindings, invocations) is the **managed** consumer surface — check each handler; anything writing `managed_*` tables moves to a `routes/managed/` module rather than being deleted. |
| Gateway experiment metering | `entry/api/src/services/gateway/evaluation.rs`; `entry/api/src/repository/evaluation.rs`; the eval branches in `entry/api/src/repository/gateway.rs` (6 refs) | — | Hot path: `services/gateway/service/mod.rs:81-86` (`evaluation_session` → `dispatch_pricing`), `:180-181` (`admit_evaluation` → retry policy), `:227` (`preflight`); `services/gateway/audit/journal/mod.rs:85`. Today every `/v1/messages` with a `session_id` does two point lookups on the empty `eval_session_bindings` (`domain/evaluation/src/repository/experiments/gateway/mod.rs:98-105`). Deleting this removes a DB hop per AI request. |
| Execution-token principal | `entry/api/src/routes/gateway/messages/auth.rs` (5 refs; the `spexec_` bearer branch), `entry/api/src/services/middleware/context/middleware/mcp_flavour.rs` (3 refs; execution capability + `evaluation_fixture` service) | — | `EXECUTION_TOKEN_PREFIX` / `speval_` worker tokens only exist for this. |
| CLI | `entry/cli/src/commands/admin/evals/**` (`promote`, `shared`) and the `Evals` arm in `admin/mod.rs` | — | Deleted with the rest. |
| Composition | `app/runtime/src/builder/composition.rs:96-107` (`EvaluationRepositories::new` + `EvaluationSeams`), `app/runtime/src/context/mod.rs:71` (`evaluation_repositories` field), `context/repositories.rs:76`, `app/runtime/src/error.rs` variant | — | |
| Profile | `shared/models/src/profile/evaluator.rs` (`EvaluatorConfig`), `profile/mod.rs:17,38,155` (`evaluator: Option<EvaluatorConfig>`) | — | `deny_unknown_fields` on the profile: any installed `profile.yaml` carrying an `evaluator:` block must have it removed first (astound's builders already write `evaluator: None`). |
| Identifiers | `shared/identifiers`: `EvalApprovalId EvalBudgetId EvalCampaignId EvalCaseId EvalExecutionId EvalExperimentId EvalHoldoutProposalId EvalReservationId EvalRevisionId EvalSuggestionId EvalWorkerId` | — | `EvalRevisionId` only if `eval_resource_revisions` goes (§3). |
| Feedback / evaluator client models | `shared/models/src/feedback/mod.rs:19-26` (`EvaluatorClient`) and anything in `shared/models` used only by `domain/evaluation` (`ClientKind` → `EvaluatorClient` map, `managed::RevisionBundle` stays) | — | Grep `systemprompt_models::feedback::EvaluatorClient` after the crate is gone; delete what has no remaining user. |
| Marketplace "improvement" gate | `domain/marketplace/src/managed/evaluation.rs` (`admit_improvement`), `schema/managed_evaluation_attestations.sql`, `schema/managed_dependency_verifications.sql` if it is only read there, `PublicationAction::PublishImprovement` (9 non-test refs) | — | A publication is `InitialAdoption`, `Withdraw` or `Rollback`. Astound already dropped the option from its form. |
| `managed_invocation_attributions.traffic_class` CHECK | `domain/marketplace/schema/managed_invocation_attributions.sql:9` | — | Narrow `('production','fixture','live_evaluation','suggestion','judge')` to `('production','fixture')` in a migration; astound's Impact filter already offers only those two. |
| Tests | `tests/unit/domain/evaluation/` (31 files), `tests/unit/app/scheduler/src/evaluator_*.rs` + `evaluation_supervisor_job.rs` (14 files), `tests/unit/app/runtime/src/optimization_{fixture,campaigns,evidence}.rs`, `tests/unit/entry/cli/src/commands/admin_evals_db.rs`, `tests/unit/entry/api/src/routes/evaluation_worker.rs`, `tests/unit/entry/api/src/services/gateway/evaluation_bound.rs`, `tests/common/fixtures/src/evaluation.rs`, `systemprompt-evaluation` deps in `tests/Cargo.toml`, `tests/common/{fixtures,migrate}/Cargo.toml`, `tests/unit/{app/runtime,app/scheduler,entry/api}/Cargo.toml` | — | `tests/common/migrate` references `EvaluationExtension` so eval migrations register in the shared test schema — remove with the crate. |
| Workspace | `systemprompt-evaluation` in the root `Cargo.toml` members and `[workspace.dependencies]`; `Cargo.toml` deps in `app/runtime`, `app/scheduler`, `entry/api`, `entry/cli`; the `systemprompt` facade re-export (`systemprompt/src/lib.rs` `pub use systemprompt_evaluation as evaluation`) | — | |

## 2. Keep — load-bearing for observe / compare / measure / version

- All of `domain/marketplace/src/managed/**` and its ~40 `managed_*` tables:
  sources, snapshots, revisions, bundles, publications, reviews, distribution
  outbox/deliveries, installation receipts, consumer grants / session bindings /
  invocation evidence, inventory, attributions. This is the narrow waist.
- `app/runtime/src/optimization/git_sources.rs` (re-homed).
- `domain/analytics` (feedback facts, snapshots, reporting privacy) — untouched.
- The consumer-facing device/receipt/invocation routes under
  `routes/evaluation/consumer/` **if** they write `managed_*` tables — move, don't delete.

## 3. One decision: `eval_resource_revisions`

Immutable content-addressed revisions of skills/rubrics/cases. Astound's only
FK into it (`reviewed_production_failures`) is gone (astound migration 069).
Inside core it is read by `ManagedRevisionOwnership` (the `managed_revisions`
seam) and by the experiment repositories. If `managed_resource_revisions` (or
whatever `domain/marketplace` stores revisions in) already carries the exact
bytes + digest, drop `eval_resource_revisions` with the crate. If the managed
side still references it for bundle assembly, migrate that read to the managed
table first. Do not keep the table "just in case".

## 4. Ordering (each step lands on core `next` and is proven by astound's build)

1. **No data sweep.** Ed's call: drop with `CASCADE`, no export. Nothing was
   ever admitted to the engine, so every `eval_*` row is engine bookkeeping;
   `DROP TABLE … CASCADE` takes the approved-receipt immutability triggers
   with the tables.
2. Remove the gateway hot-path hooks first (`services/gateway/{service,evaluation,audit}`,
   `repository/gateway.rs`, `messages/auth.rs`, `mcp_flavour.rs`). This is the
   part with a per-request cost and the part most likely to hide a behaviour
   change — land it alone, deploy, watch `infra logs request list`.
3. Delete `app/scheduler` evaluator + job; delete `app/runtime/optimization` except
   `git_sources.rs`; delete `routes/evaluation/**` (relocating any managed
   consumer handlers); delete composition/profile/CLI/identifiers; narrow the
   `traffic_class` CHECK; remove `PublishImprovement` + attestations.
4. Drop the `eval_*` schema: one migration in whichever crate now owns the
   remaining `managed_*` FK (probably `domain/marketplace`) that drops every
   `eval_*` table in dependency order (`eval_approved_operation_receipts` first,
   `eval_managed_workspace_projections` / `eval_resource_revisions` last), drops
   the `reject_eval_operation_receipt_change` and `enforce_eval_owner_scope`
   trigger functions, and drops `eval_fixture_payloads` / `eval_fixture_test_records`.
   Then delete `domain/evaluation` and the facade re-export.
5. Astound follow-up (same day): remove the `eval_session_bindings` trigger
   references (`extensions/web/schema/30_feedback_ingestion.sql:181-182`,
   `31_feedback_projection.sql:68,72`), the `"evaluation"` entry in
   `extension_impl.rs::dependencies()`, `"eval_session_bindings"` in
   `cross_extension_tables()` + its unit test, the `EvaluationExtension`
   reference in `tests/common/src/tempdb.rs:172-173`, the `[patch.crates-io]`
   / workspace-dep lines for `systemprompt-evaluation`. The schema baseline is
   **not** re-recorded: it records the 0.53.0 release schema, which still has
   the `eval_*` tables, and the upgrade test now proves marketplace 011 drops
   them on the way forward. Then `just core-pin`, `just verify`, deploy.

## 5. What this buys

- ~17k lines of core (domain 7.1k, scheduler 5.4k, api 3.5k, runtime 1.1k) plus
  ~45 test files gone.
- One DB round-trip off every gateway request.
- No `spexec_`/`speval_` bearer classes in the gateway auth path.
- `profile.yaml` loses an `evaluator:` block nobody sets.
- 30 `eval_*` tables and two trigger functions out of every fresh install and
  every schema-upgrade fixture.
