# Skills, evaluations, and the production improvement loop

Status: architecture and operator guide. Updated 13 September 2026.
This describes the current `next` checkout, including integration work that has not
yet passed an accepted paid end-to-end pilot. It is not a production-readiness claim.

## Executive summary

Astound currently has two related but operationally distinct planes:

1. **Production observation** records which installed skills users invoke and
   associates those invocations with requests, cost, latency, and outcomes where the
   available identity and accounting evidence permits it.
2. **Controlled evaluation** runs exact baseline and candidate skill revisions over
   frozen cases, rubrics, client/model configurations, and budgets in an isolated
   environment.

They can and should form one closed improvement loop. The join must be made through
verified identifiers and reviewed, sanitized evidence. Live customer traffic must
not be routed directly into the evaluator, and raw production transcripts must not
become test cases automatically.

```text
Production invocation
  -> verify the installed skill revision
  -> analyze failures, quality, cost, latency, and coverage
  -> human review and sanitization
  -> immutable development case revision
  -> current revision as baseline + independently authored candidate
  -> controlled experiment on frozen inputs
  -> evidence, scoring, hard gates, and human review
  -> reviewed publication generation
  -> signed distribution through Bridge
  -> installation receipt from the client
  -> subsequent production invocations attributed to the new revision
  -> post-release impact analysis
```

The repository contains foundations for every part of this chain. The production
failure-to-development-case action is implemented. The complete one-screen workflow
from that case through experiment launch, publication, rollout, and measured adoption
is not yet accepted end to end.

## The three systems that must not be confused

### Skill delivery and invocation

A skill is a package of instructions and supporting files. Skills are authored under
`services/skills/<skill_id>/`, selected into a plugin or assigned to a server-side
agent, packaged into a signed bundle, and installed on a user's machine by Bridge.
A signature proves the bundle's integrity and origin; it does not grant tool, MCP, or
model permissions.

For an installed client skill, traffic is selected locally:

- the user explicitly invokes a slash command such as `/plugin:skill`;
- the model chooses the client's `Skill` tool; or
- a server-side agent has the skill assigned and its instructions are injected into
  the agent prompt.

There is no central semantic router that examines every request and chooses one of
several skill revisions. Gateway model routing is a separate concern and does not
route between skills. In the normal client path, a user has one installed production
bundle generation at a time, so subsequent eligible invocations use that installed
content until it is updated.

### Production analysis

Production analysis consumes authenticated client events and gateway records after
or alongside normal work. It is observational: it tells us what was used and what
happened around that use. It does not rerun the task under controlled conditions.

The `skill_invocation_events` view normalizes two client signals:

- **Slash invocation:** extracted from the retained prompt preview because a slash
  command may never emit a tool call.
- **Skill tool invocation:** extracted from `PostToolUse` or `PostToolUseFailure` and
  accepted only when a nearby governance decision proves that the tool call passed
  through the governed client path.

A slash and tool signal for the same session and skill within five seconds are
deduplicated. This avoids counting a single run twice. Measuring only Skill tool calls
would under-report ordinary slash-command usage.

The analysis joins skill events to related AI requests only under the documented
owner and session/tool correlation rules. Request counts and spend deduplicate request
IDs. If several skills occur in the same conversation, their displayed related cost
can overlap and must not be summed as independent skill cost. Missing linkage or
accounting is reported as unknown, not zero.

### Controlled evaluation

Controlled evaluation is an intentionally separate execution environment. It freezes
the inputs needed to make two variants comparable, expands a case/variant/repetition
matrix, executes it using isolated workers, retains evidence, and scores the results.
Evaluation execution, judging, repair, and suggestion traffic use explicit traffic
classes so they do not contaminate production cohorts or recursively enter monitoring.

This separation is desirable. The evaluator needs deterministic fixtures, bounded
permissions, frozen costs, isolated credentials, retries, fenced worker leases, and
evidence retention. A production client session provides none of those guarantees by
itself.

## How the two planes join

The join has four distinct parts. A complete product workflow needs all four.

### 1. Join a production invocation to its exact revision

An invocation may claim an installation ID, publication generation, and resource
revision. Those client-supplied values are not trusted by themselves. Attribution is
marked `verified` only when authenticated ingestion ownership and a stored installation
receipt agree on all of the following:

- owner;
- native client session;
- installation ID;
- publication generation;
- managed skill resource and resource key; and
- immutable revision ID.

Otherwise the event remains `revision_unknown`. Unknown attribution must stay visible
in cohort coverage and must never be silently assigned to the latest revision.

The resulting identifier chain begins as:

```text
invocation_id
  -> installation_id
  -> resource_id
  -> revision_id
  -> publication_generation
```

`/admin/analysis/impact` groups these revision-attributed events and exposes explicit
counts for invocations, attributed events, related requests, accounted requests,
failures, tokens, related conversation cost, and latency. A before/after association
on this page is useful monitoring evidence, but it is not a causal A/B result.

### 2. Turn a reviewed production problem into a development case

An administrator can capture an owned production invocation from the version-impact
surface. The server:

1. verifies administrator access and same-origin form submission;
2. verifies that the invocation belongs to the authenticated owner and is production
   traffic;
3. accepts a bounded prompt, expected-behavior lines, and assertions;
4. performs basic secret/token redaction and rejects invalid or excessive input;
5. validates the content as an evaluation case in the `development` partition;
6. creates or reuses an immutable `eval_resource_revisions` case revision; and
7. records the reviewed link in `reviewed_production_failures`.

That creates this bridge:

```text
production invocation_id
  -> reviewer and sanitized evidence record
  -> development_case_revision_id
```

This is a reviewed capture, not an automatic transcript export. The reviewer is still
responsible for removing personal data, secrets, customer-specific facts, and any
content the evaluation environment is not permitted to retain. The current sanitizer
is a bounded safety layer, not a general-purpose PII classification guarantee.

Production-derived examples belong in the development partition. Once an example has
influenced a candidate skill, it is no longer an untouched holdout. If a holdout case
is exposed during diagnosis or authoring, consume it and replace it with a new unseen
case before claiming held-out performance.

### 3. Compare baseline and candidate under controlled conditions

The deployed revision should be captured as the baseline. A candidate is a new
immutable child revision; editing a candidate must never mutate the baseline. An
experiment then freezes:

- exact case and optional dataset revisions;
- the rubric revision, weighted dimensions, pass policy, and hard gates;
- exact baseline and candidate bundle digests;
- client name/version and worker image digest;
- provider, requested model, model parameters, and provider prices;
- tool configuration, permissions, fixture state, clock, and timezone where relevant;
- repetitions, execution mode, objective, and limits; and
- a shared budget account and maximum experiment cost.

The planned matrix is:

```text
case revisions x variants x repetitions = planned executions
```

Preflight must validate the complete matrix and reserve a defensible maximum cost. It
must not silently remove difficult cases to fit a budget. Workers claim owner-scoped
executions using environment-scoped credentials and fenced leases. Evidence and
completion events from stale leases are rejected. Execution success, evaluation
success, and complete accounting are separate states.

Every scored dimension must cite stored evidence. Missing or invalid judgments remain
unscored, and a failed hard gate cannot be hidden by a strong weighted average. Human
review precedes adoption.

The managed experiment records and worker primitives exist, but the checked-in Super
Admin benchmark has not completed an accepted paid end-to-end pilot. Queuing an
experiment proves only that a record was queued; it does not prove that a worker ran,
the candidate improved, costs settled, or publication is safe.

### 4. Publish, install, and measure the candidate

A successful experiment does not activate a skill automatically. Adoption is a
separate reviewed lifecycle:

```text
human approval
  -> publication generation
  -> signed distribution
  -> client download
  -> client installation and activation
  -> verified installation receipt
```

Publication generations are immutable historical decisions. Withdrawal and rollback
must not rewrite history; rollback publishes a new reviewed generation pointing at the
chosen prior revision. A signed manifest proves delivery authenticity but is not an
installation receipt. The receipt is what allows later invocations to be attributed
to the revision actually reported as installed.

The managed resolver is not yet proven as the universal source for every gateway,
marketplace, CLI, agent, Bridge, and evaluator consumer. Until that integration and a
clean-client acceptance run are complete, do not claim that creating a managed
publication has activated the revision everywhere.

The complete cross-plane chain is therefore:

```text
invocation_id
  -> verified installed revision_id
  -> reviewed development_case_revision_id
  -> experiment_id and frozen variant digests
  -> approved candidate revision_id
  -> publication_generation
  -> installation receipts
  -> later production invocation_ids
```

## What works now and what remains separate

| Capability | Current state | Meaning |
|---|---|---|
| Skill discovery, signed bundle projection, and client installation | Runnable | Eligible clients can receive configured skills |
| Slash/tool invocation normalization | Runnable with authenticated events | Observed skill use, with five-second duplicate suppression |
| Revision attribution using installation receipts | Implemented integration slice | Attribution is verified only when all identities agree; otherwise unknown |
| Version-impact cohorts | Implemented analysis slice | Observational quality/cost/latency association with visible coverage |
| Reviewed production failure to development case | Implemented | A human-curated immutable case is linked to the source invocation |
| Recorded-traffic LLM judge and repair loop | Runnable and paid | Assessment of retained gateway requests, not a skill A/B test |
| Immutable baselines, candidates, experiment specs, budgets, leases, and evidence APIs | Substantial foundations | Inspectable components, not accepted end-to-end proof |
| Super Admin 40-case benchmark and eight-case $5 pilot specification | Authored, not run | Test input and acceptance target only |
| Case-to-experiment launch from the impact page | Missing integrated workflow | The captured case must still be assembled into a suite/experiment |
| Universal managed-revision activation and clean-client verification | Incomplete | Publication cannot yet be described as universally active |
| Randomized live production A/B allocation | Not implemented | Users are not split between skill revisions by this system |

## How traffic routes today

There are three independent decisions:

| Decision | Mechanism | Where it happens |
|---|---|---|
| Which skill is used | User slash command, model Skill-tool choice, or assigned agent skills | Primarily on the client or in server-agent configuration |
| Which model/provider serves a request | Ordered gateway routing configuration | Server gateway |
| Which revision a user has | Published bundle selected and installed through Bridge | Distribution/client installation |

These decisions are not a live experiment allocator. There is currently no server
component that hashes a user into baseline/candidate cohorts and remotely selects a
different client skill revision for each invocation. The installed production skill
is therefore the one normally used by that client, while controlled experiment
traffic runs separately under evaluation identities and traffic classes.

## What a revision means

A revision is an immutable content snapshot, not an in-place edit and not a release.
The related identifiers intentionally prove different facts:

| Identifier | What it proves |
|---|---|
| Source snapshot | Which authoring source was imported |
| Resource revision | Exact immutable skill, case, or rubric content and provenance |
| Bundle digest | Exact assembled bytes and dependency closure |
| Experiment ID | Which frozen comparison matrix was requested |
| Publication generation | Which reviewed revision selection was published at that point in history |
| Installation receipt | Which publication and bundle a particular client reports installing |

A normal skill change should be:

1. Capture the current installed/deployed revision as the baseline.
2. Create a candidate child revision containing the proposed change.
3. Compare the exact files and provenance.
4. Run baseline and candidate over the same development cases and untouched holdouts.
5. Review evidence, failures, cost, latency, and incomplete accounting.
6. Approve and publish only the selected immutable candidate.
7. Verify installation receipts and new-session behavior.
8. Monitor subsequent production cohorts separately by revision.

Merely editing a disk `SKILL.md`, creating a candidate, or publishing a generation
does not by itself prove improvement.

## Running the evaluation paths

### Recorded gateway traffic: runnable today

Use a local or disposable instance with PostgreSQL initialized, a configured provider,
and completed gateway requests whose prompts and responses were retained. These
commands can make paid judge and replay calls.

Inspect the interface and enabled skills:

```bash
systemprompt admin evals --help
systemprompt core skills list --enabled
systemprompt core skills show admin_daily_brief
```

Start with a small sample and conservative judge budget:

```bash
systemprompt admin evals run \
  --sample-size 5 \
  --window-hours 1 \
  --budget-microdollars 100000
```

For a selected cohort:

```bash
systemprompt admin evals run \
  --sample-size 10 \
  --window-hours 24 \
  --provider anthropic \
  --model <served-model-id> \
  --judge-provider anthropic \
  --judge-model <judge-model-id> \
  --conversations \
  --budget-microdollars 500000
```

`--provider` and `--model` filter the production traffic under assessment;
`--judge-provider` and `--judge-model` select the grader. A context can be isolated
with `--context-id`, and a persisted named rubric with `--rubric`.

Inspect stored results:

```bash
systemprompt admin evals list --limit 20
systemprompt admin evals show <run-id>
```

Promote a reviewed request into the legacy golden case store:

```bash
systemprompt admin evals promote <ai-request-id> \
  --name "Daily brief with incomplete status data" \
  --expectation "Reports unknown coverage and does not default the project to green" \
  --tags daily-brief,coverage
```

This CLI promotion snapshots the canonical prompt and source request. It does not add
the case automatically to the controlled Super Admin JSON suite or launch a managed
experiment. The Analysis impact capture described above creates a managed development
case revision and is the newer bridge toward controlled experiments.

Replay unrepaired failures from a prior recorded-traffic run:

```bash
systemprompt admin evals replay <run-id> \
  --budget-microdollars 100000
```

The recorded-traffic budget is a loop stop based on recorded judge spend, not an
atomic pre-dispatch reservation. A judge call may cross the threshold, and repair
generation is outside that judge counter. Replay can also repeat external side
effects. It does not have the controlled runner's fixture isolation, approval binding,
or uncertain-write reconciliation, so use it only on traffic safe to replay.

### Controlled experiment infrastructure: preparation and inspection

Start the isolated evaluation database and probe the pinned clients:

```bash
just evals-up
just evals-probe
```

The probe checks that pinned native clients can start under container restrictions.
It has no provider credentials, network, host mounts, Docker socket, inference, MCP
execution, judging, or publication. It is not an end-to-end evaluation.

Run component suites when changing the implementation:

```bash
just evals-test
just evals-integration
```

On the application, an administrator can inspect:

- `/admin/analysis/impact` for revision-attributed cohorts and reviewed failure
  capture;
- `/admin/analysis/versions` for baseline capture, immutable revisions, candidates,
  and file comparison;
- `/admin/analysis/evaluations` for experiment records, execution states, evidence,
  and accounting;
- `/admin/analysis/publications` for review, publication generations, distribution,
  and installation receipts.

Stop the isolated environment when finished:

```bash
just evals-down
```

There is not yet a documented, accepted command that executes the full eight-case paid
pilot from clean setup through native-client execution, scoring, publication,
installation, rollback, and post-release measurement. `just evals-e2e`, fault, and
live acceptance remain release-boundary work. Do not substitute component tests,
screenshots, queued rows, or client probes for that evidence.

## Recommended integrated operator workflow

The next product increment should turn the existing pieces into one auditable workflow:

1. **Investigate.** Open a failed or suspicious invocation in version impact.
2. **Verify provenance.** Require verified revision attribution or explicitly record
   that the production revision is unknown.
3. **Review and sanitize.** Show retained evidence, permissions, and retention policy;
   require a human to write safe test input, expected behavior, and assertions.
4. **Create development case.** Persist the immutable case revision and the source
   invocation link. This server-side operation already exists.
5. **Build experiment.** Offer the currently installed revision as baseline, select a
   candidate, development cases, untouched holdouts, rubric, repetitions, clients,
   models, tools, and budget.
6. **Preflight.** Display the entire frozen matrix, compatibility failures, maximum
   spend, permissions, fixture/live boundary, and missing provenance.
7. **Run and review.** Show every attempt, hard failure, unscored result, evidence
   reference, settled and reserved cost, latency, and cost per verified success.
8. **Publish deliberately.** Require human approval and an optimistic generation
   check; never auto-publish from an LLM score.
9. **Verify adoption.** Track manifest delivery separately from installed/activated
   receipts and clean-client smoke evidence.
10. **Measure.** Compare later verified revision cohorts while retaining unknown
    attribution and accounting coverage. Label these results observational.

No automatic candidate generation should receive untouched holdout contents. No raw
production transcript should be copied without consent and review. No evaluation
identity should gain production administrator or provider credentials. No experiment
should directly authorize publication.

## Would live A/B testing improve this?

Yes, but it is a later capability with a different risk profile. Controlled offline
experiments estimate the effect under repeatable conditions; post-release cohorts show
real-world association. Neither provides randomized production causality.

A genuine live A/B mechanism would require:

- a server-controlled assignment service using stable, owner-scoped user or device
  cohorts;
- two simultaneously publishable, signed generations;
- Bridge support for receiving and activating the assigned revision;
- sticky assignment for a session and explicit reassignment rules;
- exclusion of administrators, evaluator sessions, and ineligible clients;
- exposure logging before outcome measurement;
- minimum sample, duration, guardrail, and stopping policies;
- immediate kill switch and reviewed rollback;
- privacy, consent, regional, and customer-contract controls; and
- analysis that handles missing receipts, noncompliance, multiple skill exposure,
  interference, and delayed outcomes.

Randomizing each invocation would be unsafe and analytically weak because skill state
and conversation context persist on the user's machine. A canary rollout should start
with a small, sticky cohort of installations, not per-prompt randomization. Until such
a system exists, describe releases as controlled evaluation followed by monitored
rollout, not production A/B testing.

## Is this a world-class evaluation framework?

The design has many world-class ingredients: immutable provenance, exact bundle
digests, case and rubric revisions, hard gates, evidence-backed scoring, isolated
workers, fenced leases, explicit traffic classes, shared budget accounting, reviewed
publication, installation receipts, and post-release revision cohorts.

The system as a delivered product is not yet world class because the defining proof is
operational, not architectural. It still needs:

- a successful paid end-to-end pilot using real supported clients;
- complete shared accounting for generation, tools, judges, repair, retries, and
  unknown usage;
- failure-injection proof for cancellation, worker loss, lease expiry, uncertain
  writes, cleanup, and recovery;
- universal managed-revision resolution across all consumers;
- clean installation, activation, withdrawal, and reviewed rollback evidence;
- outcome definitions and larger statistically useful holdout sets;
- evaluator calibration against human reviewers and disagreement reporting;
- drift monitoring, retention enforcement, privacy review, and reproducibility checks;
- an integrated production-finding-to-release operator experience; and
- optional safe canary/randomized assignment if causal production measurement is a
  business requirement.

The correct current description is: **a strong governance-oriented evaluation
architecture with runnable recorded-traffic evaluation and substantial controlled
experiment foundations, but without accepted end-to-end controlled-experiment or live
A/B evidence.**

## Evidence required before claiming an improvement

Retain and report:

- source snapshot, resource revision, bundle and dependency digests;
- case, dataset, rubric, policy, configuration, client image, model, tool, permission,
  fixture, and price revisions;
- experiment, variant, execution, attempt, lease, provider request, and evidence IDs;
- every planned and attempted execution, including hard failures and unscored results;
- settled cost, outstanding reservations, unknown usage, latency, tokens, and cleanup;
- human review, publication generation, distribution event, installation receipt, and
  rollback history; and
- cohort size, attribution coverage, accounting coverage, observation window, and
  whether the result is controlled, randomized, or merely observational.

Cost per successful task is total attempted spend divided by verified successful
tasks. If nothing succeeds, it is undefined, not zero. A small pilot can prove the
pipeline and reveal defects; it cannot establish broad statistical superiority.

## Source map

- Public maintainer guide: `services/content/documentation/skills-and-evaluations.md`
- Analysis measurement contract: `docs/analysis/implementation.md`
- Invocation normalization: `extensions/web/schema/migrations/040_skill_invocation_view.sql`
- Verified version attribution and reviewed failure link:
  `extensions/web/schema/migrations/056_skill_version_impact.sql`
- Failure capture handler:
  `extensions/web/admin/src/handlers/ssr/analysis/impact.rs`
- Controlled benchmark input: `services/evaluations/super-admin/`
