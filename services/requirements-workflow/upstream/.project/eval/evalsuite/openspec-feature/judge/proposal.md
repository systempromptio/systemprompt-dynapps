---
schema: openspec-feature
artifact: proposal
context:
  - .context/initial-context
  - discovery/final
  - design/final
sources:
  - ../../../../templates/openspec/schemas/feature/templates/proposal.md
  - ../../../../../skills/project-context/SKILL.md
scoring:
  pass_threshold: 0.7
---

# Proposal Standard — Contract and Rubric

This is the canonical definition of a good `proposal.md` working copy in the
OpenSpec `feature` workflow. The judge uses it to score a candidate proposal. It
is intentionally explicit and self-contained: do not assume the judge has any
ambient project knowledge beyond this standard, the inlined SOURCES, and the
CONTEXT (the source packet plus the reconciled discovery trace and the
architecture/solution design the proposal derives from).

## What a proposal artifact is (essence)

- A proposal is the **BA/SA delivery offer** produced after discovery and
  design and before grooming. It lives as a working copy at
  `openspec/changes/<name>/proposal.md`.
- It **locks scope; it does not re-decide it**. The Requirements Lock mirrors
  the discovery Requirements Trace — every trace row exactly once, disposition
  inherited verbatim — and is the scope source of truth downstream artifacts
  (grooming, specs, FSD/ISD, stories, Planning, tasks) consume.
- It **converts the locked scope into the offer**: the Delivery Breakdown
  slices the in-scope requirements into per-slice BLOCKS — each a
  `### S# — <WBS> — <name>` heading, a machine-readable metadata list
  (`Covers (WBS)`, `Capability`, `FSD` or `ISD`, `Depends on`), a
  `#### User Story`, and a `#### Acceptance Criteria` — the concrete "what you
  will get" a human approves at grooming. Story + Acceptance Criteria are
  authored NEAR-FINAL here: the proposal is the SINGLE SOURCE OF TRUTH that the
  FSD later PROJECTS read-only (the functional-artifacts script consumes these
  blocks to project the FSD + stories), not thin seeds. They stay offer-level, client-facing
  criteria, NOT the exhaustive behaviour canon — specs still own the full
  Given/When/Then scenario sets, so an Acceptance Criteria list carries only
  numbered SHALL statements, never Gherkin (GIVEN/WHEN/THEN) blocks.
- It **proposes; it does not approve**. It runs pre-grooming: Change Request
  dispositions are proposed for a named human to approve in grooming; upstream
  Open Questions stay open; unresolved facts (accounts, keys, quotas, approval
  states) stay recorded as the sources leave them.
- It **inherits the solution shape from design**. The Approach compresses the
  design's decisions to one delivery paragraph and, for an integration change,
  points at the design's Integration Specification instead of restating the
  service contract. It never re-opens or re-decides architecture and never
  carries implementation detail (our file/route/component wiring, our code,
  our tuned numbers).
- It carries the **External Dependencies & Access** register — the build
  prerequisites grooming turns into its Prerequisites table — with verified,
  factual rows (or an explicit `UNVERIFIED` marker), each tied to the slices
  it blocks. Phantom dependencies propagate into grooming and tasks.
- Its **delivery scope (functional vs integration) is inherited from the
  discovery trace in CONTEXT**, not re-decided. A `functional` proposal slices
  the shopper experience (end-user persona); an `integration` proposal stays
  thin — few slices, addressed to the consuming developer (a storefront
  developer building on the capability), with the contract pointed at, not
  restated. Depth shifts; both scopes fill every section.
- It opens with a **derived reader interface**, both blocks derived LAST from the
  finished body so a downstream reader consumes them before the detail sections:
  an **Executive Summary** that reads as a reader map (what the offer settles,
  what it hands downstream, what stays open — by subject and count, no
  inventories) and a **Document Consumption Guide** that maps the body (one row
  per `##` section it carries, its `Key data` the identifiers a consumer joins
  on). Neither is a new source of truth: every value must already be readable in
  the body it projects.

## Invariants the candidate MUST satisfy

1. **Lock fidelity.** The Requirements Lock mirrors THIS flow's discovery
   Requirements Trace: the same RQ row set (none dropped, none added), each RQ
   exactly once, dispositions inherited token-for-token (`in scope` /
   `out of scope` / `deferred` / `CR candidate`), RQ/WBS identifiers preserved
   verbatim, rows grouped by WBS. Notes may compress upstream notes but must
   not contradict them.
2. **Breakdown coverage (bijection).** Each slice block's `Covers (WBS)`
   metadata value holds the in-scope requirement's clean WBS code (the leaf code
   the lock's WBS column and the FSD block headings use, NOT an RQ ID). The
   breakdown is 1:1 with the in-scope lock rows: each `in scope` requirement's
   clean WBS is delivered by EXACTLY ONE slice block, and each slice block's
   `Covers (WBS)` names EXACTLY ONE in-scope WBS — no grouping (several
   requirements folded into one slice), no splitting (one requirement spread
   across slices), and no clean WBS the lock does not hold as `in scope` (no
   deferred, out-of-scope, or CR scope smuggled into the offer). Slice count
   equals the in-scope requirement count. Each slice names a capability from
   discovery's Affected Capabilities in its `Capability` metadata.
3. **Acceptance Criteria discipline.** Each slice's `#### Acceptance Criteria`
   is a NEAR-FINAL, client-facing numbered SHALL list (one obligation per
   criterion, observable rather than subjective, a concrete subject),
   grounded in the upstream artifacts. The criteria sit in H5 (`#####`)
   RE-USABILITY MARKER groups, each carrying one marker from the closed legend
   (`[SF Next OOTB]`, `[3rd party OOTB]`, `[SF Next Modified]`, `[Legacy AS IS]`,
   `[Legacy Modified]`, `[New]`) and its OWN numbered SHALL list; the
   `## Re-usability Legend` section prints that whole key whichever markers the
   groups use. A
   slice takes ONLY the markers that apply — one group when its criteria share one
   footing — and never restates the same criterion under two markers (that is
   fabrication). Marker groups are the ONLY heading axis inside Acceptance
   Criteria: device context belongs inside the criterion and the administering
   actor belongs in its subject, so a `Desktop` / `Mobile & Tablet` /
   `System Configuration` heading is a stale shape, not a canonical one. Which
   marker a group carries is the author's call: NEVER grade the marker CHOICE, in
   either direction, under any dimension. An AC block MAY open with a one-line
   BOUNDARY NOTE naming the sibling change or vendor that owns a surface this
   slice merely hosts — that note is expected content, never an extra section, and
   never fabrication when the CONTEXT grounds the ownership.
   The AC is NOT the spec behaviour canon: no Given/When/Then Gherkin
   blocks, no code tokens or invented magnitudes/routes. Vendor naming branches
   on scope: a FUNCTIONAL AC names NO consumed-dependency vendor (they sit in
   `Depends on` / External Dependencies & Access); an INTEGRATION AC may name the
   OWNED vendor at GOVERNANCE level only (key / quota / billing / privacy posture
   — the golden's `sa` AC does this) and SHOULD leave its SDK, endpoint, method,
   field, or call mechanism to the design's Integration Specification — but naming
   one is an over-specification WARNING, not a critical (the scope-breaking fault
   is PARROTING the seed, per the SEED-ELABORATION test). An undecided item
   stays undecided with its Q# reference, never hardened into a chosen value.
4. **Scope fit (inherited from discovery).** The proposal matches the delivery
   scope and change identity the discovery trace resolved. Out of Scope &
   Boundaries carries the deferred rows, unsigned CR topics, and consumed
   capabilities with their owners; consumed capabilities and other changes'
   surfaces are boundaries, not owned scope. Functional proposals carry the
   delivery detail; integration proposals stay thin. Each slice's story is a
   VERBATIM LOOKUP, not authored prose: the discovery Requirements Trace
   `Requirement` cell for the slice's `Covers (WBS)`, copied byte-for-byte
   (persona / wording / punctuation intact; only a trailing `[Q#]` may be
   added). This is IDENTICAL for FUNCTIONAL and INTEGRATION scope — the
   integration framing lives in the slice name and Acceptance Criteria, never
   in the story. A story that re-authors, paraphrases, shortens, swaps persona
   (e.g. source "As a user" rendered "As a shopper"), re-casts a shopper story
   as an authored "As a <consuming developer>, I want …" obligation, or has no
   verbatim basis in the trace, is a violation.
5. **Proposes, never approves (pre-grooming discipline).** No CR is asserted
   approved / absorbed / descoped as fact; no upstream Open Question is closed
   or answered with an invented value (in prose OR inside an Acceptance
   Criterion); no unresolved fact or status (account or credential existence,
   quota, billing guardrails, approval state) is asserted settled or
   "confirmed"; no human sign-off is implied.
6. **No fabrication (invent nothing).** No RQ/WBS/CR/Q/S identifiers, tracker
   refs, branches, pages, services, access items, owners, or statuses that are
   not grounded in the CONTEXT.
7. **Grounded dependencies & access.** Every External Dependencies & Access
   row traces to the upstream artifacts or cited sources with a factual
   status; an unverifiable item is dropped or carries the explicit
   `UNVERIFIED` marker. Membership branches on delivery scope: a functional
   proposal lists the consumed upstream deliveries / existing services and
   holds NO vendor account/credential/billing rows of its own (the owning
   integration changes carry those); an integration proposal lists its
   operational prerequisites (account, credentials, billing,
   sandbox/environment) with their unresolved statuses preserved. Rows never
   list the change's OWN work items.
8. **Inherited approach.** The Approach compresses design decisions without
   contradiction, re-opening, or new architecture, names the FSD/ISD working
   copies exactly as discovery's Delivery Scope & Targets records them, and
   contains no implementation detail.
9. **Template & structure conformance.** Follows the `proposal.md` template
   section order and headings (Executive Summary, Document Consumption Guide,
   Tracker, Proposal Source Inventory, Requirements Lock, Change Request
   Dispositions, Re-usability Legend, Delivery Breakdown, Out of Scope &
   Boundaries, External Dependencies & Access, Approach, Open Questions, Proposal
   Gate). The `## Document Consumption Guide` sits directly after the Executive
   Summary and is a MAP THAT FOLLOWS THE DOCUMENT: one row per `##` section the
   body carries in document order, excluding only the Executive Summary, the guide
   itself, and the Proposal Gate — every carried section earns a row and a section
   the body omits earns none — with each row's `Key data` the identifiers/sets
   readable in the section it names (`None` when empty). The Re-usability Legend is
   a FIXED KEY, not a selection: it carries EVERY canonical bullet the template
   holds, in template order and wording verbatim, whichever markers the Delivery
   Breakdown uses — a member listed while no slice carries it is CORRECT, never a
   finding, and a legend trimmed to the markers used is the defect. A terse canonical
   gloss is CORRECT and complete by definition — never call it thin and never ask
   it to describe what this change does with the marker. A dropped member or a
   gloss re-authored into case-specific prose is a
   `structure_conformance` warning, never a critical. Tables use the template columns;
   empty registers carry the explicit `None` row. Delivery Breakdown carries
   per-slice blocks (`### S# — <WBS> — <name>` H3, `#### User Story` /
   `#### Acceptance Criteria` H4, and any `#####` AC groups), not a table — the
   deterministic gate does not see that H3/H4/H5 shape, so judge it here:
   every in-scope slice has the metadata list, a User Story, and an Acceptance
   Criteria section under the right heading levels. A deterministic code gate
   already enforces exact H2 section presence, order, and table columns against
   the golden; do not re-litigate what that gate covers.

## Forbidden (each is a critical finding when present)

- A Requirements Lock that re-decides scope versus THIS flow's discovery trace:
  a dropped trace row, an added non-discovery requirement, or a changed
  disposition.
- A Delivery Breakdown that breaks the coverage-law bijection: an in-scope
  requirement's clean WBS absent from every slice (the offer misses signed
  scope); a slice whose `Covers` carries a clean WBS the lock does not hold as
  `in scope` (deferred / out-of-scope / CR scope offered as if signed); a slice
  whose `Covers` names more than one in-scope WBS (grouping); or one in-scope
  WBS placed in more than one slice (splitting). Slice count must equal the
  in-scope requirement count.
- Any fabricated identifier, tracker ref, branch, page, service, access item,
  owner, or status not grounded in the CONTEXT.
- A pending item asserted settled: a CR marked approved/absorbed as fact, an
  upstream Open Question closed with an invented value (in prose or an
  Acceptance Criterion), an unconfirmed account/key/quota/approval asserted as
  existing or "confirmed", or implied human/grooming sign-off.
- Implementation detail presented as the offer or approach: our named code
  artifacts (internal file/module paths, route/loader/action or component
  wiring) or specific magnitude values we chose that no cited source states —
  in the Approach OR inside a slice's Acceptance Criteria.
- Spec leakage in the breakdown: Acceptance Criteria authored as formatted
  Given/When/Then scenario blocks (multi-line Gherkin), turning the offer into
  the behaviour contract that belongs to specs; or a marker split that only
  restates the SAME criterion under two marker groups (duplication is
  fabrication — one group carries it).
- Restating the discovery trace's verbatim requirement texts or authoring new
  requirement/behaviour contracts as if the proposal were discovery, an FSD,
  or specs.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE place.

- Artifact presence (run FIRST, before anything else): if the candidate is not
  a proposal.md working copy at all — a refusal or STOP message, a status
  report, planning notes, or an empty document — record the critical finding
  `no_artifact`, score EVERY dimension 0.0, and stop. A missing-upstream
  complaint is never a valid candidate: the upstream artifacts are given in
  CONTEXT.
- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A difference
  from GOLDEN, or something GOLDEN omits, is NEVER by itself a finding. Every
  finding MUST quote the candidate text AND the STANDARD rule or CONTEXT fact
  it violates. "Not in GOLDEN" or "not mentioned" alone is not evidence —
  especially against summary wording, slice grouping, story phrasing, note
  compression, or the approach phrasing.
- Closed-world vs open-world. Identifiers (RQ/WBS/CR/Q/S), dispositions,
  statuses, ownership, tracker ids, delivery targets, capabilities, and access
  items are CLOSED-world: they must trace to CONTEXT, and inventing or flipping
  one is a violation. Each slice's story text is ALSO closed (STORY-LOOKUP LAW):
  it is the discovery Requirements Trace `Requirement` cell for the slice's
  `Covers (WBS)`, copied VERBATIM — re-authoring, paraphrasing, shortening, or
  swapping the persona (e.g. "As a user" -> "As a shopper") is a violation. This
  closed rule is scope-independent: FUNCTIONAL and INTEGRATION slices copy the
  same cell, and a story re-cast as an authored consuming-developer obligation is
  a violation, not the required handling. Requirement summaries, Acceptance
  Criteria phrasing, business value, and approach wording remain OPEN-world
  synthesis: judge them for grounded, faithful compression, not for verbatim
  presence in a source.
- Lock mirroring (closed rule — run BEFORE scoring `scope_lock_fidelity`):
  list THIS flow's discovery Requirements Trace RQ ids from CONTEXT, then diff
  the candidate's lock against it. (1) Same RQ set — a missing or extra row is
  a critical. (2) Same disposition per row, token-for-token — a flip (e.g.
  `deferred` -> `in scope`, `out of scope` -> `in scope`) is a critical.
  (3) Summaries and Notes are open-world: compression is fine; a Note that
  contradicts the upstream disposition or invents a fact is not. Do not fire on
  wording differences.
- Breakdown coverage (closed rule — run BEFORE scoring `delivery_breakdown`):
  list the lock rows whose disposition is `in scope` (by their clean WBS code),
  then read each slice block's `Covers (WBS)` metadata value (a clean WBS code,
  not an RQ ID). The breakdown MUST be a bijection to the in-scope set. A
  coverage-law CRITICAL fires in ANY of these forms: (1) an in-scope
  requirement's clean WBS absent from every slice's `Covers (WBS)` (missing
  scope); (2) a clean WBS outside the in-scope set (deferred / out-of-scope / CR
  / invented) present in some slice's `Covers (WBS)` (smuggled scope); (3) a
  slice whose `Covers (WBS)` names more than one in-scope WBS (GROUPING —
  fold-in of several requirements); (4) one in-scope WBS present in more than one
  slice (SPLITTING — a requirement spread across slices); (5) slice count not
  equal to the in-scope requirement count. The breakdown must be 1:1 — one slice
  block per in-scope requirement — so it projects to stories.md and the FSD
  deterministically (one slice : one story : one Acceptance Criteria block). (6) Story text is CLOSED against the
  discovery source in EVERY flow (STORY-LOOKUP LAW): each slice's story must be
  the discovery Requirements Trace `Requirement` cell for the slice's
  `Covers (WBS)`, reproduced VERBATIM (persona included). A re-authored,
  paraphrased, or re-personaed story (e.g. source "As a user…" rendered "As a
  shopper…"), or a story with no verbatim basis in the trace (a fabrication,
  typically from splitting one source requirement across slices), is a critical.
  Do NOT reward a "nicer" or normalized rewrite — closeness to GOLDEN wording is
  irrelevant; fidelity to the discovery source is the test. FUNCTIONAL and
  INTEGRATION slices copy the same cell; re-casting a story as an authored
  consuming-developer obligation ("As a <developer>, I want …") is itself a
  re-persona critical — not the required handling.
- Acceptance Criteria discipline (apply BEFORE firing spec-leakage): fire the
  Forbidden spec-leakage item ONLY when an Acceptance Criterion is a formatted
  multi-line Given/When/Then scenario block, OR when two marker groups merely
  restate the SAME criterion. Short conditional phrasing inside a criterion
  ("when X, Y happens"; "only on explicit shopper action") is the standard's own
  event-response criterion shape and NEVER fires. A near-final, fuller criterion
  is the intended shape now (NOT a thin seed) — do NOT fire for a criterion being
  detailed. A criterion whose CONTENT promises another change's behaviour is an
  ownership fault — route it to `scope_fit`, not spec-leakage. A group heading
  outside the closed marker legend — including the stale `Desktop` /
  `Mobile & Tablet` / `System Configuration` surface headings, or a flat AC list
  under no marker group at all — is a `delivery_breakdown` WARNING, not a
  critical. Which marker a group carries is NEVER graded: do not fire (and do not
  reward) a marker you would have chosen differently, and never route a
  marker-choice disagreement into `no_fabrication` or `scope_fit`.
- Flow-scoping. Judge the candidate ONLY against ITS OWN flow's discovery and
  design in CONTEXT. In a functional flow the lock keeps functional rows in
  scope, slices address the shopper, and consumed services are dependencies;
  in an integration flow only the selected service's obligations are in scope,
  slices address the internal consumer, and every other row is out of scope
  naming its owner. Never import the other flow's dispositions, questions,
  slices, or prerequisites.
- Dependencies & access placement (closed rule). Functional: rows are the
  consumed upstream deliveries / existing services the locked scope relies on;
  a vendor account/credential/billing row in a functional proposal is a
  `scope_fit` finding (the owning integration change carries it). Integration:
  rows are the capability's own operational prerequisites with factual
  statuses; the service the change OWNS is never its own dependency. In either
  scope, a row for the change's OWN work or artifacts (a redirect/route it
  implements itself, its own FSD/ISD working copies or publication targets) is
  a `scope_fit` finding — the register is external prerequisites only. A
  missing grounded prerequisite is `missing_evidence`, not a critical. A
  status is a fact: `unconfirmed (Q2)` for an unconfirmed account is correct
  handling, and flagging it as a gap is wrong; asserting `have` for it — or
  translating a source's `client-owned` / unresolved status into `have` — is
  an `approval_discipline` critical.
- Approval discipline (HARD GATE — run these steps IN ORDER before firing):
  (1) NAME the pending item: the exact CR#, Q#, or unresolved fact that THIS
  flow's discovery/design carries as pending, which the candidate text settles.
  If you cannot name one, DO NOT fire — score 1.0 and stop.
  (2) CONFIRM IT IS ASSERTED SETTLED. The text must state it as fact (a CR
  Status `approved`, CR scope inside a slice / the lock as `in scope`, a
  chosen redirect type or call approach with no Q# reference, an account
  "confirmed"). A clause carrying an inline pending marker — `pending`,
  `proposed`, `unconfirmed`, `(Q#)`, `raised for grooming` — is proposed, not
  settled, and does NOT fire, whatever its verb sounds like.
  (3) An omission NEVER fires here: a missing CR row or dependency row is
  `missing_evidence` or a lock-fidelity matter, not an approval violation.
  (4) RAISING A QUESTION NEVER FIRES HERE. Approval discipline fires ONLY on a
  pending item asserted SETTLED. Keeping a question open — re-listing an upstream
  Q# in Open Questions, or adding a NEW proposal-stage Q# (Q3/Q4…) that re-raises
  an already-undecided upstream item — is the OPPOSITE of settling: it can NEVER
  be an `approval_discipline` critical. A redundant re-raise is at most a
  `warning`; do not reframe it as "proposes, never approves" and cap the score.
  - an identifier / status / item not in CONTEXT -> `no_fabrication`;
  - a real, grounded fact placed against the flow's ownership (consumed
    service as owned scope, vendor credentials in a functional proposal,
    consumer UI inside an integration proposal — including an Acceptance
    Criterion that promises behaviour another change owns, e.g. map/list
    synchronization promised by the integration) -> `scope_fit`;
  - a lock row dropped / added / disposition flipped -> `scope_lock_fidelity`;
  - a slice coverage-law break (missed in-scope RQ, smuggled non-in-scope RQ),
    Gherkin-block Acceptance Criteria, or a duplicate criterion across two marker
    groups -> `delivery_breakdown`;
  - a pending item asserted settled or approved -> `approval_discipline`;
  - our named code artifact or our chosen magnitude in the approach or an
    Acceptance Criterion -> `approach_inheritance`.
  Do not raise the same underlying claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension: (a) upstream
  working-copy anchors (`openspec/changes/<name>/discovery.md`, `design.md`)
  and project-context inbox anchors — the citation convention; (b) referencing
  an upstream Q#/CR# in Notes, Acceptance Criteria, boundaries, or the
  Approach — that is inheritance, not re-raising or hardening; (c) the explicit `None` row in an
  empty register; (d) a user-facing URL / page path / redirect subject (e.g.
  `/legacy-path`) inherited from the sources — an IA/navigation fact, not a code
  leak (only a chosen redirect TYPE with no Q# reference hardens Q2);
  (e) restating the mandatory automated-testing obligation — it is the
  schema's own rule, not fabrication; (f) a tracker reference reused verbatim
  from CONTEXT, including "no dedicated ticket exists" when the sources record
  none; (i) a working-copy anchor whose `<name>` segment uses the harness /
  case placeholder instead of the discovery-resolved change name — that is a
  NAMING SLIP: route it as a `scope_fit` or `structure_conformance` WARNING,
  never `no_fabrication` (fabrication requires an identifier grounded
  NOWHERE, not a mis-assembled anchor of a real artifact);
  (g) business-value phrasing — open-world synthesis, judged for faithful
  compression, not verbatim GOLDEN match (NOTE: this does NOT extend to slice
  STORY text, which is closed — carried verbatim from the discovery source per
  Breakdown coverage above —   nor does it sanction grouping/splitting); (h) `Depends on` naming the same
  items as the External Dependencies & Access register — that is the required
  wiring, not duplication; (j) the `## Re-usability Legend` section — carrying the
  WHOLE canonical key, including members no slice uses, which is the required shape,
  never padding or extra content — plus the `#####` marker group headings and a
  one-line boundary note opening an AC block: all of them are the template's own
  shape, never a scope concession; (k) the `## Document Consumption Guide` block
  and its rows — a required derived interface that maps the body's sections,
  never extra content, padding, or fabrication when the named section grounds
  the row's `Key data`.
- Omission is not a critical — under ANY dimension. A missing expected item
  (a dependency row, a cross-cutting obligation in an Acceptance Criterion, a
  delivery-target mention) is `missing_evidence` (or a warning), never
  `no_fabrication` and never `approval_discipline`. The exceptions are the Forbidden items: a
  dropped Requirements Lock row IS a critical (`scope_lock_fidelity`), and an
  in-scope RQ absent from every slice IS a critical (`delivery_breakdown`),
  because lock completeness and offer coverage are the artifact's core
  contract.
- Completeness does not offset a violation, and fluency is not correctness.
  Inspect each identifier, disposition, status, owner, Acceptance Criterion, and
  approval verb on its own merits regardless of how polished the document is.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by the
dimension id below. For each dimension, 1.0 = fully satisfied with evidence,
~0.5 = partially satisfied or minor gaps, 0.0 = violated or absent.

- `intent_and_readiness` — the Executive Summary reads as a READER MAP, not a
  recap: after a reader-first, grounded opening (problem/change context + one
  source-grounded business-value statement) it states, by SUBJECT and COUNT, what
  the offer SETTLES (the slices at headline level, the proposed CR dispositions,
  the recorded dependencies), what it HANDS DOWNSTREAM (the Delivery Breakdown as
  the read-only source the FSD/specs project, and who owns what is held Out of
  Scope), and what STAYS OPEN plus grooming readiness that mirrors the Proposal
  Gate; it names subjects not identifiers, dumps no RQ/CR/Q/S inventory, and
  implies no approval for a pending item. A count it states must equal the size of
  that set in the body; a plainer or richer phrasing than GOLDEN's is never a
  defect.
- `scope_lock_fidelity` — the lock mirrors THIS flow's discovery trace exactly
  (row set, one row per RQ, verbatim identifiers and dispositions, WBS
  grouping); Notes are grounded compressions carrying the inherited
  cross-links (Q#/CR#/dependency/owner).
- `delivery_breakdown` — the coverage-law bijection holds (each in-scope
  requirement's clean WBS in exactly one slice block, each slice exactly one
  in-scope WBS, slice count = in-scope requirement count, nothing else — no
  grouping, no splitting); each slice block carries its `Covers (WBS)` /
  `Capability` / `Depends on` metadata and a `#### User Story` + `####
  Acceptance Criteria`. Each slice's story is the discovery Requirements Trace
  `Requirement` cell for its `Covers (WBS)`, reproduced VERBATIM (persona
  included; no re-authoring, paraphrase, or persona swap) — scope-independent,
  so an integration slice copies the SAME cell (integration framing lives in the
  slice name and Acceptance Criteria, not a rewritten story); each names a
  discovery capability.   Acceptance Criteria are near-final client-facing numbered SHALL
  statements grounded in the resolved upstream (discovery `Requirement Details` /
  `Notes`; design `Gap Analysis` / `Base Configuration & Cross-Cutting` /
  `Architecture Decisions` / `Integration Specification`), not re-mined from the raw
  SRD, in a valid shape (`#####` marker groups from the
  closed legend, only the markers that apply, no criterion duplicated across two
  markers, no Gherkin block; the marker CHOICE is not graded, and a vendor
  mechanism token is only a WARNING, not a shape breach — see the MECHANISM
  TEST), with undecided items kept undecided via Q# references; `Depends on` ties slices to the access register;
  integration breakdowns stay thin (few slices, flat AC). The scope-breaking AC
  fault is PARROTING the seed (SEED-ELABORATION test), not a vendor word.
- `scope_fit` — delivery scope and change identity inherited, not re-decided;
  Out of Scope & Boundaries carries deferred rows, unsigned CR topics, and
  consumed capabilities naming owners, consistent with the lock and slices;
  depth weighted to the scope (functional detailed, integration thin).
  INTEGRATION-AC SUBJECT TEST (deterministic — run before firing an ownership
  breach): read each integration-slice criterion. FIRE a `scope_fit` breach when
  its subject is the shopper/user ("allow the shopper to enter…") OR it NAMES a
  consumer-UI construct or outcome — a results list, a map selection,
  highlighting, synchronization / "in tandem" / "vice versa", a permission
  prompt, a page / filter / navigation, localized copy, accessibility. This fires
  REGARDLESS of framing: "expose a hook / events for the consumer to synchronize
  / highlight …" STILL fires, because the criterion names the consumer's UI
  outcome. INPUT-ACCEPTANCE CARVE-OUT (deterministic): a criterion where the
  capability ACCEPTS or SUPPORTS a shopper-initiated INPUT — "the capability SHALL
  accept a typed postal code / city", "SHALL support a shopper-initiated
  current-location request", "SHALL not persist the supplied coordinates" — is the
  capability's CONTRACT SURFACE and is CLEAN; the shopper is the source of an input
  the capability accepts, not the subject of a UI outcome. FIRE only when the
  criterion promises a shopper-VISIBLE RESULT the consumer renders — a displayed
  list, a highlight, a map selection reflected in the UI, a permission prompt
  shown. HOSTED-NOT-OWNED CARVE-OUT (deterministic): on a slice whose covered
  requirement is closed by CONSUMING a vendor-hosted or sibling-owned artifact,
  a criterion stating how this change REFERENCES that artifact, what propagates
  when the owner re-publishes it, how the change behaves when it is unavailable,
  or that the artifact is the other party's to author, is this change's OWN
  obligation and is CLEAN — it is the honest form for a surface the signed work
  cannot otherwise make true. Do NOT read it as promising another change's
  behaviour, and do NOT read the accompanying BOUNDARY NOTE as ceding scope.
  The `scope_fit` breach on such a slice is the OPPOSITE: a criterion asserting
  that this change MANAGES, AUTHORS, or CONFIGURES the vendor-owned content it
  only references. Do NOT fire when the criterion names the capability's own provided
  surface / interface / governance and STOPS there (e.g. "expose a stable interface
  that resolves a postal code to coordinates", "expose map rendering as a
  lazy-loadable entrypoint", "keep the key domain-restricted") — no rendered
  list / selection / highlight / sync outcome at all. A verbatim Story or a
  discovery-trace `Requirement Details` line that mentions shopper UI is NOT itself a breach —
  judge the AC criteria, not the Story or Details.
  INTEGRATION-AC MECHANISM TEST (deterministic, TOKEN-BASED — do not judge by
  "sounds technical"): record a `delivery_breakdown` WARNING (a quality issue that
  does NOT enter `critical_findings` and does NOT cap the score) when an
  integration AC (or an External Dependencies & Access row) contains one of these
  CONCRETE mechanism TOKENS: (a) a named vendor PRODUCT / SDK proper noun ("Cartora
  Maps SDK", "Cartora Geocoding API" used as the product you call); (b) an HTTP
  verb or endpoint / URL path (GET /geocode, `/v1/...`); (c) a method or function
  signature / field name (`geocode()`, `results[].lat`); (d) a wire protocol
  (REST, gRPC, GraphQL, WebSocket); (e) a numeric or SLA magnitude (a latency, a
  rate / quota number, "rate-limited"). Rationale for WARNING (not critical): the
  owned vendor is already named upstream (the design Integration Specification /
  discovery trace), so carrying a mechanism token is an
  over-specification to tidy on the way to the FSD/ISD, NOT a scope-breaking fault
  — the CRITICAL is the SEED-ELABORATION test below (stopping at the seed), not the
  presence of a vendor word. ALLOW-LIST — these are the capability's OWN contract
  vocabulary and never even warn: naming its own capability / service ("the Cartora
  maps integration SHALL…"), "expose a / callable / stable interface", "an
  entrypoint", "a render surface", "coordinates", "proximity ranking" /
  "proximity-ranked", "lazy-loadable", and governance ("domain-restricted key",
  quota / billing / privacy). Paired examples: "render an interactive map USING THE
  CARTORA MAPS SDK" → WARN (token: named SDK); "expose a callable interface that
  returns coordinates and a proximity ranking, keeping the key domain-restricted"
  → CLEAN (no token, own contract). Route the mechanism-token warning CONSISTENTLY
  to `delivery_breakdown`, never `approach_inheritance`.
  INTEGRATION-AC SEED-ELABORATION TEST (deterministic — anti-parroting; THIS is the
  critical that mechanism-token presence is NOT). This is a WHOLE-BLOCK test: NEVER
  decide it from one criterion (especially the first), from opening-verb similarity,
  or from any surface wording. Run this COUNT procedure over the entire slice AC
  block, lined up against the upstream obligation for its `Covers (WBS)` — the
  discovery trace `Requirement Details` bullets (and its `Notes`); the raw SRD is
  cited evidence behind that trace, never the alignment basis:
  (0) TWO PRE-GATES — both MUST pass before you run the count; if either fails this
  critical CANNOT fire (route any weakness to `missing_evidence` instead):
    - SCOPE — this critical applies ONLY to an INTEGRATION-consumed slice (one whose
      requirement is served by a consumed integration capability, where the seed
      names a vendor mechanism the AC could copy). On a FUNCTIONAL slice the seed IS
      the shopper behaviour, so restating it is an ABSENCE of added obligations, not
      a positive scope-leak: judge any shortfall as `missing_evidence` (missing
      warranted elaboration — an edge / empty behaviour, a merchant-configuration
      obligation a design Base-Configuration row licenses, or a cross-cutting
      privacy / a11y / i18n obligation) and do NOT fire parroting.
    - NON-EMPTY DETAILS — parroting is echoing the `Requirement Details` BULLETS, so
      if the slice's Details are empty (`—`, no bullets) there is NOTHING to echo:
      the AC is elaborating a bare one-line requirement from scratch. Never fire;
      judge only thinness (`missing_evidence`).
  (1) Scan EACH criterion for OBLIGATION MARKERS and label it by PRESENCE, not by
  semantic novelty against the Details. A criterion is ELABORATED the moment it
  carries ANY marker from this CLOSED list — check the four in order and STOP at the
  first hit (do NOT re-weigh once one matches):
    (a) PROVIDED-CONTRACT marker — the capability offers a surface others
    call/consume: expose / provide / make available an interface, an entrypoint, a
    render surface, a hook, a callable or stable contract.
    (b) GOVERNANCE marker — a key, quota, billing, privacy, domain-restriction,
    retention, or data-handling posture (incl. a `SHALL NOT persist / SHALL NOT
    send …` constraint).
    (c) ROBUSTNESS marker — an error, empty, no-result, invalid-input, timeout,
    unavailable, or fallback behaviour.
    (d) BASELINE-DELTA marker — it names a platform baseline primitive from CONTEXT
    and what THIS capability adds on top (builds on / extends / reuses / lazy-loads
    an existing convention).
    (e) CONSUMPTION-CONTRACT marker — the criterion states how this change
    REFERENCES a vendor-hosted or sibling-owned artifact, what propagates when the
    owner re-publishes it, that it holds a reference rather than a copy, or that
    the artifact is the other party's to author. On a slice closed by consuming
    such an artifact this is the strongest obligation available, so its presence
    ELABORATES the seed exactly like (a)-(d).
  A criterion carrying NONE of (a)-(e) is a SEED-ECHO. Do NOT judge whether a
  marker's content is "already implied by the Details" — the Details are high-level
  and never state these as the capability's OWN obligations, so marker PRESENCE is
  sufficient and decisive.
  (2) COUNT the ELABORATED criteria in the block.
  (3) FIRE a `delivery_breakdown` critical ONLY when that count is ZERO — the ENTIRE
  block is SEED-ECHO. If the count is >= 1 the block ELABORATES the seed: do NOT
  fire. (An elaborated-but-sparse block is at most a THIN-AC `missing_evidence`,
  never this critical.)
  When you fire, QUOTE every SEED-ECHO criterion paired with the Details bullet it
  echoes as verbatim evidence. Two things are NEVER SEED-ECHO evidence on their own
  and never push a criterion toward SEED-ECHO: reusing a Details noun, and carrying
  a mechanism token (that is the separate WARNING above — route it there, not here).
  Judge OBLIGATIONS, not wording. This is DISTINCT from a THIN AC (too few /
  underspecified criteria), which stays `missing_evidence` per the demotion gate —
  parroting is a WHOLE-BLOCK positive duplication you quote; thinness is an absence.
- `approval_discipline` — nothing pending asserted settled: CR dispositions
  proposed with pending status; upstream Q#s stay open (referenced, not
  closed, and not re-listed as new questions); unresolved facts keep their
  factual unresolved statuses; no implied sign-off. Apply the HARD GATE above
  before firing.
- `cr_dispositions` — every discovery CR candidate carried exactly once with
  CR# preserved, a proposed disposition, and pending status; the explicit
  `None` row when discovery carries none; no invented CRs.
- `dependencies_and_access` — rows grounded and scope-branched per the closed
  rule, statuses factual, specific items named (not vague "access needed"),
  unverifiable items marked `UNVERIFIED`, rows tied to the slices they block;
  no phantom or padded rows; missing grounded prerequisites recorded as
  `missing_evidence`.
- `approach_inheritance` — the Approach compresses design decisions at BA/SA
  level without contradiction or new architecture, points at the design's
  Integration Specification instead of restating the contract (integration),
  names the delivery targets verbatim from discovery, keeps undecided items
  undecided with their Q# references, and carries no implementation detail.
- `no_fabrication` — every identifier, status, owner, service, access item,
  and tracker reference traces to the CONTEXT; nothing invented.
- `structure_conformance` — template section order, headings, and table
  columns, including the `## Document Consumption Guide` block (with its
  `Section | What it settles & why it matters downstream | Key data` columns)
  directly after the Executive Summary; explicit `None` rows for empty
  registers. The guide MAPS THE BODY'S CURRENT `##` sections — one row each in
  document order, excluding only the Executive Summary, the guide itself, and
  the Proposal Gate, a row for every section the body carries and none for one
  it does not — and each row's `Key data` set agrees with the section it
  projects (the in-scope WBS set, the `S#`/`CR#`/`Q#` sets with status, owners,
  `None` for empty). A dropped section-row the body carries, or a `Key data` set
  the named section contradicts, is a `structure_conformance` defect; a `What it
  settles & why` phrase is judged only for grounding, and ordering/wording
  differences from GOLDEN are not defects. Score 1.0 when the guide is complete
  and body-faithful; deduct per dropped section-row or contradicted `Key data`
  set. A deterministic code gate already enforces exact H2 section presence,
  order, and table columns against the golden; do not re-litigate that — grade
  the guide's ROW-SET fidelity to the body and its `Key data` accuracy here.

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it: if
  any item in `critical_findings` is present, `score` MUST NOT exceed 0.5.
- `pass` is true only when `score >= threshold` (given in the user message)
  AND `critical_findings` is empty.

### Finding severity

- **SEVERITY DEMOTION GATE (apply before writing ANY critical).** An ABSENCE is
  never a critical: expected-but-missing content — a dropped row, a prerequisite
  the design carries but the proposal omits, a `Depends on` tie the metadata
  omits, a cross-cutting / merchant-configuration obligation absent from an
  Acceptance Criteria list, or an AC that is merely thin — is `missing_evidence`
  and MUST NOT cap the score. EXCEPTION — a PARROTING AC block on an
  INTEGRATION-consumed slice (Details carry bullets AND every criterion is a
  SEED-ECHO of them, per the whole-block SEED-ELABORATION count with its two
  pre-gates) is NOT an absence: it is a present-in-text duplication you can quote, so
  it IS a `delivery_breakdown` critical. A FUNCTIONAL slice, a slice with empty (`—`)
  Details, or a block with even ONE elaborated criterion is NOT parroting — and a
  block framed as the CONSUMPTION CONTRACT for an artifact this change does not own
  (marker (e): reference, propagation, unavailability, explicit other-party
  ownership) is elaborated by definition, however close its nouns sit to the
  Details. Thinness
  is an absence; parroting is a whole-block positive copy on an integration slice —
  keep them distinct. A pattern this section lists as a warning (a
  re-listed upstream Q#, a system-persona story, an over-detailed non-Gherkin AC,
  a heading / WBS slip) is a `warning`, never a critical. Only a POSITIVE,
  present-in-text violation you can quote verbatim may enter `critical_findings`.
- **critical_findings** — violations of any Forbidden item or of invariants
  1-8 (lock fidelity, breakdown coverage, Acceptance Criteria discipline at
  Gherkin-block level, scope fit, approval discipline, fabrication, dependency
  grounding, approach inheritance). These block acceptance.
- **warnings** — quality/structure issues that should be fixed but do not by
  themselves make the proposal wrong (e.g. a thin Note where upstream carried
  a material cross-link, an RQ repeated across slices, a system-persona story,
  an over-detailed Acceptance Criteria list without Gherkin, a re-listed
  upstream Q# in Open Questions, a heading slip, weak WBS grouping).
- **missing_evidence** — expected content the CONTEXT implies but the
  candidate omits (e.g. a consumed dependency with no External Dependencies &
  Access row, an operational prerequisite the design carries that the proposal
  drops, a cross-cutting obligation absent from the Acceptance Criteria of the
  slice that owns it). Record it ONLY here: it MUST NOT appear in
  `critical_findings` and MUST NOT cap the score. An upstream Q# that is referenced but not re-listed
  in Open Questions is the REQUIRED handling, not an omission.

Always ground findings in concrete evidence. Every critical finding MUST quote
a string that appears VERBATIM in the candidate. Before recording a critical,
locate that exact string (the table row, the sentence) in the candidate text;
if it is not literally present, DROP the finding — do not infer, paraphrase, or
assume a row or section exists. For a missing-item finding, name the specific
CONTEXT / discovery / design fact the candidate omits. A finding whose evidence
you cannot copy verbatim from the candidate is a hallucination, not a
violation.
- VERBATIM IS NORMALIZATION-INSENSITIVE. "Verbatim" / "reworded" / "fabricated"
  judgements compare TEXT CONTENT, not typography. Ignore purely typographic
  differences: straight vs curly quotes (`"` vs `“ ”` / `'` vs `’`), ASCII vs
  Unicode hyphen/dash (`-` vs `‑ – —`), non-breaking vs normal spaces, and
  collapsed whitespace or line wrapping. If a candidate story or quote matches
  the source once these are normalized, it IS verbatim — do NOT fire a
  `no_fabrication` "reworded / fabricated story" critical. A story-fabrication
  critical is valid ONLY when the WORDS or MEANING differ (a changed persona,
  added/dropped clause, altered obligation), not when only glyphs differ. If
  your own evidence line shows the candidate and source strings reading the
  same, that is proof of verbatim, not of rewording — DROP it.
- COUNT BEFORE YOU FIRE A COVERAGE CRITICAL. Before recording a "missing slice"
  / "slice count mismatch" / coverage-law critical, enumerate the candidate's
  `### S#` slice headings and their `Covers (WBS)` values and confirm the
  claimed-missing WBS is genuinely absent from ALL of them. If the block exists,
  DROP the finding — a present slice miscounted as missing is a hallucination.
