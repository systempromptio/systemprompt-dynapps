# Grooming contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
Grooming is the human decision ledger and approval GATE produced after
proposal and before specs. It is the first point discovery, design, and
proposal CONVERGE: it gathers everything the pre-grooming pipeline left
open — their Open Questions, the proposal's Change Request dispositions
and build prerequisites, the design's Planning Deferrals, and the offer
itself — into one session-ready checklist a named human works through.
The agent PREPARES the ledger; humans DECIDE in the session. Grooming is
the scoped source of truth for the DECISION/APPROVAL layer only — which
option was chosen and by whom. It does not own scope (the proposal's
lock), architecture (design), or behaviour (specs), it never re-decides
a fact it inherits, and it routes what another phase owns rather than
deciding it. There is no functional-vs-integration branching here: one
contract for every change; only the migrated content differs.

AUTHORITY ORDER:
1. The reconciled upstream artifacts — discovery.md, design.md, and
   proposal.md — and the source evidence they cite are the factual
   source of truth. Grooming inherits their facts verbatim.
2. This `grooming` instruction is the artifact contract: follow it for
   migration, approval discipline, categorization, routing, and the
   quality bar.
3. The `grooming.md` template defines section order and table shape.
4. Project `config.rules.grooming`, workflow rules, and skills provide
   supporting context; they do not override this artifact contract.

DECISION KERNEL (each rule is a task to establish, not a slot to fill):
- Prepare, do not decide. The generated artifact is a DRAFT: every
  `Decision` and `Approved by` cell is `—`, every Status OPEN (or
  grounded BLOCKED), and every Grooming Approval Gate checkbox
  UNCHECKED. Two exceptions carry an upstream decision into the draft:
  an item the user explicitly answered in the current conversation, and
  a RESOLVED-UPSTREAM item an upstream already closed
  (RESOLVED-UPSTREAM RULES).
- Establish migration completeness by ENUMERATING THE CLOSED SET: walk
  every upstream register that can raise an item (INPUTS AND SOURCE
  POLICY) and for each candidate decide include-once or exclude, with a
  one-line reason; record each included item as its row, deduplicated by
  identifier. An item no register carries into this ledger never gets
  decided.
- Establish each item's home by PROVING MEMBERSHIP: test the item
  against the target register's definition; if it fails, it belongs in
  another register (QUESTION RULES, PLANNING ROUTE RULES, CR RULES).
  Route by nature; never park an item in the wrong register.
- Inherit facts verbatim: severities (CRITICAL / NICE-TO-HAVE), current
  factual statuses, proposed CR dispositions, and design's routed
  deferrals are copied, never rewritten, answered, closed, or
  re-scoped. A recommendation is not a decision.
- Back-link every item: each migrated or raised row records the upstream
  artifact section or decision that raised it as its Origin. A row with
  no traceable origin is ungrounded.
- Route, do not decide, what another phase owns: design's Planning
  Deferrals are acknowledged as ROUTED-TO-PLANNING (PLANNING ROUTE
  RULES); the developer-HOW itself is settled at planning, not here.
- Reference design's settled architecture, do not adopt it as a decision.
  Only an upstream REGISTER raises a migratable item — open Q#s, proposal
  CR rows, External Dependencies & Access rows (P#), and design Planning
  Deferrals (PD#). Design's settled Architecture Decisions (its `D#` set)
  are design-owned HOW, NOT a register: a settled `D#` enters ONLY as the
  Origin of a Q# or PD# it grounds, and is NEVER carried as its own row
  that restates the `D#` with a filled Decision / Approved by / APPROVED.
  A genuine open decision the session must confirm is raised as a Q# with
  Decision `—`, citing the `D#` as Origin; it never asserts the design's
  choice as already approved.
- "Approved by" must name a real human or human role from the session
  ("BA", "SA lead", "client PO"). "Agent", "LLM", "Cursor", blank, or
  implied approval is invalid. A deferral still requires a named human
  and a concrete constraint or owner.

PIPELINE (follow each referenced section; do not improvise content):
1. Clear the ENTRY GATE and read discovery.md, design.md, and
   proposal.md fully; take the change identity verbatim from discovery
   (INPUTS AND SOURCE POLICY).
2. Fill the Grooming Source Inventory under its FIXED header row —
   reproduce the three column names `Input | Reference | Used for`
   VERBATIM as template scaffold (do NOT relabel `Used for` to
   `Purpose`, `Use`, `Notes`, or any synonym): one row per upstream
   artifact (discovery, design, proposal), the tracker reference from
   the sources, and any client FSD/ISD source pages discovery
   recorded — never invent one. Every template table header is fixed
   scaffold: copy each section's column names exactly, never paraphrase
   a heading.
3. Enumerate the closed set of upstream registers and migrate each
   still-open item to its home, back-linked to its origin (DECISION
   KERNEL; INPUTS AND SOURCE POLICY):
   - proposal External Dependencies & Access -> Prerequisites & Access
     as P1..Pn (PREREQUISITE RULES);
   - business/requirements Q#s -> BA / Requirements Questions
     (QUESTION RULES);
   - technical/operational Q#s and design-raised undecided decisions ->
     Architecture / Tech Decisions (QUESTION RULES, ARCH FILTER);
   - design Planning Deferrals -> Planning Routes as PD# route
     acknowledgements (PLANNING ROUTE RULES);
   - proposal Change Request rows -> Change Request Decisions (CR RULES,
     RESOLVED-UPSTREAM RULES).
4. Put the offer up for sign-off as O1/O2 in Scope & Offer Approval
   (OFFER RULES).
5. Write the Executive Summary from the completed sections
   (EXECUTIVE SUMMARY).
6. Emit the Grooming Approval Gate with every box unchecked
   (GATE RULES).
7. In live use, run the grooming session per SESSION RULES; the eval /
   generation step ends at the prepared draft.

ENTRY GATE (blocking — evaluate BEFORE authoring anything):
- discovery.md, design.md, and proposal.md must exist as completed
  upstream artifacts for this change, with the Proposal Gate
  self-checked. If any is missing, STOP and produce it first.

INPUTS AND SOURCE POLICY:
- Read all three upstream artifacts fully; each carries registers that
  raise items. The sections named below are the CLOSED SET to enumerate
  for migration completeness — walk every one, wherever it sits in the
  file, and decide include/exclude per candidate:
  - discovery.md: Open Questions (Q#s with severities and "why unclear"
    context), Change Request Candidates (the origin of CR rows), source
    anchors for citations.
  - design.md: Open Questions (usually the same Q#s, refined),
    Architecture Decisions marked proposed / raised for grooming
    (undecided directions the session must settle), Risks / Mitigations
    (decision context and alternatives), and Planning Deferrals (the
    PD# developer-HOW items design routed to technical-design planning —
    the only source of Planning Routes).
  - proposal.md: the Requirements Lock and Delivery Breakdown (the offer
    O1/O2 summarizes), Change Request Dispositions (the CR rows with
    proposed dispositions, including any already decided upstream),
    External Dependencies & Access (the P# rows with factual statuses),
    proposal-stage Open Questions.
- Deduplicate by identifier: a question discovery raised and design
  refined is ONE row carrying the richest grounded context. Preserve
  Q#/CR#/PD#/S#/RQ identifiers verbatim; number prerequisites P1..Pn in
  register order and offer rows O1/O2.
- Determine each row's Origin — the upstream artifact section or
  decision that raised it (for a session-raised item, the grounded fact
  it rests on) — and record it in the Origin column with the same
  anchors the upstream artifacts use. Never invent tracker ids, page
  ids, owners, or humans.
- A NEW item may be added only for a genuine grounded gap found while
  preparing (a decision the sources leave open that no upstream register
  covers); continue the upstream numbering and back-link the grounding.
  Never add boilerplate items.
- Do NOT read or anticipate downstream artifacts (specs, FSD/ISD,
  stories); they do not exist yet at this pipeline position.

EXECUTIVE SUMMARY:
- Keep the DRAFT status line under the title, before this section:
  "Status: **DRAFT — human approval required.**" plus one sentence
  that the agent prepared the ledger and no item is decided until a
  named human approves it.
- Reader-first, written after the detailed sections: what upstream
  settled (one sentence), what this session must settle — the pending
  items by type with blockers named, the planning routes to
  acknowledge, and any resolved-upstream item migrated for the record —
  and the explicit statement that nothing is approved yet and the
  pipeline stays blocked until the gate passes.
- Do not duplicate the registers (no full Q#/CR#/P#/PD# inventories) and
  do not imply readiness to proceed.

PREREQUISITE RULES (Prerequisites & Access):
- Establish, for each proposal External Dependencies & Access row, what
  the build needs before it can start: what is needed, which
  slices/requirements it gates (from the register's "Needed for"), the
  owner/provider, and the CURRENT factual status inherited verbatim
  ("to be delivered", "have (already integrated)", "unconfirmed (Q#)",
  "client-owned"). Record one P# row with its Origin as the proposal
  section that raised it.
- Determine the grooming Status: OPEN by default; BLOCKED only where a
  source records the item as blocking (an unconfirmed credential the
  build cannot start without). Missing access, credentials, design
  assets, CMS setup, API keys, environments, or contracts stay
  OPEN/BLOCKED unless a named human explicitly approves a deferral —
  never "ready with stubs" by inference.
- Cross-link, do not duplicate: a prerequisite gated by an open Q#
  references that Q# in its context; the question itself lives once,
  in its own register.

QUESTION RULES (BA / Requirements Questions and Architecture / Tech
Decisions):
- Establish each question's home by PROVING MEMBERSHIP against the
  register definition: a question about what the experience or scope
  should be is business/requirements -> BA / Requirements Questions; a
  decision about how it is built or operated (call approaches, keys,
  redirects, quotas, guardrails, environments) is technical/operational
  -> Architecture / Tech Decisions. If a candidate is a routed
  developer-HOW deferral, it is not a grooming question — it belongs in
  Planning Routes (PLANNING ROUTE RULES). Either register may hold the
  explicit `None` row naming where the items actually live.
- Each row records: the Q# and severity verbatim; its Origin; the
  question; grounded context (which slice/requirement/prerequisite it
  gates); 2-3 option one-liners with implications drawn from the
  upstream analysis (design alternatives, risk notes); an optional
  recommendation labeled as the agent's, mirroring the design's
  proposed direction when one exists.
- Presenting the decision space settles nothing: options may name the
  concrete mechanisms the sources name (server proxy, permanent
  redirect); the Decision cell stays `—`.
- ARCH FILTER for NEW architecture items only (a migrated Q# always
  stays, including one grounded in a design `D#`; a settled design `D#`
  is itself NOT an item — it appears only as a Q#'s Origin, never as a
  row that restates it with a filled Decision): include one only if all
  three hold — (a) multiple viable options exist, (b) the choice
  meaningfully affects scope, risk, or timeline, (c) deferring it to
  implementation would cause rework. No boilerplate ("which base
  branch?", "which testing framework?" — the project owns those
  conventions). An empty register with the explicit `None` row beats
  padded questions.

PLANNING ROUTE RULES (Planning Routes):
- Establish, for each design Planning Deferrals (PD#) row, the route to
  acknowledge: the deferred developer-HOW item, its target phase, and
  its owner, all inherited verbatim from design; record one PD# row with
  its Origin as the design section/decision that raised it.
- Grooming acknowledges the ROUTE, not the developer-HOW: the draft
  leaves Route acknowledgement and Approved by `—` and Status OPEN; the
  session sets Status ROUTED-TO-PLANNING with a named human when it
  confirms the route. Never decide the deferred item here — that is
  technical-design's planning.
- ALWAYS a table. When design defers nothing (Planning Deferrals: None),
  carry the explicit `None` row; do not invent deferrals.

CR RULES (Change Request Decisions):
- Establish, for each proposal Change Request row, the disposition to
  decide: CR#, its Origin (the discovery Change Request Candidate),
  description, impact, and the PROPOSED disposition inherited verbatim.
  An undecided CR ships Decision `—` and Status OPEN; the session
  approves, absorbs, or descopes. A CR the upstream already decided is a
  RESOLVED-UPSTREAM item (RESOLVED-UPSTREAM RULES).
- ALWAYS a table. When the proposal carries the explicit `None` row,
  carry it with its reason.

RESOLVED-UPSTREAM RULES (any migrated item an upstream already closed):
- Determine whether a migrated item arrives already decided: an upstream
  artifact records a disposition AND a named human approver for it (a CR
  declined at the discovery review, a question answered and signed off
  upstream). If both hold, it is RESOLVED-UPSTREAM.
- Record it for the record, do not re-open it: carry its upstream
  Decision, its named Approved by, and its Origin (the artifact and
  review that closed it) into the row, with a settled Status. This is
  the one migrated case whose Decision/Approved by are filled in the
  DRAFT. Never blank it back to OPEN and never re-decide it.

OFFER RULES (Scope & Offer Approval):
- Exactly two rows, each with its Origin (the proposal section it
  projects): O1 — the Requirements Lock boundary summarized as
  proposal.md locks it (in-scope set, deferred rows, nothing else
  introduced); O2 — the Delivery Breakdown slice set (S#s and what
  approving them means: the slicing specs and stories will consume).
- Summarize, never restate or alter: changing the lock boundary or the
  slice set here is re-deciding scope, which grooming never does.

GATE RULES (Grooming Approval Gate):
- Emit the gate after the body with EVERY box unchecked ([ ]) — the
  prepared draft has nothing approved by definition. Boxes are checked
  only during the live session as items resolve.
- The gate items are the downstream contract: before specs, FSD/ISD,
  or stories are generated, all of the following must be true in
  grooming.md:
  - every prerequisite (P#) is APPROVED, or DEFERRED-APPROVED with a
    named human approver and an explicit constraint;
  - every BA / requirements question (Q#) is APPROVED or
    DEFERRED-APPROVED by a named human;
  - every architecture / tech decision is APPROVED or
    DEFERRED-APPROVED by a named human, or a named human confirms none
    is required;
  - every planning route (PD#) carried from design is acknowledged as
    ROUTED-TO-PLANNING by a named human, or the register is empty;
  - every Change Request (CR#) has a disposition decided by a named
    human;
  - the Requirements Lock and Delivery Breakdown (O1, O2) are approved
    by a named human;
  - every migrated or raised item back-links to the upstream artifact
    section or decision that raised it;
  - approved decisions are written back into proposal.md / design.md
    where they change scope or approach;
  - no approver is "Agent", "LLM", "Cursor", blank, or implied.
- If any gate item is not satisfied, downstream generation STOPS at
  this artifact.

SESSION RULES (live workflow, after the draft is generated):
- Start a chat-based grooming session immediately after generating the
  draft: ask ONE item at a time — brief context, why it matters, the
  options with one-line implications, the labeled recommendation —
  then WAIT for the human answer before the next item. Do not use the
  AskUserQuestion tool for grooming and do not dump the full
  questionnaire at once.
- Record each answer as it lands: fill Decision, Approved by (real
  name/role), and Status (including ROUTED-TO-PLANNING for an
  acknowledged route); check a gate box only when its item set is fully
  resolved.
- Write approved decisions back into proposal.md / design.md where
  they change scope or approach, keeping the artifacts consistent.

OUTPUT CONTRACT:
- Emit every section in the `grooming.md` template order and table
  shape, with the DRAFT status line under the title; do not add, drop,
  or reorder sections (the template is canon). A register with nothing
  to carry holds its explicit `None` row instead of being dropped.

SELF-CHECK BEFORE OUTPUT:
- Change identity taken verbatim from discovery; every
  `openspec/changes/<name>/…` anchor uses it.
- Migration completeness holds: the closed set of upstream registers was
  enumerated and every still-open Q# (deduped), every proposal CR row,
  every External Dependencies & Access row (as P#), every design
  Planning Deferral (as PD#), and O1/O2 are present exactly once;
  identifiers verbatim.
- Membership holds: every item proves it belongs in its register (BA vs
  Architecture vs Planning Routes vs Change Requests); no routed
  deferral sits in a question register and no question sits in Planning
  Routes; no Architecture / Tech Decisions row restates a settled design
  `D#` with a filled Decision / Approved by — a settled `D#` appears only
  as a Q#'s Origin.
- Back-links hold: every migrated or raised row records the origin that
  raised it.
- Zero self-approval, with the two exceptions only: all Decision /
  Approved by cells are `—`, all statuses OPEN or grounded BLOCKED,
  every gate box unchecked — except an item the user answered in the
  current conversation and a resolved-upstream item, which carry their
  upstream decision and named approver.
- Planning Routes acknowledge the route (Status OPEN in the draft),
  never decide the developer-HOW; when design defers nothing the
  explicit `None` row is present.
- Severities, factual statuses, proposed dispositions, and routed
  deferrals match upstream verbatim; no optimistic rewrites.
- Every in-session item is session-ready: grounded context, options with
  implications, labeled recommendations only.
- No invented items, owners, humans, statuses, tracker refs, or
  identifiers; no boilerplate architecture questions.
- The Executive Summary states that nothing is approved and names the
  blockers; it does not imply readiness.
