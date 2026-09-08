# Specs contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
Specs are the behaviour canon of the pipeline: the formal, testable
contract of WHAT the system does after this change. They are written
as DELTAS against the baseline capability specs — one file per
affected capability at `openspec/changes/<name>/specs/<capability>/
spec.md` — and they are the ONLY artifact that survives into the
`openspec/specs/` archive when the change completes. Everything
downstream (FSD/ISD, stories, tasks, tests) derives from them. The
primary audience is LLM planners and executors; the contract must
still read cleanly for a human QA. Specs own the exhaustive edge and
error-case canon that the proposal's seeds only hint at.

AUTHORITY ORDER:
1. The APPROVED grooming decisions and the reconciled upstream
   artifacts — discovery.md (verbatim requirements, codes,
   capabilities), design.md (behaviour-shaping constraints),
   proposal.md (Requirements Lock and Delivery Breakdown) — and the
   source evidence they cite are the factual source of truth.
2. This `specs` instruction is the artifact contract: follow it for
   inventory, coverage, delta discipline, and the quality bar.
3. The `spec.md` template defines the ADDED / MODIFIED / REMOVED
   block shapes.
4. Project `config.rules.specs` and the workflow rules provide
   supporting context; they do not override this artifact contract.

DECISION KERNEL:
- Formalize, never decide: every requirement and scenario traces to a
  signed requirement, an approved slice seed, or an approved grooming
  decision. If a behaviour cannot be stated precisely from those, STOP
  and return the ambiguity to grooming — never encode a guess.
- Approved decisions are law: encode each as behaviour in its covering
  requirement, and a rejected option never appears (APPROVED-DECISION
  BINDING).
- Behaviour only: state observable outcomes in RFC 2119 language; HOW the
  behaviour is built is design's and technical-design's (REQUIREMENT
  RULES).
- One subject, one requirement: a WBS's criteria divide by the subject that
  owes them, and each requirement's NAME is the group label its scenarios
  carry into the FSD/ISD table — so the partition is a reading structure the
  client sees, not bookkeeping (REQUIREMENT PARTITION).
- Delta is the unit: write only what this change ADDs, MODIFIes, or
  REMOVEs against the baseline canon (DELTA RULES).
- Coverage is total: every in-scope lock requirement, every Delivery
  Breakdown slice, and every Applicable cross-cutting area is realized;
  deferred, declined, and out-of-scope items produce nothing.

PIPELINE (follow each referenced section; do not improvise content):
1. Clear the ENTRY GATE: verify the Grooming Approval Gate passed
   (INPUTS AND SOURCE POLICY).
2. Enumerate the capability files from discovery's Affected
   Capabilities table into the closed FILE INVENTORY / FILE OUT plan
   commitment (FILE INVENTORY RULES).
3. For MODIFY / REMOVE capabilities, load the baseline spec from
   `openspec/specs/<capability>/spec.md` (DELTA RULES).
4. Formalize the approved offer by COMPOSING two committed layers, never
   the seed alone: (i) TRANSCRIBE each vendor-free acceptance-criteria seed
   as the behaviour skeleton (the named SEED SKELETON commitment) — the
   seed's phrasing is the base requirement and scenario language; (ii) LAYER
   onto that skeleton every approved bound (the named DECISION LEDGER, one
   line per grooming Q#/CR#/P#) as an added observable clause + scenario, and
   every Applicable cross-cutting area (step 5). A requirement that
   stops at the bare seed and omits a covering ledger bound or an Applicable
   cross-cutting is INCOMPLETE, a silent drop. What design and discovery may
   NOT add is a NEW behaviour clause of their own, a vendor / SDK / endpoint
   noun, or the richer consumer-UI wording a signed discovery Requirement
   or its Requirement Details carries; a
   clause traceable to none of the seed, a ledger bound, or a cross-cutting
   area is anchor drift, dropped. Commit each requirement's and scenario's
   seat in the plan via the OBLIGATION-SEAT TEST before drafting, so an
   integration capability specs only its own interface obligation and never
   the consumer's UI behaviour, and commit how each WBS's criteria divide
   into requirements via the REQUIREMENT PARTITION block ledger (SEED
   SKELETON, APPROVED-DECISION BINDING; REQUIREMENT RULES, SCENARIO RULES).
5. Add behaviour for every cross-cutting area marked Applicable in
   discovery (REQUIREMENT RULES).
6. Carry the WBS in each ADDED/MODIFIED requirement heading (Option-H)
   and attach the Trace / Source metadata comment (TRACE METADATA RULES).
7. Run the SELF-CHECK before emitting.

ENTRY GATE (blocking — evaluate BEFORE authoring anything):
- discovery.md, design.md, proposal.md, and grooming.md must exist as
  completed upstream artifacts for this change, and grooming's
  Grooming Approval Gate must have PASSED: every P#/Q#/CR#/O# item
  APPROVED or DEFERRED-APPROVED by a named human and every gate box
  checked. If the gate has not passed, STOP — do not generate specs
  from agent-resolved or unapproved decisions.

INPUTS AND SOURCE POLICY:
- proposal.md is the behaviour-authoritative parent: the Delivery Breakdown
  acceptance-criteria SEEDS are the vendor-free behaviour skeleton this
  contract FORMALIZES (expand each seed into a full requirement + scenarios),
  the Requirements Lock is the coverage set with obligation-boundary notes,
  and Out of Scope & Boundaries bounds what may NOT appear. The consumed
  vendors/services named in its Depends on / External Dependencies & Access
  are the seat's OUT-set — recorded there, never named in a functional
  capability's behaviour text (OBLIGATION-SEAT TEST).
- grooming.md: the approved Decision cells are the law this contract
  encodes; prerequisite decisions carry the constraints under which
  behaviour ships.
- discovery.md: the Requirements Trace supplies verbatim requirement
  text and RQ/WBS codes; Affected Capabilities drives the file
  inventory; Cross-cutting Requirements drives cross-cutting
  behaviour; its source anchors are the citation vocabulary.
- design.md is consulted for HOW only: cite it for a specific approved
  MECHANISM bound a requirement must carry (a lazy-loaded payment form becomes
  "checkout is usable before the payment form loads"; a privacy constraint becomes a
  SHALL NOT; a redirect type). Its vendor / SDK / endpoint prose is design's
  implementation detail — never imported as behaviour, and the consumed
  vendors it names are part of the OUT-set above.
- Baseline specs `openspec/specs/<capability>/spec.md`: the current
  behaviour MODIFY and REMOVE deltas are written against.
- Do NOT read or anticipate downstream artifacts (FSD/ISD, stories,
  tasks); they derive from specs, never the reverse.
- Cite with the same anchors the upstream artifacts use; an approved
  grooming decision is citable as `grooming Q#`. Never invent codes,
  sources, or capabilities.

FILE INVENTORY RULES:
- One spec file per row of discovery's Affected Capabilities table:
  `specs/<capability>/spec.md`. The row's disposition determines the
  file's delta section: ADD -> `## ADDED Requirements`; MODIFY ->
  `## MODIFIED Requirements`; REMOVE -> `## REMOVED Requirements`.
- A MODIFY file changes only requirements its baseline already
  carries. A requirement with NO baseline counterpart is ADDED
  behaviour and belongs to the capability that owns it going forward:
  behaviour of a new surface — including how it is reached, such as a
  retired URL's redirect landing on it — lands with the NEW
  capability, not as an ADDED section inside a MODIFY file.
- Never create a file for a capability another change owns; consumed
  dependencies get no spec here.
- CLOSED FILE SET — commit the file inventory in the PLAN before drafting,
  exactly as a closed-membership set (like the section ROWS/OUT commitment):
  `FILE INVENTORY = [<one specs/<capability>/spec.md per Affected
  Capabilities row>]` and `FILE OUT = [<capability that appears in CONTEXT
  — a baseline spec, a consumed dependency, another change's surface — but
  is NOT an Affected Capabilities row -> its owning change>]`. A baseline
  spec in CONTEXT is supplied ONLY to decide ADD/MODIFY/REMOVE for a
  capability you already own; its presence NEVER licenses a file. The emit
  is a VERBATIM application of FILE INVENTORY: as the last step before
  output, diff the emitted `--- specs/<cap>/spec.md` markers against it — a
  file with no inventory line (or one named on FILE OUT) is a placement
  error; delete it, never author or annotate it "for completeness".

REQUIREMENT RULES:
- Heading = Option-H `### Requirement: <WBS> — <behaviour name>` — the WBS
  code lives IN the heading (the single FSD join key; ` — ` is an em-dash
  with surrounding spaces). The archive matches MODIFIED/REMOVED headings
  verbatim, so an ADDED requirement always carries its WBS here and a
  MODIFIED/REMOVED heading reproduces the baseline heading exactly (a
  purely legacy requirement that never had a WBS keeps its
  behaviour-name-only heading). Several requirements MAY share one WBS
  (aggregated into one FSD block downstream) — reuse the same WBS across
  ADDED/MODIFIED requirements that belong to the same block; do NOT force
  one requirement per WBS. The RQ IDs stay in the metadata comment.
- BEHAVIOUR ONLY: requirement text uses RFC 2119 keywords (SHALL /
  SHALL NOT / MUST / SHOULD / MAY) and states observable behaviour — what
  a shopper, consumer, or test can see. It never names a file path,
  component, route literal, loader, hook, helper, endpoint, payload
  field, SDK, library, product, internal step, sequencing, or agent
  commentary; those are design and technical-design.
- IMPLEMENTATION SCAN — OpenSpec's own contract excludes from a spec:
  internal class/function names, library or framework choices,
  step-by-step implementation, and execution plans (those live in design
  / technical-design). As the last step before output, scan every
  requirement and scenario line and DROP it, or rewrite it to the
  observable bound, on any hit: (a) a library / SDK / product / framework
  name (e.g. a vendor's SDK / client library, "using the <Vendor> SDK"); (b) an
  endpoint, URL, HTTP verb, status code, or data-format token (`GET …`,
  `200`, `429`, "returns a JSON array"); (c) internal-step narration
  ("forwards the request to <vendor>", "initializes the SDK", "returns a
  payload containing …"). Keep a line that states an observable bound;
  drop one that narrates a mechanism a consumer cannot observe. (The
  own-vendor-at-interface vs consumed-vendor-name boundary and the
  approved-MECHANISM carve-out are defined once in OBLIGATION-SEAT TEST —
  apply that boundary here, do not restate it.)
- REQUIREMENT PARTITION — the committed SUBJECT (below) is the partition
  axis: one WBS yields one requirement PER DISTINCT SUBJECT its covering
  criteria address, and downstream each requirement's NAME becomes the
  group label of its scenarios in the projected FSD/ISD table. Commit the
  partition in the PLAN before drafting, as a closed-membership set:
  `BLOCK LEDGER = [ <WBS> -> <requirement name> | SUBJECT = <actor> :=
  <the criteria and bounds it carries> ; … one line per requirement … ]`;
  every covering criterion sits on exactly ONE line, and the draft is a
  verbatim application of that ledger.
  (a) A criterion whose SUBJECT is the ADMINISTERING business user
  (authoring, configuring, enabling, curating the feature) never shares a
  requirement with an observer-outcome criterion: it is its own
  requirement, named `Management`, emitted LAST for that WBS and never
  split further (the projection closes the block's table with that group
  whatever the file order, so the reader always meets it last). A shopper-outcome criterion whose scenario merely
  PRESUPPOSES an administered record (an admin GIVEN) is not one of these
  — the axis is the SUBJECT of the requirement's SHALL, never a scenario
  precondition.
  (b) An Applicable cross-cutting guarantee that constrains this WBS
  (privacy, accessibility, localization) MAY stand as its own requirement,
  named for the guarantee it holds; it is then a seat of its own and, like
  Management, is exempt from (c)'s two-scenario bar.
  (c) The remaining observer-seat requirement splits further ONLY when the
  same trigger yields a DIFFERENT observable outcome in a distinct context
  (a viewport class, a channel, an authenticated state) AND each resulting
  requirement carries >=2 scenarios; name each for the outcome it holds,
  never for the context alone. Otherwise the context sits INSIDE a
  scenario and the seat stays ONE requirement — a proliferation of
  one-scenario groups is the defect this bars.
  (d) Requirement names are UNIQUE within a WBS (they are the downstream
  group labels and part of the archive merge key) and each requirement's
  text carries only the clauses its own criteria state.
- OBLIGATION-SEAT TEST — commit each requirement's seat in the PLAN before
  drafting, not at emit time: extend its placement-ledger line with
  `seat = THIS capability's | SUBJECT = <the grammatical actor of its SHALL>
  | consumed deps = <named upstream, abstracted here>`, inheriting the
  boundary from the lock Notes and Out of Scope & Boundaries. The draft is a
  verbatim application of that commitment; a clause that fails its seat is
  dropped from this spec, not rephrased. Commit by capability type:
  (a) Integration capability: SUBJECT is "the capability" — never the
  vendor-bearing capability NAME as the actor of every SHALL ("The capability
  SHALL authorize…", not "The Acme payments integration SHALL authorize…"). It
  owes only what the consumer receives AT the interface (an authorization
  result, a ready-to-use entrypoint, a clear error). A clause describing what the
  consumer's UI does with that result — presenting, ordering, synchronizing
  surfaces, layout, filtering, paginating, accessibility, empty-state, or
  localized copy — fails its seat; it belongs to the consumer named in Out
  of Scope & Boundaries. The capability's OWN vendor may be named only as the
  service at the interface — WHO supplies the obligation (the owned key, what
  that vendor can resolve); its SDK, library, or product as the execution
  mechanism is design's HOW, not behaviour.
  (b) Functional capability: SUBJECT is the shopper's observable outcome. In
  the plan, copy the consumed vendors/services from the proposal's Depends on
  / External Dependencies & Access into a closed OUT-set, and pair each with
  the observable phrase its covering seed already uses (seed "address
  suggestions appear" -> the requirement and scenarios say that, never the
  supplier's name). The draft uses the seed's PHRASE in both requirement text
  AND every scenario line; a final scan finds no OUT-set name anywhere.
  Attributing the supplier ("via <vendor>", "suggestions from the <vendor>
  service", "provided by the <vendor> capability") is a leak, not a citation —
  the dependency is recorded upstream, not here.
- GROUNDING: mint no route, path, number, or magnitude. A `/…` path
  appears only when a cited source OR an upstream artifact states that
  exact path — including a user-facing URL an upstream design fixed as an
  information-architecture decision (the new surface's path, a navigation
  or redirect target); inherit such a path VERBATIM. A threshold, amount,
  count, time, or limit appears only when a source or approved decision
  states it. A surface whose path NO source or upstream artifact states is
  named ("the <page name> page"), never slugged — in requirement text,
  footer/nav lines, and the Migration note. Neither a sourced legacy path
  nor an upstream-fixed path licenses minting a DIFFERENT slug the upstream
  does not carry.
- OPERATIONAL GROUNDING: an operational property — a cache or TTL, a
  retry / back-off / retry-after hint, a quota, rate-limit, or timeout
  number — appears ONLY when an approved decision or a cited source states
  that exact property. A vendor / design doc merely noting a value is
  "cacheable", or documenting a rate limit, is HOW reference, not a
  decision to impose that behaviour; absent an approved decision the
  property is fabrication and is omitted, not invented as a SHALL.
- SEED SKELETON — commit the behaviour skeleton in the PLAN before
  drafting, exactly like the closed-membership ROWS/OUT commitment, not a
  loose mention: `SEED SKELETON = [ <slice S#> -> <its acceptance-criteria
  seed phrase, verbatim> -> covering requirement(s) ; … one line per
  slice … ]`. The proposal's seeds are the human-approved, vendor-free
  behaviour; the draft TRANSCRIBES each seed phrase into its requirement
  text and scenarios AS THE SKELETON, then COMPLETES it by layering every
  covering DECISION LEDGER bound (APPROVED-DECISION BINDING) and every
  Applicable cross-cutting area as added observable clauses + scenarios —
  the seed alone is an incomplete contract, and a requirement that omits a
  covering ledger bound or cross-cutting behaviour is a silent drop. design
  and discovery add NOTHING BEYOND those committed bounds: no NEW behaviour
  clause of their own, no consumed-vendor / SDK / endpoint noun
  (OBLIGATION-SEAT TEST), and never the richer
  wording a signed discovery Requirement or Requirement Details carries
  (trace wording that describes the consumer's UI — a synchronized list, a
  highlighted selection — is that
  consumer's behaviour, not this capability's, even when the seed's slice
  traces to the same RQ). A clause that traces to none of a SEED SKELETON
  phrase, a DECISION LEDGER bound, or an Applicable cross-cutting area is
  anchor drift; drop it before output. (`Source:` citation provenance is a
  separate concern from this behaviour-import rule — see TRACE METADATA
  RULES.)
- APPROVED-DECISION BINDING — commit the decision set in the PLAN before
  drafting, not at emit time (this is the pass most often skipped, because
  a base requirement reads complete without its bound). Straight from
  grooming, enumerate EVERY APPROVED or DEFERRED-APPROVED decision (each
  Q#/CR#/P#) and commit an explicit, NAMED plan block — exactly like the
  closed-membership ROWS/OUT commitment, not a loose mention in the
  extracted-items prose: `DECISION LEDGER = [ <Q#/CR#/P#> -> <decided
  bound, verbatim> -> covering requirement <heading or RQ> -> carries: text
  clause + scenario ; … one line per decision … ]`. Grooming's Decision
  cell already names the slice / requirement its bound constrains — read
  the covering requirement from there; do NOT wait for a seed to point at
  it (a decision with no seed marker binds the same way). The draft is a
  VERBATIM application of these ledger lines: each ledgered decision emits
  its bound as an observable clause in the covering requirement's TEXT AND
  a scenario that exercises it, and the seed's open phrasing is REPLACED by
  the decided bound, never formalized as "undecided". A decision left off
  the ledger, or stated without its bound, is a silent drop; a rejected
  option that surfaces is a leak. As the last step before output, diff
  every ledger line against the draft and fix any decision missing its
  clause + scenario. Bind by type:
  (a) CONFIGURATION (administrator- or merchant-tunable): the base
  requirement reads complete without it, so it is the bound most often
  dropped — land it as a trailing SHALL-configurable clause on the
  requirement AND a dedicated scenario where the configured value governs
  the outcome.
  (b) SCOPING (a v1 limit, a declined CR): an explicit
  inclusion/exclusion statement.
  (c) MECHANISM or QUALITY (a permanent redirect, a server proxy, a
  domain-restricted key): a SHALL stating the decided property and its
  observable guarantee.
  A DEFERRED-APPROVED prerequisite does not thin the contract — spec the
  full approved bound against the stable interface. A decided bound
  replaces upstream pre-decision phrasing.
- Group by behaviour, not slice arithmetic: a slice may yield several
  requirements (per REQUIREMENT PARTITION) and two slices may share one.
  Coverage is judged on the lock and the slices, never on requirement
  arithmetic.

SCENARIO RULES:
- Shape `#### Scenario: <name>` (exactly 4 hashtags) with
  bold **GIVEN** / **WHEN** / **THEN** (/ **AND**) lines. Each line is
  testable as written: one observable condition or outcome per line, a
  concrete subject, no subjective wording ("fast", "user-friendly"), and
  nothing a tester cannot verify from outside (see also Deterministic and
  observable below).
- The whole heading text after `Scenario:` is the NAME, and nothing in it is
  interpreted: no leading bracketed label, no grouping tag, no re-usability
  marker. Downstream, the FSD/ISD renders the scenarios of all requirements
  sharing a WBS as ONE table in which each requirement contributes a NAMED
  group of rows, so grouping is expressed by the requirement a scenario sits
  under (REQUIREMENT PARTITION), never by its name.
- PROJECT-ALL / specs_from_ac COUPLING: author the COMPLETE grounded
  scenario set — the FSD projects ALL of them (no downstream curation), so
  every near-final proposal acceptance-criteria criterion is covered by
  >=1 scenario here. A criterion with no covering scenario is a silent
  drop; a scenario covering no criterion (and entailed by no seed/source/
  decision) is fabrication.
- Deterministic and observable: a tester executes each line without
  interpretation. No "works correctly", no invented magnitude, no
  implementation assertion.
- SCENARIO-SET COMPLETENESS: every ADDED and MODIFIED requirement carries
  its full scenario set, and every scenario is grounded — it exists only
  because a seed, source line, or approved decision names or directly
  entails it. Walk the requirement's seeds and approved bounds and write
  one scenario per case they raise: the main flow; each named edge (an
  empty result, a cleared input, a sourced boundary, a
  configuration-governed value); each named error or denied case (a
  permission or absent dependency the source names). Two defects are
  equally wrong: a lone happy path when the seeds raise more (the common
  under-specification), and a plausible case no seed raises (a generic
  service error, data-unavailable, or fallback is fabrication). If the
  seeds raise no edge or error case, carry only the flows they name.
- Observer's seat: every scenario INHERITS its requirement's plan-committed
  seat and SUBJECT and passes the IMPLEMENTATION SCAN line-by-line
  (OBLIGATION-SEAT TEST) — GIVEN a request or state, WHEN the capability
  serves it, THEN what the observer receives at the boundary. No
  GIVEN/WHEN/THEN/AND line narrates an internal step (a proxy hop, an SDK
  init/load, an own endpoint, a payload): the WHEN names the observable
  trigger ("WHEN the capability processes the request"), and an
  integration THEN asserts only the resolved value, ready entrypoint, or
  clear error the consumer receives — never a downstream artifact the
  consumer builds (a store list, a rendered or synchronized surface). A
  scenario asserts an approved mechanism's observable guarantee, never the
  mechanic.

DELTA RULES:
- MODIFIED: copy the FULL requirement block from the baseline spec —
  heading VERBATIM incl. its WBS (the merge key) — then edit the text and
  scenarios to the new behaviour and attach the Trace/Source comment. A
  diff note or a heading the baseline does not carry breaks the archive
  merge.
- REMOVED: name the baseline requirement heading verbatim and record
  `**Reason:**` (why it is retired and what supersedes it) and
  `**Migration:**` (where the behaviour or data went — the redirect,
  the re-pointed entry, the replacing capability). No trace metadata
  or scenarios on REMOVED blocks.
- ADDED: only requirements with no baseline counterpart.
- RENAMED: when a change only changes a baseline requirement's NAME (the
  same requirement, same WBS, a new name — the WBS is the immutable join
  key and NEVER changes on a rename), record it under
  `## RENAMED Requirements` as a FROM/TO pair — a `- FROM: ` line
  wrapping the baseline heading VERBATIM in backticks
  (`### Requirement: <WBS> — <old name>`) then a `- TO: ` line with the new
  heading (`### Requirement: <WBS> — <new name>`), the same WBS on both
  sides. Never model a rename as a REMOVED + ADDED
  pair: that drops the archive's rename linkage and can trip its
  uniqueness / cross-section checks. A rename pair carries no
  Trace/Source/Scenario. If the behaviour ALSO changes, keep the rename
  pair AND add a MODIFIED block whose heading is the NEW name with the
  full updated block. This section is orthogonal to the capability's
  ADD/MODIFY/REMOVE disposition — it lives in the file of the capability
  that owns the renamed requirement; omit it entirely when nothing is
  renamed.

BASELINE & DELTA TIMELESSNESS:
- Baseline (`specs/`) is timeless: it states current behaviour only — no
  change names, no "Phase N", no changelog / migration narrative, no
  forward-reference to `changes/` paths (they break on archive).
- ADDED / MODIFIED requirement bodies and scenarios merge into the baseline
  VERBATIM on archive, so they inherit that timelessness: describe the END
  STATE ("the meter is placed in the right column…"), never the act of
  change ("Phase 2 adds…", "now moves…"). The `## ADDED` / `## MODIFIED`
  header already encodes the change-nature — do not restate it in prose.
- "Phase N" is tolerable only in proposal / design / grooming / tasks
  (discarded on archive), never in a requirement body or scenario.
- Whole-block restatement is required, not redundant: a MODIFIED block
  restates the unchanged scenarios it keeps — but never adds a
  "no regression" / "behaviour unchanged" scenario (that is a tasks.md QA
  gate, not a permanent spec scenario).

REVERSE-ENGINEERED BASELINE vs AUTHORED DELTA:
On a brownfield project the storefront is adopted from a vendor baseline
(e.g. the Storefront Next template), so much of `specs/` is
reverse-engineered from inherited code, not authored intent. Do NOT
bulk-rewrite it — grow specs one change at a time.
- Reverse-engineered baseline: characterises inherited behaviour; the CODE
  is the source of truth; impl symbols (`useX`, `FooModal`, `getBar`) are
  TOLERATED as a traceability anchor into unfamiliar vendor code; mark
  provenance in the capability's baseline `## Purpose`
  (`> Provenance: reverse-engineered from the storefront baseline; code is
  source of truth.`). Never bulk-convert it to declarative purity.
- Authored change delta: DRIVES new code; the SPEC is the source of truth;
  impl symbols are FORBIDDEN — declarative behaviour only (`THEN the item
  quantity updates`, not `THEN updateItemInBasket is called`). Clean the
  slice you touch to declarative form; leave the rest.

CAPABILITY DECOMPOSITION — which capability owns a requirement (apply
top-down, stop at the first match):
- Level 0 (primary axis: surface/page): group by surface first (PDP, cart,
  mini-cart, checkout, account…); behaviour belonging to exactly one
  surface lives under that surface's capabilities by default.
- Level 1 (lift ABOVE a single surface) only when it has its OWN external
  integration / data source (→ own capability) or is REUSED across
  surfaces (→ own shared capability; each surface keeps a thin "placement"
  requirement cross-referencing it).
- Level 2 (split WITHIN a surface) by self-contained neighbour areas; keep
  states (empty / loading / error), nested sub-components, and page-local
  shared rules in the page-shell capability.
- Decision order (stop at first yes): (1) a state of the surface → page
  shell; (2) own external integration / data source → own capability;
  (3) reused across surfaces → own shared capability; (4) large
  self-contained area on this surface → own capability; (5) otherwise →
  containing area or page shell.
- Delta consequence: archive matches deltas to the baseline by capability
  folder AND requirement header (verbatim, whole-block). If this change
  needs a different decomposition than the baseline (split a monolith, a
  new capability, rename / re-own an area), RESTRUCTURE THE BASELINE FIRST
  (move requirement blocks verbatim into the target capabilities, `openspec
  validate --strict`, commit that restructure), THEN author deltas that
  line up 1:1. Never author a delta against a capability or header absent
  from `specs/`.

TRACE METADATA RULES:
- The WBS code lives in the Option-H heading (REQUIREMENT RULES), NOT in a
  metadata line. Every ADDED and MODIFIED requirement carries, on the line
  immediately after the heading, a single HTML comment holding Trace +
  Source: `<!-- Trace: <RQ IDs> · Source: <citation> -->`. Use a comment,
  never bold `**Trace/WBS/Source**` lines — the CLI folds any text between
  the heading and the first `#### Scenario:` into the archived baseline
  requirement text, so bold metadata pollutes the canon while a comment
  stays invisible in the render and out of the baseline.
- RQ and WBS codes come VERBATIM from the discovery trace / proposal
  lock — never minted, never blank, never `N/A`. Commit each requirement's
  `WBS` (for its heading) and `Trace` (for its comment) in its plan ledger
  line before drafting: a cross-cutting requirement (privacy, accessibility,
  i18n) copies the RQ/WBS of the traced requirement(s) whose behaviour it
  constrains, or its own cross-cutting RQ row upstream when one exists — a
  ledger line with blank or invented (`PRIV`) codes is the defect, caught
  before emit, not a heading WBS or `Trace:` left empty in the draft.
- Sources cite the signed grounding a behaviour derives from: the SRD
  requirement (its RQ id and inbox file), technical/meeting notes, a
  grooming decision (`grooming Q1`), a design mechanism, a discovery
  cross-cutting row, or the baseline spec path for MODIFIED blocks —
  never the intermediate proposal as the derivation. A behaviour bounded
  by an approved decision cites that decision alongside its signed source.

OUTPUT CONTRACT:
- `openspec/changes/<name>/specs/<capability>/spec.md`, one file per
  affected capability.
- Each file contains exactly its delta section(s) with requirement
  blocks and scenarios. The file MUST begin with a `## <DELTA>
  Requirements` header — no document title, no Purpose section (the
  capability Purpose lives in the baseline spec), no executive
  summary, no status line, no gate checklist, no commentary, and NO
  title line before the first operation header (a leading title makes
  `openspec validate` reject the delta). Delta files must merge into
  the archive clean.

SELF-CHECK BEFORE OUTPUT (fix any "no" before emitting):
- Grooming Approval Gate verified before authoring (ENTRY GATE)?
- One file per Affected Capabilities row, under the delta section its
  disposition names (FILE INVENTORY RULES)?
- Coverage total — every in-scope lock RQ traced, every slice formalized,
  every Applicable cross-cutting area captured, and nothing deferred,
  declined, or out of scope specced (DECISION KERNEL)?
- Every ADDED/MODIFIED requirement carries its full grounded scenario
  set, not a lone happy path, and no ungrounded case (SCENARIO RULES)?
- Every requirement transcribes its covering proposal seed AS THE SKELETON
  and is COMPLETED with every covering DECISION LEDGER bound and Applicable
  cross-cutting layered on (no requirement left at the bare seed) — with
  design/discovery adding nothing beyond those bounds: no NEW clause,
  vendor/SDK/endpoint noun, or consumer-UI wording pulled from a discovery
  Requirement / Requirement Details (SEED SKELETON), and every Source cites
  the signed grounding (SRD requirement, technical notes, grooming, design,
  or baseline), not the intermediate proposal as the derivation (TRACE
  METADATA RULES)?
- Every requirement and scenario line passes the OBLIGATION-SEAT TEST and
  the IMPLEMENTATION SCAN — no consumer-UI behaviour in an integration
  spec, no implementation token (SDK/product, endpoint/URL/verb/status/
  payload, internal-step narration), and no consumed-vendor OUT-set name in
  a functional spec (REQUIREMENT RULES, SCENARIO RULES)?
- Every WBS matches its BLOCK LEDGER line-for-line — administering criteria
  in a single trailing `Management` requirement, each Applicable
  cross-cutting guarantee its own requirement, no observer seat split below
  two scenarios, and names unique within the WBS (REQUIREMENT PARTITION)?
- No fabricated operational property — no cache/TTL, retry/retry-after,
  quota, rate-limit, or timeout that no approved decision or cited source
  states (OPERATIONAL GROUNDING)?
- Every approved/deferred-approved decision bound appears as an
  observable clause AND a scenario, with no rejected option
  (APPROVED-DECISION BINDING)?
- No minted route, path, number, or magnitude; a new surface named, not
  slugged (GROUNDING)?
- MODIFIED blocks carry verbatim baseline headings with full updated
  blocks; REMOVED blocks carry Reason + Migration (DELTA RULES)?
- Every cross-cutting behaviour, including a retired URL's redirect,
  exists as a requirement with a scenario — never only a Migration note?
- Every heading is Option-H `### Requirement: <WBS> — <name>` with the
  WBS in the heading (MODIFIED/REMOVED match the baseline verbatim), Trace
  and Source in a single HTML comment after the heading (no bold metadata
  lines), and every WBS/RQ verbatim and grounded (REQUIREMENT RULES, TRACE
  METADATA RULES)?
- Every near-final proposal AC criterion covered by >=1 scenario, with no
  fabricated scenario (SCENARIO RULES)?
