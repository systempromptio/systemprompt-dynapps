---
schema: openspec-feature
artifact: grooming
context:
  - .context/initial-context
  - discovery/final
  - design/final
  - proposal/final
sources:
  - ../../../../templates/openspec/schemas/feature/templates/grooming.md
  - ../../../../../skills/project-context/SKILL.md
scoring:
  pass_threshold: 0.7
---

# Grooming Standard — Contract and Rubric

This is the canonical definition of a good `grooming.md` working copy in the
OpenSpec `feature` workflow. The judge uses it to score a candidate grooming
ledger. It is intentionally explicit and self-contained: do not assume the
judge has any ambient project knowledge beyond this standard, the inlined
SOURCES, and the CONTEXT (the source packet plus the settled discovery,
design, and proposal artifacts the grooming derives from).

## What a grooming artifact is (essence)

- Grooming is the **human decision ledger and approval gate** produced after
  proposal and before specs. It lives as a working copy at
  `openspec/changes/<name>/grooming.md`. There is no functional-vs-integration
  branching in this contract: the same structure and laws apply to every
  change; only the migrated content differs.
- It is the **first convergence** of discovery, design, and proposal, and the
  **scoped source of truth for the decision/approval layer only** — which
  option was chosen and by whom. It owns decisions, not facts: not scope (the
  proposal's lock), not architecture (design), not behaviour (specs). It never
  re-decides a fact it inherits, and it **routes** what another phase owns
  (design's planning deferrals) rather than deciding it.
- The **agent prepares, humans decide**. The generated artifact is a DRAFT:
  it migrates every still-open upstream item, adds session-ready context,
  options with implications, and labeled recommendations — and leaves every
  decision empty. Approvals happen later, live, with named humans.
- Its core law is **migration completeness**: every still-open upstream item
  lands here exactly once — the open Q#s from discovery/design/proposal
  (deduplicated by Q#: one question refined across artifacts is ONE row),
  every proposal Change Request row, every External Dependencies & Access row
  as a P# prerequisite, **every design Planning Deferral (PD#) as a Planning
  Routes row**, and the offer itself as O1 (Requirements Lock) and O2
  (Delivery Breakdown). An item this ledger misses never gets decided.
- Its second law is **zero self-approval**: at generation time every
  `Decision` and `Approved by` cell is `—`, every item Status is `OPEN` (or
  `BLOCKED` when a source records a blocker), and every Grooming Approval
  Gate checkbox is UNCHECKED. This is the one artifact whose gate ships
  unchecked by design. Exactly TWO exceptions carry an upstream decision into
  the draft: an item the user explicitly answered in the recorded
  conversation, and a **resolved-upstream** item an upstream already closed
  with a named human approver (see below). Absent such a record in CONTEXT,
  any approval is fabricated.
- It **inherits facts, it does not re-decide them**: severities, current
  statuses, proposed dispositions, and design's routed deferrals arrive
  verbatim from upstream. Grooming never answers its own questions, never
  closes an item, never changes a CR's proposed disposition, never alters the
  offered scope, and never decides a routed developer-HOW.
- Every migrated or raised row **back-links to its origin** — the upstream
  artifact section or decision that raised it — in the Origin column.
- A **resolved-upstream** item (a CR the client declined at an upstream
  review, a question answered and signed off upstream) arrives already decided:
  it is migrated for the record carrying its upstream Decision, its named
  Approved by, and its Origin, and is **never re-decided** (its upstream decision
  changed to a different value). This is the only migrated row whose
  Decision/Approved by are filled in the DRAFT. If the draft simply leaves that
  row `—`/OPEN, that is an omission to fix (a `missing_evidence` gap), NOT a
  re-opening and NOT a critical: a blank cell decides nothing.

## Invariants the candidate MUST satisfy

1. **Migration completeness.** Every still-open upstream item appears
   exactly once: each open Q# from THIS flow's discovery/design/proposal
   (deduped by Q#), each proposal CR row (or the explicit `None` row), each
   External Dependencies & Access row as a P# prerequisite, each design
   Planning Deferral as a PD# Planning Routes row (or the explicit `None` row
   when design defers nothing), and the two offer rows O1/O2. Identifiers
   (Q#/CR#/PD#/S#/RQ) are preserved verbatim; nothing is renumbered, dropped,
   or duplicated.
2. **Zero self-approval.** No filled `Decision`, no named `Approved by`, no
   `APPROVED`/`DEFERRED-APPROVED`/`ROUTED-TO-PLANNING` status, no checked gate
   checkbox — unless the CONTEXT explicitly records a named human answering
   that exact item (a user answer in the recorded conversation, or a
   resolved-upstream decision). An approver that is "Agent", "LLM", "Cursor",
   blank-but-implied, or a fabricated person is always a violation.
3. **Faithful inheritance.** Severities (CRITICAL / NICE-TO-HAVE), current
   factual statuses (e.g. `unconfirmed (Q2)`, `client-owned`, `to be
   delivered`, `have`), proposed CR dispositions, and design's routed
   deferrals match the upstream wording; `BLOCKED` appears only where a source
   records a blocker. A workflow `Status` mismatch — a source-blocked item left
   `OPEN` instead of `BLOCKED`, or `BLOCKED` without a source blocker, with the
   `—` cells intact — is a faithful-inheritance WARNING, not a critical (the `—`
   cells approve nothing). No question is answered, closed, or re-scoped, and no
   resolved-upstream item is **re-decided** — its upstream Decision changed to a
   different value, a different approver, or the item re-litigated with new
   options — in any cell or prose. Leaving a resolved-upstream row `—`/OPEN is an
   omission, not a re-decision (see invariant 8).
4. **Session-ready preparation.** Each open item carries grounded context
   (why it matters for this change, which slice/requirement it gates),
   2-3 option one-liners with implications drawn from the upstream analysis
   (design alternatives, risk notes), and an optional recommendation that is
   labeled as non-approval. When design already proposed a direction, the
   recommendation mirrors it and says so.
5. **Sound categorization.** Business/requirements ambiguity sits in BA /
   Requirements Questions; technical/operational decisions (call approaches,
   keys, redirects, guardrails) sit in Architecture / Tech Decisions; a routed
   developer-HOW deferral sits in Planning Routes, never in a question
   register. New (non-migrated) architecture items pass the filter: multiple
   viable options, material scope/risk/timeline impact, rework if deferred —
   no boilerplate (base branch, test framework, conventions the project owns).
6. **Route acknowledgement.** Every design Planning Deferral is migrated to
   Planning Routes carrying its deferred item, target phase, and owner
   verbatim; in the DRAFT its Route acknowledgement and Approved by are `—`
   and Status is `OPEN`. The candidate never decides the routed developer-HOW,
   and never fabricates a `ROUTED-TO-PLANNING` acknowledgement in the draft.
7. **Back-link presence.** Every migrated or raised row records an Origin
   tracing to the upstream artifact section or decision (or grounding fact)
   that raised it.
8. **Resolved-upstream discipline.** An item the CONTEXT records as decided
   upstream with a named human approver should be migrated carrying that
   Decision, that Approved by, and its Origin, with a settled Status. Leaving its
   Decision/Approved by `—`/OPEN is an omission (`missing_evidence`), NOT a
   violation of this invariant — a blank cell re-decides nothing. The invariant
   is violated ONLY when the draft ACTIVELY re-decides the item: a Decision or
   Approved by filled with a value DIFFERENT from the upstream record, a flipped
   disposition, or the item re-litigated with new options.
9. **Offer put up for sign-off, unchanged.** O1 summarizes the Requirements
   Lock boundary exactly as proposal.md locks it; O2 names the proposal's
   slice set as the slicing specs/stories will consume. Changing, extending,
   or trimming the offer here is re-deciding scope.
10. **No fabrication.** No invented items, questions, owners, humans,
    statuses, severities, tracker refs, or identifiers absent from CONTEXT.
11. **Template & structure conformance.** Template section order and headings
    (Executive Summary, Grooming Source Inventory, Prerequisites & Access,
    BA / Requirements Questions, Architecture / Tech Decisions, Planning
    Routes, Change Request Decisions, Scope & Offer Approval, Grooming
    Approval Gate); template table columns including the Origin column;
    explicit `None` rows for empty registers; the DRAFT status line under the
    title. A deterministic code gate already enforces exact section presence,
    order, and table columns against the golden; score this dimension for
    heading fidelity only.

## Forbidden (each is a critical finding when present)

- A fabricated approval: an actual FILLED approval artifact for which CONTEXT
  records no explicit human answer — a non-`—` `Decision`, a non-`—`
  `Approved by`, an `APPROVED`/`DEFERRED-APPROVED`/`ROUTED-TO-PLANNING` `Status`,
  or a checked gate checkbox — including "Approved by: Agent/LLM/Cursor" and
  invented approver names. A `—`-celled row in the `OPEN` or `BLOCKED` draft
  state fills no approval and can never be this.
- A dropped upstream item: an open upstream Q#, a proposal CR row, an External
  Dependencies & Access row, or a design Planning Deferral (PD#) absent from
  the ledger; a missing O1 or O2 row.
- An answered or closed question: a Decision cell, Status, or prose that
  settles an item the sources leave open (a chosen redirect type, a chosen
  call approach, a confirmed account) — settling upstream facts is the
  session's job, not the draft's.
- A decided route: a Planning Routes row whose deferred developer-HOW is
  resolved in the draft (a chosen cache TTL, a chosen bundle-load mechanism),
  rather than acknowledged as a route to planning.
- A re-decided upstream fact: a changed CR proposed disposition, a flipped
  severity, a RE-DECIDED resolved-upstream item (its Decision or Approved by
  filled with a value DIFFERENT from the upstream record — a blank/OPEN cell is
  never this), an offer row (O1/O2) that alters the lock boundary or slice set,
  or a factual status rewritten optimistically in the "Current status (from
  sources)" cell (e.g. `unconfirmed` -> `have`). The DRAFT workflow `Status`
  column (`OPEN`/`BLOCKED`) is NOT a factual-status cell: an `OPEN`-vs-`BLOCKED`
  mismatch there is a faithful-inheritance warning, never this critical (see the
  approval-state scan).
- Any fabricated identifier, item, owner, human, status, or tracker ref not
  grounded in the CONTEXT.
- Downstream anticipation: generating, quoting, or promising specs/FSD/ISD/
  stories content, or marking the pipeline ready to proceed while items are
  open.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE
place.

- Artifact presence (run FIRST): if the candidate is not a grooming.md
  working copy at all — a refusal or STOP message, a status report, planning
  notes, or an empty document — record the critical finding `no_artifact`,
  score EVERY dimension 0.0, and stop. A missing-upstream complaint is never
  a valid candidate: the upstream artifacts are given in CONTEXT.
- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A
  difference from GOLDEN, or something GOLDEN omits, is NEVER by itself a
  finding. Every finding MUST quote the candidate text AND the STANDARD rule
  or CONTEXT fact it violates — especially for option wording, context
  phrasing, recommendation choice, Origin phrasing, and summary style, which
  are open-world.
- Closed-world vs open-world. Identifiers (Q#/CR#/P#/PD#/O#/S#/RQ),
  severities, statuses, proposed dispositions, routed deferrals, owners, and
  tracker ids are CLOSED-world: they must trace to CONTEXT. Context cells,
  option phrasing, implications, recommendations, Origin phrasing, and the
  Executive Summary are OPEN-world synthesis: judge them for grounded,
  faithful compression, not for verbatim presence.
- ENUMERATE gate (closed rule — run BEFORE scoring `migration_completeness`):
  from THIS flow's CONTEXT enumerate the closed set of upstream registers —
  (1) the still-open Q#s across discovery/design/proposal (dedupe by Q#),
  (2) the proposal CR rows, (3) the External Dependencies & Access rows,
  (4) the design Planning Deferrals (PD#) rows. Those four registers plus the
  offer rows O1/O2 are the ENTIRE migration set; design's settled `Architecture
  Decisions` (D#) are NOT a register — they enter grooming only as an Origin
  reference, never as their own decision row (a candidate decision row that
  re-states a design architecture D# is an out-of-set item, judged under
  `no_fabrication`, and a fabricated approval if it also fills an approval).
  Diff the candidate against that inventory. A missing item is a critical; a
  duplicate row for the same
  Q#/CR#/P#/PD# is a warning; a new item is judged by grounding (a genuine
  grounded gap with continued numbering is ALLOWED; an ungrounded or
  boilerplate item is a finding under `no_fabrication` or a warning per the
  arch filter). When a register is empty upstream (no CR candidates, no
  planning deferrals), the explicit `None` row is the correct migration.
- PROVE-MEMBERSHIP gate (closed rule — run BEFORE scoring
  `categorization_and_severity` and `route_acknowledgement`): for each
  candidate row, confirm it belongs in its register — a business/requirements
  question in BA, a technical/operational decision in Architecture, a routed
  developer-HOW deferral in Planning Routes, a change request in Change Request
  Decisions. A routed deferral parked in a question register, or a question
  parked in Planning Routes, is a mis-route.
- Approval-state scan (closed rule — run BEFORE scoring
  `approval_discipline`): scan every `Decision` and `Approved by` cell,
  every `Status` cell, and every gate checkbox. PROVE-MEMBERSHIP before firing
  a fabricated-approval critical: you MUST quote the candidate's actual
  approval artifact — a FILLED (non-`—`) `Decision`, a FILLED (non-`—`)
  `Approved by`, an `APPROVED`/`DEFERRED-APPROVED`/`ROUTED-TO-PLANNING`
  `Status`, or a checked `[x]` gate box — for which the CONTEXT records no human
  answer. Absent such a filled artifact there is nothing to fabricate: STOP, do
  not fire. The two recorded exceptions (a user answer, a resolved-upstream
  decision) carry their filled cells legitimately. `OPEN` and `BLOCKED` are the
  two DRAFT workflow states, NOT approvals — neither can ever be a fabricated
  approval, and a `—`-celled row in either state is the REQUIRED draft state
  (flagging it as incomplete, or as an approval, is wrong). Status-word mismatch
  is not an approval issue: an item left `OPEN` where a source blocker would
  warrant `BLOCKED` (or `BLOCKED` written without a source blocker) is at most a
  faithful-inheritance WARNING — the `—` cells approve nothing and the blocker
  fact still sits in the item's Current-status cell — never a critical and never
  a fabricated approval. `BLOCKED` grounded in a source blocker (e.g. an
  unconfirmed credential) is correct handling, not pessimism.
- RESOLVED-UPSTREAM scan (run BEFORE scoring `resolved_upstream_discipline`):
  from CONTEXT identify items already decided upstream with a named human
  approver (e.g. a CR declined at a review). A resolved-upstream item is a
  MEMBER of one of grooming's own migratable tracks — an open Q#/CR#/P# that
  CONTEXT records as CLOSED at a review by a NAMED human. Design's settled
  Architecture Decisions (the solution-shape D# entries in design's
  `Architecture Decisions` section — build-vs-reuse, single-vendor, key/quota
  posture, lazy-load, and the like) are design's OWNED how, NOT resolved-upstream
  items: grooming references a design architecture D# only as an Origin (of a Q#
  it raises or a PD# it routes) and NEVER carries it as its own filled grooming
  decision row. A candidate row that RE-STATES a design architecture decision
  with a FILLED Decision/`Approved by`/`APPROVED` (e.g. rows whose Origin points
  at "design Architecture Decisions") is an out-of-set fabrication, not faithful
  migration — route it to `approval_discipline` (fabricated approval) plus
  `no_fabrication` (invented decision row); do NOT clear it as a carry.
  A genuine resolved-upstream item should carry its Decision, Approved by, and
  Origin; carrying it is REQUIRED faithful migration (not a self-approval) — but
  CLEAR a filled row as legitimate ONLY after PROVE-MEMBERSHIP: quote the CONTEXT
  record that (1) closes THAT exact item at a review and (2) names the SAME human
  the candidate wrote in `Approved by`, that human appearing VERBATIM in CONTEXT.
  An `Approved by` name that does NOT appear verbatim anywhere in CONTEXT is a
  fabricated approval (`approval_discipline`) even when the row is dressed as a
  carried/resolved-upstream decision — an invented approver is never faithful
  migration. PROVE-MEMBERSHIP before firing a RE-DECISION critical: to record a
  `resolved_upstream_discipline` critical you MUST quote BOTH (1) the candidate's
  FILLED Decision or Approved by cell AND (2) the differing upstream value from
  CONTEXT — the two must be different NON-BLANK values (an active re-decision).
  If the candidate's cell is `—`/OPEN, it re-decides nothing: record
  `missing_evidence` and STOP — a blank cell can never be a re-decision or a
  re-opening. Only a filled cell that differs from the upstream record is the
  re-decision critical.
- BACK-LINK scan (run BEFORE scoring `back_link_presence`): each migrated or
  raised row should carry an Origin tracing to an upstream section/decision or
  a grounding fact. A missing or ungrounded Origin is a warning /
  `missing_evidence`, not a critical.
- Categorization is judged leniently for borderline items: a defensible
  routing of a question that is both requirements- and API-shaped is fine; an
  obviously wrong route (a pure business question in Architecture, a routed
  deferral filed as a grooming question) is a WARNING under
  `categorization_and_severity`, never a critical.
- One defect, one primary critical. Classify by the actual fault:
  - an identifier / item / owner / status not in CONTEXT -> `no_fabrication`;
  - a dropped upstream item, missing O-row, or missing PD# route ->
    `migration_completeness`;
  - a fabricated approval / answered question / checked box / decided route ->
    `approval_discipline`;
  - a re-decided resolved-upstream item (a FILLED cell differing from the
    upstream record) -> `resolved_upstream_discipline`; a resolved-upstream
    decision left `—`/OPEN -> `missing_evidence`, never a critical;
  - a workflow `Status` mismatch (`OPEN` where a source blocker warrants
    `BLOCKED`, or `BLOCKED` without a source blocker, cells still `—`) ->
    faithful-inheritance WARNING, never a critical and never a fabricated
    approval;
  - a changed disposition / severity / offer boundary -> judge where it
    lives: offer rows -> `offer_approval`, CR rows -> `migration_completeness`
    if dropped vs `approval_discipline` if settled, a SETTLED status
    (`APPROVED`/`DEFERRED-APPROVED`/`ROUTED-TO-PLANNING`) -> `approval_discipline`
    (an `OPEN`/`BLOCKED` workflow status is never this critical).
  Do not raise the same underlying claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension: (a) upstream
  working-copy anchors and project-context inbox anchors — the citation
  convention; (b) a labeled agent recommendation, including one mirroring
  the design's proposed direction — that is REQUIRED preparation, not
  pre-deciding; (c) the explicit `None` row in an empty register, with its
  explanation; (d) `BLOCKED` status grounded in a source blocker; (e) a Q#
  referenced from several Context cells (e.g. a key question gating two
  prerequisites) — cross-linking, not duplication; (f) a tracker reference
  reused verbatim from CONTEXT, including "no dedicated ticket" when the
  sources record none; (g) NICE-TO-HAVE severities kept NICE-TO-HAVE — do
  not demand escalation; (h) option lists that name concrete mechanisms the
  upstream analysis already names (a server proxy, a permanent redirect) —
  presenting the decision space is the artifact's job and settles nothing;
  (i) a PD# row acknowledging a route with Status `OPEN` and `—` route-ack in
  the draft — that is required prep, not pre-deciding; (j) a resolved-upstream
  row carrying its upstream Decision, named Approved by, and Origin in the
  draft — that is faithful migration, not self-approval, when CONTEXT records
  the upstream decision.
- Omission is not a critical — under ANY dimension — EXCEPT the migration
  law: a dropped upstream Q#/CR#/access row, a dropped design PD# route, or a
  missing O1/O2 IS a critical (`migration_completeness`), because completeness
  is this artifact's core contract. Everything else missing (a thin context
  cell, a single option where two exist upstream, an absent recommendation, a
  missing Origin) is `missing_evidence` or a warning.
- Completeness does not offset a violation, and fluency is not correctness.
  Inspect each identifier, status, approver cell, and checkbox on its own
  merits regardless of how polished the document is.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by
the dimension id below. For each dimension, 1.0 = fully satisfied with
evidence, ~0.5 = partially satisfied or minor gaps, 0.0 = violated or absent.

- `intent_and_readiness` — the DRAFT status line is present; the Executive
  Summary is reader-first and grounded: what upstream settled, what this
  session must settle (pending items by type, naming blockers, planning routes
  to acknowledge, any resolved-upstream item), and the explicit statement that
  nothing is approved yet; it implies no readiness to proceed.
- `migration_completeness` — the ENUMERATE gate holds: every open upstream Q#,
  CR row, access row, and design PD# lands exactly once with identifiers
  verbatim; O1/O2 present; nothing renumbered or duplicated; empty registers
  carry the explicit `None` row.
- `approval_discipline` — the zero-self-approval law holds: all Decision /
  Approved by cells `—`, statuses OPEN/BLOCKED only (grounded), every gate
  box unchecked; no answered, closed, or decided-route items anywhere, save
  the two recorded exceptions. Apply the approval-state scan before firing.
- `item_preparation` — each open item is session-ready: grounded context tying
  it to the slice/requirement it gates, 2-3 option one-liners with
  implications from the upstream analysis, and a labeled, grounded
  recommendation where one helps; prerequisites carry inherited factual
  statuses and named providers.
- `categorization_and_severity` — items sit in the right register by nature
  (proved-membership: BA vs Architecture vs Planning Routes vs Change
  Requests), severities inherited verbatim, new items pass the arch filter,
  explicit `None` rows explain where the items actually live.
- `route_acknowledgement` — every design Planning Deferral is migrated to
  Planning Routes with its deferred item/target/owner verbatim; the draft
  acknowledges the route (Status OPEN, route-ack `—`) without deciding the
  developer-HOW; when design defers nothing the explicit `None` row is present.
- `back_link_presence` — every migrated or raised row records an Origin
  tracing to the upstream section/decision or grounding fact that raised it.
- `offer_approval` — O1/O2 request sign-off of the lock boundary and slice
  set exactly as proposal.md states them, with no re-decided scope.
- `no_fabrication` — every identifier, item, owner, status, severity, routed
  deferral, and tracker reference traces to the CONTEXT; nothing invented.
- `resolved_upstream_discipline` — an item CONTEXT records as decided upstream
  is carried with its upstream Decision, named Approved by, and Origin, and is
  not actively re-decided (a filled cell differing from upstream). Leaving it
  `—`/OPEN is a partial gap (score ~0.5, a `missing_evidence` omission), not a 0
  and not a critical; an active re-decision is a 0. When the flow has no such
  item this dimension is fully satisfied.
- `structure_conformance` — template section order, headings (including
  Planning Routes), table columns (including Origin), DRAFT line, and explicit
  `None` rows.

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it:
  if any item in `critical_findings` is present, `score` MUST NOT exceed
  0.5.
- `pass` is true only when `score >= threshold` (given in the user message)
  AND `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or of invariants
  1-3 / 6 / 8-10 (a dropped upstream item incl. a PD# route, a fabricated
  approval, an answered / re-decided item, a decided route, a changed offer,
  fabrication). A resolved-upstream item counts here ONLY when ACTIVELY
  re-decided (a filled cell differing from the upstream record); a
  resolved-upstream row merely left `—`/OPEN is `missing_evidence`, never a
  critical. These block acceptance.
- **warnings** — quality issues that should be fixed but do not make the
  ledger wrong (e.g. a duplicate row for one Q#, a mis-routed borderline
  item, a boilerplate-ish new question, a thin context cell, a missing or
  ungrounded Origin, a heading slip).
- **missing_evidence** — expected content the CONTEXT implies but the
  candidate omits without breaking the migration law (e.g. an option the
  design's alternatives clearly offer, a missing recommendation where the
  design proposed a direction, an absent S#/RQ cross-link in a prerequisite
  row, a resolved-upstream decision left OPEN). Record it ONLY here: it MUST
  NOT appear in `critical_findings` and MUST NOT cap the score.

Always ground findings in concrete evidence. Every critical finding MUST
quote a string that appears VERBATIM in the candidate. Before recording a
critical, locate that exact string (the table row, the cell, the checkbox
line) in the candidate text; if it is not literally present, DROP the finding
— do not infer, paraphrase, or assume a row or section exists. For a
missing-item finding, name the specific CONTEXT fact the candidate omits. A
finding whose evidence you cannot copy verbatim from the candidate is a
hallucination, not a violation.
