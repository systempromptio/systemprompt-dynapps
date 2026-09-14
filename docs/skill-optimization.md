# Organizational skill optimization

This change introduces the campaign and portfolio layer over the existing managed
revision/evaluation foundations. Source implementation is not deployment or paid
pilot acceptance. See the limitations below before enabling automatic campaigns.

## Ownership and boundaries

| Responsibility | Home |
| --- | --- |
| Immutable sources, files, revisions, dependencies, publication, Git verification | Core marketplace domain |
| Campaign policy, shared budgets, frozen experiments, paired comparisons, retained suggestions | Core evaluation domain |
| Request/session deduplication and metric definitions | Core analytics domain |
| Binding campaign evidence to evaluated and committed content | Core runtime application service |
| Bounded automatic development iterations | Core runtime service, invoked by the evaluator scheduler |
| Authenticated versioned REST resources and problem responses | Core API |
| Signed client-neutral skill publication identity | Core shared manifest and marketplace projection |
| Astound catalog, hook-event adapter, dashboard templates and forms | Astound web extension |
| Astound benchmark content and provider/client deployment configuration | Astound services/configuration |

Managed resources and campaigns use the configured system owner's organizational
scope, not the consuming administrator's personal scope. Requests retain the actual
actor for campaign changes and publication reviews. Distribution still applies the
existing user's grants. Shared authoring and evaluation pages require administrators;
project-manager access is not widened to organizational evidence. Previously created
personal assets are not silently reassigned or deleted.

## Lifecycle

```text
Git/local authoring -> immutable revision -> baseline + candidate experiment
                                               |
production associations -> reviewed cases       v
                                      development suggestions
                                               |
                                      bounded candidate iterations
                                               |
                                     fresh independent holdout
                                               |
                                    report / manual source change
                                               |
                            server verifies exact Git commit content
                                               |
                                retained evaluation attestation
                                               |
                                  human publication review
                                               |
                               signed publication distribution
                                               |
                      client installation/invocation evidence (see gaps)
```

Campaign creation does not start inference. The dashboard can launch a development
comparison using an existing frozen experiment as the environment/dataset template
and a selected immutable candidate. Only development cases are copied. Explicitly
enabling automatic follow-up authorizes development iterations up to the campaign's
iteration limit and shared budget; it does not authorize publishing or committing.
The dashboard defaults to manual follow-up.

The supervisor uses retained, development-only suggestions with a bounded
`proposed_changes.files` list of complete text replacements. It creates new immutable
revisions and queues an iteration atomically with the experiment. Idempotency keys
prevent duplicate campaign dispatch; worker leases and budget accounting remain the
existing evaluation mechanisms. A cancelled/paused campaign prevents new iterations,
but does not cancel already running experiments; cancel those separately.

Publication eligibility requires completed paired evidence, settled accounting,
quality floors, verified success, no candidate hard-gate failures, and positive
conservative paired confidence bounds for the selected objective. Repetitions of
one case count as one sampling unit. A separate fresh holdout must establish the
same result. Confidence bounds are screening evidence, not a guarantee of production
generalization or a correction for unlimited adaptive testing.

`publish_improvement` now requires a retained attestation for the exact resource,
revision, bundle and experiment. A caller-supplied JSON score is insufficient.
Initial adoption remains explicitly unevaluated; rollback remains a separately
reviewed generation change. Source verification fetches an exact Git commit without
executing its files and compares file bytes and modes with the retained revision.
This supports a local-authored revision committed to a registered Git source without
making the database the authoring authority. Changed content requires reevaluation.

## REST resources

All routes below are under `/api/v1`, protected by the existing core admin auth and
rate limiting. Cookie-authenticated mutations require same-origin requests; bearer
clients do not need a browser Origin. Responses are non-cacheable. Domain failures
use problem JSON: 400 invalid input, 404 unavailable in scope, 409 state/evidence/
budget conflict, 500 sanitized infrastructure failure. Extractor/auth middleware
errors retain the existing core response conventions.

| Resource | Operations |
| --- | --- |
| `/campaigns` | GET cursor page, POST idempotent policy creation |
| `/campaigns/{id}` | GET retained policy and generation |
| `/campaigns/{id}/transitions` | POST action plus expected generation |
| `/campaigns/{id}/experiments` | GET iteration IDs, POST attach same-budget experiment |
| `/campaign-runs` | POST frozen paired spec plus campaign and idempotency key |
| `/campaigns/{id}/experiments/{experiment}/report` | GET measured report and development suggestions |
| `/experiments`, `/experiments/{id}` | GET retained experiments and execution state |
| `/experiments/{id}/cancellation` | POST cancellation |
| `/budgets`, `/budgets/{id}` | POST idempotent cap, GET accounting |
| `/evaluation-revisions`, `/evaluation-revisions/{id}` | POST immutable content, GET content |
| `/sources`, `/sources/{id}` | POST source registration, GET specification |
| `/sources/{id}/captures` | POST selected skill IDs from the configured authoring root only |
| `/revisions/{id}/bundle`, `/revisions/{id}/workspace` | GET immutable bundle, POST verified evaluator projection |
| `/source-verifications` | POST revision, registered Git source, relative root and exact commit |
| `/source-changes` | POST campaign, experiment, evaluated/committed revision IDs and verified source commit |
| `/evaluator-capabilities` | GET versioned client capability and unsupported-reason registry |

Campaign creation/run keys are in JSON, matching the existing evaluation protocol.
Creation returns Location; queued runs return 202 with a resolvable experiment URL.
Campaign transitions require `expected_generation`; identical retained action retries
are acknowledged, while conflicting stale requests are rejected.

## Portfolio semantics

`/admin/analysis/resources` and its `.json` endpoint show configured skills, plugins,
and marketplaces, including unused entries. Counts deduplicate invocations, users,
requests, and assessed conversations. Costs include recorded failed requests.
Missing cost/usage stays unknown, and coverage denominators remain visible.
Legacy default-zero cost fields cannot prove free usage and are treated as unknown
by the Astound adapter until pricing completeness is recorded independently.
Token averages include input, output and separately accounted cache tokens; reasoning
tokens already included in output are not added again. Quality is conversation-
weighted, not request-weighted. These are related conversation costs, not additive
per-skill bills or causal improvements. Marketplace cohorts use retained effective-dated membership; unverifiable historical
membership remains unknown.

## Implemented behavior and remaining acceptance

Portfolio serving uses incrementally refreshed snapshots and durable SSE generations.
Normalized facts, daily aggregates, distinct identities, latency histograms and
custom-range jobs preserve coverage and freshness diagnostics. Inventory includes
configured and managed/imported resources, explicit bindings, reconciliation and
effective-dated membership. Historical membership without evidence remains unknown.

Device-authenticated installation receipts and separate native-session bindings
support immutable ingestion evidence and correctable attribution. Historical hooks
without device proof remain unknown. Private HTTPS Git orchestration resolves each
registered source credential and requires complete dependency verification before
publication attestation. Guided independent holdout and persistent blocked campaign
diagnostics are implemented; benchmark thresholds have not been lowered.

All four CLI adapters use shared admission before reservations. OpenCode's parent
relay enforces the request ceiling independently of advisory native steps, and its
review workspace is read-only with execution tools restricted. Codex preserves
global noninteractive approval and explicitly approves only the allowlisted local
fixture MCP tool. Its credential probe validates the complete bounded isolated
child environment against fixed keys/values and native session identifiers; it
does not claim that reading the sandbox's own `/proc/1/environ` is denied. Generic
sandbox startup failure cannot satisfy boundary evidence. Complete corrected native
and live gateway acceptance remains pending, and automated targets remain disabled.

Typed OpenAPI, bounded cursor collections, durable API operation status and cookie
Origin protections are implemented. Human publication approval remains mandatory.
Personal-asset adoption and additional non-admin reviewer grants remain outside
scope. The corrected 18-test migration/privacy/ingestion/parity batch passed.
Validation after the incoming core merge, coverage, CI and exact
pin/push verification remain tracked in [the completion ledger](skill-optimization-completion.md).
Windows/macOS native bridge acceptance is unavailable here. No load testing,
throughput or p95 measurements, paid inference or deployment is claimed.

### Authenticated ingestion and correction capture

Astound migration 059 replaces the publisher-equals-consumer attribution trigger.
The historical immutable attribution rows remain evidence, but are not used as
proof of device identity. Hook-JWT-only events retain unknown consumer/device
coverage. `/hooks/track` additionally accepts a device credential forwarded by the
bridge from its protected enrollment store: the authenticated device consumer
must equal the hook token consumer. Raw event and separate device context commit
atomically; retries cannot replace a retained device, host or session context.
Active Claude Code and Claude Desktop hook files contain a host marker only.
The bridge strips caller device credentials and supplies its enrolled credential
only to the gateway hook route; secrets are absent from authored hooks and outbox
items. Native transports without this hook path retain unknown device evidence.

Publication attribution requires an unambiguous retained session binding matching
consumer, device, host, session, current grant and canonical resource. A late
receipt/session binding reprocesses the existing invocation key. Context removal,
revocation of a resource grant, source corrections and source deletion queue
replacement facts or tombstones. Submitted installation/revision/device fields in
raw hook metadata are never evidence of successful installation.

The `feedback_ingestion` job runs every five seconds under its explicitly declared
organization administrator. PostgreSQL source triggers maintain independent
invocation, request, assessment and resource-association keys, fenced row revisions,
leases, retry deadlines and redacted diagnostics. They lock the shared ingestion
producer barrier before queue writes. Facts admission, queue acknowledgement and
barrier decrement commit together. Restarted workers reclaim expired leases; a
new source revision invalidates an old lease. A bounded keyset backfill runs with
capture triggers active, so an earlier-key transaction committing later is still
queued. Checkpoints describe completed work, not allocation order.

Requests remain independent across multiple resource associations. Failed spend
is retained. Zero/default costs without pricing provenance are explicitly unknown
pricing; absence of a raw request's device proof remains unknown identity rather
than borrowing another device's identity from a conversation. Assessment records
carry a conversation key independent of the invocation key, preventing a shared
assessment from creating extra conversations. Synthetic and non-production
evaluation traffic is excluded from production facts.

The coordinated expiry function refuses work while producers, normalized changes
or snapshot deltas are pending. After snapshots retain daily totals, expiry removes
raw rows, reviewed-evidence references and identity-bearing side tables in the
same transaction. Expiry alone suppresses deletion tombstones; ordinary privacy
and correction deletions continue to enqueue them with the original event time.
The snapshot compactor owns the retention transaction and its checkpoint locks.
The corrected stateful upgrade, privacy expiry, producer-barrier and attribution
fixtures passed; the incoming core merge still requires revalidation. Throughput,
p95 latency and native Windows/macOS acceptance remain unmeasured.

Retention uses the snapshot service's exact rolling 90-day evidence cutoff. The expiry
transaction first takes the same ordered exclusive table locks as core compaction,
covering producer registration, direct fact admission, backfills and consumer
leases while allowing ordinary reads. Any pending evidence or an owner whose
compaction watermark has not reached the cutoff raises an error and rolls the
whole transaction back. Expiry also removes completed orphan queue entries,
including associations whose request expired before their newer invocation,
so queue keys cannot preserve the removed raw identifiers. The current
`ai_requests` schema has no pricing-provenance flag: a default zero therefore
remains unknown pricing; no provider-response metadata is treated as authoritative
currency or pricing proof.

## Inventory observations and baseline preparation

Core inventory records configured skills, plugins, rules, hooks, artifacts, agents,
and MCP servers alongside managed and imported resources, including unused and
unpublished entries. Disabled, malformed, missing, withdrawn, and conflicting
entries retain explicit diagnostics. Configured identities require an explicit,
immutable resource binding before an existing managed resource is adopted; matching
names alone never establish a binding. Git verification continues to use the
retained registered-source binding and complete dependency manifest.

The configured-owner `managed_inventory_refresh` job observes inventory during
bootstrap and every minute. Each complete observation has a durable generation.
Semantic membership changes retain effective intervals; membership before the first
observation is unknown. Failed scans preserve the prior observation and expose
processing diagnostics. An observed empty inventory is distinguished from an
unobserved inventory. Snapshot consumers use the owner-scoped coverage read rather
than counting catalog files on dashboard requests.

Administrative inventory APIs provide bounded cursor lists, current processing
status, effective-time membership, immutable bindings, reconciliation summaries,
and operation-addressed baseline outcomes. Baseline preparation accepts at most
100 entries per page and retains successful or blocked outcomes under the supplied
operation ID. Configured files are captured twice with bounded bytes and file
counts, including executable modes on Unix; symlinks and changing trees fail
capture. Imported resources reuse retained immutable bundles. Changed authoring
content retains upstream heads and uses the existing three-way reconciliation when
a managed candidate has diverged. Candidates and publication selections remain
unchanged until their existing review workflows approve changes.

Inventory fixtures cover configured-only and imported-only entries, unavailable
sources, naming conflicts and immutable binding retries, source removal with
publication preservation, effective membership and unknown history, non-skill
baseline capture, replayed operation results, retained scan failures, and candidate
reconciliation. These fixtures require the coordinated database validation gate;
implementation alone does not establish native Windows or macOS mode verification,
throughput, or p95 performance.

## Campaign completion and independent confirmation

Campaign reports include the admission result for their exact client versions and
frozen execution variants. Setup and automatic follow-up failures retain safe,
actionable diagnostics independently of the failed request. Diagnostic collections
are owner-scoped and cursor bounded. Unsupported execution remains disabled before
budget reservation; fixture availability does not enable a native target.

The campaign dashboard prepares inventory baselines in bounded, retryable pages,
including imported resources and retained unavailable/conflicting outcomes. Guided
holdout preparation retains a proposal containing the frozen matrix and digest,
requires completed qualifying development measurements and unchanged minimum
counts, and preserves baseline/candidate environment parity. Human confirmation
must name that digest. Dispatch still applies current admission and budget checks.
A transactional content-consumption guard rejects relabelled or previously exposed
holdout inputs, and identical confirmation retries reuse the retained experiment.
Publication remains a separate human approval.

The authored four-skill benchmark now contains eighty cases: ten development and
ten independent holdout cases per skill. The eight-case pilot is a smoke fixture,
not sufficient evidence for the ten-pair campaign threshold. Contract tests cover
case partition counts/content separation, diagnostic persistence, proposal retries,
digest confirmation, actual admission visibility and relabelled holdout rejection.
These tests await the orchestrator's integrated gates; no paid inference or native
acceptance is claimed here. Throughput and p95 targets remain unmeasured.

## Durable analytics snapshots and retention

The snapshot consumer processes leased, fenced batches of committed fact deltas.
A transactional shadow projection reconstructs only affected UTC days; daily
requests and resource associations deduplicate shared request keys. Portfolio spend
counts each request once, while related resource spend is explicitly non-additive.
Failed requests retain known spend, and pricing, latency, token, invocation
attribution, and assessment coverage retain independent denominators. Conversation
assessments use the latest retained assessment per conversation, with deterministic
ties. Corrections, deletion tombstones, and late associations replace prior daily
contributions and invalidate affected custom-range results.

Workers refresh changed scopes for 1/7/30/90/365-day snapshots, plus all existing
scopes at UTC-day rollover. Dashboard snapshot endpoints read retained results.
Custom UTC-date ranges up to365 days use bounded asynchronous jobs with idempotent
operation IDs, recoverable leases, and resolvable status. Exact distinct users and
sessions are counted across the whole retained range, never by adding daily counts.
Version1 geometric microsecond histograms merge bucket counts and expose percentile
upper bounds; they never average daily percentiles. PostgreSQL notifications only
announce durable generation changes and act as wake-up hints.

The configured-owner snapshot job runs every five seconds. Nightly raw expiry and
aggregate compaction share one transaction. Retention first takes consistently
ordered PostgreSQL table/row locks that block new producer registrations and direct
fact admissions, then requires all producer evidence, privacy corrections, fact
changes, backfills, and snapshot-consumer checkpoints to be drained. The common
cutoff is exactly90 days before the trusted invocation time; future or regressing
compaction clocks are rejected. Applied changes, deltas, normalized evidence,
projection identities, backfill cursors, raw source rows, and indirect reviewed
references are erased at expiry. No identifying contribution ledger survives.

Only non-identifying daily totals remain through365 days. Older daily cohorts with
fewer than five authenticated users are suppressed. The partially expired boundary
UTC day is also suppressed because its full contribution can no longer be rebuilt.
Older user/session distinct metrics are unavailable. A correction to sealed history
suppresses the affected day; when no retained provenance can bound that correction,
all sealed history is suppressed. Cached snapshots refresh and custom jobs requeue,
with suppression coverage exposed explicitly. This provides safe historical
correction behavior rather than claiming exact subtraction after identity erasure.

Snapshot fixtures cover raw-reference parity, shared-request duplication, failed
spend and unknown pricing, conversation assessment replacement, exact cross-day
identities, histogram merging, late association corrections and tombstones,
concurrent workers, expired leases, restart recovery, idempotent custom jobs,
retention barriers, producer-registration races, sealed-history suppression, and
midday cutoff/clock guards. The functional freshness fixture reports one committed
request through direct local workers. These fixtures do not measure deployment
freshness, throughput, p95 latency, burst behavior, or large-reader concurrency.

## Snapshot portfolio and live updates

The resource portfolio reads bounded canonical inventory pages joined to retained
usage and installation coverage snapshots. It includes unused, unpublished,
unavailable, withdrawn, conflicting, configured-only and imported entries.
Unbound inventory entries show usage unavailable. Unknown invocation attribution
remains a separate portfolio denominator. Historical marketplace membership is not
inferred from current plugin lists.

Installation coverage is refreshed by the configured-owner snapshot job. Its
expected denominator is currently granted, actively enrolled devices. A current
acknowledgment matches the selected publication generation; verified devices have
at least one installation with successful native readback. Historical authenticated
acknowledgments and unverifiable installations remain distinct. Legacy receipts
never manufacture consumer/device identities. The server cannot count disconnected
bridge outboxes; those installations remain unobservable until acknowledged.

The page shows snapshot/fact watermarks, pending work, inventory and installation
observation generations, unknown usage, histogram measurement coverage, and
historical suppression. Arbitrary retained UTC-day ranges enqueue deterministic,
idempotent bounded jobs and display preparation status until results are ready.

Authenticated `/api/v1/analytics/live` streams reuse core broadcaster connection
limits, keep-alives and connection guards. Composite durable cursors contain the
snapshot, inventory and installation generations. Reconnects and generation gaps
instruct clients to reload retained snapshots. PostgreSQL notifications are wake-up
hints; periodic generation reads recover lost notifications. A single listener is
shared across active streams and exits after the last connection closes. Browser
teardown closes streams, cancels reconnect/poll timers, aborts outstanding requests
and removes listeners. Hidden pages release transport; superseded responses cannot
replace current results.

Dashboard fixtures compare additive legacy source metrics with retained snapshots
and explicitly verify the corrected authenticated-identity boundary, bounded unused
inventory pages, receipt/readback coverage, core SSE caps and guard release, durable
resync, browser reconnects, stale-response rejection, authorization expiry, and
teardown. Additive parity passed and the legacy raw query now lives only in the
test suite. A single authenticated hook with an identical retry reached a retained
snapshot 221 ms after commit (292 ms total), using one consumer, one configured
owner and serial local worker drains on PostgreSQL 18.6. This is a small functional
freshness check. No throughput, p95 performance, burst, million-event or 100-reader
result is claimed.

### Served API contracts and recoverable administration

`GET /api/v1/openapi.json` serves an authenticated OpenAPI 3.1 document generated
from the handler DTOs. It covers device enrollment handoff, installation receipts,
separate session bindings and invocation status; inventory, baseline preparation,
membership and reconciliation history; exact evaluator capabilities; complete
source and dependency verification; snapshot ranges, asynchronous jobs and SSE;
and campaign setup, transitions, dispatch, reports, guided holdout confirmation,
diagnostics, source acceptance and human publication review.

Consumer routes authenticate the enrolled-device credential independently of
administrative JWT/session authorization. Submitted identity fields and a user
bridge secret cannot establish device identity. Cookie mutations require the
configured exact Origin. Extraction failures, invalid bounds and domain failures
share `application/problem+json`; missing analytics jobs return 404. JSON bodies
are limited to 1 MiB, nested inputs are bounded, and collections expose bounded
cursor traversal, including experiment executions and campaign experiments.

Inventory refresh, authoring capture, source verification and device credential
issuance require `Idempotency-Key`. `GET /api/v1/operations/{id}` resolves their
owner-scoped state and typed result. Conflicting key reuse is rejected; an active
lease returns 202. Fenced retries retain captured inputs. Inventory generation and
operation completion commit together; authoring recovery reuses its deterministic
immutable snapshot and revisions. Verification recovery reuses retained complete
attestation evidence and rejects different dependency provenance. Campaign
transition retries acknowledge the same retained generation/action.

Credential issuance stores only the credential digest and operation metadata.
The plaintext token is delivered once. A repeated request reports the retained
status without rotating the token or reconstructing it; lost delivery requires a
deliberate new operation key to rotate. Session-binding and invocation status
resources remain restricted to the authenticated consumer/device (and host for
invocations). Publication remains an explicit human-reviewed mutation.

API fixtures cover stale lease fencing, response-loss recovery, conflicting
requests, credential token secrecy, actual HTTP extraction and origin boundaries,
forged device credentials, missing-job status, cursor bounds, and OpenAPI reference
integrity. Their observed integrated results are recorded by the completion ledger;
these contract additions do not establish throughput or p95 performance results.
