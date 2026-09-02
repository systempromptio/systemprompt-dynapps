# 2026-09-02 — The OCI target architecture is published as proposed, and kept apart from what runs

**Status:** accepted (Ed, 2026-09-02). Follows
`2026-09-02-self-host-multi-node.md`, which recorded what the deployment
actually does; this one records where it is going.

## Context

The platform team supplied a target architecture for the Oracle Cloud
deployment: DNS and WAF in front of the region, a public subnet with a flexible
load balancer terminating TLS, a private subnet running two container instances
blue/green, a second private subnet with managed PostgreSQL, NAT and service
gateways, Vault, Bastion, and Logging/Monitoring/Notifications, all provisioned
by Terraform through Resource Manager. Its stated principles are immutable
infrastructure, IaC, GitOps, multi-environment, managed services, observability
from day one, and vendor agnosticism.

The deployment that exists is one node against one managed PostgreSQL. Two
risks followed from having both pictures in circulation with neither written
down: a reader taking the target for the runbook, and the gap between them
being assumed to be zero because most of it is.

## Decision

1. **The target architecture is published as documentation, not kept in a
   slide** — `/documentation/deploy-target-architecture`, with the diagram at
   `storage/files/images/architecture/oci-target-architecture.png`.
2. **It is labelled proposed at the top, in the title, in the description, in a
   `proposed` tag, and in a blockquote before any content.** Where it and
   `/documentation/deploy-oracle-cloud` disagree, the deploy page wins and says
   so.
3. **Current and aspirational are bucketed per component in a table**, not
   narrated. Every row is one of: already built, deliberate non-goal, or a
   numbered gap.
4. **The boundary is stated explicitly.** Almost every component in the diagram
   is the platform team's. This software is the container image, the database
   schema, the probes and metrics port, and the outbound provider calls.
   Six named seams are the only integration points.
5. **A line moves from the proposed page to the deploy page only when it has
   been run on a real instance.** That is the sole criterion separating them.

## The four gaps, as recorded

Verified against the core source, not from memory. Each is a seam that exists
with a driver that does not.

1. **Secrets have no Vault provider.** The source resolves to file or
   environment, once, at boot, into memory. A provider is a fetch driver plus —
   if leases are wanted — a renewal path. Satisfied operationally today: the
   secret lives in Vault and the deployment places it on the node.
2. **File storage has no object-storage backend.** One implementation, a local
   directory, behind a vendor-agnostic interface. Multiple replicas need a
   shared mount today.
3. **Generated static content is written to node-local disk.** With two
   replicas, only the node that ran the job has fresh content. Same fix as (2).
4. **Locally spawned processes are not scoped per node.** MCP and agent child
   processes are recorded by raw PID in a shared table; one node can judge
   liveness against, or terminate, another node's PID. This is the same defect
   `2026-09-02-self-host-multi-node.md` recorded as "the MCP service registry is
   per server name, not per instance", and it is why multi-node boots are
   serialised. Fix is the instance-scoping the job scheduler already has.

Minor and open: the database connection maps to `sslmode` require/disable/prefer
with no CA verification — encrypted but unauthenticated. Low severity inside a
private subnet; the kind of thing a security review picks up.

## Explicitly already built (so nobody re-plans them)

Replica-safe job scheduling with instance-scoped claims and node/cluster
policies; session and identity state in PostgreSQL; distinct `/livez` and
`/readyz` plus a separate metrics port; structured JSON logs on stdout needing
no agent; a single outbound-URL validation guard applied at every user- and
agent-supplied URL, complementing the NAT chokepoint.

## Staged path

One node → Vault secrets (gap 1) → object storage (gaps 2 and 3) → blue/green
two replicas (needs gap 4) → WAF and multi-AZ. Incremental because every
component in the diagram is a failure mode somebody has to diagnose; adding one
at a time keeps the cause knowable. The first stage reaches none of the gaps,
which is the argument for starting there.

## Consequences

- The proposed page is public documentation and ships with the release, so the
  target and its gaps are persisted and citable rather than re-litigated.
- Gaps 1–4 are core work, not repo work, and are the standing backlog for
  multi-replica on OCI.
- `/documentation/deploy` and `/documentation/deploy-oracle-cloud` link to the
  proposed page as a related playbook; the proposed page links back and defers
  to them.

## Addendum (same day) — the second projection is published with it

**Decision.** A companion diagram, `/files/images/architecture/value-architecture.svg`,
is published on the same page: the control plane drawn full size above a
compressed data-plane strip of the platform team's resources. It uses their
structural grammar — labelled bands, dashed boundaries, a numbered principles
panel — with the astound product palette rather than Oracle's icon set, so it
reads as a companion sheet rather than either a copy or vendor marketing.

**Why.** A topology diagram measures what is provisioned, so it renders this
software as one container among twenty-odd boxes and, more consequentially,
frames it as an application the platform runs rather than the control plane it
is. That framing follows the diagram into review and procurement. The answer is
not to argue with their diagram — it is correct — but to supply the projection
it does not have.

**How.** `scripts/make-value-architecture.py` emits **both** artefacts from one
model: `docs/architecture/value-architecture.drawio` (uncompressed mxGraph XML,
editable by the platform team in app.diagrams.net) and the published SVG. One
generator because two hand-maintained copies of the same diagram drift, and the
whole point is that the editable file and the published file agree. Edit the
script, never the outputs; it is deterministic and stdlib-only.

**Load-bearing content.** The panel titled "why this cannot live in the layer
below" is the substance and should survive any redesign: WAF inspects HTTP and
cannot see an exfiltration inside a legitimate tool call; IAM authenticates
humans to the cloud and cannot express "this agent, acting as this user, on this
project"; Vault stores a secret and cannot notice one leaving in a model
response; monitoring counts requests and cannot attribute $3.40 to a person, an
agent and a tool call. Each pair looks redundant on a topology diagram and is
not, because the lower layer is blind to agent semantics.
