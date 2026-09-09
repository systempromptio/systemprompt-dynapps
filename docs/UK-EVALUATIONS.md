# UK requirements workflow and evaluation implementation

The UK workflow uses three skills: `requirements_context`, `requirements_specify`, and `requirements_publish`. It requires a signed browser decision on the context and a second decision on the specification and Confluence destination. Changes to the approved inputs invalidate the relevant approval. The server validates structure synchronously and verifies the published content through Confluence readback.

The workflow freezes explicitly selected, caller-readable Knowledge Bank sources. Selected Confluence pages are read through the connected Atlassian MCP server. Evidence records preserve both the indexed hash and the live content hash. Missing access, missing version metadata, and oversized evidence stop the operation; the server does not truncate evidence or fall back to the demonstration corpus.

The OpenSpec engine and its bundled upstream checkout have been removed. Existing historical workflow records remain available as read-only downloads. New workflows use `requirements-v2.1`; historical approvals do not authorize new content.

## Evaluation foundation

Core now provides typed immutable input revisions, experiment specifications, queued client/model matrices, fenced worker leases, deterministic evidence-based weighted scoring, and transactional budget reservations. Astound exposes owner-scoped authoring and read APIs beneath the existing admin API router:

- `GET/POST /evals/experiments`
- `GET /evals/experiments/{id}`
- `POST /evals/experiments/{id}/cancel`
- `POST /evals/revisions`
- `GET /evals/revisions/{id}`

Writes require the authenticated admin session and a same-origin browser request. Creating an experiment only queues records; it does not execute a client or call a model. Budget values use integer microdollars. Only a newly admitted reservation permits dispatch; an idempotent duplicate reservation does not permit a second call. Uncertain spend stays reserved, and an overage freezes further admission.

Structural workflow validation explicitly reports semantic quality as `not_scored`. Weighted scoring requires the exact configured dimensions and hard gates, and rejects unknown evidence references. Neither operation replaces a human approval.

## Worker transport and accounting

Core exposes `POST /api/v1/evaluation/worker/{claim,heartbeat,evidence,complete}`. Every call requires an expiring worker bearer credential enrolled for this server environment. Lease mutations check the credential's worker identity, owner, fencing token and expiry. Claiming admits at most two active executions per owner, and heartbeats cannot extend the 30-minute active deadline. Completion does not confer human approval or a quality score.

Admin writes additionally support `POST /evals/workers`, `POST /evals/workers/{id}/revoke`, and `POST /evals/workspaces`. Enrollment returns the credential once with `Cache-Control: no-store`; the database stores its hash. These routes inherit the existing admin authentication and same-origin requirement. Owner-scoped `GET /evals/executions/{id}/evidence` and `/artifacts` return stored manifests and artifact contents.

Evidence submissions include the manifest and UTF-8 artifact bytes. The server checks sizes and SHA-256 hashes, the frozen client/image/bundle variant and the exact set of admitted request IDs. Evidence is immutable. Uploaded bytes are execution evidence, not proof of semantic correctness.

Server-bound evaluation gateway sessions require a pending audit record before admission. Dispatch reserves a conservative text request bound, disables transport retries and uses pinned selected-provider prices for completion. Missing usage retains the reservation. Requests requiring routing selectors or safety scanners are rejected until those auxiliary calls can share the same budget. Ordinary sessions do not acquire evaluation reservations.

The native-client container adapter uses core's existing supervised spawn thread. `deploy/evaluator/Dockerfile` pins the client versions; executions require an image digest and named network. A named Docker network alone does not enforce the required egress allowlist.

## Remaining integration

The full platform in [evals.md](../evals.md) is not complete. Outstanding work includes the evaluator service role and worker polling loop, restricted per-execution gateway/MCP credential provisioning, complete auxiliary-call budget attribution, persisted judgments, compatibility probes, comparison reports, the improvement dashboard, managed skill publication and distribution, reviewed rollback, and production monitoring. Immutable snapshots also need to be linked to the existing logical case and rubric records before exposing unified version authoring.

No paid experiment or deployment is established by this implementation. The initial paid pilot remains limited to the separately authorized $5 total and must wait for end-to-end budget admission and verified accounting. Do not interpret an empty accounting record as a measured zero-cost model run.

## Verification commands

The repository [justfile](../justfile) defines `just requirements-test`, `just evals-check`, `just evals-test`, and `just evals-integration`. Database integration tests use disposable PostgreSQL databases and the existing `SYSTEMPROMPT_TEST_DATABASE_URL` configuration. They exercise admission races, idempotent settlement, owner isolation, immutable inputs, cancellation, stale leases, expiry persistence, wrong-environment and revoked worker credentials, tampered artifacts and incorrect request evidence.
