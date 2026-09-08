# Proposal contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
Proposal is the BA/SA delivery offer produced after discovery and design
and before grooming. It converts the reconciled discovery trace and the
design's solution shape into delivery-space: WHY the change is needed,
the Requirements Lock downstream artifacts consume, proposed Change
Request dispositions, the Delivery Breakdown — the offer's core: the
in-scope requirements sliced into per-slice blocks carrying a user story
and NEAR-FINAL acceptance criteria that grooming approves and
stories/specs/FSD consume (the proposal is the single source of truth the
FSD later PROJECTS read-only) — the external dependencies and access the
build needs, and the delivery approach. It proposes and locks; it does not approve (grooming
owns human sign-off), does not re-decide dispositions (discovery owns
them) or architecture (design owns it), and does not write behaviour
contracts (specs own the exhaustive scenarios) or implementation plans
(technical-design).

AUTHORITY ORDER:
1. The reconciled upstream artifacts — discovery.md and design.md — and
   the source evidence they cite are the factual source of truth.
2. This `proposal` instruction is the artifact contract: follow it for
   the lock, dispositions, verification, gates, and quality bar.
3. The `proposal.md` template defines section order and table shape.
4. Project `config.rules.proposal`, workflow rules, and skills provide
   supporting context; they do not override this artifact contract.

DECISION KERNEL:
- Proposal is produced for every change once the ENTRY GATE clears;
  discovery.md and design.md are its required inputs.
- The Requirements Lock is a freeze, not a re-decision: every discovery
  Requirements Trace row appears exactly once with its disposition
  inherited verbatim. A lock that drops, adds, or re-decides a row is
  wrong.
- Scope absent from discovery is never introduced here. A new ask found
  at proposal time goes back to discovery as a Change Request candidate
  first.
- Change Request dispositions are PROPOSED for grooming; none is approved
  here, and no human sign-off is implied.
- The Delivery Breakdown obeys the coverage law as a BIJECTION: every
  in-scope lock row is delivered by exactly one slice and every slice
  delivers exactly one in-scope lock row (one WBS in `Covers`) — no
  grouping, no splitting — and no slice carries scope the lock does not
  hold as `in scope` (deferred, out-of-scope, and CR scope never enter
  the offer). Slices are the decomposition downstream consumes: grooming
  approves them, stories.md materializes them 1:1, specs formalize their
  seeds.
- STORY-LOOKUP LAW: slice stories are NOT authored here. Each slice's Story
  is a deterministic VERBATIM lookup — take the slice's `Covers (WBS)`, find
  the discovery Requirements Trace row with that WBS, and copy that row's
  `Requirement` cell byte-for-byte (persona, wording, punctuation intact).
  Discovery owns story wording (it mirrors the signed SRD); the proposal
  only transcribes it, identically for FUNCTIONAL and INTEGRATION scope.
- PROJECT, DON'T REDISCOVER: like the story, the Acceptance Criteria are
  PROJECTED from resolved upstream — the obligation from discovery
  (`Requirement Details` / `Notes`) and the baseline from design
  (`Gap Analysis` / `Base Configuration & Cross-Cutting` / `Architecture
  Decisions` / `Integration Specification`) — never re-derived from the raw
  SRD: the disposition is discovery's, the solution shape is design's, and the
  raw SRD is cited evidence only. The Delivery Breakdown AC rule is the home.
- Acceptance Criteria are authored NEAR-FINAL per slice — the near-final bar
  is on what YOU produce (the client-facing contract the FSD projects
  read-only), NOT on the input. DETERMINE each criterion by PROJECTING the
  resolved upstream obligation, never by re-mining the raw SRD: the seed is
  the discovery Requirements Trace `Requirement Details` and `Notes` for this
  slice's `Covers (WBS)` — the atomised obligation and its ownership boundary;
  the raw SRD is cited evidence only, never the AC source. ELABORATE that
  upstream atom into this slice's own criteria — never copy a `Requirement
  Details` line as a finished criterion. Operationally, elaboration is a
  BASELINE-DELTA test: read the baseline from design — `Gap Analysis` (the
  current state and gap class for this WBS), `Base Configuration &
  Cross-Cutting`, the `Architecture Decisions`, and (integration slice) the
  `Integration Specification` — and, for EACH criterion, make it one of two
  things — (a) a NET-NEW obligation (gap class `rebuild`, no platform
  primitive exists) stated as this capability's own observable contract, or
  (b) a DELTA (gap class `config` / `adapt`) that names the platform
  primitive it builds on / adapts and states what THIS capability adds on top
  of it; a `remove` gap yields a migration / boundary criterion naming where
  the behaviour goes. A criterion that merely re-states or re-numbers a
  `Requirement Details` line — adding no net-new obligation and no delta
  against the baseline — is PARROTING; rewrite or drop it. PHRASING
  DISCIPLINE (makes elaboration verifiable, not cosmetic): open each
  criterion with a verb that names what THIS capability provides or
  guarantees as its own observable contract; a clause lifted from the seed
  may appear ONLY as the object of that verb — never as the criterion's own
  leading verb, and never carried with the seed's mechanism token. The
  platform primitive named in (b) is a design-stated baseline fact (from
  `Gap Analysis` / `Base Configuration` / an `Architecture Decision` — e.g. a
  lazy-load convention, server tier, analytics sink, or existing platform
  capability), NOT the owned vendor — vendor mechanism still stays out of the
  AC per the token test below. Shape: numbered SHALL lists under the
  re-usability marker groups the template's Acceptance Criteria comment
  defines; `## Re-usability Legend` prints the whole closed key whichever
  markers those groups use. Still offer-level,
  NOT the exhaustive behaviour canon: no Given/When/Then blocks (specs own
  those). An undecided item stays undecided with its Q# reference.
- Every External Dependencies & Access row must trace to the upstream
  artifacts or cited sources; an item that cannot be verified is either
  dropped or marked `UNVERIFIED — confirm before proposal merge`.
  Phantom dependencies propagate into grooming and tasks.
- Weight the proposal to the inherited delivery scope: a functional
  proposal slices the shopper experience in delivery detail; an
  integration proposal stays thin — few slices addressed to the internal
  consumer, pointing at the design's Integration Specification.

PIPELINE (follow each referenced section; do not improvise content):
1. Clear the ENTRY GATE, read discovery.md and design.md fully, and take
   the change identity, delivery scope, and targets verbatim from
   discovery (INPUTS AND SOURCE POLICY). Do not re-derive them.
2. Record the tracker reference from the sources; never invent one.
3. Fill the Requirements Lock from the discovery Requirements Trace,
   grouped by WBS (LOCK RULES).
4. Carry each discovery Change Request candidate into Change Request
   Dispositions with a proposed disposition (CR RULES).
5. Slice the in-scope lock rows into the Delivery Breakdown
   (DELIVERY BREAKDOWN RULES), then emit Re-usability Legend as the
   template's COMPLETE canonical bullet list, in template order and wording
   verbatim — never trimmed to the markers those slices used, nothing
   re-authored.
6. State Out of Scope & Boundaries (BOUNDARY RULES).
7. Compile External Dependencies & Access (DEPENDENCY AND ACCESS RULES).
8. Write the Approach weighted to the delivery scope (APPROACH RULES).
9. Record NEW proposal-stage Open Questions only (OPEN QUESTION RULES).
10. DERIVE THE READER INTERFACE LAST — from the FINISHED body, never from
    intentions. The Executive Summary and Document Consumption Guide are
    placed first in the file but derived here, after every detail section is
    final, so each is a copy of what the body says NOW (a claim the sections
    no longer support is wrong however well written):
    a. Build the Document Consumption Guide (DOCUMENT CONSUMPTION GUIDE ROWS):
       walk the `##` sections the body carries in document order, emit one row
       per section EXCEPT the three that cannot map (Executive Summary — the
       reader map; this guide itself; Proposal Gate), and read each row's Key
       data off the section it names — never carried over from an earlier draft.
    b. Answer the Executive Summary reader map (EXECUTIVE SUMMARY) from those
       same finished sections: what this offer settles, what it hands
       downstream, and what stays open — by subject and count, re-listing no
       section's rows.
11. Set the Proposal Gate checkboxes (PROPOSAL GATE).

ENTRY GATE (blocking — evaluate BEFORE authoring anything):
- The Discovery Gate must be clean: no CRITICAL Open Question still Open
  without recorded user sign-off. If it is not clean, STOP; do not author
  proposal.md.
- design.md must be available as the completed upstream artifact for this
  change. If it is missing, STOP and produce design first; do not author
  proposal.md from discovery alone.

INPUTS AND SOURCE POLICY:
- Read discovery.md as the resolved WHAT and treat each of its sections
  as a typed input — inherit its dispositions, do not re-decide them.
  The sections below are the TYPICAL discovery shape, not a closed
  checklist: review the WHOLE file and map every section it actually
  carries (a case may add, rename, merge, or omit some); never limit
  yourself to the names here.
  - Requirements Trace: the row set and dispositions the Requirements
    Lock mirrors; its in-scope rows are the material the Delivery
    Breakdown slices. Preserve RQ/WBS identifiers verbatim. Each row's
    `Requirement` cell is the slice story source: the Delivery Breakdown
    joins by WBS (`Covers (WBS)` -> the trace row's `Requirement`) and
    copies that cell verbatim (STORY-LOOKUP LAW); it is never re-authored.
    The SAME row's `Requirement Details` and `Notes` cells are that slice's
    ACCEPTANCE-CRITERIA seed — the atomised obligation and its ownership
    boundary the AC elaborates (see the Delivery Breakdown AC rule); the raw
    SRD is cited evidence only, never the AC source.
  - Affected Capabilities / Delivery Scope & Targets: the owned change
    identity, delivery scope, FSD/ISD targets, and the capability names
    the breakdown's Capability column uses. Take them verbatim; the raw
    feature name is not the change identity.
  - Dependencies: the consumed capabilities/services that become the
    delivery-prerequisite rows of External Dependencies & Access for a
    functional change and the `Depends on` entries of the slices they
    block.
  - Change Request Candidates: the CR# set carried into Change Request
    Dispositions. Unsigned scope never enters the lock's in-scope rows.
  - Open Questions: already registered for grooming. Reference Q#s in
    Notes/Scope/Approach where they refine an item; do NOT re-raise them
    as proposal Open Questions and do NOT close them.
  - Cross-cutting Requirements: applicable product concerns the in-scope
    summary reflects (accessibility, SEO, privacy, i18n).
- Read design.md as the resolved solution shape:
  - Solution Overview / Architecture Decisions: the decisions the
    Approach restates at BA/SA level. Inherit them; do not re-open,
    re-decide, or contradict them.
  - Dependencies: operational prerequisites (integration scope: account,
    credentials, billing, sandbox/environment) that become External
    Dependencies & Access rows with their unresolved statuses preserved.
  - Integration Specification: for an integration proposal, the Approach
    points here instead of restating the service contract.
  - Open Questions / Risks: still-undecided items; keep them undecided.
- Do NOT read grooming.md or downstream artifacts; they do not exist yet
  at this pipeline position.
- If discovery records "No BRD", keep the proposal proportionate: a light
  lock over the rows discovery actually carries, no invented codes.
- Cite evidence with the same anchors the upstream artifacts use
  (project-context inbox paths, spec paths, working-copy paths, URLs,
  tracker ids). Never cite a source you did not use, and never invent
  tracker ids, branch names, page ids, or access items.

EXECUTIVE SUMMARY:
- Place it immediately after the title; derive it LAST (PIPELINE step 10b),
  from the finished sections, so it reflects final facts.
- It reads as a READER MAP, not a recap. After a reader-first opening (the
  problem/change context and one source-grounded business-value sentence),
  establish and record three things by SUBJECT and COUNT: (a) what this offer
  SETTLES — the locked scope as a slice count, the proposed CR dispositions,
  the recorded external dependencies; (b) what it HANDS DOWNSTREAM — the
  Delivery Breakdown as the read-only source the FSD/specs project, and who
  owns everything held Out of Scope; (c) what STAYS OPEN plus grooming
  readiness.
- Name subjects, not identifiers: re-list no detail table (no RQ/WBS/CR/Q/S
  inventories), and any count it states equals the size of that set in the
  body.
- Imply no approval for a pending decision or CR; readiness wording mirrors
  the Proposal Gate.

LOCK RULES (Requirements Lock):
- Enumerate every discovery Requirements Trace row BEFORE drafting and
  commit one ledger line per row: `RQ -> inherited disposition -> WBS
  group`. The emitted table is that ledger verbatim: every trace row
  appears exactly once, no row is added, none is dropped, no disposition
  changes, and the lock row count equals the trace row count.
  Cross-cutting rows (accessibility, i18n, SEO, privacy) are trace rows
  like any other — locked, and sliced when in scope; never dropped as
  "non-functional overhead".
- Group rows by WBS code; rows sharing a WBS sit together.
- `Requirement summary` is a short faithful summary of the source
  requirement, not new scope wording. Do not copy the full verbatim
  requirement text; discovery owns it.
- Dispositions use discovery's exact values: `in scope`, `out of scope`,
  `deferred`, `CR candidate`.
- Notes carry only grounded material inherited from discovery/design:
  consumed dependencies, Q#/CR# cross-links, the owning capability for
  out-of-scope rows, service-slice splits for integration scope. Use `—`
  when nothing material exists; never restate the requirement or invent
  notes.

CR RULES (Change Request Dispositions):
- One row per discovery Change Request candidate, preserving its CR#.
- Propose a disposition (`confirm with client`, `absorb`, `descope` — or
  carry discovery's proposed one) with Status `pending grooming
  approval`. Proposing is not approving: never mark a CR approved or
  absorbed as fact, and never move CR scope into in-scope lock rows or
  a Delivery Breakdown slice.
- ALWAYS a table. When discovery carries no CR candidates, emit the
  single explicit row `| None | — | — | — | <why> |`.

DELIVERY BREAKDOWN RULES:
- This is the offer's core: slice the locked in-scope rows into per-slice
  BLOCKS a human approves at grooming. Emit ONE block per in-scope lock
  row. The breakdown is a BIJECTION to the in-scope requirements:
  enumerate the lock's `in scope` rows BEFORE drafting and give EACH ONE
  its own block — every in-scope requirement is delivered by EXACTLY ONE
  block, and every block delivers EXACTLY ONE in-scope requirement (one
  WBS in `Covers (WBS)`). NEVER group (fold several requirements into one
  block) and NEVER split (spread one requirement across blocks); deferred
  / out-of-scope / CR scope never enters any block. The block count EQUALS
  the in-scope requirement count EQUALS the FSD in-scope block count — that
  1:1:1 correspondence is what makes stories.md AND the FSD deterministic
  projections (one block : one requirement : one story : one Acceptance
  Criteria block), and the scaffold script (`generate-functional-artifacts.mjs`, which
  projects the FSD + stories) ERRORs if it does not hold. If a user outcome seems to
  span two requirements, that is a signal the lock granularity is wrong —
  refine the requirement grain upstream (one requirement per deliverable
  outcome), do not group.
- Block shape — each slice is authored NEAR-FINAL (the proposal is the
  single source of truth the FSD later PROJECTS read-only, so Story and
  Acceptance Criteria are written to client-facing quality here, not as
  thin seeds):
  * Heading `### S# — <WBS> — <requirement name>` (H3 under the "## Delivery
    Breakdown" H2), where `<WBS>` is the covered requirement's clean WBS
    code and `<name>` is its short requirement name.
  * A machine-readable metadata list directly under the heading (a
    `- **key:** value` bullet list the downstream scripts parse):
    - `- **Covers (WBS):**` the ONE in-scope requirement's clean WBS code
      this slice delivers (e.g. `CHK.PAY`), verbatim — the SAME leaf code
      the FSD block is keyed by and the lock's WBS column holds, NOT an RQ
      ID, exactly one code (never a list). This is the stories/FSD join key
      (Covers WBS -> FSD block heading), so it MUST match exactly one FSD
      in-scope heading and the lock's WBS column.
    - `- **Capability:**` the owning capability name(s) from discovery's
      Affected Capabilities, verbatim.
    - `- **FSD:**` (FUNCTIONAL change) the target FSD working-copy path from
      discovery's Delivery Scope & Targets (e.g. `fsd/<feature>.md`) — the
      routing pointer the FSD scaffold uses. On an INTEGRATION change use
      `- **ISD:**` with the ISD working-copy path instead (an integration
      change has no FSD).
    - `- **Depends on:**` the External Dependencies & Access row(s) blocking
      this slice, or `—`. Same names as the register; no new items minted.
  * `#### User Story` (H4): the story per the Story rule below.
  * `#### Acceptance Criteria` (H4): the criteria per the Acceptance
    Criteria rule below.
- Story (STORY-LOOKUP LAW — a copy operation, not authoring): take this
  slice's `Covers (WBS)`, find the discovery Requirements Trace row with
  that WBS, and copy its `Requirement` cell VERBATIM — byte-for-byte,
  persona / wording / punctuation unchanged. The ONLY permitted edit is a
  trailing `[Q#]` marker flagging an open question. NO re-authoring,
  paraphrase, shortening, re-persona, or "normalization"; NO
  functional/integration distinction (both scopes copy the same cell); NO
  invented story. If that `Requirement` cell carries no "As a…" sentence,
  copy the cell as-is. One WBS : one trace row : one story — a slice covers
  a whole requirement, never a fraction, so there is never a sub-story to
  author; fix requirement grain upstream instead.
- Acceptance Criteria: the NEAR-FINAL, client-facing contract the FSD will
  project verbatim (CRITERION BAR below is the phrasing standard) —
  numbered SHALL lists grounded in the upstream artifacts, grouped under the
  `#####` RE-USABILITY MARKER headings the template's `Re-usability Legend`
  section defines (that legend is the closed marker set). The
  `## Re-usability Legend` section is a FIXED KEY, never a selection: print
  EVERY canonical bullet the template carries, in template order and wording
  VERBATIM — never drop a member because no slice uses its marker, and never
  re-author a gloss into case-specific prose (the gloss defines the marker, it
  never describes what this change does with it). Selection applies to the
  GROUPS instead: take ONLY the markers that apply and never pad an empty group; a
  slice whose criteria share one footing carries ONE group. Marker groups are
  the ONLY heading axis inside `#### Acceptance Criteria`: there is no
  device / surface / configuration axis, so device context sits INSIDE the
  criterion ("on mobile and tablet, …") and the administering actor is the
  criterion's SUBJECT ("Business users SHALL be able to …") instead of a
  heading. Never restate one criterion under two markers (duplication is
  fabrication). The
  AC is the OFFER-level contract, NOT the exhaustive behaviour canon: NO
  Given/When/Then Gherkin blocks (specs own the full scenario sets), no
  code tokens (incl. HTTP status codes — say "permanently redirected", not
  "HTTP 301"), no internal file/route/component wiring. Vendor naming
  branches on scope: a FUNCTIONAL slice's AC names NO consumed-dependency
  vendor at all (the consumed vendors sit in `Depends on` / External
  Dependencies & Access); an INTEGRATION slice may name the OWNED vendor at
  GOVERNANCE level only (key / quota / billing / privacy / domain
  restriction) but NEVER its SDK, API, endpoint,
  method, field, protocol, or call mechanism, and NEVER a vendor magnitude
  (an SLA, latency, or rate / quota number, incl. "rate-limited") — that
  contract stays thin and points at the design's Integration Specification.
  GOVERNANCE-vs-MECHANISM test (token-based): the FORBIDDEN tokens are a
  named vendor product / SDK proper noun ("Acme Payments SDK"), an HTTP verb
  or endpoint path, a method / field signature, a wire protocol, or a
  magnitude — never write those in an AC. The capability's OWN contract
  vocabulary is ENCOURAGED and is NOT a leak: "expose a callable / stable
  interface", "an entrypoint", "a render surface", "an authorization
  result", "a settlement status", "lazy-loadable", and governance
  ("domain-restricted key"). Write "expose a callable interface that
  authorizes a charge and returns an authorization result", NEVER "authorize
  via the Acme Charges API" or "collect card data using the Acme Payments
  SDK". Reflect the applicable cross-cutting
  requirements in the slice that owns them. An undecided item stays
  undecided with its Q# reference, never hardened into a chosen value.
- CRITERION BAR — the phrasing standard EVERY criterion meets, so a reader
  other than the author can verify it:
  * ONE obligation per criterion. Merge overlapping lines; split a criterion
    that hides a second promise behind an "and".
  * OBSERVABLE, never subjective: no "fast", "easy", "user-friendly",
    "seamless", "early enough". State the observable condition instead
    ("before any marketing or tracking script runs"), and assert nothing a
    reader cannot verify from outside the implementation.
  * A CONCRETE SUBJECT the reader can point at — the capability, the named
    surface, or the business user who administers it — in preference to a
    bare "the system"; keep terminology consistent across the block.
  * Reusable shapes: composition ("SHALL contain <listed elements>"),
    event-response ("When <event>, <subject> SHALL <response>"), condition
    ("If <precondition>, <subject> SHALL <response>"), state ("While
    <state>, <subject> SHALL <behaviour>"). Short conditional phrasing
    INSIDE one criterion is this shape, not a Gherkin block.
  * EDGE CASES belong in a near-final offer. Ask, per slice: empty or
    no-result input, a boundary value, the operation failing or a consumed
    dependency being unavailable, and an actor without permission. Emit the
    ones the sources ground; invent none.
- AC shape is a PRE-DRAFT commitment, not a drafting-time choice: in the
  planning pass, for EACH in-scope slice commit one ledger line
  `S# -> the marker groups that apply`, and NAME the grounded upstream fact
  putting each group on that footing (what design's `Gap Analysis` /
  `Base Configuration` records exists on that surface today, and whether THIS
  change writes it, administers it, or only references it). A marker group
  with no grounding fact is NOT emitted. In
  the SAME ledger line, enumerate every design `Base Configuration &
  Cross-Cutting` row whose `Applies to` names this slice's WBS and commit the
  criterion each becomes — a merchant/admin-configuration row becomes a
  criterion whose SUBJECT is the administering business user, a
  privacy / security / accessibility / i18n / performance row becomes a
  cross-cutting criterion in this slice. An applicable row with no committed
  criterion is a DROPPED obligation. Emit each slice's AC to match its
  committed shape — never improvise or drop a grounded group while drafting.
- AC ELABORATION is the companion PRE-DRAFT commitment to shape: in the same
  planning pass, for EACH intended criterion commit one ledger line
  `criterion -> {net-new | delta | seed-echo}`, where `seed-echo` marks a
  criterion that only re-states a `Requirement Details` bullet (no
  capability-owned obligation, no baseline delta). Rewrite or drop every
  `seed-echo` line BEFORE drafting — emit ONLY `net-new` and `delta` criteria.
  This is a binding row-set like the shape ledger: do not defer the
  elaboration decision to emit time, and emit no criterion that reaches draft
  time still labelled `seed-echo`.
- AC promise ONLY what THIS change owns — prove each criterion against the
  RQ's obligation boundary recorded in the lock Notes before emitting it.
  For a shared in-scope RQ (an integration slice serving a UI-consumed
  requirement, or a functional slice consuming a service), the AC promises
  this change's own obligation and NEVER the other side's behaviour: a
  criterion in an integration slice that describes consumer UI (list/selection
  synchronization, accessibility, localized copy — owned by the functional
  change) or one in a functional slice that describes vendor internals
  fails the test → drop it and leave that behaviour to the owning change
  named in Out of Scope & Boundaries. The lock Note's boundary is binding.
- INTEGRATION-AC BOUNDARY SOURCE — when you elaborate the high-level seed
  (see the AC rule above) for an INTEGRATION slice, the seed's Details mix
  the consumer's UI with the vendor's mechanism, both written from the
  outside. Two moves turn a seed line into this capability's obligation:
  (a) drop the shopper-visible UI outcome — that is the consumer's, owned by
  the change named in Out of Scope & Boundaries; (b) lift a named vendor
  mechanism UP to the capability surface it provides, leaving the concrete
  SDK / API / endpoint to the design's Integration Specification. Worked
  transforms (seed -> AC):
    * "collect card details in an embedded payment form via the Acme
      Payments SDK" -> "expose payment-field collection as a lazy-loadable
      entrypoint" (the render surface is the obligation; the SDK is the
      design's HOW).
    * "authorize a shopper's charge ... via Acme" -> "expose an interface
      that authorizes a charge and returns an authorization result".
- INTEGRATION-AC SUBJECT TEST (apply to EVERY integration-slice criterion
  before emitting it): the grammatical subject/actor MUST be the
  capability/service and the promise MUST be its OWN observable contract
  obligation — perform its core operation, expose a stable interface, make a
  render surface available, lazy-load, or govern key/quota/billing/privacy.
  FAIL and DROP a criterion when its subject is the shopper/user ("allow the
  shopper to enter…"), OR when it names a consumer-UI construct or outcome —
  a results list, a selection, highlighting, synchronization / "in
  tandem" / "vice versa", a permission prompt, a page/filter/navigation,
  localized copy, accessibility — EVEN framed as "expose a hook / events for
  the consumer to <highlight / synchronize>". That interaction wiring is the
  consumer's functional UI (owned by the change named in Out of Scope &
  Boundaries) and the design's Integration Specification — never an
  integration AC. This holds EVEN WHEN the verbatim Story or the Details
  mention that shopper behaviour (the Story is copied whole; the AC states
  ONLY what the capability provides). Litmus: the AC names the capability's
  provided surface / interface / governance and stops there; it never names
  what the shopper or the consuming page does with it. INPUT-ACCEPTANCE
  CARVE-OUT: a criterion where the capability ACCEPTS or SUPPORTS a
  shopper-initiated INPUT ("SHALL accept a submitted payment token", "SHALL
  support a shopper-initiated save-card request", "SHALL not persist
  the raw card number") is the CONTRACT SURFACE and is KEPT — the shopper
  is the source of an input the capability accepts, not the subject of a
  rendered UI outcome. Only a shopper-VISIBLE RESULT the consumer renders (a
  displayed list, a highlight, a selection reflected in the UI) is dropped.
- AC mint NO numbers: never introduce a threshold, amount, time,
  latency, count, or limit that no cited source states (no "under $50",
  "in 2 seconds", "5 results" unless a source says so). Testable means
  observable business behaviour, not an invented metric; quantification is
  Planning's job.
- Weight by delivery scope: a functional breakdown carries the shopper
  journey in delivery detail; an integration breakdown stays thin (few
  slices — the capability's consumer-facing obligations, flat AC), pointing
  at the design for the contract.
- Automated tests (unit, E2E) are mandatory across the offer; slice so
  each slice is independently deliverable and testable.

BOUNDARY RULES (Out of Scope & Boundaries):
- Carry the deferred lock rows, unsigned CR topics, and the
  capabilities/services this change consumes but does not own — name
  the owning change/capability for each. An integration proposal also
  excludes the consumer UI/data it does not own.
- The boundary must match the lock and the breakdown: nothing listed
  here appears in a slice, and no in-scope lock row is pushed out here.

DEPENDENCY AND ACCESS RULES (External Dependencies & Access):
- This is the build-prerequisite register grooming turns into
  prerequisites: every external delivery, service, account, credential,
  environment, or tool/admin access the implementation needs.
- CLOSED MEMBERSHIP — prove EXTERNAL per row. For each candidate row,
  prove its Owner/Source is a party OTHER than this change: another
  change to be delivered, an existing service, or a vendor. A row whose
  owner resolves to THIS change FAILS the test and is NOT a dependency —
  a redirect/route/page it implements itself, its own FSD/ISD working
  copies or publication targets, its own artifacts or sections are own
  work whose home is the Delivery Breakdown slice that delivers them (and
  the Approach), never this register. The Owner/Source column IS that
  proof: if it names this change, the row is a placement error — delete
  it and slice the item instead. Commit the register's `ROWS` (the
  proven-external items) and `OUT` (each own-work item -> its breakdown
  slice) in the planning pass before drafting; an `owner = this change`
  line belongs on `OUT`, never as an emitted row.
- Tie rows to the offer: lead `Needed for` with the S# slice(s) the row
  blocks, then the RQ(s)/build need. The breakdown's `Depends on` and
  this register must name the same items.
- Derive rows from the upstream artifacts, branching on delivery scope:
  - functional: the consumed capabilities from the discovery/design
    Dependencies (upstream integration changes to be delivered, existing
    services), plus any access the feature build itself needs. The
    feature holds no vendor account/key/billing — do not add credential
    rows the owning integration changes carry.
  - integration: BEFORE committing ROWS, enumerate EVERY row of the
    design's operational-prerequisites / Dependencies table (account,
    credentials, billing, sandbox/environment) and commit one ledger line
    per row — each is external by definition (client/vendor-owned, not
    this change), so it lands on ROWS with its factual status preserved
    (`unconfirmed (Q#)`, `client-owned`). `ROWS = [None]` is valid ONLY if
    the design records no operational prerequisites; for a vendor
    integration it never does.
- Verification gate: every row must trace to the upstream artifacts or
  cited sources. Where the live workflow can check (branches, paths,
  tracker/doc refs, service endpoints/scopes), verify before listing;
  name the specific thing, not a vague "access needed". An unverifiable
  item is either dropped or marked `UNVERIFIED — confirm before proposal
  merge`. Statuses are facts, never optimistic: inherit the source's
  status wording verbatim (`client-owned`, `unconfirmed (Q#)`,
  `requested`, `not requested`). Ownership is not possession: never
  translate `client-owned` or an unresolved upstream status into
  `have`. Use `have` only when a source states the item already exists
  and is in hand (e.g. an already-integrated service). Do not assert an
  account, key, or approval exists when the sources leave it open.
- ALWAYS a table. When genuinely nothing external is needed, emit the
  single explicit row `| None | — | — | <why> |`.

APPROACH RULES:
- The Approach is a SHORT compression (one-two paragraphs): how the
  offer lands per the design — one grounded sentence of the solution
  shape, an explicit pointer that decisions and rationale live in
  design.md (and, for integration scope, that the service contract
  lives in the design's Integration Specification), the FSD/ISD
  delivery targets, and the testing obligation. The delivery detail
  itself lives in the breakdown's slices, not here.
- Inherit, never re-decide: add no new architecture, re-open none, and
  contradict nothing the design settled.
- Name the FSD/ISD working copies and targets exactly as discovery's
  Delivery Scope & Targets records them.
- No implementation detail: no file/route/component wiring, no our-code
  artifacts, no tuned numbers, and no restated service contract —
  endpoints, payloads, and data mapping live in the design's Integration
  Specification; the Approach points there and never enumerates them.
  Undecided items stay undecided with their Q# references.

OPEN QUESTION RULES:
- This table carries ONLY questions raised at proposal time: a genuine
  new ambiguity discovered while locking scope or verifying
  dependencies/access that no upstream Q# already covers. Number new
  questions by continuing the discovery sequence.
- HUNT before you emit (do this BEFORE writing the table, never default to
  `None`): walk the Requirements Lock, the Delivery Breakdown slicing, and
  the External Dependencies & Access verification and look for genuine
  proposal-time ambiguities — an UNVERIFIED dependency needing
  confirmation, a slice boundary or acceptance criterion with more
  than one viable reading, an acceptance-criterion obligation a slice needs
  that neither discovery nor design carries (surface it here as a new Q#,
  never fill it from the raw SRD), an access item with no owner. Each
  surviving one is a new Q#. A brand-new ASK (not a question) is out of
  bounds here — it goes back to discovery as a CR candidate, not into this
  table.
- Do not re-list discovery/design Open Questions (they are already
  registered for grooming) and do not close or re-scope them here.
- ALWAYS a table. `None` is valid ONLY as an evidenced result whose
  `<why>` names what you actually checked (the lock / breakdown /
  dependency verification passes) — never a bare default from "everything
  is inherited from discovery/design". With no new questions, emit the
  single explicit row `| None | <why> | — | — |`.

DOCUMENT CONSUMPTION GUIDE ROWS:
The Document Consumption Guide is this artifact's interface — the map a
downstream reader (an LLM first, a human second) uses to consume the file
without re-reading the body. Placed DIRECTLY AFTER the Executive Summary, above
the detail sections it maps. It is a PROJECTION, never a new source of truth:
every value must already be readable in the section the row names, and a value
that section no longer supports is wrong however well written.
MEMBERSHIP (the guide MAPS THE DOCUMENT'S CURRENT STRUCTURE): establish the row
set by a prove-membership walk — enumerate the `##` sections the body carries in
document order; each earns exactly ONE row EXCEPT the three that cannot map
content (Executive Summary — the reader map; this guide itself; Proposal Gate —
a self-check). The set FOLLOWS the document: a section a real case adds earns a
new row in place, and a section the body does not carry has no row; do not carry
a fixed list. `What it settles & why` is a short human-readable navigation
phrase; `Key data` carries identifiers and sets only — never a restated
requirement title or prose — and an empty set is `None`.
Each row's key-data authority — what it IS and what it must NEVER be:
- Tracker: the tracker reference the offer is booked under (or the context
  ticket the sources provide) — never an invented ticket.
- Proposal Source Inventory: the upstream artifacts and key sources behind the
  lock — the source set, never the content of those sources.
- Requirements Lock: the locked scope boundary — the in-scope WBS set (the
  bijection key downstream joins on) plus the deferred / out-of-scope WBS with
  their owners; a member is a WBS code, never an RQ id in its place.
- Change Request Dispositions: the `CR#` set with proposed disposition and
  status (or `None`) — never a CR asserted approved.
- Re-usability Legend: the fixed marker key the template prints in full — its
  members, never a per-slice selection.
- Delivery Breakdown: the `S#` slice set (slice count = in-scope requirement
  count) — the offer core the FSD/specs project read-only.
- Out of Scope & Boundaries: what no slice delivers, each with its owner — never
  an in-scope WBS.
- External Dependencies & Access: the consumed deliveries / operational
  prerequisites, each with the slice it blocks and a factual status (or `None`).
- Approach: the working-copy target (the FSD or ISD path) and the testing
  obligation — never re-decided architecture.
- Open Questions: the NEW proposal-stage `Q#` set (or `None`) — never an
  upstream Q# re-listed.
- Review & Approval: the approval state (Approved yes/no with approver).
For any OTHER `##` section a real case adds, map it the same way: its export set
as identifiers, or `None`.

PROPOSAL GATE:
State it after the body with each item checked ([x]) as the passed
self-check; leave a box unchecked only when the item genuinely does not
hold (and then resolve the gap before handing off to grooming):
- the Requirements Lock mirrors every discovery trace row exactly once
  with inherited dispositions;
- the Delivery Breakdown is 1:1 with the in-scope requirements — each
  in-scope requirement is delivered by exactly one slice and each slice
  delivers exactly one in-scope requirement (no grouping, no splitting,
  nothing outside the in-scope set); every slice traces to the lock;
- no scope absent from discovery was introduced;
- every Change Request disposition is proposed, none approved;
- every External Dependencies & Access row is verified or explicitly
  marked UNVERIFIED;
- the slices and Approach inherit the design without re-deciding
  architecture or adding implementation detail.

OUTPUT CONTRACT:
Emit every section in the `proposal.md` template order and table shape
(Executive Summary, Document Consumption Guide, Tracker, Proposal Source
Inventory, Requirements
Lock, Change Request Dispositions, Re-usability Legend, Delivery
Breakdown, Out of Scope & Boundaries, External Dependencies & Access,
Approach, Open Questions, Proposal Gate); do not add, drop, or reorder
sections. Keep every
section present for both delivery scopes — a register with nothing to
carry holds its explicit `None` row instead of being dropped.

SELF-CHECK BEFORE OUTPUT:
- Change identity, delivery scope, and targets are taken verbatim from
  discovery; nothing is re-derived from the raw feature name.
- Lock row count equals the discovery trace row count; identifiers and
  dispositions match verbatim; rows are grouped by WBS.
- The breakdown's coverage law holds as a bijection: each in-scope lock
  row appears in exactly ONE slice block and each slice's `Covers (WBS)`
  names exactly ONE in-scope WBS (slice count = in-scope requirement count
  = FSD in-scope block count; no grouping, no splitting, nothing outside
  the in-scope set); each slice names a discovery capability; each slice's
  story is the discovery Requirements Trace `Requirement` cell for the
  slice's `Covers (WBS)`, copied VERBATIM (STORY-LOOKUP LAW — persona and
  phrasing unchanged, no re-authoring/paraphrase/persona swap, no
  fabricated story, no functional/integration distinction).
- Each slice block carries its metadata list (`Covers (WBS)`,
  `Capability`, `FSD`/`ISD`, `Depends on`), a `#### User Story`, and a
  `#### Acceptance Criteria` under the right heading levels.
- Acceptance Criteria are near-final numbered SHALL statements grouped
  ONLY by the reusability markers the slice earns (`#####` marker
  headings; a single group when one marker covers the whole slice).
  There is NO device / surface / configuration heading axis: device
  context sits inside the criterion and the administering actor is the
  criterion's subject. The same obligation is never repeated in two
  groups. A surface this change hosts or consumes but does not own opens
  its block with the boundary note plus the owner pointer, and its
  criteria assert the consumption contract, never management of the
  other side's content. No Given/When/Then Gherkin blocks, no vendor
  names, no code tokens; undecided items carry their Q# references.
- `## Re-usability Legend` carries EVERY canonical bullet the template
  holds, in template order and wording verbatim: none dropped because no
  slice uses its marker, and no gloss re-authored into case-specific prose.
- No invented RQ/WBS/CR/Q/S identifiers, tracker refs, branches, pages,
  services, access items, owners, or statuses.
- CR dispositions and unresolved facts (accounts, keys, quotas,
  approvals) are proposed/open, never asserted settled or approved.
- External Dependencies & Access rows are grounded and scope-correct
  (functional: consumed deliveries/services, no vendor credentials;
  integration: operational prerequisites with factual statuses); no
  phantom, padded, or own-work rows; unverifiable items are marked
  UNVERIFIED; `Depends on` and the register name the same items; no row
  whose source status is `client-owned` / `unconfirmed` / open carries
  Status `have`.
- No Acceptance Criterion or approach sentence mints a number (amount,
  time, count, limit) absent from the cited sources.
- Slices, boundaries, and Approach restate upstream decisions without
  contradiction, new architecture, or implementation detail.
- Open Questions carries only NEW proposal-stage items, surfaced by
  actually walking the lock / breakdown / dependency verification for new
  ambiguities; a `None` row's `<why>` names what was checked and is never
  a bare default from "everything inherited". No upstream Q# is duplicated
  or closed.
- The Executive Summary reads as a READER MAP: grounded and reader-first with
  business value, it states what the offer settles, what it hands downstream,
  and what stays open, by subject and count — naming subjects not identifiers,
  re-listing no table's rows, implying no approval for a pending item, and
  mirroring the Proposal Gate.
- Document Consumption Guide MAPS the current structure: one row per `##` section
  in document order, excluding only the Executive Summary, the guide itself, and
  the Proposal Gate; every section the body carries has a row and one it does not
  carry has none. Every `Key data` cell is identifiers/sets (`None` when empty),
  each already readable in the named section — a carried-over row the body no
  longer supports is a defect.
