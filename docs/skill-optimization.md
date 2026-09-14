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
Campaign transitions require `expected_generation` and reject stale requests.

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
per-skill bills or causal improvements. Marketplace cohorts use current plugin
membership and explicitly disclose that historical membership is not established.

## Remaining acceptance and implementation gaps

The versioned evaluator registry was verified on 2026-09-14 by the Astound static
gate and the complete unit tier (884 tests). A focused registry contract also
passed after the final lint correction. These are software contract results, not
native-client isolation, paid-inference or throughput acceptance results.

- Portfolio queries are on demand, bounded to 100,000 joined evidence rows and a
  366-day window. There is no incremental rollup, refresh subscription, percentile
  store, or throughput acceptance result yet. Larger windows fail explicitly.
- The inventory starts with configured local catalog entries. Imported-only Git
  marketplace resources need catalog reconciliation before claiming complete
  external marketplace coverage. Generalized baseline capture remains separate from
  the existing four-skill benchmark bootstrap.
- Signed manifests carry publication/resource/revision/generation/bundle identity
  for every host, but this is not an installation receipt. Bridge emitters and hook
  ingestion still need end-to-end consumer/device/session receipt attribution.
  Existing owner-equals-consumer receipt joins must be replaced before attributing
  shared organizational publications to other users. Such usage remains unknown.
- The versioned evaluator registry reports Claude Code, OpenCode, Codex, Hermes and
  Claude Desktop. All are installation-capable; automatic evaluation fails closed
  with a typed reason because no adapter has yet passed the required isolation and
  metering contract tests. Current frozen run admission is still Claude-Code-specific;
  all-client execution adapters and their isolation tests remain required.
- Git content verification currently supports public credential-free sources and
  single-resource bundles. Private credential resolution and dependency-source
  verification must be completed before those inputs can receive attestations.
- Fresh holdout confirmation is available through frozen REST campaign runs, not
  yet through a guided dashboard confirmation form. The default 10-case threshold
  intentionally cannot be met by the existing three-holdout-case per-skill demo.
- Automatic iteration errors are logged, not yet retained as campaign-level blocked
  diagnostics. No experiment template means no runnable campaign; no valid retained
  development suggestion means no follow-up candidate. There is no automatic Git
  commit, pull request, publication, or production-triggered campaign creation.
- Existing personal asset adoption, complete OpenAPI coverage, cursor pagination on
  every collection, grants for non-admin reviewers, and high-volume/privacy/retention
  acceptance tests remain work before calling the system production-ready.

No live deployment, paid inference, source push or client installation is implied by
the presence of these routes or dashboard pages.

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
These are implementation guarantees awaiting the coordinated acceptance gates;
throughput, p95 latency and native Windows/macOS acceptance remain unmeasured.

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
