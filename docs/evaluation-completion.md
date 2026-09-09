# UK evaluation implementation and acceptance

The target is three UK skills with two human approvals: scope, then specification
and Confluence destination. Knowledge Bank retrieval and live Confluence evidence
must be attributable to the frozen execution. The runner is a supervised core
service using the existing agent/MCP process architecture. Astound owns product
configuration and presentation.

## Execution order

1. **Restricted execution boundary.** Issue five-minute opaque credentials bound
   to a worker lease, owner, environment, execution, fence and session. Rotate
   credentials on refresh. Reject expired/revoked workers and mismatched models
   or providers before dispatch. Accept execution credentials only at inference
   and explicitly scoped MCP endpoints, never ordinary administrative routes.
2. **Worker lifecycle.** Add the evaluator role to service registration,
   reconciliation and CLI start/stop/restart. Retrieve complete immutable
   assignments, poll and heartbeat, supervise each native client, collect bounded
   artifacts, and complete using fenced writes. Human approval waits release
   capacity; crashes must not replay uncertain external writes.
3. **Docker execution.** Run the control plane, PostgreSQL, evaluator and restricted
   proxy locally. Only the trusted evaluator receives the Docker socket. Use
   per-execution Docker volumes and archive transfer. Apply time, memory, CPU,
   process, output and disk limits. Clients have no direct external network or
   provider credentials. Preserve existing containers and volumes.
4. **Native clients and MCP.** Probe pinned Claude Code and OpenCode capabilities,
   provision frozen skills/configuration and verify hashes. Route approved
   Knowledge Bank and Confluence operations through existing synchronous
   governance. Bind browser approval to the exact reviewed content and destination.
5. **Accounting.** Charge generation, judging, scanners, selection, repairs and
   retries to the same pilot budget. Freeze prices and request purpose. Reserve
   before dispatch and reconcile actual usage; retain unresolved reservations.
   The authorized paid pilot ceiling is $5 total, not $5 per client or run.
6. **Managed skills.** Introduce immutable database bundles in a shared skills
   domain. Use one resolver for runtime, marketplace, bridge and CLI, preserving
   skill IDs, plugin membership and access rules. Managed/tombstoned skills must
   not fall back to disk. Link existing case/rubric records to revisions.
7. **Improvement and publication.** Provide experiment creation, preflight,
   evidence, approvals, costs, judgments, suggestions, immutable drafts and diffs
   in UI and CLI through shared application services. Human publication uses a
   generation check, outbox and signed manifest. Verify installation in a clean
   client. Rollback creates a reviewed publication and retains history. Escape
   artifacts and exclude holdout contents from improvement inputs.
8. **Monitoring.** Keep scoring-only production monitoring disabled until an
   owner and daily budget exist. Exclude evaluation traffic. Never automatically
   replay, publish or roll back. Benchmark retention is 90 days and raw production
   retention 30 days, subject to stricter existing policy.

## Evidence gates

- `just evals-up` and `just evals-probe` currently start isolated PostgreSQL and
  run offline version probes for both pinned native clients.
- `just evals-test` and `just evals-integration` exercise Rust unit and database
  contracts. They are not live workflow proof.
- Add `evals-e2e` for real service/auth/supervision tests against deterministic
  external protocol fixtures. Label fixture results explicitly.
- Add `evals-faults` for revocation, cancellation, approval tampering, egress,
  crash recovery, uncertain accounting, publication failures and rollback.
- Add `evals-live` only after the free gates pass. Run the complete UK workflow
  once per native client and a second compatible model on read-only cases.
  Obtain genuine browser approvals; do not simulate them as live evidence.
- Retain run IDs, binary/client/configuration hashes, approvals, Confluence
  readback, installed skill hashes, rollback proof, total spend and unresolved
  reservations. Do not claim end-to-end completion without this evidence.

## Current checkpoint

Local Docker connectivity and both offline native-client probes are verified.
Execution credentials, provider admission and job attribution pass the focused
Rust lint check. Database tests cover rotation, expiry, foreign environments,
stale fences, worker revocation and provider mismatches.
Worker supervision, restricted MCP dispatch, shared managed skills, improvement
UI/publication and live end-to-end acceptance remain implementation work.
No paid model run or remote deployment has been performed by this work.
