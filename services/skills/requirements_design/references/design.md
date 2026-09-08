# Design contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
Design is the architecture / solution-design step, right after discovery and
before proposal. It decides the solution SHAPE and the reasoning behind it —
which pattern, posture, or capability, and WHY (with alternatives and
trade-offs) — and RAISES anything still needing human sign-off as open
questions for the single downstream grooming to approve. It produces
architecture-level decisions for BOTH delivery flows (a functional design and
an integration design alike). It decides WHAT the solution is and WHY, never
HOW it is built: file-by-file changes, component/route/loader/action wiring,
import mechanics, and concrete numbers belong to technical-design.md during
/opsx:plan.

AUTHORITY ORDER:
1. Source inputs and evidence — the reconciled discovery trace, existing
   `openspec/specs/`, and the current implementation / codebase patterns — are
   the factual source of truth.
2. This `design` instruction is the artifact contract: follow it for when to
   generate, what to cover, the gate, and the quality bar.
3. The `design.md` template defines section order and shape.
4. Project `config.rules`, workflow rules, and the referenced build skills
   provide supporting domain context; they do not override this contract.

DECISION KERNEL:
- Design is the standard architecture step for every scoped change: produce
  design.md for each change once discovery resolved its scope. The
  architecture-bearing concerns (cross-cutting behaviour, a new pattern, a
  new dependency, a data-model change, integration/security/performance/
  migration complexity, or ambiguity an SA should resolve before Planning)
  set the design's depth, not whether it exists; downstream proposal
  requires it.
- Record each architecture decision with rationale, alternatives, and
  trade-offs. Decisions that still need human sign-off, or that remain
  ambiguous, are RAISED as open questions / risks for grooming — never asserted
  as approved or final here. Only discovery-signed facts and clearly
  source-grounded constraints are stated as settled.
- CLASSIFY EACH DISCOVERY ITEM BY ITS DISPOSITION, then place it. The trace
  carries the disposition — read it, do not re-decide it. Every item falls into
  exactly one of three classes, and each class has ONE home:
  - SETTLED and in scope. Disposition tells you the item is in scope; it does NOT
    by itself make the item an architecture decision. Read what KIND of item it is:
    - a behaviour REQUIREMENT (a signed "SHALL" the specs / FSD own): honour it —
      it shapes the Solution Overview and is the REASON a decision exists — but do
      NOT restate it as its own Architecture Decision. Design owns HOW the solution
      is shaped, not a re-listing of WHAT was required.
    - a CONSTRAINT that narrows the solution space, or a CHOICE among viable
      solution shapes that the source settles: OWN it as an Architecture Decision
      (or a Base Configuration & Cross-Cutting row) with rationale, alternatives,
      and trade-offs. Derive it from what the source states; add no specifics the
      source does not state.
    Elevating the settled architecture choices and solution-narrowing constraints
    IS the design's job — a design that defers its own settled architecture is
    under-built. But minting one decision per requirement row is
    requirement-restatement, not architecture — that is the over-built failure.
  - IN SCOPE but UNDECIDED (an item discovery's OWN Open Questions table
    carries — that table is the ONLY set eligible for a proposed direction and
    the ONLY source of Q# numbers; an "undecided" remark in raw sources or
    notes creates neither): PROPOSE a direction but do NOT settle it. Give a
    qualitative lean with its
    main alternative and the trade-off between them, authored as a proposed,
    undecided decision (a `Decision (proposed, raised for grooming — Q#)` block
    carrying a Proposed direction and a `Status: Undecided` line), and mirror the
    Q# in Open Questions. Never present it as a settled Architecture Decision,
    Base Configuration & Cross-Cutting row, or Integration Specification entry,
    and never invent a concrete value to close it (a type, an exact approach, a
    location, a numeric limit) — the lean is a qualitative direction, and an
    unstated specific asserted as chosen is a fabrication. The lean is confined to
  its proposed-decision block: every OTHER section (Risks / Mitigations, Data
  Flow, Dependencies, Cross-Cutting) must still read the item as undecided — do
  not silently harden the lean into a settled mitigation, a committed
  data-flow path, or a fixed posture elsewhere (a Data Flow stays Q-neutral — see
  the DATA FLOW law below). Leaving the item as a bare
  question with no proposed direction is under-built; asserting it as decided —
  in its own block or by hardening it in another section — is over-reach. Do
  neither.
  - UNRESOLVED CHOICE vs UNRESOLVED FACT. A proposed direction answers an open
    architecture CHOICE (which solution shape to build) — that is design's job. It
    does NOT answer an open FACT, STATUS, VALUE, or POLICY: whether an account or
    credential exists, a quota / rate limit, an auth transport, an approval state.
    Those are not the design's to settle — record them as the source leaves them
    (unknown / pending / undecided) and never assert a value or a "confirmed"
    status the source does not state. Proposing a direction changes neither the
    factual record nor the approval state.
  - OUT OF SCOPE by disposition (deferred / later-phase, an unsigned / pending
    Change Request, or an approach the trace hands to another change). Reason
    from what a row MEANS before placing it. Every table here is a register of
    what is being BUILT: a Dependencies row says "the design relies on this,
    live, now", a Risks row says "this threatens what we are building", a
    Decision says "we are doing this". An out-of-scope item makes none of those
    true, so no honest row exists for it — its columns cannot be filled without
    stating something false, which is why a row labelled "out-of-scope" or
    "pending" is self-contradictory, not cautious. The item is still
    acknowledged, in the one honest form design has for it: a single non-goal
    line in the Solution Overview naming it and why it is out of scope (deferred
    / unsigned / owned elsewhere). That line IS the complete, thorough treatment
    — it, not a row, is how a rigorous design shows it saw the item and chose not
    to build it.
    Watch the look-alike: an unsigned CR often names a third-party service, so it
    resembles the integrations the design genuinely consumes. What separates them
    is what PULLS the item in — a consumed integration is required by a SIGNED
    requirement of this change (the current build cannot function without it); an
    unsigned CR is a NEW ask no signed requirement needs. Same "external / separate
    change" shape, opposite disposition — so trace every candidate Dependencies /
    Risks row back to the signed requirement that needs it; a row that traces only
    to a CR candidate has nothing pulling it in and is not earned.
    Discovery has already sorted these: its Dependencies table holds what the
    build consumes, its Change Request Candidates table holds the new asks. Read
    each candidate row's membership from that sorting — not from a raw source's
    "named together" service list — then re-state it at the design's technical /
    operational level per the Dependencies content rule.
  State a constraint as settled only when a source states it or unambiguously
  forces it. SHARED SUBJECT — a source states a base fact while an Open Question
  leaves a narrower refinement of that SAME subject open: settle ONLY the grounded
  base fact and keep the exact sub-choice the Q# owns undecided, mirrored as that
  Q#. Do NOT widen the base fact to swallow the open sub-choice, and do NOT let a
  note that phrases the refinement as a fact ("must be X", "has to be Y") close the
  Q#. Settling the base fact is correct; hardening the open sub-choice — naming its
  value, tier, mechanism, or code — is the violation. The line is per-subject: the
  base posture a source states can be a settled Decision or Base Config row, while
  the value / tier / placement the Q# still owns stays undecided in that same block.
- MARKER, NOT VERB. When you PROPOSE a direction for an item this flow's discovery
  carries as an Open Question, the proposed value MUST carry an inline undecided
  marker (`(Q#, decision pending)` / `(raised for grooming)`) in EVERY place it
  appears — the Decision block AND any Base Configuration / Cross-Cutting / Security
  row, Risks / Mitigations cell, or Data Flow step that repeats it. The marker
  governs, not the verb: "permanent redirect (Q2, decision pending)" is a labelled
  proposal and passes even with an imperative verb, while the same value stated bare
  in ANY one section — "use a permanent redirect", a fixed Security posture, a chosen
  tier or code — hardens the Q# and is a violation even when its own block is
  labelled. Re-state the same lean identically wherever it recurs; a lean that reads
  settled in even one section fails. Also do not invent a concrete value that closes
  the Q# (a specific guardrail, quota, redirect target) — propose only a qualitative
  direction.
- DATA FLOW stays Q-NEUTRAL for an open call-approach / physical-tier question.
  Draw the flow at the capability boundary (consumer -> capability -> vendor) as a
  single sequence that names no undecided physical tier; do NOT split it into
  client-side vs server-proxy branches, and do NOT annotate a step with a
  tier-presuming detail (a key-exposure mode, a proxy hop) when that tier is what a
  Q# leaves open. A boundary-level flow settles no tier and is correct; naming the
  tier in the diagram hardens the Q# just as a settled prose decision would.
  The diagram is a BOUNDARY sketch, not the wire contract: its arrows carry no
  vendor endpoint / path / query-param / field and no chosen key-transport — a step
  reads `capability -> vendor: <capability call> (placement per Q#)`, never
  `GET /v1/... ?key=...`. The concrete wire lives ONLY in the Integration
  Specification's contract and ONLY for an API shape; a vendor-SDK or reuse-cartridge
  approach has no vendor endpoint to draw at all. An endpoint / param / key in the
  Data Flow is the same fabrication (or, for an open tier, hardening) as in prose.
- INFORMATION ARCHITECTURE: the user-facing URL / route PATH of a NEW surface
  this change introduces — and the target of a navigation entry or a retired-URL
  redirect — is design's to DECIDE as an information-architecture decision,
  recorded with rationale, EVEN WHEN no source states a path: this is design's
  product, not fabrication, and is distinct from the internal route module /
  loader / action wiring that belongs to technical-design. State the path once as
  a settled Decision; downstream specs inherit it verbatim. A retired URL's
  redirect TYPE, or any narrower sub-choice a Q# explicitly owns, stays undecided
  and mirrored as that Q# (SHARED SUBJECT).
- Do NOT re-decide the delivery scope; discovery already resolved it. Read the
  trace for the direction (functional vs integration) and weight the design to
  it — functional owns the storefront solution and consumes integrations as
  dependencies (thinner base design); integration owns the agnostic service
  capability and excludes consumer UI / other services (richer service design).
  A consumed integration's INTERNAL choice — its call placement (client-vs-proxy),
  key exposure, or quota governance — is owned by that integration change, so in a
  functional design it is NOT an Open Question of yours: do not mint a Q# for it,
  give it no proposed-direction block, and do not harden it as settled; at most note
  it as a consumed-dependency risk (the feature relies on the capability either way).
  That risk's Mitigation cell must NOT commit a tier or use a directional verb
  (`default to`, `use`, `enforce`, or naming a specific mechanism / value): the only
  allowed mitigation is to defer the choice to the owning integration change /
  grooming. Naming a chosen tier in the mitigation is the hardening this forbids.
- SOLUTION APPROACH (build-vs-reuse) — decide HOW the need is met before shaping the
  solution, preferring REUSE over BUILD in priority order: (1) reuse an existing cartridge
  / accelerator or platform module discovery's prior-art surfaced AND that fits the stack;
  (2) integrate a vendor SDK / client library / documented API; (3) build a connector from
  scratch. An explicit USER approach signal in the sources overrides; absent one, apply
  Enumerate-then-Justify — enumerate the closed set {reuse, SDK, from-scratch}, state FITS
  / DOES-NOT-FIT per rung against discovery's prior-art + the platform baseline, then pick
  the highest-priority fitting rung. This applies to INTEGRATION changes ONLY: it is the
  FIRST Architecture Decision of an integration change (for its OWNED service capability)
  and SHAPES the Integration Specification. A FUNCTIONAL change makes NO Solution Approach
  decision — it does not run a build-vs-reuse ladder, and (per the consumed-integration
  firewall above) NEVER decides, proposes, or open-questions a CONSUMED integration's
  build/call approach (SDK vs API vs proxy, key placement, contract). Never invent a
  cartridge/module/SDK/API discovery did not surface.
- Derive the change IDENTITY from discovery, not the feature name. The change
  name and title are the discovery-resolved owned change (the capability the
  trace owns), never the broader storefront feature the raw sources describe: an
  integration design is scoped as its agnostic service capability, a functional
  design as its feature surface. The feature name is not the change identity.
  The reconciled discovery trace in CONTEXT IS this change's discovery — read the
  change name and owned capability from it. Do NOT adopt the raw packet / fixture
  slug (a consumer feature name) as the change id and then declare "discovery is
  missing" for that slug and collapse to an empty stub: there is no separate
  slug-named change, the trace in CONTEXT is it. When the trace owns an INTEGRATION
  capability, produce the integration design even though the raw packet reads like
  a functional feature — never pivot to a functional `N/A` design. The fixture /
  working-directory name (a directory slug, often the raw feature name) is NOT the
  change id either: when a discovery trace is present in CONTEXT under a DIFFERENT
  name, that trace IS this change's discovery — do NOT halt, refuse, or emit a
  "discovery.md missing for <slug>" stub merely because no discovery file matches
  the directory slug.

PIPELINE (follow each referenced section; do not improvise content):
1. LOCK IDENTITY & SCOPE FIRST (DESIGN GATE). Before drafting any section, read discovery's
   Delivery Scope & Targets and Affected Capabilities and take the change title + delivery scope
   (functional vs integration) verbatim from there. The raw requirements / meeting packet
   describes the BROADER storefront feature and its consumers — it is NOT the change
   identity: an integration change is titled and scoped as its agnostic service
   capability (e.g. a payment-integration capability), never as the consumer feature or
   its UI. When the raw packet pulls toward the consumer surface but discovery resolved an
   integration scope, the discovery scope wins. Fix the change name from the trace's owned
   capability at PLAN time; the trace in CONTEXT IS this change's discovery, so never record
   it as "missing" and never fall back to the raw packet slug.
2. Gather the discovery-resolved scope, dependencies, and open questions, plus
   existing specs / current patterns, and record the inputs actually used in the
   Design Source Inventory (INPUTS AND SOURCE POLICY). The Solution Overview and
   Document Consumption Guide are DERIVED LAST from the finished body (step 8),
   not drafted here.
3. Establish the solution SHAPE at architecture level, weighted to the inherited
   delivery scope (functional vs integration) — the reasoning that drives the
   decisions below. This is the thinking, not the Solution Overview block: that
   reader map is derived from the finished body at step 8.
3a. State the Affected Capabilities: one pointed row per owned capability — what
   this change adds / modifies / removes and where, at capability level, grounded
   in that capability's baseline spec.
3b. GAP ANALYSIS. Before deciding anything, state what EXISTS today for each in-scope
   requirement and how far it is from the target (GAP ANALYSIS RULES). This is the factual
   input the decisions are made FROM; it decides nothing itself.
4. Record each architecture decision with rationale, alternatives, trade-offs.
5. Fill Base Configuration & Cross-Cutting only where a concern affects the
   solution choice; these apply to both functional and integration changes. Tag
   each row's `Applies to` column with the clean WBS codes (from discovery's
   Requirements Trace) the concern shapes, or `all` — this is the story-friendly
   source the functional-artifacts script (generate-functional-artifacts.mjs) projects into
   per-story Implementation notes. Never reference slices (S#) here: slices are defined
   downstream in proposal.md and do not exist at design time.
6. For an INTEGRATION change, decide the Solution Approach (build-vs-reuse ladder — see
   DECISION KERNEL) as the FIRST decision, then fill the Integration Specification SHAPED
   BY it: service overview + integration documentation always, then the contract in the
   approach's shape (from-scratch/API → endpoints + data mapping; vendor SDK → the SDK
   interface; reuse → the cartridge config/extension contract), plus data flow — at
   capability level, not implementation. For a FUNCTIONAL change there is NO Solution
   Approach decision: mark the Integration Specification N/A and point to the integration
   change(s) that own the consumed services.
7. Record risks and mitigations. Then build Dependencies by the Dependencies rule as a
   classify-then-prove subtask: for each candidate decide its disposition, prove the
   ones this change RELIES ON but does not own, emit a row ONLY for those, and send the
   rest to their real home.
7a. HUNT (b) DESIGN QUESTIONS as the Open Questions CLOSED-MEMBERSHIP LEDGER —
   MANDATORY, do this BEFORE writing Open Questions and EMIT it in the planning pass
   as the section's `ROWS` / `OUT` ledger. A self-check attestation that the hunt "ran"
   WITHOUT the emitted ledger does not satisfy this step: the ledger is the work, the
   attestation is not. Diff the TARGET solution against the CURRENT implementation and
   the baseline specs — open the concrete files / components / config the change touches
   (read them, do not reason from memory) — and ENUMERATE every divergence the solution
   introduces. Put EACH divergence through the owner gate, naming the one class it is:
   - MINE — a choice whose RESOLUTION changes a surface or decision THIS change builds or
     owns (a storefront route / IA / redirect target, a rendering / caching / prefetch
     posture on our tier, an owned capability's behaviour), with multiple viable options
     and rework if deferred. PROVE membership before it may enter `ROWS`: name the OWNED
     surface the answer changes. "It affects my risk / scope / timeline" is NOT proof —
     a consumed service's mechanic affects those too; the test is whether the answer
     changes something THIS change owns. If the answer instead changes HOW A CONSUMED
     SERVICE is called, built, keyed, or metered (client-vs-proxy, key placement, quota,
     contract), it is NOT MINE — reclassify it as ANOTHER CHANGE'S even though it touches
     this change's risk. Only a proven-MINE choice enters `OPEN QUESTIONS ROWS`, recorded
     with a proposed direction and mirrored as a Q#.
   - ANOTHER CHANGE'S — a divergence living INSIDE a consumed integration (how it is
     called or built, key placement, quota governance) belongs to the change that owns
     that integration. It is BARRED from `ROWS`: record it on `OPEN QUESTIONS OUT` as
     `<divergence> -> consumed-dependency risk` (a Risks row whose Mitigation defers the
     choice to the owning change / grooming — never a tier, mechanism, or directional
     verb), with NO Q#, NO proposed-direction block, and NO decision anywhere.
   - NOT A CHOICE OF MINE TO OPEN — already resolved by discovery, an unsigned / pending
     Change Request, or an open FACT / STATUS / VALUE rather than a CHOICE. Also BARRED
     from `ROWS`: record it on `OPEN QUESTIONS OUT` as `<item> -> <its real home>` (the
     source as-is, a Solution-Overview non-goal line, or its owning change).
   EMIT the ledger as ONE ROW PER ENUMERATED DIVERGENCE, each carrying three cells —
   `<divergence> | <MINE | ANOTHER CHANGE'S | NOT-A-CHOICE> | <PROOF>` — and DERIVE the two
   binding lines FROM it; never assert them. The PROOF cell is what makes the class checkable
   and must be NON-EMPTY: for MINE it NAMES the OWNED surface whose answer changes (a route /
   IA / redirect target, a rendering-caching-prefetch posture on our tier, an owned
   capability's behaviour); for ANOTHER CHANGE'S it NAMES the consumed service the choice
   lives inside; for NOT-A-CHOICE it NAMES the resolving source / home. A divergence phrased
   as a consumed service's call placement, client-vs-proxy, key placement, quota, or contract
   HAS NO owned surface — its MINE proof cell cannot be filled — so it is ANOTHER CHANGE'S by
   construction and can NEVER be recorded as MINE, however much it touches this change's risk,
   scope, or timeline; "we will call it" is not owning it. A blanket `OUT = [] (all MINE)`
   without a filled per-row proof is the rubber-stamp this ledger exists to stop. Then READ
   `OPEN QUESTIONS ROWS = [<the MINE rows: carried (a) discovery Q#s + new (b), or None>]`
   and `OPEN QUESTIONS OUT = [<every ANOTHER / NOT-A-CHOICE row -> its named home>]` OFF the
   ledger and treat them as binding: the Open Questions table emits EXACTLY `ROWS`, and
   nothing on `OUT` may surface anywhere as a Q#, a proposed-direction block (no `D#`), or a
   decision. A Q# whose class is anything but MINE is a defect. Do NOT collapse to "discovery
   resolved everything -> no questions": inheritance covers product / scope, NOT the
   architecture that only exists once the solution is shaped. `OPEN QUESTIONS ROWS = [None]`
   is an allowed PASS ONLY when the ledger NAMES what you inspected (which files / surfaces),
   so `None` is an evidenced result, never a default. Then raise all `ROWS` decisions needing
   sign-off as Open Questions for grooming.
8. DERIVE THE READER INTERFACE LAST — from the FINISHED body, never from
   intentions. The Solution Overview and Document Consumption Guide are placed
   first in the file but derived here, after every detail section is final, so
   each is a copy of what the body says NOW (a claim the sections no longer
   support is wrong however well written):
   a. Build the Document Consumption Guide (DOCUMENT CONSUMPTION GUIDE ROWS): walk
      the `##` sections the body carries in document order, emit one row per
      section EXCEPT the three that cannot map (Solution Overview — the reader
      map; this guide itself; Design Gate), and read each row's Key data (its
      identifiers/sets, `None` when the set is empty) off the section it names —
      never carried over from an earlier draft.
   b. Answer the Solution Overview reader map (its CONTENT RULE) from those same
      finished sections: what the design settles, why it is shaped that way, what
      constrains downstream, and what is passed forward (grooming / planning) — by
      SUBJECT and COUNT, re-listing no section's rows. Any count it states equals
      the size of that set in the body.
9. State the Design Gate after the body with each item checked ([x]): it is the
   passed pre-grooming self-check, not an open to-do list. Set each box from the
   final body, the Open Questions table, and the derived reader interface
   (step 8). Leave a box unchecked only if that item genuinely does not hold.

DESIGN GATE (blocking — evaluate BEFORE authoring anything):
- Entry: design.md is produced for every change once the Discovery Gate is
  clean; downstream proposal requires it. Depth scales with the change's
  architecture-bearing concerns — a simple change yields a short design,
  never a skipped one.
- Design runs pre-grooming: do NOT wait for or require an approved grooming.
  Decisions needing sign-off are recorded as proposed and raised for grooming,
  not held back and not asserted as decided.

INPUTS AND SOURCE POLICY:
- Read discovery.md as the resolved upstream trace: it has ALREADY passed its own
  review gate, so its dispositions are settled — inherit them, never re-decide.
  Consume it through its two published reader aids rather than re-scanning raw prose:
  its `Executive Summary` orients you (the WHAT, the business context, what discovery
  settled and what it left open), and its `Document Consumption Guide` is the
  authoritative map of every `##` section the file carries and the key identifiers in
  each — use it to find what you need. Both are reading AIDS only; the design derives
  its OWN Solution Overview + Document Consumption Guide last, from THIS file's finished
  body, never by copying discovery's forward.
  That guide is the section map (a case may add, rename, merge, or omit some), so do
  NOT re-enumerate or re-describe discovery's sections here; instead inherit each item's
  disposition and place it by the rules this instruction already states (DECISION KERNEL
  and the per-section CONTENT RULES), in short, per disposition:
  - a SIGNED, in-scope requirement -> HONOUR it in the Solution Overview and in the
    rationale of the decision that realises it, and give it one Gap Analysis row per
    its in-scope WBS (GAP ANALYSIS RULES); elevate to its own Architecture Decision
    ONLY where it carries an architecture choice or a solution-narrowing constraint,
    never one decision per requirement.
  - an owned Affected Capability -> the design's OWN Affected Capabilities section,
    one level deeper (the concrete add / modify / remove and WHERE), never a
    Dependencies row.
  - a consumed-not-owned Dependency -> the Dependencies content rule (normalise the
    Type to the design closed set; `None` never licenses `None`).
  - an Open Question -> the ONLY source of Q# numbers and of proposed-direction
    eligibility; maps to this design's Open Questions / Risks, never a settled Decision.
  - an unsigned Change Request Candidate, or any row the disposition marks deferred /
    later-phase / out-of-scope -> a single non-goal line in the Solution Overview, no
    table row (OUT OF SCOPE by disposition).
  - a Cross-cutting Requirement -> a Base Configuration & Cross-Cutting row where grounded.
  - the resolved Delivery Scope & Targets and change identity -> weight the design to
    it; do not re-derive it.
  - a settled, architecture-bearing choice discovery states in PROSE (its Executive
    Summary, the Source Inventory "purpose" notes, or the technical / meeting notes it
    cites), not only in its tables -> elevate to exactly one Architecture Decision or
    Base Configuration row; do not leave it un-elevated (under-built) nor elevate a bare
    behaviour requirement (over-built); on a SHARED SUBJECT elevate only the base fact
    and keep the Q#'s sub-choice undecided.
  Also read existing `openspec/specs/` and the current implementation for baseline
  patterns. Do NOT read proposal.md or grooming.md — they are downstream of
  design in this pipeline.
- A research report committed for this change under
  `openspec/changes/<change>/research/<slug>/report.md` is a valid, citable
  input for an empirical fact that no document, baseline spec, or code
  reading settles. Anchor a specific fact as `<that path> · Q#`, and carry
  the finding's recorded status and basis as they stand — an unconfirmed or
  documentation-only finding is never hardened into a settled Decision,
  Gap Analysis Current State, or Integration Specification entry.
- Reference existing codebase patterns at the level needed to justify a
  decision. Do NOT list files to modify or wire components; that is
  technical-design.md.
- For an integration change, derive the third-party service contract (endpoints,
  formats, auth, data mapping) from the vendor's OFFICIAL documentation. Gather
  it in this order: search the web first, then local / project-context material,
  then the user; record what you used under Integration Specification ->
  Integration Documentation with realistic anchors (e.g. project-context `web` /
  the vendor doc URL). Never fabricate endpoints, fields, or auth; if the
  contract cannot be grounded, raise it as an Open Question.
- Consult the build skills for domain constraints when they affect the solution
  choice, risk, or client/BA sign-off:
  - Rendering / routing: `sfnext-routing`.
  - Caching: storefront/edge (HTTP + CDN) — `b2c-ecdn`; commerce-backend (custom SCAPI / cartridge / job) — `b2c-custom-caches`.
  - Security: storefront/shopper auth — `sfnext-authentication` / `b2c-slas-auth-patterns`; outbound third-party service credentials/secrets — `b2c-webservices`.
  - Performance: `sfnext-performance`.
  - i18n / content: `sfnext-i18n`.
  - CMS: `sfnext-page-designer` / `b2c-page-designer`.

CONTENT RULES (cover where applicable; omit non-applicable concerns):
- Solution Overview — a READER MAP that orients a reader (an LLM first, a human
  second) before the detail, never a summary or a retelling. Answer, from the
  sections below as they stand NOW, in order: WHAT THIS DESIGN SETTLES (the
  solution at architecture level, the inherited delivery scope, and the
  owned-vs-consumed boundary); WHY IT IS SHAPED THIS WAY (the load-bearing
  approach/decision by SUBJECT and COUNT — the build-vs-reuse rung, "seven
  architecture decisions" — not a retelling of each block); WHAT CONSTRAINS
  DOWNSTREAM technical-design (the conclusions the next phase must honour); and
  WHAT IS PASSED FORWARD (the open questions handed to grooming and, when present,
  the items routed to planning as deferrals). Its machine-readable counterpart is
  the Document Consumption Guide; the two split the same facts along the
  prose/identifier line and never overlap. Name SUBJECTS, never identifiers ("the
  retired /legacy-path redirect", not "Q2"); counts/magnitudes are welcome, but it does
  NOT enumerate what a section below carries (the owned capabilities, the consumed
  services, the `D#` decisions) — naming the boundary is not listing its members.
  Each out-of-scope item (a deferred / unsigned Change Request the build brushes
  against) gets one non-goal line here — its only record. Write it after the body
  is final and re-derive it when the body changes; a statement the sections no
  longer support is wrong however well written.
- Affected Capabilities: one row per capability THIS change owns, stating at
  architecture / behaviour level WHAT it adds / modifies / removes and WHERE,
  grounded in that capability's baseline spec (an ADD capability has no baseline —
  its spec path is a future target). Discovery already enumerated WHICH capabilities
  are affected and their ADD / MODIFY / REMOVE direction; this section does not
  restate that list — it goes one level deeper and states the SPECIFIC change per
  owned capability (discovery = high-level enumeration; design = the concrete change).
  It is not file / route / component wiring (that is technical-design). Every owned
  surface belongs here, never as a Dependencies row.
- GAP ANALYSIS RULES — the factual current-vs-desired assessment the decisions are made FROM.
  Run it as Enumerate-then-Justify. The row keys are a set with TWO parts, and you build
  it before writing anything: (i) every in-scope WBS code of discovery's Requirements
  Trace, and (ii) every capability in discovery's Affected Capabilities whose MODIFY /
  REMOVE direction no in-scope WBS row already covers — a surface this change retires or
  re-points often carries no signed requirement of its own, and its current state is
  exactly the evidence the retirement rests on, so it would otherwise be assessed
  nowhere. Then JUSTIFY each row's verdict in the row itself. Coverage is a law
  in BOTH directions — a missing key from either part is a dropped subject, not brevity, and an
  extra row is padding: an item the trace does not carry as an in-scope requirement of THIS
  change, and that is not such an owned capability, earns no row, because this change
  assesses nothing about it. Excluding it here is
  SILENCE, not a re-homing — it does not become a row in any other section either. Granularity
  of part (i) follows the delivery scope: a FUNCTIONAL change gets one row per in-scope
  requirement (this is the section's main value); an INTEGRATION change gets lighter
  capability-level rows for the obligations it owns, sized to feed the Solution Approach
  decision rather than duplicate it. A part-(ii) row is keyed by the capability itself in
  either scope, since it has no signed WBS to key on. The first column stays headed
  exactly `WBS` — never widened to `WBS / Capability` or similar — because the placeholder
  already admits a capability key for these part-(ii) rows; renaming the header breaks the
  structural contract.
  - `Current State` — what exists TODAY and WHERE, with an anchor (a baseline spec path, a
    fact the discovery trace states, or a live URL). The sources are a closed set: the
    platform / OOTB baseline, a third-party-managed surface, a legacy implementation, or
    nothing. Cite that anchor IN the cell, and make a "nothing exists" cell say what the
    shopper or consumer has INSTEAD today (the legacy surface, the OOTB default) rather than
    stopping at the bare negative. SWEEP for the anchor before concluding "nothing" — the
    baseline specs, the technical / meeting notes, and the trace's Cross-cutting Requirements
    table (a concern listed there as applicable states the MECHANISM that already carries it,
    and that row IS the anchor). Concluding "none" for a concern one of those sources shows as
    existing is a fabrication, and the most common one: the requirement is new, the mechanism
    it needs is not. Grounding is symmetric — an "already covered" with no
    anchor is a fabrication exactly as an invented gap is, so a `none` verdict carries the
    same evidence burden as a `rebuild` one. Never assert an OOTB / vendor capability no
    source states.
  - `Desired State` — the target the signed requirement sets. Inherit it; do not re-decide,
    widen, or narrow it here.
  - `Gap` — the DISTANCE from what exists to what the requirement needs, from the closed set
    `none` / `config` / `adapt` / `rebuild` / `remove` (a value outside it is a defect).
    Assess the MECHANISM the requirement needs, NOT this change's own instance of it — ask
    "what would we have to BUILD?". A requirement satisfied by authoring content, keys, or
    settings against a mechanism the platform already provides is `config` even though nothing
    exists for THIS feature yet; that is the usual shape of a cross-cutting requirement
    (localization, an existing accessibility or analytics convention). Reserve `rebuild` for a
    mechanism that genuinely does not exist. The trace's DELIVERY MODE is a different axis and
    never lands in this column: `build` there means THIS change delivers the requirement
    rather than consuming it from another change, and says nothing about the distance — a
    `build` requirement whose mechanism already exists is still a `config` gap. Choose the
    LIGHTEST verdict that honestly closes the requirement, and when it IS `rebuild`, `Notes`
    states in one line why `config` and `adapt` do not close it. The verdict is an ASSESSMENT,
    not a decision or an approval: it settles no Open Question, and no other section may state
    it as a chosen approach.
  - `Notes` — the one-line proof: the evidence for the verdict, and for a `remove` row whether
    the removal is already SIGNED upstream (a discovery Affected Capabilities REMOVE — carried
    as a fact) or NEWLY PROPOSED here. A newly proposed removal is class A when it only cleans
    up an OOTB / baseline default outside the signed scope — design decides it and records it
    as an Affected Capabilities REMOVE row; it is class B when it touches signed scope or
    client-visible behaviour — raise it for grooming, never record it as decided.
  A `remove` verdict IS the removal record; do not add a second removal table.
  `### Legacy Behaviour Overview` — emit this subsection ONLY when a legacy implementation of
  the behaviour exists: a short narrative of how it works today, what it covers, what it does
  not, and what (if anything) is portable. Facts, not decisions. Omit it entirely when there
  is no legacy.
  The whole section states FACTS. It settles no Open Question, proposes no direction, and
  asserts no approval — those stay with Architecture Decisions and Open Questions. A cell that
  names a value an Open Question leaves open (a redirect type, a chosen call placement) hardens
  that Q# exactly as settled prose would; write the target the requirement sets and point at
  the Q#, never the answer.
- Solution Approach: for an INTEGRATION change, apply the build-vs-reuse ladder per the
  DECISION KERNEL principle as the FIRST Architecture Decision for its OWNED capability; it
  SHAPES the Integration Specification (see that content rule for each approach's contract
  shape). A FUNCTIONAL change makes NO Solution Approach decision.
- Architecture decisions with rationale (why X over Y), alternatives considered,
  and trade-offs — never a bare assertion. Number the decision blocks
  `D1, D2, …` in document order; the `D#` is the stable joinable id the Document
  Consumption Guide's Architecture Decisions row exports and a downstream artifact
  cites, and a proposed-direction block links its `D#` to the mirrored `Q#`. State
  each at the level of the CHOICE
  and its pattern (e.g. "lazy-load the heavy client widget", "call the
  provider through a server-side proxy"), not
  the mechanism that realises it. Our concrete build HOW belongs to
  technical-design and stays OUT of the Architecture Decisions, Cross-Cutting,
  and prose sections: our file/module paths and route names, component/hook
  wiring, import mechanics (e.g. `dynamic import()`), our function/method
  signatures and headers, and our own config numbers (TTL, debounce, cache size,
  timeouts, platform/library versions). The build skills show that HOW so you can
  name the right decision — record the decision, not the skill's code, paths, or
  numbers. The one exception is the Integration Specification of an integration
  change, which states the third-party CONTRACT concretely — see its rule below.
  REGISTER — each bullet is a dry technical statement, not prose: ONE claim,
  stated directly. Cut everything that does not change the decision — a second
  sentence that re-argues the first, a restatement of what the Solution Overview /
  Gap Analysis / a Cross-Cutting row already carries, benefit framing, hedges, and
  lead-ins. This is a register rule only: it never drops one of the four bullets,
  an RQ# / Q# citation, or an undecided marker — those survive every compression.
- Base Configuration & Cross-Cutting: rendering, caching, security, performance,
  privacy, i18n/content, and CMS constraints ONLY when they affect the solution
  choice, risk, or client/BA sign-off (see the build skills above). These apply
  to both functional and integration changes. Each row's `Applies to` column
  carries the clean WBS codes (space-separated) the concern shapes, or `all`;
  keep the Constraint cell to one intent line ending with its primary skill in
  backticks. The four technical concerns (Rendering, Caching, Security,
  Performance) are the deterministic source the functional-artifacts script projects into each
  story's Implementation notes by intersecting `Applies to` with the story's
  covered WBS; Privacy / i18n / CMS stay solution constraints (i18n and consent
  surface as acceptance criteria, never Implementation notes).
  A MAGNITUDE in a Constraint cell — a cache TTL / duration, a latency / paint / TTI
  target, a bundle-size or payload budget, a device / network baseline — is stated as a
  settled value ONLY when a source grounds it. Absent a source, state the QUALITATIVE
  posture that shapes the solution (read-only reference data is edge-cacheable, the map is
  off the critical path, the list/shell renders first) and leave the specific number to
  technical-design / performance testing. A number the design picks itself is neither a
  signed FACT nor a CHOICE this design owns — it is downstream tuning — so asserting it
  here as settled (a `5 min` TTL, a `< 1 s` TTI, a `200 KB` budget, `< 2 s on 3G`) is
  fabrication, not a constraint (per KERNEL CHOICE-vs-FACT).
- Integration Specification (integration changes only): describe the contract the
  consuming changes integrate against, SHAPED BY the chosen Solution Approach (above).
  ALWAYS present: an Overview (a LOCAL READER-MAP for this section — 1-2 sentences that LEAD
  WITH the contract SHAPE the Solution Approach fixed (API / SDK / cartridge, and why the
  subsections below are what they are); when the section carries more than one service or
  contract, or more than one contract shape, NAME each and state how the section splits by
  them — that is the single most important fact for a reader — and note what the section
  defers out (an open call placement, our build → technical-design). It does NOT re-describe
  what the capability settles or why it exists — that is the Solution Overview's job, and
  repeating it is the defect to avoid. It MUST END WITH a `Read in order:` signpost that names
  the subsections actually present, in document order, each with a one-clause purpose: a
  single compact line when the section is small, GROUPED by service/contract cluster when the
  section is large (so the path scales instead of becoming one unreadable run), so a
  downstream reader navigates a large contract without re-reading it. It orients WITHIN this
  section, never restating the Solution Overview or re-listing Document Consumption Guide
  rows, and is a projection of this section's body),
  Integration Documentation (the official vendor / cartridge docs used, per the source
  policy, cited with realistic anchors), and a Data Flow (the sequence; a diagram is
  welcome). The MIDDLE — the concrete contract — takes the shape of the chosen approach:
  - FROM-SCRATCH / API: Endpoints (channels, protocol/method, data format, auth) + Data
    Mapping (what is sent/received, field-level) — stated concretely from the vendor's
    real endpoints/paths, params, data formats, auth, documented status/fault codes, and
    documented quota/rate-limit/cache (TTL) guidance, each grounded in the cited docs.
  - VENDOR SDK: the SDK INTERFACE — initialization / loader config, the methods we call
    (name, inputs/returns, when to call), the events we subscribe to (when they fire +
    payload), and the lifecycle — grounded in the SDK docs. Endpoints appear ONLY for
    real network assets (a loader/CDN URL, an allowlist origin); an in-browser SDK call
    is NOT an endpoint, and inventing a request/response Data-Mapping field table for an
    SDK channel is a defect — its absence is CORRECT.
  - REUSE cartridge/module: the CONFIGURATION / EXTENSION contract — the cartridge's
    integration points, the config/settings we set, and any extension seam we use —
    grounded in the cartridge/module docs. Do NOT invent vendor endpoints/fields the
    cartridge encapsulates.
  Grounding is symmetric across shapes: an invented value (endpoint, field, method,
  event, config key) is fabrication; if the contract cannot be grounded, raise an Open
  Question. For the SDK / reuse shapes the ABSENCE of an endpoints/data-mapping table is
  correct, never a gap. Beyond the third-party contract, OUR build stays OUT of every
  shape (per the Architecture decisions rule): our own (not vendor-documented) tuning
  numbers defer to technical-design, while vendor-documented values (quota/TTL) stay in
  this contract.
  For a FUNCTIONAL change this whole section is exactly one line —
  `N/A — functional change.` plus a pointer to the
  integration change(s) that own the consumed services (see Dependencies). Do NOT
  emit the subsections or any vendor endpoint/field/protocol detail: a functional design
  does not own the service contract, so filling it is out-of-scope fabrication.
- Dependencies — what this change RELIES ON but does NOT own (a capability, service,
  or prerequisite owned by another change or a vendor). Ownership is the first gate:
  if this change's own work delivers the item, it is an Affected Capabilities row, not
  a dependency — however much the rollout relies on it. A row asserts a live, in-scope
  reliance now, stated at the technical / operational level:
  - functional rows are the consumed capabilities / services discovery already sorted,
    re-stated on the change that owns each — an integration change to be delivered, or
    an already-integrated service — and the feature holds no vendor account / key /
    billing, so those are never rows here. Discovery's list is the CANDIDATE set, not
    the final one: this table is what the build WAITS ON, so a candidate earns a row
    only when it is a DELIVERY (a capability another change must still deliver, or a
    service this build calls). A platform-managed surface that is neither — the back-
    office settings the client's own administrators configure, or content components
    another workstream authors — is nothing to wait for: it is current-state evidence,
    so carry it in the Gap Analysis row it shapes (and, where this change owns the
    surface, in Affected Capabilities), never as a Dependencies row. Dropping it here
    is that filter working, not a lost fact. NORMALIZE the Type to
    the design closed set — discovery may label a consumed integration `external
    integration` / `third-party` / `vendor` / `data service`; those are NOT design Type
    values, so re-type each as `upstream integration change` (owned by a change still to
    be delivered) or `existing service` (already integrated), never copied verbatim. Each
    Note cites the signed requirement (RQ#) that makes this change rely on it; a row that
    cannot name one is not earned.
  - integration rows are the operational prerequisites the service contract you
    specified makes necessary — for an authenticated or metered vendor API the account
    and credentials rows are REQUIRED (billing when metered), and a bare `None` there is
    a defect. Derive them from that contract; discovery's `None` denies owned-service
    reliance, not these prerequisites, so it never licenses a `None`.
  Do NOT add a row to show a category was considered — a prerequisite genuinely not
  needed earns no row (note the exclusion instead). Carry an unconfirmed one with its
  discovery status (never upgraded to "confirmed", never justified by an invented Q#).
  An unsigned / pending Change Request is not a dependency; a candidate whose own note
  admits it is owned / here "for completeness" is a misplaced Affected Capabilities row
  — delete it. ALWAYS a table; if nothing relies-on-not-owned, one explicit
  `| None | — | — | <why> |` row. Type is a closed set — functional:
  `upstream integration change` | `existing service`; integration:
  `account` | `credentials` | `billing` | `sandbox` | `environment`. A Type outside
  this set (e.g. `external integration`, `third-party`, `data service`, `functional
  surface`) is a placement error: remap it to the member above, never emit the synonym.
- Risks and trade-offs with mitigations, including the undecided items the
  discovery trace still carries.
- Planning Deferrals — a bounded developer-HOW the design
  deliberately does NOT settle and routes to technical-design planning (an
  implementation strategy, a tuning approach, a build sequencing), recorded as a
  `PD#` row (`PD# | source design section/decision | deferred item | why not
  decided now | target phase (=planning) | owner | status`). Establish the
  deferral set by a prove-membership pass — for each developer-HOW the design
  brushes against, classify it as exactly one of: (a) a CHOICE grooming signs →
  an Open Question `Q#`, not a deferral; (b) a consumed integration's internal
  choice → stays with the owning change, not here; (c) a signed requirement →
  built, not deferred; (d) a bounded HOW this design OWNS but leaves for planning
  → the ONLY class that earns a `PD#` row. `PD#` is the design-owned
  planning-deferral namespace; owner is per-row (`BA` / `SA` / `tech-design`) —
  who owns the deferred item. The section is ALWAYS PRESENT, like Open Questions
  and Dependencies: when the prove-membership pass earns no `PD#` row, write a
  single `None.` line — do NOT omit the section, and its Document Consumption Guide
  row then reads `None`. Downstream technical-design consumes these rows;
  this artifact routes them, it does not decide them.

DOCUMENT CONSUMPTION GUIDE ROWS:
The Document Consumption Guide is this artifact's interface — the map a downstream
reader (an LLM first, a human second) uses to consume the file without re-reading
the body. Placed DIRECTLY AFTER the Solution Overview, above the detail sections
it maps. It is a PROJECTION, never a new source of truth: every value must already
be readable in the section the row names, and a value that section no longer
supports is wrong however well written.
DESIGN-SPECIFIC MEMBERSHIP (this is NOT discovery's fixed 7-row interface): the
guide MAPS THE DOCUMENT'S CURRENT STRUCTURE. Establish the row set by a
prove-membership walk — enumerate the `##` sections the body carries in document
order; each earns exactly ONE row EXCEPT the three that cannot map content
(Solution Overview — the reader map; this guide itself; Design Gate — a
self-check). The set FOLLOWS the document: a section a real case adds earns a new
row in place, and a section the body does not carry has no row. Do not carry a
fixed list; derive the rows from the sections actually present. `What it settles & why` is a short human-readable navigation
phrase; `Key data` carries identifiers and sets only — never a restated
requirement title or prose — and an empty set is `None`.
Each row's key-data authority — what it IS and what it must NEVER be:
- Affected Capabilities: the owned capabilities with their ADD / MODIFY / REMOVE
  direction — fixes the specs file inventory.
- Design Source Inventory: the input sources this design reasons from (discovery,
  BRD/SRD, technical/meeting notes, vendor docs, a review session) — the source
  set, never the content of those sources.
- Gap Analysis: the assessed keys with their gap class (`WBS →
  none/config/adapt/rebuild/remove`) — the distance, never a chosen approach.
- Architecture Decisions: the `D#` set (a proposed-direction `D#` notes its linked
  `Q#`) — the joinable decision ids, never a restated decision title.
- Base Configuration & Cross-Cutting: the cross-cutting concerns carried (with
  their `Applies to` WBS) — the rows the functional-artifacts script reads.
- Integration Specification: for an integration change, the contract keys
  (endpoints + data-mapping fields for an API shape; the SDK interface for an SDK
  shape); for a functional change, `N/A — functional change` with Key data `None`.
- Risks / Mitigations: the risk set (short labels) this change carries.
- Dependencies: the consumed-not-owned capabilities/services (owned vs consumed) —
  each becomes a proposal External Dependencies & Access row.
- Open Questions: the `Q#` set raised for grooming with owner/status — the ONLY
  source of design `Q#` downstream.
- Planning Deferrals: the `PD#` set routed to planning — the ONLY source of `PD#`;
  the section is always present, so this row is too, reading `None` when the design
  defers nothing.
- Review & Approval: the approval state (Approved yes/no + approver).
For any OTHER `##` section a real case adds, map it the same way: its export set as
identifiers, or `None`.

OUTPUT CONTRACT:
Emit the sections in the `design.md` template order and shape (Solution
Overview, Document Consumption Guide, Affected Capabilities, Design Source
Inventory, Gap Analysis, Architecture Decisions, Base Configuration &
Cross-Cutting, Integration Specification, Risks / Mitigations, Dependencies,
Open Questions, Planning Deferrals, Design Gate); do not add, drop, or reorder
sections. Keep the Integration Specification section present even for a functional
change (rendered N/A). Planning Deferrals is ALWAYS present, like Open Questions /
Dependencies (and always carries a Document Consumption Guide row); write `None.` in
the section and `None` in its guide row when the design defers nothing.

SELF-CHECK BEFORE OUTPUT:
- The design's depth matches the change's architecture-bearing concerns; it
  states why the design matters for this change.
- Delivery scope matches the discovery trace and is not re-decided; the change
  name and title are the discovery-resolved owned change, not the feature name.
  The trace in CONTEXT IS this change's discovery — it is never recorded as
  "missing" and the raw packet / fixture slug is never adopted as the change id.
- Solution Overview reads as a READER MAP: it says what the design settles, why it
  is shaped that way, what constrains downstream, and what is passed forward
  (grooming / planning), by subject and count. It names subjects not identifiers,
  re-lists no section's rows, and implies no approval for an unresolved decision.
- Document Consumption Guide MAPS the current structure: one row per `##` section
  in document order, excluding only Solution Overview, the guide itself, and
  Design Gate; every section the body carries has a row and a section the body does
  not carry has none. Every `Key data` cell is
  identifiers/sets (`None` when empty), each already readable in the named section
  — a carried-over row that the body no longer supports is a defect.
- For an INTEGRATION change, the Integration Specification opens with an Overview that
  reads as a LOCAL READER-MAP of that section: 1-2 sentences that LEAD WITH the contract
  shape (API / SDK / cartridge), naming each service/contract and how the section splits when
  there is more than one, plus what the section defers out, NOT re-describing what the
  capability settles (that is the Solution Overview's job), and it MUST END WITH a `Read in
  order:` signpost of the subsections present in document order, each with a one-clause
  purpose — a single compact line when small, grouped by service/contract cluster when large,
  so a downstream reader navigates the section without re-reading it. It is a projection of
  THIS section's body — every claim readable in the subsections it maps — and it neither
  restates the Solution Overview nor re-lists Document Consumption Guide rows.
- Planning Deferrals holds ONLY class-(d) bounded developer-HOW items the design
  owns and routes to planning as `PD#` rows; a grooming CHOICE stays a `Q#`, a
  consumed integration's internal choice stays with its change, a signed
  requirement is built. The section and its guide row are ALWAYS present; both
  read `None` when the design defers nothing.
- CONSUMED-INTEGRATION FIREWALL (check FIRST): I minted NO Q#, NO proposed-direction
  block, and NO settled/hardened Decision, Risk-mitigation, Cross-Cutting row, or Data
  Flow annotation for a CONSUMED integration's INTERNAL choice (call placement / client-
  vs-proxy, key exposure, quota governance) — those belong to the owning integration
  change, carried here at most as a consumed-dependency risk whose mitigation DEFERS to
  that change (no tier, no directional verb). A new / unsigned Change Request ask (e.g.
  an appointment-booking capability no change owns) appears only as one Solution Overview
  non-goal line — never a Decision, Risk, Dependency, or Open Question.
- For an INTEGRATION change, the Solution Approach is decided FIRST (build-vs-reuse): an
  explicit user approach signal is honoured, else the ladder reuse > SDK > from-scratch is
  applied via Enumerate-then-Justify with a per-rung fit verdict; nothing is invented
  beyond what discovery's prior-art surfaced. A FUNCTIONAL change makes NO Solution
  Approach decision and never decides a consumed integration's call approach.
- For an integration change, the Integration Specification is grounded in the vendor's /
  cartridge's official documentation and SHAPED by the chosen approach — from-scratch/API
  → concrete endpoints/params/auth + field-level data mapping; SDK → the SDK interface
  (init/config, methods, events, lifecycle) with NO invented request/response field
  table; reuse → the cartridge's config/extension contract with NO invented vendor
  endpoints/fields. For a functional change it is N/A with a pointer to the owning
  integration change(s). Nothing is fabricated, and the ABSENCE of endpoints/data-mapping
  for the SDK/reuse shapes is correct, not a gap. Only OUR build (per the Architecture
  decisions rule) is deferred to technical-design.
- Decision inventory is complete AND not padded: walk the discovery trace — its
  tables AND its prose (Executive Summary, Source Inventory notes, cited technical /
  meeting notes) — and confirm every settled architecture choice or
  solution-narrowing constraint (capability shape, vendor choice, key / security
  posture, loading / performance posture, data / privacy rule) is recorded as an
  Architecture Decision (or Cross-Cutting row) with rationale, alternatives, and
  trade-offs. A behaviour requirement is HONOURED in the Solution Overview and in
  the rationale of the decision that realises it — it is NOT restated as its own
  Architecture Decision. The design neither defers its settled architecture nor
  manufactures a decision per requirement row.
- Approval discipline holds (per KERNEL "classify by disposition" + MARKER-NOT-VERB):
  every sign-off-needing item is a proposed, Undecided-marked direction mirrored in
  Open Questions — none asserted as settled/approved and no value invented to close a
  Q#; only discovery-signed / source-grounded facts are stated as settled. Deferred /
  later-phase / unsigned-CR / owned-elsewhere items appear only as a Solution-Overview
  non-goal line, never as a Decision, Dependency, Cross-Cutting row, Integration
  Specification entry, or Open Question.
- A discovery Open Question's value — especially a redirect TYPE (`301` / `302`) —
  appears ONLY inside its `Decision (proposed … Q#)` block with `Status: Undecided`.
  EVERYWHERE else in the artifact it is a hardening: not just the obvious tables but
  any narrative subsection too, including a background account of the legacy surface,
  where "the retired URL will use a permanent redirect" settles the Q# exactly as a
  decision row would. Check every section, not a remembered short list. A parenthetical such as `(decision pending / deferred to
  Q#)` does NOT license stating the settled value (the `301`) elsewhere — that still
  hardens the Q# (per KERNEL MARKER-NOT-VERB).
- Dependencies pass the Dependencies rule: every row is a DELIVERY this build waits on,
  traced to a signed requirement or the specified contract; no owned / modified /
  removed surface (an owned MODIFY / REMOVE capability — a re-pointed navigation entry,
  or a retired legacy page — is an Affected Capabilities row, NEVER a Dependencies row),
  no platform-managed surface there is nothing to wait for (client-administered
  back-office settings, content components another workstream authors — those are Gap
  Analysis evidence),
  no unsigned CR, no operational task merely gated on a Q#; an integration table is not
  a bare `None` when its contract calls an authenticated / metered API. Every Type is a
  member of the closed set (functional: `upstream integration change` / `existing service`;
  integration: `account` / `credentials` / `billing` / `sandbox` / `environment`) — a
  discovery synonym (`external integration`, `data service`, `functional surface`) is
  remapped to a member, never emitted.
- Open Questions provenance: a design Q# is EITHER (a) a discovery Q# for THIS
  flow, carried forward and refined, OR (b) a NEW architecture question the
  design itself surfaces while shaping the solution — a genuine choice with
  multiple viable options that affects scope / risk / timeline and would cause
  rework if deferred (the same bar grooming uses to raise a new architecture
  question). Design is EXPECTED to raise (b): questions that could not exist at
  discovery time because they only arise once the solution is being shaped are
  yours to open, not something to suppress as "already handled at discovery".
  What a design Q# is NOT: anything the PIPELINE 7a owner gate classes as ANOTHER
  CHANGE'S or as NOT A CHOICE OF MINE TO OPEN.
- The (b)-question hunt (PIPELINE 7a) actually ran: I diffed the target solution
  against the current implementation / baseline specs, put every divergence through
  the owner gate, and either opened each surviving MINE divergence as a new design Q#
  with a proposed direction, or recorded `None` as an EVIDENCED result that names
  what I inspected. A hunt whose divergences all classed away legitimately records
  `None`, and that `None` is a PASS. A bare `None` justified by "all discovery
  questions resolved" is still a FAIL — that inheritance covers product / scope, not
  design-time architecture.
- Each in-scope UNDECIDED architecture CHOICE carries a proposed direction
  (qualitative lean + main alternative + trade-off), Undecided-marked and mirrored
  in Open Questions, never bare and never asserted — and the lean stays inside its
  block (no Risks / Data Flow / Dependencies / Cross-Cutting section hardens it into
  a settled mitigation, path, or posture), per KERNEL. "In-scope undecided" is a
  discovery Q# for THIS flow or a new (b) architecture question that passed the owner
  gate as MINE — nothing else earns a proposed direction. An UNDECIDED fact / status
  / value / policy is recorded
  as pending, never given a direction or a "confirmed" status (per KERNEL
  CHOICE-vs-FACT).
- Gap Analysis keys exactly the two-part set — every in-scope WBS code of the trace PLUS
  every owned capability no in-scope WBS row covers — none dropped, none added (no
  deferred / out-of-scope requirement, no component another change owns) —
  every Current State cell carries its anchor, every `Gap` value comes from the closed
  set, and no cell names a value an Open Question leaves open. The table header is exactly
  `WBS | Current State | Desired State | Gap | Notes` — the first column stays `WBS` even
  where a part-(ii) row keys on a capability. `### Legacy Behaviour Overview` is present exactly when
  a legacy implementation of the behaviour exists.
- For a functional change, the Integration Specification is only the
  `N/A — functional change.` line plus a pointer (no Overview / Endpoints
  / Data Mapping / Data Flow subsections and no vendor endpoint/field detail).
- No OUR-build implementation detail leaked in ANY section (per the Architecture
  decisions rule — our paths / wiring, import mechanics (a `dynamic import()`),
  signatures / headers, own tuning numbers (a chosen TTL / debounce / cache-size, a
  req/s or ms budget), and task sequencing stay for technical-design.md). Vendor
  contract detail (the third-party's endpoints/paths, params, documented codes and
  quota/TTL) is NOT a leak: it belongs in the Integration Specification of an
  integration change, grounded in cited docs.
- Base Configuration & Cross-Cutting cells carry NO chosen number: no `N per page`,
  `N stores`, `NNN ms` debounce, `N min` / `N h` TTL, `< N KB` bundle, or `< N s`
  FCP / TTI target. State the qualitative posture instead (a small result batch, a
  debounced input, a short-lived cache, a bundle-size budget, a fast-first-paint
  target); the concrete number is technical-design's.
- Data Flow is boundary-neutral: the diagram names NO vendor endpoint / path / param /
  field and NO chosen key-transport, and it is not split into client-vs-proxy branches
  when the call placement is an open Q# or the approach is SDK / reuse (no vendor endpoint
  exists to draw). The concrete wire, if any, lives only in the Integration Specification
  contract for an API shape.
- Every architecture decision has rationale, alternatives, and trade-offs.
- Base Configuration & Cross-Cutting rows are grounded and limited to concerns
  that affect the solution choice; non-applicable concerns are omitted.
- Nothing is fabricated: decisions, constraints, and patterns trace to the
  discovery trace, existing specs, the current implementation, or cited docs.
- The Design Gate is present after the body with each item marked checked ([x]);
  an unchecked item is left only when it genuinely does not hold.
