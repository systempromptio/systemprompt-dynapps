---
schema: openspec-feature
artifact: design
context:
  - .context/initial-context
  - .context/integration-docs
  - discovery/final
  - specs/global-navigation/spec
  - specs/store-directory/spec
sources:
  - ../../../../templates/openspec/schemas/feature/templates/design.md
  - ../../../../../skills/project-context/SKILL.md
scoring:
  pass_threshold: 0.7
---

# Design Standard — Contract and Rubric

This is the canonical definition of a good architecture / solution `design.md`
working copy in the OpenSpec `feature` workflow. The judge uses it to score a
candidate design. It is intentionally explicit and self-contained: do not assume
the judge has any ambient project knowledge beyond this standard, the inlined
SOURCES, and the CONTEXT (the source packet and the reconciled discovery trace
the design derives from).

## What a design artifact is (essence)

- A design artifact records **architecture / solution decisions, constraints, and
  trade-offs** that shape a change before Planning. It lives as a working copy at
  `openspec/changes/<name>/design.md`.
- It is **SA-level**, not developer implementation: it does NOT own file maps,
  component/route/loader/action wiring, endpoint implementation details, or task
  sequencing. Those belong to `technical-design.md` during `/opsx:plan`.
- It is the **standard architecture step right after discovery and before
  proposal**, produced for every change once discovery resolved its scope; the
  change's architecture-bearing concerns set its depth, not whether it exists.
- It runs **pre-grooming**: it PROPOSES decisions and RAISES the ones needing
  human sign-off as open questions/risks for the single downstream grooming to
  approve. It does NOT wait for or assert grooming approval.
- It derives from the reconciled **discovery** trace, existing specs, and the
  current implementation; it is not a source of truth for requirements. Behaviour
  authority stays with the specs and the signed requirements.
- Its **delivery scope (functional vs integration) is inherited from the discovery
  trace in CONTEXT**, not re-decided here. A `functional` design covers the
  storefront solution and treats integrations as dependencies it consumes; an
  `integration` design covers the agnostic service capability and treats consumer
  UI / other integrations as out of scope. The design must match the scope its
  discovery trace resolved, and weight itself accordingly (functional = thinner
  base design; integration = richer service design).

## Invariants the candidate MUST satisfy

1. **Warranted + gated.** The candidate is a genuine SA-level design: it states
   why design is warranted and references the related grooming/discovery decisions
   it builds on. It does not read as a restated requirements list or an FSD.
2. **Scope fit (inherited from discovery).** The design matches the delivery scope
   of the discovery trace in CONTEXT. A functional design owns the storefront
   solution and treats named integrations as consumed dependencies; an integration
   design owns the agnostic service capability and puts consumer UI / other
   integrations out of scope. Re-deciding scope, or mixing the other scope's
   ownership in, is a scope-fit failure.
3. **Architecture, not implementation.** No developer-implementation detail: no
   file maps, no component/route/loader/action wiring, no our-code function/module
   names, no task sequencing. Codebase patterns are referenced only to justify a
   decision, at architecture level.
   What this doc EXISTS to carry is the opposite of a leak: an architecture
   decision, strategy, or posture that a developer LATER implements is the PRODUCT
   of the design, never a violation, wherever it appears (Architecture Decisions,
   Base Configuration & Cross-Cutting, Integration Specification, or Risks /
   Mitigations). This explicitly includes a rendering model (SSR/RSC page-shell,
   client boundary), a load / prefetch strategy (lazy-load, prefetch-on-focus), a
   caching posture, a throttling / quota-governance posture (rate-limit, debounce,
   server-proxy fallback), a key / auth policy, and a dependency-risk mitigation
   (version-pinning, changelog monitoring, contract tests). Naming the TECHNIQUE is
   architecture; only performing it is implementation.
   The leak is narrow and is exactly ONE of two things: (a) OUR named code artifact
   — an internal file / module path or route / loader / action WIRING of our build
   (e.g. `src/routes/_app.store-locator.tsx`, a named loader / action / component /
   variable). A user-facing URL / page path / redirect target (e.g. `/store-locator`
   or a `/stores` → new-page redirect) is NOT this: choosing site URLs, slugs, and
   redirects is an IA / SEO / navigation architecture decision the SA owns. (b) a
   specific magnitude VALUE WE chose (ms, KB, %, req/s, TTL hours, a pinned version)
   that is not a cited vendor-contract literal. A standard protocol constant or
   status code (an HTTP `301` / `302` / `404`, a standard cache-control directive)
   is NOT this: selecting the correct standard semantics — a `301` permanent vs a
   `302` temporary redirect — is an architecture / SEO decision, not a tuned number.
   Tie-breaker: if the text says WHAT to do and WHY and a developer would still
   have to implement it, it is architecture; if it IS the implementation or the
   exact tuned value, it is a leak — when genuinely ambiguous, treat it as
   architecture. A single stray chosen value inside an otherwise architecture-level
   decision / risk is a WARNING (tighten it to a policy), not a critical; escalate
   to a critical only when implementation detail is pervasive or names OUR code.
   Exception — the Integration Specification of an integration-scope design IS the
   service contract and MUST be concrete, IN THE SHAPE OF the chosen Solution Approach:
   for a FROM-SCRATCH / API approach — real vendor endpoints / paths, methods, query
   params, auth, field-level request/response mapping, documented quotas / rate-limits
   / cache (TTL) guidance, and status/fault codes; for a VENDOR SDK approach — the SDK
   interface (initialization / config, methods with inputs/returns and when-called,
   subscribed events with payload, lifecycle), with endpoints ONLY for real network
   assets (a loader/CDN URL, an allowlist origin) since an in-browser SDK call is not
   an endpoint; for a REUSE cartridge/module approach — the cartridge's configuration /
   extension contract. Plus the data flow. That concrete detail is NOT an
   implementation leak here — it is the contract, and for the SDK / reuse shapes the
   ABSENCE of an endpoints/data-mapping field table is CORRECT, not a gap. The only
   guard is grounding: every concrete value (endpoint, param, method, event, config
   key, code, number) MUST trace to a cited vendor / cartridge doc — an invented one,
   OR an endpoint/field a vendor SDK or cartridge does not actually expose, is a
   fabrication. What remains out even inside this section is OUR internal build — our
   file / module layout, our route / loader / action wiring, and our code — which
   belongs to technical-design.
4. **No fabrication (invent nothing).** No decisions, constraints, dependencies,
   integrations, RQ/section codes, or approvals that are not grounded in the
   provided CONTEXT. Undecided items and unsigned asks are NOT presented as
   decided; they appear as risks/open trade-offs or non-goals.
5. **Sign-off discipline (pre-grooming).** Decisions needing human sign-off are
   PROPOSED and RAISED as open questions/risks for grooming, never asserted as
   approved or final. Open Questions the discovery trace still carries as undecided
   (e.g. a third-party client-vs-proxy call approach, or a retired-URL redirect)
   and unsigned Change Request candidates (e.g. appointment booking) MUST NOT be
   stated as decided design constraints. Only discovery-signed facts and clearly
   source-grounded constraints are stated as settled. This binds each flow to its
   OWN discovery Open Questions: a question another change owns is not this design's
   to keep open. In a functional (consuming) design the third-party call approach /
   key placement is the integration change's question, so a functional Risks row
   noting "keys exposed if called client-side" with a server-proxy mitigation is a
   consumed-dependency risk, not a hardening violation. Fire this dimension only
   against an item that appears in THIS flow's own Open Questions table.
   Distinguish a GROUNDED FACT from an OPEN SUB-CHOICE before flagging. A base fact
   a CONTEXT source states or unambiguously forces (e.g. Cartora keys are
   domain-restricted; billing sits on the client's Cartora account) is
   source-grounded and is correctly stated as settled — EVEN WHEN a narrower
   refinement of the same subject is an open Q (the exact guardrails, quota, key
   count, environment split). Only the specific sub-choice the discovery Q actually
   leaves open (the guardrail values, the client-vs-proxy call placement, the auth
   transport) must stay undecided; hardening THAT sub-choice is the violation,
   restating the grounded base fact is not. When a grounded constraint and an Open
   Question share one subject, stating the grounded part while raising the open part
   as the Q is the correct handling, not a hardening violation.
   For a discovery-undecided item the COMPLETE and correct handling is to raise it
   as an Open Question/risk — optionally with a clearly-labelled PROPOSED DIRECTION
   (chosen lean + rationale + alternatives) whose status is explicitly
   undecided / raised-for-grooming and mirrored in Open Questions. Raising it this
   way fully satisfies the discipline: a clearly-labelled undecided proposed
   direction is NOT an approval-discipline violation, and the ABSENCE of a settled
   decision on a discovery-undecided item is NOT a gap. The violation is only
   asserting such an item as settled — presented as a plain decision with no
   undecided status and not mirrored as an Open Question.
   Eligibility for a proposed direction is limited to an item the discovery trace
   itself carries as one of its OWN Open Questions. An unsigned / pending Change
   Request candidate (e.g. appointment booking), a deferred / later-phase item
   (e.g. real-time inventory), and anything the trace hands to another change are
   NOT eligible: each gets at most one non-goal line in the Solution Overview and
   NO decision block, NO proposed-direction block, NO dependency row, and NO
   open-question row. Giving such an item a proposed direction — even one labelled
   undecided — or any decision / dependency / question row is a fabrication /
   approval-discipline violation, because a block or row asserts the item is live
   in this design's scope.
6. **Decision quality.** Each architecture decision records the chosen approach
   with rationale (why this option), alternatives considered, and trade-offs — not
   a bare assertion.
7. **Grounded cross-cutting constraints.** Cross-cutting rows are limited to
   concerns that actually affect the solution choice, risk, or client/BA sign-off,
   and each is justified. Non-applicable concerns are omitted, not padded.
8. **Template & structure conformance.** Follows the `design.md` template section
   order and headings — GOLDEN is the canon for the exact set; the typical shape is
   Solution Overview, Affected Capabilities, Design Source Inventory, Gap Analysis,
   Architecture Decisions, Base Configuration & Cross-Cutting, Integration
   Specification, Risks / Mitigations, Dependencies, Open Questions, Design Gate.
   Tables use the template columns. A deterministic code gate already enforces exact
   section presence, order, and table columns against the golden; score this dimension
   for heading fidelity only and do not re-litigate what that gate covers.

## Forbidden (each is a critical finding when present)

- Developer-implementation detail leaking in: file-by-file changes, component/
  route/loader/action wiring, endpoint implementation, or a task/work breakdown.
- Any fabricated decision, dependency, integration, code, or approval not grounded
  in the CONTEXT.
- Presenting an undecided Open Question or an unsigned Change Request as a decided
  solution constraint, or asserting grooming approval (sign-off-discipline
  violation). Design runs pre-grooming; it proposes and raises, it does not approve.
- Restating requirements/acceptance criteria as if the design were an FSD or a
  discovery trace.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE place.

- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A difference from
  GOLDEN, or something GOLDEN omits, is NEVER by itself a finding. Every finding MUST
  quote the candidate text AND the STANDARD rule or CONTEXT fact it violates. "Not in
  GOLDEN" or "not mentioned" alone is not evidence — especially against a proposed
  design direction, an alternative, a trade-off, or a foreseeable risk. GOLDEN is
  deliberately austere about mechanisms and numbers; that austerity is one
  acceptable minimal example, not a maximum. A candidate that carries MORE
  architecture — an extra decision, strategy, posture, or risk mitigation — is not
  leaking merely because GOLDEN stated it more qualitatively; judge the added
  content against the STANDARD's leak definition, not against GOLDEN's brevity.
- No-Fabrication trace gate (closed rule — run these steps IN ORDER before scoring
  `no_fabrication`, and record the working in `gates[]`). A fabrication is a CLOSED-world
  literal PRESENT in the candidate that no CONTEXT (or GOLDEN) source supports — never an
  omission, never a judgement.
  - PARTITION first. CLOSED-world = requirements, identifiers, statuses, approvals,
    ownership, and vendor-contract literals (endpoints, fields, codes, quotas, and the auth
    TRANSPORT — header vs query param vs a `Bearer`/scheme the docs do not state). OPEN-world
    = design synthesis: alternatives, trade-offs, reasonable risks and mitigations, an
    eligible proposed direction, a mapping / surfacing / omission of a documented vendor
    field, and any Gap distance GRADE (`none` / `config` / `adapt` / `rebuild` / `remove`) —
    a grade is a judgement, so it is NEVER a fabrication under any wording ("fabricates token
    `none`", "invented verdict"). EXCLUDE every open-world item here; it is judged for sound
    reasoning grounded in the architecture, not for verbatim presence in a source.
  - ENUMERATE the closed-world literals the candidate actually contains, quoting each. This
    is the gate's `enumerated` set.
  - TRACE, per literal. Locate and quote the CONTEXT / GOLDEN line that supports it — a
    vendor-contract value the cited docs DO define (a documented quota, a `Cache-Control`
    directive, an endpoint, a field) is GROUNDED, so EXCLUDE it. Stating an auth transport
    the vendor docs leave unspecified is unsupported. A literal with no supporting line to
    quote SURVIVES as a `no_fabrication` critical, evidence = the quoted candidate literal.
  Only a survivor reaches `critical_findings`; a literal you cannot copy verbatim from the
  candidate is a hallucination, not a fabrication.
- Flow-scoping. Judge the candidate ONLY against ITS OWN flow's scope and Open
  Questions (the discovery trace in CONTEXT for this flow). The "undecided items"
  that must stay unhardened are exactly the Open Questions THIS flow's discovery
  carries — never import the other flow's questions. In a functional (BA) design the
  third-party CALL APPROACH (client-vs-proxy geocoding, key placement) is owned by
  the integration change the feature consumes, NOT a functional Open Question: a
  functional risk that notes "keys exposed if called client-side" with a mitigation
  is a consumed-dependency risk, not a hardening of a functional undecided item, and
  is not an approval violation. Fire a hardening / approval finding only against an
  Open Question that literally appears in THIS flow's Open Questions table.
- Scope-Placement gate (closed rule — run these steps IN ORDER before scoring
  `scope_fit`, and record the working in `gates[]`). A scope-fit misplacement is a real,
  grounded surface sitting in the WRONG section — it is judged from the candidate's OWN
  Affected Capabilities and the CONTEXT ownership, NEVER from whether a row matches a
  GOLDEN row. Ownership and placement are two axes: OWN = a capability THIS change adds /
  modifies / removes; CONSUME = an upstream delivery / existing service / operational
  prerequisite the build waits on but does not own. The Dependencies table is NOT a copy
  of discovery's Dependencies, so a difference from discovery's rows is never by itself a
  finding.
  - ENUMERATE the OWNED set: from the Affected Capabilities section, list each capability
    with its disposition (`ADD` / `MODIFY` / `REMOVE`). These are the surfaces this change
    owns; their correct home IS Affected Capabilities.
  - ENUMERATE the Dependencies rows: list each row VERBATIM from the Dependencies table as
    `<subject> | <Type>`. This is the gate's `enumerated` set for the placement check. A
    "misplaced in Dependencies" critical may cite ONLY a row in THIS enumerated set — a
    surface that appears solely in Affected Capabilities (or any other section) is
    CORRECTLY placed and can NEVER be a Dependencies misplacement; asserting it is one is a
    hallucination, drop it.
  - TEST each Dependencies row, and EXCLUDE by the first rule that hits:
    - Subject is a member of the OWNED set, OR its Type is outside the closed set
      (functional: `upstream integration change` / `existing service`; integration:
      `account` / `credentials` / `billing` / `sandbox` / `environment`) such as
      `internal capability` / `functional change` / `static content` -> it is an owned
      surface in disguise: it SURVIVES as a `scope_fit` critical, and a Note like "MODIFY /
      owned by this change" or "will be updated here" is confirming evidence. A framing
      like "depends on the <X> integration change / to be delivered" is a consumed upstream
      delivery, NOT owned -> EXCLUDE (`consumed`).
    - Subject is a platform-managed surface neither this change nor an upstream delivery
      owns — settings the client's own administrators configure in the platform back
      office, or content components another workstream authors -> its home is the Gap
      Analysis evidence or Affected Capabilities, but listing it here — under any Type,
      in-set or not — is a placement WARNING, not a critical, because the fact is real and
      grounded and no scope is claimed by it. Its ABSENCE here is likewise correct and
      never a `missing_evidence` item or a finding. EXCLUDE from criticals
      (`platform-managed -> warning`).
    - Otherwise (a consumed upstream delivery or an already-integrated service, functional;
      an operational prerequisite, integration) -> EXCLUDE (`grounded dependency`). In an
      INTEGRATION design the rows are its own operational prerequisites and it does NOT list
      the service it owns.
  - TEST each Affected Capabilities row: is its subject a consumed service or a capability
    ANOTHER change owns (not this change's own ADD / MODIFY / REMOVE)? If YES -> it
    SURVIVES as a `scope_fit` critical (a consumed service's home is Dependencies). Else
    EXCLUDE (`owned`).
  - An unsigned / pending Change Request or a deferred / later-phase item placed as a
    Dependencies row is `approval_discipline`, not `scope_fit` — route it there and do not
    also fire here.
  Only a survivor reaches `critical_findings`, and its evidence MUST be the verbatim row
  you enumerated. A misplacement is always `scope_fit`, never `no_fabrication` (the fact is
  real, only its section is wrong).
- Affected Capabilities section. This section is the owned-scope footprint: one row
  per capability THIS change adds / modifies / removes, stated at capability /
  behaviour level and grounded in the capability's baseline spec (an ADD capability
  has no baseline). Naming an owned capability and WHAT changes and WHERE at behaviour
  level is the section's PRODUCT, not a leak or fabrication — judge it for owned-scope
  completeness and correct disposition against CONTEXT, and flag `no_fabrication` only
  for an owned capability with no CONTEXT basis, `scope_fit` for a consumed service or
  another change's capability listed here, and `architecture_not_implementation` only
  for OUR named code artifact / route wiring (a user-facing surface or URL is not a
  leak). A consumed service the change does not own belongs in Dependencies, not here.
- Gap Analysis gate (closed rule — run these steps IN ORDER before scoring `gap_analysis`,
  and record the working in `gates[]`). This section carries the FACTS decisions are made
  from: one row per in-scope requirement (functional) or owned obligation (integration),
  keyed on the discovery trace's WBS codes, with `Gap` drawn from the closed distance set
  `none` / `config` / `adapt` / `rebuild` / `remove`. The distance GRADE is the designer's
  judgement — two competent architects grade a row differently — so it is NEVER scored and
  NEVER a finding, in either direction (a lighter or a heavier verdict you would have
  written is not a defect). Only membership, Current-State honesty, and coverage are judged.
  - ENUMERATE. List every Gap row as `WBS | <verbatim distance token>`. This list is the
    gate's `enumerated` set.
  - MEMBERSHIP, per row. Is the verbatim token one of the five members? If YES, EXCLUDE it
    (`member of set`) — the grade closes here; you may NOT name a different member as the
    "correct" one, because naming a member proves the token was inside the set. If NO (the
    literal token is none of the five), it SURVIVES as a `no_fabrication` critical, evidence
    = the quoted `WBS | token`.
  - HONESTY, per row's Current State prose. Does it assert a platform / legacy capability,
    status, or anchor? If it does, quote that clause AND the CONTEXT line it contradicts —
    both quotes present, it SURVIVES as a `no_fabrication` critical (e.g. "the storefront
    has no localization mechanism" when a source states it is already multi-locale). No
    contradicting CONTEXT line to quote -> EXCLUDE (`grounded`). A terse negative scoped to
    THIS feature ("no locator copy yet", "not present on the new surface", "none") asserts
    nothing about the platform -> EXCLUDE (`thin`); it is at most a warning, never a critical.
  - COVERAGE. A dropped in-scope WBS row, a row for an out-of-scope / deferred item or a
    capability ANOTHER change owns, or an invented row key (a key naming no capability in
    CONTEXT) SURVIVES as `scope_fit`; a cell that names the value an Open Question of THIS
    flow leaves open SURVIVES as `approval_discipline`. A redundant capability-level row for
    a capability THIS change OWNS whose coverage a WBS / behaviour row in the same table
    already carries (a capability-name row that merely restates an owned WBS row) is NOT a
    coverage breach — the scope is already covered, only restated — so it is at most a
    redundancy WARNING, never `scope_fit`. Judge coverage by ownership + CONTEXT, never by
    whether a row matches a GOLDEN row.
  Only a survivor reaches `critical_findings`. The distance verdict assesses the MECHANISM
  the requirement needs, not this change's own instance of it — a requirement satisfied by
  authoring content / keys / settings against a mechanism the platform already provides is
  `config` even though nothing exists for THIS feature yet; the trace's delivery mode
  (`build` / `consume`) is a different axis and never dictates it — but that only informs
  the designer's grade, never your re-grading. `### Legacy Behaviour Overview` is
  CONDITIONAL: expected only when CONTEXT shows a legacy implementation of the behaviour;
  its absence when there is none is correct and never a finding. A `remove` row IS the
  removal record — no second removal table is expected; a removal the discovery trace
  already signed is carried as a fact, and only a removal the design itself newly proposes
  must be raised rather than decided.
- Solution Overview & Document Consumption Guide (the derived interface). These two
  blocks sit at the top of the artifact (Solution Overview, then the guide) and are
  DERIVED from the body — a downstream reader consumes them before the detail
  sections. Judge them under `handoff_fidelity`, and derive their correct content
  from the body, never from GOLDEN's wording.
  - The Solution Overview is a READER MAP, not a summary dump: it orients a reader to
    what the design settles, why it is shaped that way, what constrains downstream, and
    what is passed forward, by SUBJECT and COUNT (e.g. "seven architecture decisions,
    two raised for grooming"; "two items routed to planning"). Stating a subject and a
    count, or naming a non-goal in one line, is the intended reader-map shape. Two
    defects only: a count or claim the body CONTRADICTS (e.g. "six decisions" when the
    body carries seven `D#`) is a `handoff_fidelity` deduction; RE-LISTING table rows —
    a `D#` roll-call, a per-question walkthrough, a row-by-row inventory that belongs to
    the guide — is a reader-map warning. Prose that agrees with the body is correct
    however worded; a plainer or richer phrasing than GOLDEN's is never a defect.
  - The Document Consumption Guide is a MAP THAT FOLLOWS THE DOCUMENT, not a fixed row
    set. Its rows are DERIVED: one row per `##` section the body carries, in document
    order, EXCLUDING only Solution Overview, the guide itself, and Design Gate. FIRST
    enumerate the body's `##` sections to get the correct row set — every section the
    body carries EARNS a row (Planning Deferrals is always present and always earns one,
    reading `None` when the design defers nothing); a section the body does not carry has
    NO row and its absence is CORRECT, never a drop. A row
    for a section the body does not carry is drift. THEN check each row's `Key data`
    against the section it names — Architecture Decisions → the `D#` set (a proposed
    `D#` notes its `Q#`); Open Questions → the `Q#` set with status; Planning Deferrals
    → the `PD#` set; Affected Capabilities → owned capabilities with ADD/MODIFY/REMOVE;
    Integration Specification → contract keys or `N/A — functional change`; and so on,
    with an empty set written `None`. `Key data` is identifiers and sets, not prose.
    A dropped section-row the body carries, or a `Key data` set the section contradicts,
    is a `handoff_fidelity` defect; ordering/wording/separator differences from GOLDEN
    are not. A `What it settles & why` phrase is judged only for grounding.
  - Planning Deferrals is ALWAYS present and design-owned: it holds only bounded
    developer-HOW items design routes to planning (`PD#`), never grooming choices
    (those are `Q#`), consumed integration internals, or signed requirements. When there
    is nothing to defer it reads `None.` — a present, empty section, never an omission
    and never a `missing_evidence` item. When it carries rows, each `PD#` row carries a
    source, the deferred item, why it is not decided now, a target phase, an owner, and a status.
    Judge it under `handoff_fidelity` for completeness of the register; misclassifying a
    grooming choice as a `PD#` (or vice versa) is the defect to catch.
  - The Integration Specification `### Overview` is a LOCAL DERIVED reader-map for that
    one section (INTEGRATION changes only; a functional design renders the section
    `N/A — functional change` and this bullet does not apply). It is 1-2 sentences that
    LEAD WITH the contract SHAPE (API / SDK / cartridge) — naming each service/contract and
    how the section splits when there is more than one — and what the section defers out,
    NOT re-describing what the capability settles (that is the Solution Overview's job) —
    then a `Read in order:` signpost of the subsections present in document order, a single
    compact line when small and grouped by service/contract cluster when large. Judge it under
    `handoff_fidelity` for AGREEMENT with the section body: the shape it claims matches the
    subsections actually present (an SDK-only contract with no Data-Mapping table must not be
    signposted as carrying one), and the `Read in order:` signpost names real subsections in
    their document order. Two defects only: a claim the section body CONTRADICTS (a
    signposted subsection that is absent, a wrong order, a shape the contract does not take)
    is a `handoff_fidelity` deduction; RESTATING the Solution Overview, RE-LISTING Document
    Consumption Guide rows, or dumping contract detail into the Overview is a reader-map
    warning. Prose that agrees with the body is correct however worded; the Overview's
    absence on an integration design is a defect, its absence on a functional design correct.
- One defect, one primary critical. Classify by the actual fault:
  - a fact / contract literal not in CONTEXT -> `no_fabrication`;
  - a real, grounded fact placed in the wrong scope / ownership -> `scope_fit`
    (NOT `no_fabrication` — it was not invented, only misplaced);
  - a discovery-undecided item or unsigned ask asserted as settled, or a proposed
    direction / dependency / question row given to an ineligible item ->
    `approval_discipline`;
  - OUR-build realization detail — narrowly: OUR named code artifact (internal
    file / module path or route / loader / action wiring, a named component /
    variable) OR a specific magnitude WE chose that is not a cited vendor-contract
    literal -> `architecture_not_implementation`. A decision / strategy / posture a
    developer later implements is NOT this, and neither is a user-facing URL / page
    path / redirect target or a standard protocol status code — those are the doc's
    product.
  Do not raise the same underlying claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension. Before recording a
  finding, check whether a STANDARD rule permits the thing; if it does, do not file
  it (and do not re-file it under a different dimension). Three the STANDARD
  explicitly permits: (a) a build-skill anchor (`sfnext-*` / `b2c-*`) cited in a
  Base Configuration row — the template's grounding convention, NOT a tool-name
  leak; (b) surfacing, renaming, or omitting a documented vendor field (e.g. a
  display `label`) — a design choice, not fabrication or a scope violation; (c) a
  clearly-labelled undecided proposed direction on a discovery Open Question — not
  an approval violation; (d) a named architecture technique / strategy / posture a
  developer later implements (rendering model, lazy-load / prefetch strategy,
  caching or throttling / quota-governance posture, key / auth policy,
  dependency-risk mitigation such as version-pinning or contract tests) — the
  PRODUCT of the design, NOT an `architecture_not_implementation` leak. The leak is
  only OUR named code artifact or OUR chosen numeric / version value; (e) a
  user-facing URL / page path / redirect target (e.g. `/store-locator`, a `/stores`
  redirect) and a standard protocol constant / status code (HTTP `301` / `302` /
  `404`, a standard cache-control directive) — an IA / SEO / navigation
  architecture decision, NOT a code-path or tuned-value leak. The route/path leak
  is only an INTERNAL module / file / loader / action of our build (e.g.
  `src/routes/_app.store-locator.tsx`); (f) an Integration Specification whose
  contract is an SDK interface (methods / events / init-config) or a cartridge
  configuration / extension contract INSTEAD OF endpoints + field-level data-mapping,
  when the chosen Solution Approach is a vendor SDK or a reused cartridge/module — the
  absence of an endpoints/data-mapping table is the CORRECT shape, never a
  `missing_evidence` or fidelity gap; the fabrication guard is the reverse (inventing
  an endpoint/field the SDK/cartridge does not expose); (g) the Solution Approach
  decision itself (a build-vs-reuse ladder pick, or an Enumerate-then-Justify per-rung
  fit/no-fit verdict when no user approach signal is given) — the design's expected
  FIRST decision, scored under `decision_quality`, never padding or a leak; (h) a Gap
  Analysis distance grade you would have graded differently while its Current State prose
  is honest — the grade is the designer's judgement, excluded at the Gap Analysis gate's
  membership step, and never a finding under any dimension.
- Omission is not a critical — under ANY dimension. A missing expected item is
  `missing_evidence` (or a warning), never `no_fabrication` (fabrication requires an
  INVENTED item actually present in the candidate) and never `approval_discipline`
  (an omission asserts nothing). In particular, an integration `Dependencies: None`
  that should have listed an operational prerequisite is `missing_evidence`, not a
  critical: writing "None" is a gap, not an assertion that an undecided fact is
  settled. An omission is a critical only when a Forbidden item names it as one.
- Completeness does not offset a violation, and fluency is not correctness. Inspect each
  concrete number, OUR-stack tool name, endpoint / auth / default, dependency status, and
  approval verb on its own merits regardless of how polished the surrounding document is.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by the
dimension id below. For each dimension, 1.0 = fully satisfied with evidence,
~0.5 = partially satisfied or minor gaps, 0.0 = violated or absent.

- `warranted_and_gated` — states why design is warranted and the grooming/
  discovery decisions it builds on; reads as an SA design, not a requirements dump.
- `scope_fit` — matches the delivery scope inherited from the discovery trace
  (functional owns the storefront solution and consumes integrations; integration
  owns the agnostic service capability and excludes consumer UI / other services);
  scope is not re-decided or mixed.
- `gap_analysis` — the Gap Analysis section is complete and evidence-bearing: every
  in-scope WBS code of the discovery trace has a row and nothing else does; Current State
  cells are grounded rather than asserted; `Gap` values come from the closed set; and the
  conditional Legacy Behaviour Overview appears exactly when a legacy implementation
  exists. Score the EVIDENCE and the COVERAGE only — WHICH enum member a row grades is not
  scored here at all, in either direction. A thin cell is a ~0.9 with a warning. Reserve
  low scores for prose CONTEXT contradicts, a dropped in-scope row, padding or invented row
  keys, or a cell that closes an open Q#.
- `architecture_not_implementation` — stays at solution/architecture level. This
  dimension credits the doc's PRODUCT and penalizes only true build leaks; it is
  NOT a test of austerity. Full credit when every entry is a decision, strategy,
  posture, constraint, or risk mitigation that a developer LATER implements —
  rendering model (SSR/RSC shell, client boundary), load / prefetch strategy
  (lazy-load, prefetch-on-focus), caching posture, throttling / quota-governance
  posture (rate-limit, debounce, server-proxy fallback), key / auth policy, or
  dependency-risk mitigation (version-pinning, changelog monitoring, contract
  tests). Naming such a technique is architecture, not a leak, wherever it appears.
  The Integration Specification of an integration-scope design is the service
  contract and SHOULD carry concrete vendor-documented endpoints / paths, params,
  auth, field mapping, codes, and quota / TTL detail (each traceable to a cited
  source) — that is the contract, not a leak. A build-skill anchor (`sfnext-*` /
  `b2c-*`) cited for grounding is the template convention, NOT a leak.
  Count against this dimension ONLY: (a) OUR named code artifact — an internal file
  / module path or route / loader / action wiring, a named component / variable of
  our build (e.g. `DynamicImage`, `src/routes/_app.store-locator.tsx`); a
  user-facing URL / page path / redirect target (e.g. `/store-locator`) is NOT a
  leak — it is an IA / SEO / navigation decision; or (b) a specific magnitude VALUE
  WE chose (a TTL of N hours, ≤N req/s, N ms debounce, N KB, a pinned SDK version)
  that is not a cited vendor-contract literal; a standard protocol constant /
  status code (HTTP `301` / `302` / `404`) is NOT a chosen magnitude — it is a
  standard semantics decision. Tie-breaker: WHAT + WHY that a developer still
  implements = architecture; the implementation itself or the exact tuned value =
  leak; when genuinely ambiguous, score as architecture. A single stray chosen
  value inside an otherwise architecture-level decision / risk is a ~0.9 with a
  WARNING (tighten to a policy), not a 0.0; reserve low scores and a critical for
  pervasive build detail or OUR named code.
- `no_fabrication` — every decision/constraint/dependency/integration traces to
  the CONTEXT; nothing invented. Run the No-Fabrication trace gate first (partition out
  open-world synthesis and Gap grades, enumerate the closed-world literals, trace each to a
  quoted CONTEXT/GOLDEN line). In an Integration Specification, fabrication is a
  field, value, endpoint, code, or auth NOT present in the cited vendor docs — a
  concrete value the docs DO define is grounded, not invented. A field-mapping
  choice that is consistent with the docs (surfacing, renaming, or omitting a
  documented field such as a display label) is a design choice, not fabrication.
  Before calling any quota / rate / limit / TTL a fabrication, CHECK the cited vendor
  docs in CONTEXT first: a count the docs DO state (e.g. a documented "10,000 requests/day",
  "~50 req/s", `Cache-Control: max-age=86400`) is GROUNDED — never a fabrication, and
  repeating it is not a chosen-magnitude leak. At most, stating such a documented number in
  a cross-cutting / Risks / Affected-Capabilities row instead of the Integration
  Specification is a ~0.9 placement WARNING (tighten to a policy / move it to the contract),
  NOT a critical under `no_fabrication`, `architecture_not_implementation`, or
  `approval_discipline`. Only a concrete count with NO vendor-doc basis, invented to close an
  open Q (e.g. "N requests/day", "N/hour", a default `limit=N` the docs do not state), is a
  fabrication here — a critical. A RELATIVE threshold (a percent of an as-yet-unknown limit,
  e.g. "alert at 80% of quota") closes nothing and is at most a warning.
- `approval_discipline` — pre-grooming sign-off discipline: decisions needing
  sign-off are proposed and raised for grooming; undecided Open Questions and
  unsigned CRs are risks/non-goals, never stated as decided or grooming-approved.
  A proposed direction that is explicitly marked undecided / raised-for-grooming
  and mirrored in Open Questions SATISFIES this dimension ONLY for an item the
  discovery trace carries as its own Open Question. Asserting such an item as
  settled violates it — and so does giving a proposed direction (even one marked
  undecided), a dependency row, or an open-question row to an unsigned CR, a
  deferred / later-phase item, or an item handed to another change. A proposed
  direction that is marked undecided in its own block but then HARDENED elsewhere
  (a settled Risks / Mitigations entry, a fixed Cross-Cutting posture, or a Data
  Flow that commits the undecided physical tier — a browser-direct call vs a server
  proxy — or a tier-specific annotation such as a key-exposure mode) violates it
  too — the lean must read as undecided in every section. A Data Flow drawn at the
  capability boundary (consumer -> capability -> vendor) is Q-NEUTRAL and is NOT a
  hardening violation even as a single sequence: it settles no physical tier. Fire
  only when the diagram names the undecided tier or a tier-presuming detail as
  settled. Finally, an unresolved FACT / STATUS / VALUE / POLICY (account or
  credential existence, quota, auth transport, approval state) asserted as settled
  or "confirmed" when the source leaves it open violates it; a design may propose a
  direction for an open CHOICE, but it does not resolve an open fact or status.
  Discriminator — a HARD GATE, run these steps IN ORDER before firing this dimension:
  (1) NAME THE Q#. Point to the exact Q# that literally appears in THIS flow's own
  Open Questions table and that the candidate text closes. If you cannot name such a
  Q#, DO NOT fire — score 1.0 and stop. A sub-choice ANOTHER change owns is never in
  this table: in a functional (consuming) design the client-vs-proxy call approach and
  key placement are the integration change's questions, so a functional Risks /
  Base-Config mitigation mentioning a server proxy or client-side keys is a
  consumed-dependency risk and CANNOT fire here — quoting this carve-out and then
  firing anyway is the error to avoid.
  (2) CONFIRM IT IS SETTLED. The text must assert that Q#'s sub-choice as settled (a
  plain decision, a fixed cross-cutting posture, or a Data Flow naming the undecided
  physical tier) — NOT a clearly-labelled proposed/undecided direction. A clause that
  carries an inline undecided marker — `(decision pending)`, `(raised for grooming)`,
  `(undecided)`, `(proposed)`, or a `(Q#)` reference to this flow's Open Question — is a
  labelled proposed direction and does NOT fire, even when its verb sounds imperative
  ("Implement a permanent 301 redirect (decision pending)"): the marker governs, not the
  verb. A base fact a
  source states or forces — a domain-restricted key TYPE, billing on the vendor
  account, a read-only cacheable dataset — is grounded and PASSES even when a narrower
  refinement is an open Q; do NOT fire on it. Then, for each Q# in this flow's table,
  scan the WHOLE artifact — every section and every narrative subsection, tables and
  prose alike, not a short list of the usual suspects — for the SAME sub-choice hardened
  as settled anywhere in it. The lean must read undecided everywhere it appears, so a
  settled commitment outside its proposed block fires even when the block itself is
  labelled undecided, and a background narrative describing what will happen to a legacy
  surface is scanned exactly like a decision table.
  (3) An omission NEVER fires here: `Dependencies: None` that should have listed an
  operational prerequisite is `missing_evidence`, not an approval_discipline critical —
  writing "None" asserts nothing settled.
- `decision_quality` — each architecture decision has chosen approach + rationale
  + alternatives + trade-offs. This INCLUDES the Solution Approach (build-vs-reuse)
  decision: it should honour an explicit user approach signal, or — absent one — show
  the Enumerate-then-Justify per-rung fit/no-fit verdict and pick by priority (reuse
  cartridge/module > vendor SDK > from-scratch). A clean per-rung enumeration is
  correct, not verbose padding; a from-scratch pick while discovery surfaced a fitting
  cartridge/SDK and no user signal overrode it is a decision-quality miss.
- `cross_cutting_justification` — cross-cutting rows are applicable, grounded, and
  justified; non-applicable concerns omitted.
- `risks_and_mitigations` — real risks (including the undecided items the
  discovery trace still carries) captured with credible mitigations.
- `handoff_fidelity` — the derived interface at the top of the artifact is faithful
  to the body. The Solution Overview reads as a reader map (subjects + counts, no
  row roll-call) with no count or claim the body contradicts; the Document
  Consumption Guide maps the body's CURRENT `##` sections — one row each, in document
  order, excluding only Solution Overview, the guide itself, and Design Gate, with a
  row for every section the body carries (Planning Deferrals always among them, `None`
  when empty) and none for one it does not carry — and
  each row's `Key data` set agrees with the section it projects (`D#`, `Q#`, `PD#`,
  capabilities with direction, contract keys, `None` for empty). The always-present
  Planning Deferrals `PD#` register holds only design-owned developer-HOW items (or
  `None` when the design defers nothing). For an INTEGRATION change, the Integration
  Specification `### Overview` is likewise a local derived reader-map, faithful to that
  section — its claimed contract shape and its `Read in order:` signpost match the
  subsections actually present, in document order; its absence on an integration design is
  a defect, on a functional design correct. Score 1.0 when these blocks are complete and
  body-faithful; deduct per contradicted claim, dropped section-row, or drifted key-data
  set; a re-listing Solution Overview, an Overview that restates it or dumps contract
  detail, or a wording-only difference from GOLDEN is a warning, not a 0.0.
- `structure_conformance` — template section order, headings, and table columns,
  including the `## Document Consumption Guide` block (with its `Section | What it
  settles & why it matters downstream | Key data` columns) directly after the
  Solution Overview, and the always-present `## Planning Deferrals` block before the
  Design Gate (`None.` when the design defers nothing).

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it: if any
  item in `critical_findings` is present, `score` MUST NOT exceed 0.5.
- `pass` is true only when `score >= threshold` (given in the user message) AND
  `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or of invariants 1-5
  (warranted/gated, scope fit, architecture-not-implementation, fabrication,
  approval discipline). These block acceptance.
- **warnings** — quality/structure issues that should be fixed but do not by
  themselves make the design wrong (e.g. a decision missing its alternatives, a
  heading slip, a cross-cutting row that is thin but not wrong).
- **missing_evidence** — expected content the CONTEXT implies but the candidate
  omits (e.g. a source-stated architecture constraint — such as a lazy-load,
  privacy, or existing-service constraint — with no corresponding decision or
  handoff constraint).   For an integration design, operational prerequisites the
  capability cannot run without (vendor account / API credentials / billing /
  sandbox) omitted from Dependencies are a gap, not a clean "None". Record this
  ONLY in `missing_evidence`: it MUST NOT appear in `critical_findings` and MUST
  NOT cap the score — an omitted "None" asserts nothing settled, so it is never a
  fabrication or approval-discipline critical (even when every rubric dimension
  otherwise scores 1.0, this omission alone does not fail the candidate). An item
  the discovery trace marks undecided is NOT
  missing_evidence when the candidate raises it as an Open Question/risk without a
  settled decision — that is the required handling, not an omission. Do not ask the
  candidate to decide (or propose a settled value for) a discovery-undecided item.

Always ground findings in concrete evidence. Every critical finding MUST quote a
string that appears VERBATIM in the candidate. Before recording a critical, locate
that exact string (the table row, the sentence) in the candidate text; if it is not
literally present, DROP the finding — do not infer, paraphrase, or assume a row or
section exists. In particular, do NOT claim the candidate "lists X in Dependencies /
Risks / a table row" unless that row is literally in the text: a single mention in
one section (e.g. an out-of-scope aside in the Solution Overview) is evidence for
that section ONLY, never for rows the candidate does not contain. A finding whose
evidence you cannot copy verbatim from the candidate is a hallucination, not a
violation. For a missing-item finding, name the specific CONTEXT / discovery fact the
candidate omits.

Then re-read the finding you just wrote and run this self-test, because the failure mode
is visible in its OWN wording — delete the finding if it trips:

1. Hardening with no home Q#. A HARDENING finding claims the candidate states an
   undecided sub-choice as settled, so it must name the Q# in THIS flow's Open Questions
   table that the text closes. When the sub-choice belongs to a consumed integration
   (call placement / client-vs-proxy, key exposure, quota governance), that table holds
   no such Q# and the finding must be DELETED — widening the scan to the whole artifact
   widened WHERE you look, never WHICH questions are yours to protect. This does not
   excuse the opposite defect: the candidate MINTING a Q# or a proposed-direction block
   for that same sub-choice stays a real `approval_discipline` violation under Open
   Questions provenance. Judge the minting, never the mitigation.
