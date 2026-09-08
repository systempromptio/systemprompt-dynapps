# Discovery contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
Discovery is the first BA/SA artifact. It reconciles source requirements,
decisions, affected capabilities, CR candidates, open questions,
cross-cutting requirements, and publication targets before proposal.md.
It extracts and classifies; it does not approve scope, invent facts,
decide implementation, or write proposal/specs.

AUTHORITY ORDER:
1. Source inputs and evidence from project-context, existing
   `openspec/specs/`, current implementation, legacy/live storefront
   surfaces, and committed design assets are the factual source of truth.
2. This `discovery` instruction is the artifact contract: follow it for
   classification, allowed values, gates, and quality bar.
3. The `discovery.md` template defines section order and table shape.
4. Project `config.rules.discovery`, workflow rules, and skills provide
   supporting context; they do not override this artifact contract.

DECISION KERNEL:
- FIRST clear the DELIVERY SCOPE GATE before any template section. If it
  resolves to `ambiguous` / bundled, output ONLY the decomposition and stop.
  The remaining rules apply only after one delivery scope is resolved.
- Trace every source requirement once, preserving real RQ/WBS identifiers
  and source wording.
- SCOPE DISPOSITION is owner-based — this is the ONE law for `in scope` vs
  `out of scope`, and every section below defers to it (no section restates a
  competing test). A signed requirement is `in scope` when THIS change closes
  its acceptance criteria, WHETHER it builds the work itself OR consumes /
  configures the selected service/vendor that already fulfils it. It is
  `out of scope` ONLY when the deliverable is owned by a SEPARATE change / WBS
  / team (another feature FSD, a consumer integration, a platform-wide
  concern) or is consumed by a DIFFERENT surface — that owner is then recorded
  as a Dependency. A VALID out-of-scope owner is a NAMED separate change / WBS
  / team / feature FSD that DELIVERS the row as its own workstream. A vendor
  console, hosted authoring / no-code studio, or an operator / author who
  merely publishes settings is NOT such an owner and NEVER pushes a row out of
  scope: a row THIS change closes by consuming / configuring that vendor stays
  `in scope`, with the console/authoring surface listed as a Dependency.
  Consuming, rendering, or reading a service/vendor does NOT
  by itself push a row out of scope: a vendor-hosted / no-code /
  externally-authored artifact THIS change consumes to satisfy an in-scope
  requirement stays `in scope`, and its authoring surface is listed under
  Dependencies. A shared vendor / supplier / WBS prefix decides nothing on its
  own.
- DELIVERY MODE is a SECOND, orthogonal axis recorded per in-scope row as a
  current-state FACT, chosen by the ACTOR = the action THIS change performs on
  the row: `build` (THIS change writes the connector / UI / code), `consume`
  (THIS change uses a ready vendor/service capability or an already-published
  artifact AS-IS — including one authored elsewhere by the client, e.g.
  referencing a published style/preset ID), or `configure` (THIS change ITSELF
  sets the vendor-hosted / no-code settings). If the settings are authored /
  published by someone else and THIS change only references or renders the
  result, the mode is `consume`, not `configure`. Out-of-scope and deferred
  rows use `—`. Discovery MAY inspect existing code / specs / the live surface to
  record this fact; it does NOT choose the reuse-vs-SDK-vs-from-scratch
  approach — that deeper choice remains design's Solution Approach.
- Enumerate every touched capability — existing ones modified or retired, and
  new ones to add.
- Scope that changes WHAT is in or out versus the signed BRD is a Change
  Request candidate; do not silently absorb it into confirmed scope.
- Open Questions refine already-known scope; they must not duplicate Change
  Request topics.
- Discovery CRITICAL means the WHAT/scope/acceptance is unclear enough that
  proposal.md would likely be wrong. HOW/refinement decisions are NICE-TO-HAVE
  and flow to grooming/design.
- The Discovery Gate is a management interface: the first checkbox must
  literally match whether any CRITICAL Open Question remains Open.
- Topic partition (one place per topic):
  - Signed source requirement -> Requirements Trace.
  - Signed later-phase/could-have/postponed row -> `deferred` in
    Requirements Trace, not CR.
  - New/changed WHAT on this change's owned surface -> Change Request
    Candidate, not Dependency or Requirements Trace.
  - HOW/refinement of known scope on the owned surface -> Open Question,
    not CR.
  - External service consumed by an in-scope requirement and not owned here
    -> Dependency.
  - Service needed only by an unapproved CR -> stays with CR, not Dependency.
  - HOW owned by another feature/integration -> out-of-scope/dependency note,
    not this change's Open Question.

DELIVERY SCOPE GATE (blocking — evaluate BEFORE authoring anything):
This gate decides the one delivery scope. Produce discovery only after it
resolves to `functional` or `integration`. If it resolves to
`mixed / unresolved`, output only the decomposition; emit no discovery
sections, tables, or Discovery Gate.

Scope classification algorithm:
1. Identify the delivery surfaces in the sources.
   - `functional surface`: storefront UI/UX, navigation, page/route changes,
     shopper-facing copy, accessibility, SEO/redirects, user journeys, or
     feature behaviour a BA/client would review.
  - `integration surface`: a service capability that needs connector work in
    THIS change: new/changed API or service contract, auth/scopes, data
    mapping, credentials/keys/quotas/billing, security, errors, or an open
    client-vs-server/proxy call decision. This applies to third-party SaaS
    and internal services. A service is NOT "just a dependency" while any
    key/billing/quota/proxy/contract/security decision is open. Existing
    account/key/cloud project does NOT mean already integrated; zero open
    connector decisions does. An unresolved client-vs-server/proxy decision
    alone makes the service an integration surface, not a functional HOW
    refinement or NICE-TO-HAVE question.
2. Count integration surfaces by service boundary. Each distinct service is
   a separate integration candidate. Do not merge unrelated services into
   one integration because a single feature consumes them.
3. Read ONLY an explicit, user-stated scope signal: a stated role (BA vs
   SA/architect), an explicitly chosen service capability, or an explicit
   "functional-only" / "integration-only" instruction. The name of the change
   or feature (e.g. `<feature>`, `add-<feature>`) is NOT a scope
   signal — it names the feature, not the delivery owner. Likewise, invoking
   the workflow on a change (e.g. `/opsx:propose <change>`) selects WHICH
   change to work on; it is not a delivery-scope selection and does not signal
   `functional`. Do NOT infer `functional` from the fact that the sources
   describe a storefront feature, that the BRD focuses on shopper experience
   (WHAT), that the user is "proposing the feature", or from the change slug.
   Only when such an explicit signal exists does it resolve the one owned
   surface:
   - `functional`: the user explicitly owns the feature surface. Integrations
     are dependencies; record them in Source Inventory and requirement
     Notes, not as owned capabilities or ISD targets.
   - `integration`: the user explicitly selected one concrete service
     capability. That service is the only owned scope. Consumer UI and all
     other services are out of scope with notes naming the owning change.
4. Decide the outcome. `functional` and `integration` are BOTH resolved
   outcomes that REQUIRE an explicit signal from step 3 — neither is a
   default:
   - No integration surface exists at all (step 1 found only a functional
     surface): scope is `functional`; proceed to author.
   - An explicit signal exists (step 3): resolve to that owner accordingly.
   - Otherwise — one or more integration surfaces exist AND step 3 found no
     explicit signal — the scope is `mixed / unresolved`. STOP before
     discovery and return a decomposition: the functional FSD change plus one
     agnostic ISD change per distinct integration surface, with the
     capability/service each owns and the target document each would produce.
     Ask the user which single scope to pursue.
   This STOP is mandatory. Do NOT fall back to `functional` because the
   feature is shopper-facing, because the BRD focuses on the shopper journey,
   because the integrations "look like dependencies", or because "no new
   internal contract" is being built. The absence of an explicit signal is
   precisely what forces the STOP — it is never a reason to pick `functional`.

Rule of thumb: a service with any open key/billing/quota/proxy/contract/
security or client-vs-server decision is an integration surface, not a
dependency; an integration change owns exactly one service capability and one
discovery owns exactly one surface, so mixed surfaces with no explicit user
choice STOP with the decomposition.

PIPELINE (follow each referenced section; do not improvise content):
1. Identify and cite source inputs/evidence (INPUTS AND SOURCE POLICY).
2. Read sources in intake order; separate signed scope, current behaviour,
   meeting decisions, implementation evidence, and derived summaries.
3. Clear the DELIVERY SCOPE GATE before trace/capability dispositions.
4. Extract Requirements Trace for the resolved scope (including consumed
   sibling requirements from the cross-requirement entanglement pass in
   INPUTS AND SOURCE POLICY).
5. Enumerate affected capabilities.
6. Record Dependencies (including platform/OOTB-managed capabilities from the
   same entanglement pass).
7. Surface Change Request candidates.
8. Write Open Questions.
9. Evaluate Cross-cutting Requirements.
10. Record Delivery Scope & Targets.
11. Build the Document Consumption Guide from the completed sections
    (DOCUMENT CONSUMPTION GUIDE ROWS); derived last but rendered directly
    after the Executive Summary, above the detail sections.
12. Answer the Executive Summary question from the completed sections
    (CONTENT RULES).
13. Set Discovery Gate checkboxes from the final Open Questions table and the
    Document Consumption Guide.

INPUTS AND SOURCE POLICY:
- Fill Discovery Source Inventory with inputs/evidence actually used — not
  document-only: requirements, meeting notes, tracker records, baseline specs,
  current/legacy code evidence, legacy/live URLs, design assets, API/contracts,
  config, and explicit availability checks as needed.
- Retrieve project documents through project-context using the canonical Source
  Lookup table in the specifications rule; use `openspec/specs/`, inspected
  current implementation, or legacy/live storefront references for current
  behaviour.
- A research report committed for this change under
  `openspec/changes/<change>/research/<slug>/report.md` is a valid, citable
  input for an empirical fact no document, baseline spec, or code reading
  settles. Anchor the fact as `<that path> · Q#`, carrying the finding's
  recorded status and basis as they stand — an unconfirmed or
  documentation-only finding is never restated here as settled.
- If no relevant BRD / requirements source is found, ask the user to point to
  it or confirm "no BRD" for a small change. Record `No BRD - <one-line reason>`
  and keep downstream sections light. Never invent BRD rows.
- If no relevant workshop / meeting notes are found, ask whether any exist and
  can be added; if still none, record "none provided".
- Visual source, when UI is affected: rely on committed assets under the change
  `assets/` folder and FSD/ISD source docs. If no design asset exists, record
  that explicitly in the Discovery Source Inventory; do not fabricate
  screenshots or layout details.
- Prior-art / existing-solution evidence (BOTH flows): while enumerating
  capabilities, inventory what ALREADY EXISTS that could satisfy the need —
  installed or marketplace/partner cartridges, accelerator/platform modules or
  features, vendor-provided SDKs / client libraries / documented APIs — as FACTS
  in the Discovery Source Inventory (and requirement Notes), each with its
  grounding anchor (a returned project-context result, a vendor/marketplace URL
  the sources carry, or a code/config reference). Record "searched, none found"
  when nothing fitting exists (mirror the no-BRD / no-asset rule). FACT-GATHERING
  ONLY: discovery does NOT choose between reuse / SDK / from-scratch — that
  build-vs-reuse call is design's Solution Approach — and NEVER fabricates a
  cartridge, module, SDK, or API the sources do not support.
- Cross-requirement & platform-capability entanglement pass (BOTH flows):
  run this per-requirement prove-membership pass and commit each result to
  the placement ledger BEFORE drafting the trace (mirror the integration
  obligation pass shape):
  1. ENUMERATE: for EACH owned (in-scope) requirement, list every OTHER item
     in CONTEXT it must consume or is constrained by to function — signed
     sibling requirements (ANY WBS family) AND platform/OOTB-managed
     capability surfaces (Business Manager config/attributes, Page Designer
     authoring, Campaigns / Promotions / Source Codes, Custom Site
     Preferences, Shopper Context). Scan across WBS families; a shared WBS
     prefix neither includes nor excludes a row.
  2. JUSTIFY each enumerated candidate against the owner-based test (DECISION
     KERNEL): does a SEPARATE change / WBS / team own this item's deliverable,
     or does THIS change close it (by building OR consuming/configuring)?
     Commit ONE ledger line with the one-line proof recorded in the emitted
     `Note`:
     - owned-here -> THIS change closes its acceptance criteria (builds it OR
       consumes/configures the selected vendor that fulfils it) -> it stays its
       own in-scope trace row with a delivery mode (`build` / `consume` /
       `configure`) (already placed by the Topic / obligation partition; not
       re-placed here). A vendor-hosted / no-code artifact this change consumes
       stays here (mode `consume` / `configure`); its authoring surface is ALSO
       listed under Dependencies.
     - owned-elsewhere(sibling) -> a SIGNED requirement whose deliverable a
       SEPARATE change / WBS / team owns (a DIFFERENT feature / integration /
       FSD that BUILDS or AUTHORS it as its own workstream, or a row consumed
       by a DIFFERENT surface) -> Requirements Trace `out of scope`, delivery
       mode `—`, affected capability = the owning capability (NEVER this
       change's own owned capability), Note names that owner and that the owned
       requirement consumes it; add the consumption backlink in the owned
       requirement's own Note.
     - platform-dep -> a platform/OOTB-managed capability surface the
       requirement relies on -> a Dependencies row (`platform config`, its
       availability status, the consuming requirement(s)), optionally noted on
       the owned requirement.
     Each committed line carries its grounding anchor; write "searched, none
     found" for a requirement that consumes nothing external.
  FAILED pass (redo steps 1-2 before drafting): an owned-elsewhere sibling
  pointed at this change's own capability or a generic `<feature>-ui` bucket;
  a requirement whose deliverable a SEPARATE change / WBS / team owns marked
  `in scope`; a vendor-hosted / no-code artifact THIS change consumes marked
  `out of scope` merely because it is vendor-authored or shares a WBS prefix
  (owner-based: consuming the selected vendor is `in scope`, mode `consume` /
  `configure`); a platform surface CONTEXT clearly shows the requirement
  relying on left off Dependencies; or a speculative pull-in (a requirement,
  attribute, or capability the sources do NOT show the owned requirement
  actually consuming). Discovery MAY inspect existing code / specs / the live
  surface to record the current-state delivery-mode FACT, but does NOT choose
  the build-vs-reuse approach (design's Gap Analysis / Solution Approach owns
  that), re-scope or resurrect an item the upstream deferred, or fabricate.
  Trace-lock unchanged: only signed requirements enter the Requirements Trace,
  and every signed requirement still appears exactly once.
- Every `Reference / Anchor` MUST resolve to a source project-context actually
  returned or present in CONTEXT — never a plausible-looking URL, path, or ID
  you did not retrieve. When project-context returns an inbox file for a
  document, cite that inbox path; do not upgrade it to a synthesized wiki/live
  URL. Open cited inbox files/line ranges before relying on them. Subject to
  that grounding rule, the `Reference / Anchor` cell uses the exact anchor by
  source type:
  - project-context references use the returned workspace-relative inbox path
    plus lines, e.g.
    `.cursor/.project/project-context/inbox/meeting_notes/workshop.md:12-28`;
  - if project-context provides only an external identifier (e.g. a Jira key)
    and no file/line anchor, cite the identifier such as
    `project-context jira: PROJ-123` rather than inventing an inbox file;
  - baseline specs use `openspec/specs/<capability>/spec.md:<lines>` when line
    information is available;
  - code, config, and asset references use `file:line-start-line-end` when line
    information is available;
  - BRD/SRD/FSD/ISD/TSD wiki documents use the live Confluence URL or Page ID
    (optionally plus the section or exported file lines) ONLY when
    project-context supplied it; otherwise cite the inbox file/lines
    project-context returned — do not synthesize a Confluence URL;
  - live storefront surfaces use only a URL or route present in the sources; do
    not invent a hostname or a page the sources do not state.
  - For a REMOVE capability with no legacy implementation file anchor from
    project-context, record `— (legacy implementation not found via
    project-context; baseline spec and live URL above serve as current-state
    evidence)`. Do NOT invent a workspace path, filename, or line range.
  Do not cite a whole file when project-context returned a narrower line range.
  Mark missing inputs explicitly.
- FSD/ISD source lookup results may be Source Inventory evidence, but
  `NEW` / `EXTEND` belongs only in Delivery Scope & Targets.
- The user's stated delivery intent (role, chosen scope, selected service)
  is a legitimate input. Record it as `user request` or the command argument;
  do not invent a project-context file for a verbal decision.
- Source intake order:
  1. Signed BRD/SRD: canonical WHAT and official RQ/WBS codes.
  2. Existing OpenSpec baseline specs and current implementation / legacy
     code: current behaviour and capability ownership; use to decide
     ADD/MODIFY/REMOVE.
  3. Legacy or live storefront surfaces / URLs affected by the change:
     current user journey, SEO exposure, redirects, retirement/migration
     evidence, and visual/current-state context.
  4. Meeting notes/workshops: confirmed decisions, CR candidates, open
     questions, implementation constraints. They refine the signed source;
     they do not create new signed requirements unless explicitly stated.
  5. Jira/FSD/ISD/TSD/design assets: supporting scope, acceptance, visual,
     publication, or integration context when present.
  6. API/contracts, configuration, analytics/tracking plans, content/CMS
     inventory, vendor docs, or other domain inputs: include when they affect
     scope classification, open questions, cross-cutting requirements, or
     publication targets.
  7. Derived summaries / known-unknown indexes: convenience only — a checklist
     against the primary sources, not additional facts.

CONTENT RULES:
- Executive Summary — answers, from the detailed sections as they stand NOW:
  what has this file settled, what stays open, is the change ready? Record it
  as a READER MAP that orients a human before the detail — never a retelling.
  Its machine-readable counterpart is the Document Consumption Guide.
  - Answer from the body — Requirements Trace, Affected Capabilities,
    Dependencies, Change Request Candidates, Open Questions, Cross-cutting
    Requirements, Delivery Scope & Targets — stating what they say NOW. Every
    statement must already be readable in one of them; this block is never a
    new source of truth, and a statement they no longer support is wrong.
  - Place it immediately after the title, before Discovery Source Inventory.
  - Write it after the detailed sections are final, so it reflects final facts
    and gate state.
  - Open with the problem/change context and why it matters, including one
    source-grounded business-value sentence. Then answer, in order:
    - WHAT THIS FILE SETTLES: the delivery scope, the SHAPE of the requirement
      set (how much in scope, deferred, owned elsewhere), which capabilities
      move, and which section holds each — a route to the detail, not the detail.
    - WHAT STAYS OPEN: the subjects still undecided and their register (Open
      Questions, Change Request Candidates).
    - READINESS, worded consistently with the Discovery Gate. If any CRITICAL
      Open Question remains Open, do not say the change is ready for proposal.
  - Name SUBJECTS, never identifiers: "the redirect for the retired /legacy-path
    URL", not "Q2". Counts/magnitudes are welcome ("three refinements", "nine
    requirements in scope") — they make this block cheaply checkable against
    the Document Consumption Guide.
  - Do not re-list a table's rows. No RQ/WBS ID, capability inventory, CR/Q
    identifier, exhaustive scope list, acceptance criterion, implementation
    detail, or implied approval belongs here — those are the Document
    Consumption Guide's job or the detail sections' own.
- Delivery Scope (resolved by the DELIVERY SCOPE GATE; this section defines
  the allowed values and their effect):
  - `functional`: delivers a client/user-facing feature surface (normally
    FSD). It may depend on integrations, but does not own or specify their
    contracts. Record integrations in Source Inventory and requirement Notes.
  - `integration`: delivers a service-oriented capability (normally ISD).
    Name it for the service/capability, not the first consumer — e.g.
    `<provider>-integration` (a payments, search, or shipping provider), not
    `<consumer-feature>-<service>`.
    For integration scope, the Executive Summary, Affected Capabilities,
    Delivery Scope & Targets, and working-copy paths must use the
    service-oriented change name. Consumer feature names are context only.
    If the user-selected service differs from the command/change name,
    derive the artifact identity from the selected service, not from the
    consumer feature: `openspec/changes/<service-change>/isd/<service>.md`.
  - Bundled asks combining a substantial functional surface with real
    connector work, or an integration ask that does not name the service, are
    NOT one discovery: stop and return the DELIVERY SCOPE GATE decomposition.
    A feature references agnostic integrations; it does not own them. Trivial
    client usage of an already-governed integration may remain functional
    when no contract/credential/data-mapping work is created.
- Requirements Trace:
  - CLOSED-SET GATE (run FIRST, before any disposition; BOTH scopes) — a
    mechanical enumeration, not a scope judgement:
    1. SOURCE_RQ_SET = a LITERAL scan of EVERY `RQ-<id>` token appearing
       anywhere in the signed source set (the BRD/SRD and any additional
       requirement sources provided). List them all; do NOT pre-filter by
       scope, WBS family, or owner, and never add an id the sources lack.
       This INCLUDES signed rows another change / WBS / feature FSD authors
       or owns (for example Page-Designer-owned rows surfaced in the
       sources): they stay in SOURCE_RQ_SET and are TRACED `out of scope`,
       NEVER dropped. "Owned elsewhere" selects the disposition (out of
       scope), it never removes the row.
    2. After the placement ledger is built, compute TRACE_RQ_SET (the rows
       you are about to emit) and require TRACE_RQ_SET == SOURCE_RQ_SET
       (zero-diff). Any id in `SOURCE_RQ_SET − TRACE_RQ_SET` is a DROPPED
       signed row and a FAILED pass: place it before drafting — a
       sibling/other-owned row goes `out of scope` (delivery mode `—`, its
       owning capability, owner named in the Note), NEVER deleted — then
       re-run the check until the difference is empty. This is the executable
       form of the trace-lock (source fidelity): no signed row is dropped.
  - Extract every requirement in the source set, including later-phase,
    deferred, out-of-scope, and CR-candidate items.
  - Trace-lock (source fidelity): EVERY signed source row is extracted and
    appears exactly once, even when this change does not build it — never omit
    a signed row. Disposition follows the owner-based SCOPE DISPOSITION law
    (DECISION KERNEL): a row THIS change closes by CONSUMING / CONFIGURING the
    selected vendor is `in scope` (delivery mode `consume` / `configure`), with
    the vendor's authoring surface listed under Dependencies; a row whose
    deliverable a SEPARATE change / WBS / team owns — or that a DIFFERENT
    surface consumes — is traced `out of scope` (delivery mode `—`) against its
    owning capability. A signed row the change neither closes nor consumes (an
    unrelated same-packet or same-WBS-family row) is likewise traced
    `out of scope` for source fidelity. This is DISTINCT from the "no
    speculative whole-family pull-in" rule, which only bars inventing
    consumption links or unsigned items — it never licenses dropping a signed
    row.
  - Preserve the RQ ID and the requirement's clean/leaf WBS code — ONE
    WBS per requirement (e.g. `<CAP>.<REQ>`), the SAME code the FSD block
    heading and the proposal Delivery Breakdown `Covers` reuse downstream
    as the stories-generation join key; NOT a coarse parent-only grouping
    (`<CAP>`). When the source only gives a parent code, extend it to the
    per-requirement leaf. Also preserve title, citation, requirement
    text, requirement details, disposition, delivery mode, affected
    capability, and notes. Requirement and Requirement Details are two separate columns
    that mirror the source: Requirement holds the verbatim requirement
    (e.g. the user story); Requirement Details holds the verbatim detail
    list from the source, preserving its numbering/structure, and is `—`
    when the source has no separate details. Do not paraphrase or merge
    the two. Requirement Details are HIGH-LEVEL SEED detail from the SRD,
    captured verbatim: an SRD carries only high-level intent, so Details may
    contain anything the source happened to note (shopper journey, UI
    behaviour, a named vendor / SDK, a constraint) and are deliberately NOT
    finished acceptance criteria — those do not exist yet at this stage.
    Downstream (proposal, FSD, ISD) ELABORATES this seed into the criteria
    for what THAT artifact owns; it never lifts a Details line as a
    ready-made criterion. The `Requirement` cell is the CANONICAL story text: every
    downstream artifact (the proposal Delivery Breakdown slice, the FSD
    block, stories.md) copies it VERBATIM, joined by WBS — so author it to
    final signed wording here and nowhere else. Story wording is a
    discovery responsibility; no later phase re-authors or re-personas it.
  - Use dispositions: `in scope`, `out of scope`, `deferred`,
    `CR candidate`, per the owner-based SCOPE DISPOSITION law (DECISION
    KERNEL): `in scope` when THIS change closes the AC (build OR
    consume/configure the selected vendor); `out of scope` when a SEPARATE
    change / WBS / team owns the deliverable or a different surface consumes
    it.
  - Every in-scope row records a Delivery mode — `build`, `consume`, or
    `configure` — as a current-state FACT (DELIVERY MODE axis, DECISION
    KERNEL); out-of-scope and deferred rows use `—`. The Delivery mode column
    sits between Disposition and Affected capability. Enumerate-then-justify
    each row: first prove the owner (in vs out), THEN record how an in-scope
    row is satisfied (build/consume/configure); keep the one-line proof in the
    Note. Delivery mode never changes a row's disposition — a `consume` /
    `configure` row is still `in scope`.
  - Notes capture short, grounded material findings: client wishes, workshop
    nuance, ambiguity, constraints, or `Q#`/`CR#` links. Do not restate the
    requirement or invent notes. Use `—` when nothing material was found.
  - Use `deferred` for source requirements explicitly described as later
    phase, could-have, future, or postponed while still belonging to the
    same product scope. Use `out of scope` (per the owner-based SCOPE
    DISPOSITION law) when the deliverable is owned by a SEPARATE change / WBS
    / team or consumed by a different surface, rather than postponed.
  - Every row names an Affected capability, including `deferred` and
    `out of scope` rows: use the capability that owns the requirement (for
    out-of-scope items, the owning feature/integration). Never leave the
    Affected capability cell blank or `—`. This cell is the OWNING
    capability / workstream of the ROW's deliverable — a routing label, and
    for `out of scope` / `deferred` rows a BEST-EFFORT attribution of who
    delivers it. It is NOT a claim that THIS change touches that capability:
    the Affected Capabilities section is a DIFFERENT, narrower set — only the
    capabilities THIS change ADDs, MODIFIEs, or REMOVEs. A row may name an
    owner that never appears in Affected Capabilities, and that is correct.
  - A capability THIS change ADDs, MODIFIEs, or REMOVEs is OWNED BY THIS
    CHANGE, so it is never the "separate owner" that pushes a row out of
    scope: a signed requirement whose deliverable is exactly that
    modification / retirement stays `in scope` (mode `build`) and names that
    capability. Only a capability delivered by a SEPARATE change / WBS / team
    makes a row `out of scope`.
  - If Delivery Scope is `functional`, keep functional requirements in
    scope and record the integrations/services they consume in the
    Dependencies section (a short cross-link in Notes is fine); do not create
    ISD requirements or integration capabilities in this change.
  - Functional trace lock with signed BRD/SRD: Requirements Trace contains
    signed rows only, plus signed-source rows that truly lack IDs. Baseline
    specs, current-state evidence, live URLs, and meeting notes inform other
    sections; they do NOT become `UNTRACKED-<n>` requirements.
  - For functional scope, apply Topic partition exactly.
  - For `integration` scope, run this selected-service obligation pass
    BEFORE filling Requirements Trace:
    1. Name the selected service/capability from the delivery-scope signal
       (for example, the one integration the architect asked to build).
    2. Write the service capability slug you will use consistently in the
       Requirements Trace, Affected Capabilities, and Delivery Scope target.
    3. For each signed requirement, ask: "does the selected service have to
       do real work for this requirement to function?"
    4. Commit each answer as that requirement's placement-ledger line BEFORE
       drafting, recording the delivery mode for in-scope rows:
       `RQ -> in scope -> <build|consume|configure> -> Requirements Trace ->
       <service slug>` (THIS change closes the AC — building the connector, or
       consuming/configuring the selected vendor that fulfils it) or
       `RQ -> out of scope -> — -> Requirements Trace -> <owning capability
       slug>` (a SEPARATE change / WBS / team owns the deliverable, or a
       different surface consumes it). Copy the justification into the emitted
       row's `Note` for BOTH dispositions: an in-scope Note splits the
       service's backend slice from the functional remainder and states how it
       is delivered; an out-of-scope Note names the precise owning capability
       and why THIS change does not close it. `<owning capability slug>` is a
       concrete owning capability, never a generic `<feature>-ui` /
       `consumer-ui` bucket. A ledger or emitted row that points every
       out-of-scope row at one blanket owner, uses a generic UI bucket,
       leaves an owner blank / `—`, or marks no row in scope is a FAILED
       pass — redo steps 3-4 before drafting. Marking a vendor-family row
       THIS change consumes / configures `out of scope` by naming a vendor
       console, hosted authoring / no-code studio, or operator (e.g. a vendor
       studio) as its "owner" is ALSO a FAILED pass: that surface is a
       Dependency, not a separate delivering owner, so the row is `in scope`
       (mode `consume` / `configure`).
  - Integration obligation rationale: an integration change invents no
    requirements; it exists because signed functional requirements need the
    selected service, so the trace stays anchored to the signed SRD.
    Shopper-facing wording can still carry a backend obligation.
    - SOURCE-PARTITION FIRST (the primary test): well-formed sources SEPARATE
      requirements into a functional/consumer family (the feature's UI,
      journey, navigation, copy — what a shopper does) and an
      integration/vendor family (rows written from the selected service's
      side, often under their own integration WBS grouping, each naming the
      vendor obligation AND the consumer requirement that consumes it). When
      the source carries this split, the integration change OWNS its own
      vendor-family rows (`in scope`, all under the ONE selected-service
      capability) and dispositions EVERY functional/consumer-family row
      `out of scope` to its owning consumer feature — a SEPARATE change that
      owns that surface (owner-based SCOPE DISPOSITION law) — EVEN WHEN a
      consumer row and a vendor row share shopper-facing wording. The overlap
      means the consumer row CONSUMES the service, not that this integration
      owns the consumer surface: the dedicated vendor-family row already
      carries the backend work, so the parallel consumer row is NOT a second
      in-scope row here. Marking both in scope double-counts one obligation and
      is a FAILED pass.
    - DELIVERY MODE, NOT SCOPE (apply to EACH in-scope vendor-family row):
      owning a vendor obligation does NOT require building code for it. A
      vendor-family row THIS change closes stays `in scope` regardless of how
      it is delivered. A consumed or configured VENDOR-HOSTED / no-code /
      externally-authored artifact (a hosted console, a published
      style/config/preset ID, a dashboard-managed setting) is `in scope` — NOT
      out of scope — and its authoring/hosting surface is ALSO listed as a
      Dependencies row; it becomes `out of scope` ONLY when a SEPARATE change /
      WBS / team owns that deliverable. A shared vendor / supplier / WBS prefix
      does not decide mode or scope on its own. Record the mode itself with the
      ONE delivery-mode law in the DECISION KERNEL (the actor test) — this pass
      does not restate or vary it; the finer build-vs-configure responsibility
      split is resolved in design's Gap Analysis.
    - `in scope` (yes): the requirement is a vendor-family obligation the
      selected service must fulfil, whether THIS change builds the connector
      for it OR consumes/configures the service that performs it. Affected
      capability is this integration; record its delivery mode (`build` /
      `consume` / `configure`); add a Note splitting the service's backend
      slice from the functional remainder and stating how it is delivered, not
      an over-claim of the whole requirement.
      FALLBACK for an UN-partitioned source: when the source carries NO
      dedicated vendor-family row for an obligation, a lone shopper-facing
      requirement that can ONLY be satisfied by the service's core function
      carries that backend slice in scope even though its wording is
      shopper-facing — e.g. for a payment provider with no separate
      authorization row, authorizing and capturing a charge is the service's
      work. This fallback does NOT apply once a
      dedicated vendor-family row already exists (see SOURCE-PARTITION FIRST).
    - `out of scope` (no): the requirement's deliverable is owned by a SEPARATE
      change / WBS / team, or consumed by a different surface — feature UI,
      navigation, SEO, accessibility, localizable copy, a device hand-off, data
      owned by another service, a different integration, or an obligation a
      dedicated vendor-family row already owns. Affected capability is that
      owning feature/service, not this integration; delivery mode `—`.
    The trace still lists EVERY signed row. An integration trace owning NONE
    of them is a contradiction (nothing to build); if every row is out of
    scope or points to the consumer feature, re-check the selected-service
    obligations.
  - ONE capability (collapse): ALL of the selected service's in-scope
    vendor-family rows belong to ONE agnostic capability under a SINGLE slug
    named for the service (the `<provider>-integration` archetype), used
    verbatim across the Requirements Trace, Affected Capabilities, and the
    Delivery Scope target. A requirement identifier (its WBS / RQ code) is a
    row key in the trace, NEVER an Affected-Capabilities name. Do NOT split
    one service into per-obligation or per-requirement capabilities (e.g. a
    charge-authorization obligation and a payment-form render obligation of the
    SAME provider are ONE capability), and do NOT name the capability after a consumer
    feature. Two owned capabilities are correct ONLY when the sources name
    two DISTINCT services/suppliers/accounts. The collapse folds ALL in-scope
    obligations of the selected service — whether delivered by `build`,
    `consume`, or `configure` — into the ONE capability; only rows whose
    deliverable a SEPARATE change / WBS / team owns stay `out of scope` and are
    never folded in.
  - Integration obligation examples (a payment provider stands in for
    any integration archetype — search, shipping-rate, ratings):
    - The selected service owns its core function and interface governance
      (a render surface it provides, the operation it performs, key/billing/domain
      restrictions, the client-vs-proxy call decision) when the sources assign
      those facts to that service.
    - It does NOT own consumer UI synchronization, the consumer page's own
      content rendering, its filters or navigation links, localizable
      storefront copy, accessibility of the consumer page, or a deep-link to
      the shopper's native provider unless the source explicitly says that
      service performs those parts.
    - A second supplier (e.g. a fraud-screening service alongside a payments
      service) is a separate integration when the sources name a different
      supplier/account/contract. Do not merge distinct suppliers into one
      integration trace.
  - For integration scope, Affected Capabilities contains only capabilities
    this change owns. Do not list consumer capabilities (the consuming
    feature, its navigation, or retired legacy pages) unless this
    integration change actually modifies or retires them.
  - If a signed BRD/SRD exists, do not mint `UNTRACKED-<n>` rows for
    meeting-only scope that deviates from that BRD/SRD. Put that scope only
    in Change Request Candidates.
  - Use `UNTRACKED-<n>` and `WBS-TBD` only when the active requirements
    source truly has no identifier: either there is no BRD/SRD and meeting
    transcripts are the primary requirements source, or the signed source
    contains a requirement with no ID. Do not invent official-looking codes.
- Affected Capabilities:
  - `ADD`: the capability does not exist yet and must be created.
  - `MODIFY`: an existing capability changes behaviour.
  - `REMOVE`: an existing capability is retired or superseded by this
    change. Do not leave retired legacy capabilities as MODIFY; redirect,
    migration, or handoff work does not make a retired capability MODIFY.
  - Functional retirement pass: list every existing capability/page/surface
    this functional change replaces, retires, removes, or supersedes as
    `REMOVE`, even without a direct signed RQ row.
  - CAPABILITY CLOSED-SET GATE (run before drafting Affected Capabilities):
    do NOT derive the capability set from the Requirements Trace alone — an
    RQ-driven set misses a capability this change touches with NO signed RQ
    row (the retired-directory case). Instead walk a CLOSED set mechanically,
    like the Requirements-Trace token scan: enumerate BASELINE_CAP_SET = every
    `specs/<capability>` baseline block present in CONTEXT (the same baseline
    spec groups you list in the plan), then give EACH one an explicit verdict —
    MODIFY (its behaviour changes), REMOVE (CONTEXT marks it retired /
    superseded / replaced, even with NO signed RQ row — e.g. a directory or
    page this change removes), or untouched (omit from the table) — and add ADD
    rows for new surfaces an in-scope RQ builds. TOUCHED_CAP_SET is the ADD /
    MODIFY / REMOVE verdicts. A baseline block you listed in CONTEXT but left
    without a verdict, or a TOUCHED_CAP_SET member missing from the table, is a
    FAILED pass — resolve it before drafting.
    Flow scope for this walk: a REMOVE / MODIFY verdict is only for a baseline
    capability THIS change itself retires or whose behaviour it changes. For
    integration scope, a consumer baseline capability the integration merely
    enables or that consumes it does NOT change behaviour here — mark it
    `untouched`; an integration change ADDs only its own agnostic service
    capability and never MODIFIEs or REMOVEs a consumer capability owned by a
    separate functional change (per the integration-scope rule above).
  To enumerate affected capabilities, use the project-context skill to
  search the baseline specs, but do not rely on it alone: also inspect
  `openspec/specs/` directly and run `openspec list`. Cross-check the
  Requirements Trace so every affected capability is covered. A single-row
  table on a multi-area functional feature is almost certainly a miss, but
  a focused agnostic integration change may legitimately have one
  capability.
  - Decomposition (how many rows, and which): match the BASELINE
    capabilities that already exist in `specs/` first — do NOT invent a
    finer-grained sub-capability breakdown than the baseline carries. When
    the baseline is a single capability for the surface, this change is
    ADD/MODIFY/REMOVE on THAT capability, not a fabricated split into per-
    feature sub-capabilities. Decide ownership top-down, stopping at the
    first match: (1) group by surface/page first — behaviour on exactly one
    surface stays under that surface's capability; (2) lift ABOVE a surface
    only when it has its OWN external integration / data source (→ own
    capability) or is REUSED across surfaces (→ own shared capability); (3)
    split WITHIN a surface only for large self-contained neighbour areas —
    states (empty / loading / error), nested sub-components, and page-local
    shared rules stay in the surface capability. A requirement's `Affected
    capability` cell MUST be an existing baseline capability (or a genuinely
    new one under an `ADD`), never a re-pointed or invented sub-capability.
- Dependencies:
  - List a service here only after proving an IN-SCOPE requirement (not a CR
    candidate) relies on it while this change does NOT own it (third-party
    integration, another service integration, or already-integrated data/API
    service). A service needed only by an unapproved CR candidate — including a
    new integration ask surfaced here — stays with that CR (see Topic
    partition), not in Dependencies.
  - Include a row when an in-scope requirement needs a service owned by a
    different change to work; record consuming requirement(s) and integration
    status.
  - Status is a fact, not a guess: state `already integrated` or `separate
    integration change` only when sources support it; else unknown.
  - Do not turn a Dependency into an Affected Capability: this change does not
    own it.
  - Write "None." when this change consumes nothing external, e.g. an agnostic
    integration consumed by others but depending on nothing.
  - For integration scope, the selected service is NOT a Dependency. Do not list
    consumer-feature dependencies unless the selected service itself needs them
    for its in-scope obligation.
  - Browser/platform capabilities (e.g. geolocation permission, native browser
    APIs, device hand-offs) are not Dependencies unless the project owns or
    integrates a separate service for them. Record them as requirement notes,
    privacy/cross-cutting constraints, or Open Questions when they need
    refinement.
  - Do not decide consumer call/proxy/key mechanics here; those belong to the
    owning integration change.
- Change Request Candidates:
  - Use this table for scope that deviates from the signed BRD AND falls on
    this change's owned surface: new, removed, or altered behaviour from
    transcripts, constraints, or stakeholder input.
  - A CR is about WHAT changes versus the signed BRD. A pure HOW-to-implement
    choice is not a CR; put it in Open Questions if it still needs discussion.
  - Refinements such as redirect target/type, special-data availability,
    call/proxy mechanics, key/billing guardrails, or display details are Open
    Questions unless they change WHAT is built.
  - Do not create Requirements Trace rows, `UNTRACKED-<n>` IDs, or WBS
    placeholders for CR topics that are not signed source requirements.
  - Do not write an Open Question like "Should we include <new non-BRD
    feature>?" for a CR topic — the CR row status/disposition tracks that
    decision.
  - Use "None." if there are no CR candidates.
- Open Questions:
  - Use this table for ambiguity, missing decisions, or stakeholder input that
    refines already-known scope.
  - Write concrete, answerable questions. For a technical constraint, add a
    Tech note so the client can weigh the trade-off.
  - Severity is decided by WHAT vs HOW, not by how hard the item feels:
    - `CRITICAL`: the WHAT is unclear — scope, requirement, or acceptance
      criteria ambiguous enough that proposal.md would likely be wrong. You
      cannot write a correct requirement set until it is answered.
    - `NICE-TO-HAVE`: a HOW-to-implement or refinement decision on the owned
      functional surface that does not change WHAT is built and can be settled
      in grooming/design, e.g. redirect type (301 vs 302), exact redirect
      target, or a display refinement when the backing data schema is not yet
      known. A consumed integration's call/proxy/key/contract mechanic is NOT a
      functional Open Question (see the functional-surface restriction below).
  - When unsure, default to NICE-TO-HAVE. Zero CRITICAL questions is normal when
    requirements are clear; do not manufacture one.
  - Valid Open Questions: redirect target/type for a retired URL, a display
    refinement when backing data support is unknown. NOT Open Questions: whether
    to accept a non-BRD new feature already listed as a CR, whether to absorb a
    CR into confirmed scope, or a constraint already signed in the BRD/SRD (e.g.
    a stated privacy rule) — a settled requirement is not an open decision.
  - Functional (FSD) Open Questions restriction — prove membership before adding
    a row: it must be about the owned functional surface's WHAT, acceptance, or
    an in-scope refinement. If it is a call/proxy/key/quota/contract mechanic of
    a consumed integration, it FAILS the test and belongs to that integration's
    SA/ISD discovery — record it in the consumed dependency's Notes, not here.
- Cross-cutting Requirements:
  - Evaluate Accessibility (WCAG), SEO, Consent/privacy, and i18n for this
    change. Mark each Applicable or N/A with a grounded one-line reason.
  - If the change adds, moves, retires, or redirects a URL/page, SEO is
    Applicable.
  - For integration scope, mark Accessibility, SEO, and i18n N/A unless this
    integration itself produces UI, an indexable route/page, or user-facing
    copy; do not carry the consumer feature's UI obligations into an agnostic
    service integration. Mark Consent/privacy applicable when this service
    handles geolocation, PII, browser keys, tracking, or similar
    privacy/security-sensitive data.
  - These are BA/SA-owned product requirements. Technical concerns (rendering,
    caching, security, performance) belong in architecture only when an
    SA-level decision is warranted; otherwise Planning handles them.
- Delivery Scope & Targets:
  - Declare the delivery scope (`functional` or `integration`) and list only
    the FSD/ISD documents THIS change produces or touches.
  - A functional/user-facing surface normally needs an FSD target. If it
    depends on integrations, record those in Discovery Source Inventory and
    requirement Notes; add no ISD row unless this change owns the integration.
  - An integration surface (endpoints, contracts, auth scopes, external
    systems, Shopper APIs, a payment gateway, etc.) needs an ISD target — service-
    oriented and reusable by multiple consumers, never scoped as a child of a
    single feature.
  - `EXTEND`: a matching Confluence page exists; record URL/Page ID when
    available and the intended working copy path.
  - `NEW`: no matching page exists; record a lowercase kebab working-copy path
    under `openspec/changes/<name>/fsd/<feature>.md` or
    `openspec/changes/<name>/isd/<service-integration>.md`.
    For integration scope, `<name>` is the service-oriented change name derived
    from the selected service (e.g. `add-<service>-integration`) — NEVER the
    change/command name the workflow was invoked with, even when the scaffold
    paths here or the packet's `Change name:` show that consumer-feature name.
    An ISD working copy under the consumer feature's change folder is the exact
    defect this rule forbids.
  - Ask the user when EXTEND vs NEW is ambiguous. No `N/A` rows; absence of a
    document type means this change does not produce it.
  - Do not use `openspec_archive` or `openspec/changes/archive/**` for FSD/ISD
    Page ID lookup, publish target selection, approval status, or delta/conflict
    review. Archive sources are historical rationale only.

DOCUMENT CONSUMPTION GUIDE ROWS:
The Document Consumption Guide is this artifact's interface — the map a
downstream reader (an LLM first, a human second) uses to consume the file
without re-reading the body. Placed DIRECTLY AFTER the Executive Summary, above
the detail sections it projects. ONE row per consumed body section; each row
states WHAT that section settles and WHY it matters downstream (navigation) plus
the KEY DATA it exports (the identifiers/sets a consumer joins on). It is a
PROJECTION, never a new source of truth: every value must already be readable in
the named section, and a value that section no longer supports is wrong however
well written.
FIXED INTERFACE: emit one row per consumed section below, in order; an empty set
is `None`; never drop, merge, rename, or reorder a row. `What it settles & why`
is a short human-readable phrase (usable by human and LLM alike). `Key data`
carries identifiers and sets only — WBS codes, capability names with direction,
`Q#`/`CR#` with status, paths — the exact values downstream joins on, never a
restated requirement title or prose.
The rows, and each row's key-data authority — what it IS, where it GOES, and
what it must NEVER be:
- Requirements Trace: the signed scope boundary. Key data carries three
  labelled sets. Determine each set by walking EVERY Requirements Trace row and
  reading its WBS-column value — that WBS code is the set member, NEVER the RQ
  id and NEVER the owner/capability name — then routing the row by its
  Disposition cell. In-scope: the closed set of WBS codes dispositioned
  `in scope`, one code per in-scope row — the bijection key for proposal's
  Delivery Breakdown and the join key for spec requirement headings and FSD
  blocks; a code absent is out of scope downstream, and a consumer never
  re-derives the set by re-reading the trace. Deferred: the WBS codes
  dispositioned `deferred` — they populate the FSD's Deferred Requirements and
  never earn a design Gap Analysis row, a proposal slice, or a spec requirement
  (or `None`). Out of scope: every WBS code dispositioned `out of scope`, each
  paired with the owner its capability cell names as `<WBS> → <owner>` (the
  WHOLE set; the WBS is the key and the owner is the value, so a row is never
  keyed by its RQ id and never collapsed to `<owner> → <owner>`) — they bound
  proposal's Out of Scope & Boundaries; ownership elsewhere is never a Change
  Request and never by itself a Dependencies row.
- Affected Capabilities: which capabilities this change owns and how. Key data
  is the owned capabilities with their ADD / MODIFY / REMOVE direction — it
  fixes the specs file inventory and each file's delta section, and specs
  project this plan and never add a capability it does not carry.
- Dependencies: what this change consumes but does not own. Key data is the
  capabilities and services consumed here — each becomes a proposal External
  Dependencies & Access row and then a grooming `P#`; a status is inherited
  verbatim and never upgraded to "confirmed" (or `None`).
- Open Questions: what stays open and gates readiness. Key data is the `Q#`
  set still Open, each with its severity — the ONLY source of `Q#` numbers for
  design's mirror and grooming's migration; a question resolved this pass
  leaves the set entirely rather than staying with a Resolved marker, and an
  "undecided" remark elsewhere mints no `Q#` (or `None`).
- Change Request Candidates: asks against signed scope. Key data is the `CR#`
  set with its current status — it seeds proposal's Change Request
  Dispositions and grooming's Change Request Decisions; a candidate is an ask
  against signed scope, never agreed scope (or `None`).
- Cross-cutting Requirements: which cross-cutting areas apply. Key data is the
  areas marked applicable — the required rows of design's Base Configuration &
  Cross-Cutting; an area recorded N/A is omitted here, and the omission is
  inherited rather than revisited.
- Delivery Scope & Targets: the resolved scope and what is published where. Key
  data is the resolved scope (decided by the DELIVERY SCOPE GATE and only
  recorded here — it selects the FSD or ISD path and the design's
  owned-capability boundary, and downstream never re-decides it) plus the
  produced document type(s) with disposition and working-copy path (they
  select which of FSD / ISD is authored and where; an unknown Confluence
  target stays `TBD` and is never invented).

DISCOVERY GATE:
After discovery.md is generated, stop before design.md and proposal.md if any
CRITICAL Open Question remains Open. Resolve it with the user (often via
`/opsx:explore <change-name>`), capture the answer back into the Open Questions
table with Status Resolved plus user/BA/SA sign-off, then continue.
NICE-TO-HAVE questions do not block design or proposal; carry them into
grooming.md. A CRITICAL item that genuinely needs client input may be carried
forward only with explicit user sign-off recorded in discovery.md — a user
decision, not an agent inference.

The first Discovery Gate checkbox mirrors the Open Questions table:
- leave it `- [ ]` while any question is CRITICAL and still Open;
- check it `- [x]` only when no CRITICAL question remains Open (all are
  NICE-TO-HAVE, Resolved, or explicitly user-approved to carry forward);
- never check it while a CRITICAL question is Open without recorded user sign-off.

Keep all four template checkbox lines. With no CRITICAL questions, the remaining
boxes may be checked with a note such as "No CRITICAL answers required;
NICE-TO-HAVE questions are carried to grooming" once the Executive Summary and
detailed sections reflect that carry-forward.

The fourth checkbox reconciles the projection: check it only after comparing each
Document Consumption Guide row's key data against the section it names — scope,
WBS sets, capability directions, dependencies, open items, change requests,
cross-cutting areas, and publication targets. Fix a disagreement in whichever
side is wrong before checking; never report it as a remark beside a checked box.

OUTPUT CONTRACT:
This section applies ONLY when the DELIVERY SCOPE GATE resolved to
`functional` or `integration`. If the gate resolved to `ambiguous` / STOP,
the sole output is the decomposition and no discovery sections appear.
Otherwise, emit every section in the `discovery.md` template order and table
shape; do not add, drop, or reorder sections.

SELF-CHECK BEFORE OUTPUT:
- No fabricated IDs, URLs, requirements, capabilities, visuals, decisions, or
  user approvals. Every `Reference / Anchor` resolves to a source
  project-context returned or present in CONTEXT — no synthesized wiki/live
  URL, path, or ID.
- Executive Summary reads as a reader map: grounded, includes business value,
  says what the file settles and what stays open, states readiness consistently
  with the Discovery Gate. Every statement agrees with the sections as they now
  stand. It names subjects not identifiers, re-lists no table rows, and implies
  no approval for an unresolved decision.
- Discovery Source Inventory cites the inputs and evidence actually used,
  including relevant non-document evidence (baseline specs, legacy/current
  implementation, legacy/live URLs, missing design assets) where applicable.
- Prior-art / existing-solution evidence is captured when the sources carry it
  (existing cartridge / accelerator module, a vendor SDK / client library /
  documented API) or recorded as searched-and-absent; nothing fabricated;
  discovery records facts only and chooses NO build-vs-reuse approach.
- FSD/ISD `NEW` / `EXTEND` decisions appear only in Delivery Scope & Targets,
  not as source inventory notes.
- Every source requirement has an RQ ID and WBS/source grouping; use
  `UNTRACKED-<n>` and `WBS-TBD` only after the source truly lacks them.
- Every requirement row has a description; no blank requirement cells.
- Requirements Trace notes are grounded and material, or `—`; no fabricated or
  requirement-restating notes.
- CLOSED-SET zero-diff holds: TRACE_RQ_SET == SOURCE_RQ_SET — every `RQ-<id>`
  in the signed source set appears in the trace exactly once, none dropped or
  invented. If not, re-run the Requirements Trace CLOSED-SET GATE before emitting.
- With a signed BRD/SRD, the trace contains only its rows; transcript/workshop-
  only or CR-topic scope goes to Change Request Candidates, never as an invented
  `UNTRACKED-<n>` / `WBS-TBD` trace row (which are used only when the source
  itself lacks an ID).
- Every trace row has a non-empty Affected capability cell, including deferred
  and out-of-scope rows.
- Topic partition is obeyed: Trace vs Dependency vs CR vs Open Question; no
  duplicate CR/Open Question topics; functional Open Questions carry no HOW
  mechanics owned by separate integrations.
- Dependencies list only external capabilities this change consumes but does
  not own, with the requirement(s) that use them, or "None."; consumed
  dependencies are not listed as Affected Capabilities.
- No `deferred` or `out of scope` trace row is duplicated as a Change Request
  candidate; a signed later-phase item stays `deferred`, not resurrected as a CR.
- A third-party/external service only an unapproved/pending stakeholder ask
  needs is a Change Request candidate, not a Dependencies row (Dependencies are
  relied on by a SIGNED in-scope requirement).
- A retired-URL redirect target/type (and similar HOW/refinement details) is an
  Open Question, not a Change Request candidate.
- Capability dispositions match the baseline: ADD for new, MODIFY for changed
  existing, REMOVE for retired/superseded existing capabilities.
- Delivery Scope & Targets lists only produced/touched FSD/ISD documents, no
  `N/A` rows.
- Document Consumption Guide carries one row per consumed section in template
  order (Requirements Trace, Affected Capabilities, Dependencies, Open
  Questions, Change Request Candidates, Cross-cutting Requirements, Delivery
  Scope & Targets), none dropped, merged, renamed, or reordered, every empty
  set `None`, each with a grounded `What it settles & why` phrase.
- Every Document Consumption Guide key-data value is identifiers/sets present in
  the section its row names: the in-scope and deferred WBS sets equal the trace
  rows with those dispositions, out-of-scope owners equal every out-of-scope
  trace row's WBS with its owner, the capability plan equals Affected
  Capabilities with directions, consumed dependencies equal the Dependencies
  rows, open items equal the questions still Open with severities, change
  requests equal the CR candidates with statuses, cross-cutting applicable omits
  every N/A area, and publication targets equal the Delivery Scope & Targets rows.
- The first Discovery Gate checkbox matches the final Open Questions table, and
  the fourth is checked only after the Document Consumption Guide was reconciled
  row by row against those sections.
