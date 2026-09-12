# Evaluation status and remaining work

The active Super Admin deliverable is specified in [evals.md](../evals.md).
The requirements-workflow plan below remains separate historical planning context;
it does not expand the four-skill Super Admin benchmark or authorize Atlassian writes.

The automated evaluation workflow is **incomplete**. There is no supervised
evaluator that runs the complete requirements workflow. Component tests and
client version probes do not establish end-to-end readiness or deployment safety.

## What exists

- The requirements workflow uses `requirements_context`, `requirements_specify`
  and `requirements_publish`, with browser approvals for scope and for content
  and destination. See [the workflow reference](UK-EVALUATIONS.md).
- Evaluation foundations include case/rubric snapshots, experiment queues,
  fenced leases, short-lived execution credentials, partial gateway accounting,
  evidence storage and deterministic scoring.
- Worker assignment retrieval verifies ownership, live leases and the hashes of
  case, rubric, skill and configuration snapshots.
- Ordered progress events accept exact retries and reject conflicting duplicates,
  sequence gaps and invalid leases. Progress events are not approval receipts or
  verified recovery checkpoints.
- Client metadata validation checks version bounds and image digests.
- Docker infrastructure probes start PostgreSQL and check pinned client versions.

The latest worker-protocol checkpoint passed nine unit tests, five integration
tests, focused Rust linting and 33 static gates. These include existing tests;
they are not full-workflow acceptance results. No paid workflow run or deployment
was performed as part of that checkpoint.

## Remaining implementation

Complete these stages in order. Core owns execution, credentials, accounting and
revision lifecycle. This repository owns configuration, presentation and acceptance.

| Stage | Remaining work | Completion evidence |
|---|---|---|
| 1. Worker lifecycle | Add an evaluator service type, configuration, registry, reconciliation, health checks and CLI start/stop/restart. Run workers through the same binary and shared subprocess supervisor. Register capabilities, poll every five seconds, heartbeat every twenty seconds against the sixty-second lease, and refresh credentials before five-minute expiry. | A real supervised worker claims, executes and finishes assignments through authenticated APIs. |
| 2. Frozen execution and recovery | Freeze prices, permitted MCP operations and execution limits alongside existing snapshots. Enforce two active executions per owner and thirty minutes of accumulated active time. Approval waits release capacity and expire after twenty-four hours. Persist verified checkpoints or supported native sessions. Reconcile uncertain external writes before continuing; never automatically replay them. | Approval, cancellation, lease-loss and crash tests preserve attribution, capacity and write safety. |
| 3. Docker isolation | Add the actual control plane, trusted evaluator and restricted proxy to the isolated Compose project. Only the evaluator receives the Docker socket. Transfer inputs through Docker archives and dedicated volumes. Isolate each execution from peers, databases, administrative routes and arbitrary internet destinations. | Egress and cross-execution fault tests; confirmed container removal on cancellation, timeout and crash. |
| 4. Native clients and MCP | Complete pinned Claude Code and OpenCode adapters: capability probes, configuration installation, hash verification, structured output and supported resumption. Provide execution-only governed MCP. Intersect frozen permissions with the owner's current permissions and workflow phase on every call. Keep approval authority in human interfaces and preserve Confluence readback. | Both native clients finish deterministic fixture workflows using real application authentication and approval gates. |
| 5. Complete accounting | Inject shared billing admission into gateway and internal AI services. Charge generation, judging, selectors, scanners, repairs and retries to one parent pilot account. Reserve before dispatch using frozen prices and bounded usage. Keep unknown usage reserved; report settled cost, reservations and completeness separately. Enforce the $5 total pilot ceiling and reject an unaffordable matrix. | Concurrent admission, duplicate settlement, missing-usage and auxiliary-call tests prove the shared ceiling. |
| 6. Managed skills | Create immutable bundles and one resolver for agents, runtime, marketplace, bridge and CLI. Import only the three workflow skills, preserving IDs, membership and access rules. Managed or tombstoned skills cannot fall back to disk. Link existing case/rubric identities to revisions. | Consistent installed hashes and explicit missing/tombstoned outcomes across consumers. |
| 7. Improvement and publication | Share application services between dashboard and CLI for experiments, baseline/candidate comparisons, judgments, drafts, diffs and JSON/Markdown reports. Escape artifacts and enforce owner scope. Exclude holdout contents from improvement prompts and distinguish deterministic validation from semantic judging. Publish through human approval, generation checks and a transactional outbox producing a signed manifest. Track distribution and clean-client installation separately. Rollback is a new reviewed publication. | Publication conflicts, distribution retries, clean installation and reviewed rollback retain auditable receipts without duplicate publications. |
| 8. Monitoring and retention | Replace replaying production evaluation with scoring-only monitoring; exclude evaluation traffic and require an owner and daily budget before enabling it. Never automatically replay or publish. Retain benchmark evidence for ninety days and raw production evidence for thirty days, subject to stricter policy. | Monitoring and retention acceptance tests; monitoring remains disabled until configured and verified. |

Client isolation must enforce non-root users, read-only root filesystems, dropped
capabilities, one CPU, 2 GiB memory, 128 processes, a 256 MiB writable workspace,
16 MiB combined exported artifacts/logs, and bounded temporary storage and caches.
Native clients must receive no worker, administrator, database or provider credentials.

## Acceptance commands

| Command | Current state | Required proof |
|---|---|---|
| `just evals-up` | Starts PostgreSQL only | Healthy local control plane, database, evaluator and proxy. |
| `just evals-probe` | Offline version probes | Client availability only; not workflow acceptance. |
| `just evals-test` / `just evals-integration` | Component suites exist | Preserve component coverage as the workflow is completed. |
| `just evals-e2e` | Not implemented | Both real native clients complete the workflow against deterministic external protocol fixtures. |
| `just evals-faults` | Not implemented | Isolation, approval, recovery, accounting and publication failures behave correctly. |
| `just evals-live` | Not implemented | Genuine Knowledge Bank and Confluence workflow, browser approvals and shared-budget enforcement. |
| `just evals-down` | Stops existing Compose services and retains the database volume | Also preserve retained evidence when the complete stack exists. |

Fault acceptance must include missing, contradictory, stale and inaccessible
sources; source prompt injection and unsupported citations; forged or changed
approvals; foreign owners and expired credentials; model/provider substitution;
prohibited MCP calls and direct egress; worker death, lease loss, approval timeout,
streaming cancellation and uncertain Confluence writes; concurrent reservations,
duplicate settlement, missing usage and auxiliary charging; and artifact tampering,
publication conflicts, failed distribution, clean installation and rollback.

The live pilot must run the full workflow once per native client, plus a second
compatible model on read-only cases. Freeze model IDs and prices during preflight.
Stop for missing access or genuine human approval; fixtures cannot substitute for
live evidence.

Completion requires retained run IDs, source/bundle/binary hashes, approval
receipts, Confluence readback, installed revision hashes, rollback proof, quality
results, spend and unresolved reservations. No remote deployment is required for
this local acceptance objective.

## Use and deployment boundary

Keep evaluator workers unenrolled and `evaluation_loop` disabled while this work
is unfinished. The current scheduled implementation can replay failures; it must
not be mistaken for the proposed scoring-only monitor. Worker APIs remain mounted
and evaluation checks share gateway code, so inactivity is not a complete feature
switch. Validate migrations and ordinary gateway, authentication and workflow
behavior through the normal deployment gates before promoting a candidate.
