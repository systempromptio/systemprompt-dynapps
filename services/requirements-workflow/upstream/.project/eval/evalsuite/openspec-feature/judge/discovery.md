---
schema: openspec-feature
artifact: discovery
context:
  - .context/initial-context
  - specs/global-navigation/spec
  - specs/store-directory/spec
scoring:
  pass_threshold: 0.7
---

# Discovery Judge Contract

<judge_contract version="2">

## 1. Role And Authority

You judge a generated OpenSpec `discovery.md` artifact for the feature workflow.
Discovery is the first pipeline artifact. It extracts source facts, reconciles
scope, identifies touched capabilities, records open decisions, and hands a clean
scope contract to proposal/specification work.

Discovery is not an authoring step. A good candidate must not invent
requirements, capabilities, citations, behaviours, integrations, or decisions.

Authority order:

1. `CONTEXT` is the source of truth for project facts.
2. `SOURCES` and this `STANDARD` define the artifact template and evaluation
   rules.
3. `GOLDEN EXPECTED` is only a reference for expected coverage and shape. Do not
   require exact wording from the golden when the candidate is faithful.
4. `CANDIDATE` is the artifact being judged.

If `GOLDEN EXPECTED` and `CONTEXT` appear to disagree, trust `CONTEXT`.

## 1.0 Mode Detection (evaluate this FIRST)

Discovery has a blocking prerequisite (the DELIVERY SCOPE GATE) that runs before
any authoring: it classifies the ask as `functional`, `integration`, or
`ambiguous`. When the ask is `ambiguous` — a bundled
functional + integration request with NO stated role, selected scope, or named
service — the correct behaviour is to STOP and return a decomposition (a split
gate), NOT to author a discovery artifact. This judge therefore runs in one of
two modes; decide the mode before anything else:

Decide mode from the `GOLDEN EXPECTED` shape FIRST — it is the authoritative
signal, and the eval always supplies the flow-matched golden:

- **Split-gate mode** when the `GOLDEN EXPECTED` is a stop-and-ask decomposition
  (a "split before discovery" message with candidate changes and a scope choice,
  and NONE of the filled discovery sections). Judge with §8.6 and return the §8.6
  verdict shape. Skip §§3–8.5 and §9 (they are discovery-artifact specific).
- **Discovery mode** when the `GOLDEN EXPECTED` is a filled discovery artifact
  (it has the template sections — Requirements Trace, Affected Capabilities,
  Delivery Scope & Targets, Discovery Gate). Judge with §§1.1–9 as normal.

Do NOT infer split-gate mode from the size or messiness of the `CONTEXT` packet.
A bundled, messy source packet that describes several integrations is NORMAL
discovery input, not a split-gate signal. When `CONTEXT` carries any resolved
delivery-scope signal — a stated BA/architect role, a selected scope, or a named
service to build (typically a short role/scope statement appended to the base
packet) — the scope IS resolved and the mode is Discovery, even if the underlying
sources describe multiple integrations. Only fall back to the `CONTEXT`-only
signal (a bare bundled packet with NO stated role/scope/service anywhere,
including overlays ⇒ split-gate) when no `GOLDEN EXPECTED` is provided.

Mode is fixed by `GOLDEN EXPECTED` and `CONTEXT` ONLY. Never switch mode based on
the `CANDIDATE`'s shape. A candidate that emits a full discovery artifact while
the case is in split-gate mode is the PRIMARY failure to score in §8.6
(`stops_before_discovery` = `0.0`, a critical finding) — it is not a reason to
re-classify the case into discovery mode. If §1.0 selected split-gate mode, stay
in split-gate mode regardless of what the candidate produced.

## 1.1 Decision Kernel

Read this kernel before inspecting details. It is the shortest form of the
contract and resolves the known ambiguous cases.

<decision_kernel>

**Route to `critical_findings`** when the candidate changes source truth or lets
unsafe scope proceed: dropped/invented requirements, altered RQ/WBS facts, new
non-BRD scope absorbed as in scope, later-phase work marked confirmed in scope,
or first gate checkbox `- [x]` while an Open Question is `CRITICAL` and `Open`.

A "dropped requirement" means a signed `CONTEXT` requirement (an RQ/WBS row) that
is missing from the Requirements Trace. A missing or dropped **Open Question**
(e.g. redirect target/type, geocoding, holiday-hours) is NOT a dropped source
requirement: it is a HOW/refinement item, at most a `warnings` entry, and never a
`critical_findings`.

**Route to `warnings`** when the candidate preserves source truth but is
conservative, noisy, or less ready for downstream use: HOW/refinement questions tagged
`CRITICAL` with first gate `- [ ]`, duplicate CR/Open Question topics already
surfaced in the CR table, missing FSD/ISD rows while scope remains traceable, and
weak cross-cutting reasons.

The Document Consumption Guide is a PROJECTION of the body sections it names (§4.10).
A row whose key data disagrees with the section it projects, a missing fixed row, or a
fourth gate box checked over a visible disagreement are `warnings` under
`handoff_fidelity` — the section it drifted from still carries the truth. Only an
identifier the artifact and `CONTEXT` cannot support is a critical there, under the
ordinary fabrication rule.

**Gate rule:** only this combination is a gate critical:
`first gate = - [x]` AND `Open Questions contains CRITICAL + Open`.
`first gate = - [x]` with only `NICE-TO-HAVE + Open` questions is correct and
MUST NOT be routed to `critical_findings`: the checkbox literally says "No
CRITICAL open questions remain Open", not "No open questions remain".
The scope-fit or topic of a NICE-TO-HAVE row NEVER changes this. A NICE-TO-HAVE
question that is out-of-scope, wrong-flow, or an integration-HOW mechanic
(client-vs-proxy call, iframe-vs-API, geocoding approach) that arguably does not
belong in this discovery is a scope-hygiene `warnings` item under
`open_questions_quality` — it is NOT evidence that `- [x]` is wrong. The first
gate is decided ONLY by whether some Open Question row is literally tagged
`CRITICAL`; a merely misplaced or off-scope NICE-TO-HAVE row does not satisfy
that and does not create a gate critical.
All `first gate = - [ ]` cases are safe holds; score their quality, but route
gate/severity issues to `warnings`.

**Route examples:**

- Redirect target/type tagged `CRITICAL` with first gate `- [ ]` -> `warnings`,
  because this is a HOW/refinement severity-calibration defect.
- An excluded FSD/ISD document type omitted from Delivery Scope & Targets because
  this change does not produce it is CORRECT, not a finding. An unexplained
  required target missing while the rest remains judgeable -> `warnings`, because
  source scope is still traceable.
- Reserve-in-store present as a CR and duplicated as a `CRITICAL` Open Question
  with first gate `- [ ]` -> `warnings`, because the new scope is already surfaced
  and the candidate is safely holding the gate.

**Pass rule:** `pass = true` exactly when `score >= threshold` and
`critical_findings` is empty.

</decision_kernel>

## 1.2 Judge Flow Map

Follow this flow in order. Each step points to the detailed contract section that
defines the rule.

<judge_flow>

1. **Identify inputs** — read the outer prompt blocks as `STANDARD`, `CONTEXT`,
   `GOLDEN EXPECTED`, and `CANDIDATE`. See §2.
2. **Parse candidate markup** — treat Markdown section headings, tables, checkbox
   tokens, severity tokens, status tokens, and disposition tokens as structured
   data. See §3 and §4.
3. **Extract management facts** — collect the checklist facts before judging:
   requirements present, capabilities, CRs, Open Questions, first gate checkbox,
   FSD/ISD rows, and source-fact defects. See §5.
4. **Compare to source truth** — use `CONTEXT` as source of truth and `GOLDEN
   EXPECTED` only as shape/coverage reference. See §1 and §6.
5. **Route findings** — send each issue to `critical_findings` or `warnings` by
   actual risk, not by wording intensity. See §7.
6. **Score dimensions** — assign numeric scores for every rubric dimension. See
   §8.
7. **Return verdict JSON** — apply the pass/score consistency rules and return
   exactly one JSON object. See §9.

</judge_flow>

## 2. Input And Markup Model

The outer eval prompt already separates blocks with headings such as:

- `===== STANDARD (contract + rubric) =====`
- `===== SOURCES (authoritative rules/templates) =====`
- `===== CONTEXT (grounding - traceability is judged against this) =====`
- `===== GOLDEN EXPECTED (reference for this case) =====`
- `===== CANDIDATE (artifact under evaluation) =====`

Inside the `CANDIDATE`, read Markdown sections and tables as structured data so
you can judge their content.

Section presence, ordering, and table columns are enforced separately by a
deterministic harness gate that checks the candidate against the GOLDEN fixture
(the structural canon) and fails it on any missing, extra, or reordered section
or wrong columns. Do not re-litigate structure or deduct for it here — judge the
semantics of whatever sections are present.

## 3. Literal Token Definitions

Use these literal values when interpreting the candidate:

- Checkbox `- [ ]` means unchecked, not clean, or intentionally holding the
  pipeline.
- Checkbox `- [x]` means checked, clean, or claiming the gate condition is
  satisfied.
- Open Question severity values: `CRITICAL`, `NICE-TO-HAVE`.
- Open Question status values include `Open`, `Resolved`, and equivalent explicit
  states.
- Requirement dispositions include `in scope`, `deferred`, `out of scope`, and
  `CR candidate`.
- Delivery mode values (a separate column for in-scope rows) include `build`,
  `consume`, `configure`, and `—` (for out-of-scope / deferred rows).
- Capability dispositions include `ADD`, `MODIFY`, `REMOVE`.
- `CR#` rows in Change Request Candidates are not Open Questions.

Never paraphrase `- [ ]` into checked/clean. If the first Discovery Gate line is
`- [ ]`, the candidate is not claiming a clean gate.

## 4. Expected Artifact Semantics

Structure — which sections exist, their order, and their table columns — is
enforced by the harness gate against the GOLDEN, not here. This section defines
only what each section must MEAN and how to judge its content. Judge whatever
sections are present; never deduct for structural shape.

### 4.1 Executive Summary

Purpose: a READER MAP that orients a human before the detailed discovery
sections — what this file settles, what stays open, and whether it is ready. It is
not a scope lock, not a pipeline checklist, and not a second copy of the tables.
Its machine-readable counterpart is the Document Consumption Guide (§4.10); the two
split the same facts along the prose/identifier line.

Expected content: a short narrative near the top of the artifact that explains the
current problem/change context and why the change matters — including a
business-value sentence grounded in `CONTEXT`, such as reducing shopper friction,
supporting an identified business journey, improving operational clarity, reducing
risk, or driving the business outcome the source packet describes — and then
orients the reader by covering (a) what the file settles: the delivery scope, the
SHAPE of the requirement set, which capabilities move, and where each detail
lives; (b) what stays open: the subjects still undecided and the register each sits
in; and (c) readiness. Readiness wording must align with the Discovery Gate: if the
gate is blocked by an Open CRITICAL question, the summary must not claim the change
is ready for proposal.

**Anti-inventory rule — judge RE-LISTING, not identifiers.** The defect is
reproducing a table's rows in prose: an RQ/WBS roll-call, a capability inventory,
a per-question or per-CR walkthrough, a full scope list, acceptance criteria, or
implementation detail. Describing the same material by SUBJECT and by COUNT is the
intended shape and must NOT be penalized — "three refinements remain (holiday
hours, the `/stores` redirect, the Page-Designer boundary)" and "nine requirements
in scope, one deferred" are correct reader-map sentences, not duplication. An
occasional identifier used as a pointer rather than as an inventory (naming the
register a subject sits in) is likewise not a defect. What remains a defect: a
summary that walks the rows, invents an approval, or contradicts the gate.

Cross-check opportunity: the counts and subjects here should agree with the
Document Consumption Guide — "three refinements" against three `Q#` in the `Open
Questions` row's key data, "nine requirements" against nine codes in the
`Requirements Trace` row's In-scope set. A contradiction between the two blocks is a
`handoff_fidelity` finding (§4.10), reported once, not twice.

### 4.2 Discovery Source Inventory

Purpose: identify the primary inputs and evidence used for discovery. This
section is not limited to documents: it may include requirements documents,
baseline specs, current implementation evidence, live/legacy URLs, design assets,
tracker records, meeting notes, API/contracts, configuration, or explicit source
availability checks.

Minimum expectation: cite the BRD/SRD or equivalent requirements source, existing
OpenSpec baseline specs/current behaviour evidence, meeting notes when relevant,
and tracker source when present. When UI is affected, cite design assets or
explicitly record that none were provided. When a legacy URL or current storefront
surface is moved, retired, or redirected, cite that source/evidence explicitly.
The `Reference / Anchor` column should point at a source that actually exists in
`CONTEXT`. A workspace-relative inbox path, baseline-spec path, Confluence URL/Page
ID, tracker key, or live URL/route is all acceptable. Precise `:line-range` anchors
are a nice-to-have, not a requirement: an anchor that names the right source
without line numbers is fine and must NOT be treated as a fabrication or a
`source_fidelity` defect. If project-context provides only an external identifier
(for example a Jira key) and no file anchor, the identifier is acceptable and
should not be expanded into a fabricated inbox file. What IS a defect is an anchor
pointing at a source neither `CONTEXT` nor GOLDEN EXPECTED supports (an invented
file/path/ticket/spec/URL), or a false claim that a source was "verified".

Consumed integrations/services are recorded in the Dependencies section (§4.5),
not as Source Inventory rows; do not penalize a Source Inventory that omits
per-integration dependency rows when the Dependencies section carries them.

Prior-art / existing-solution evidence: when `CONTEXT` carries facts about what
ALREADY EXISTS that could satisfy the need — an installed or marketplace/partner
cartridge, an accelerator/platform module or feature, or a vendor SDK / client
library / documented API — capturing those as Source Inventory rows / requirement
Notes is expected coverage (score positively under `source_fidelity`). Omitting a
prior-art fact `CONTEXT` clearly provides is a `source_fidelity` gap; listing a
cartridge/module/SDK/API that neither `CONTEXT` nor GOLDEN EXPECTED supports is a
`no_fabrication` defect. Discovery only RECORDS these facts (and may record the
user's stated approach preference as an input) — it must NOT choose a
build-vs-reuse approach (reuse vs SDK vs from-scratch); that decision belongs to
design, so its ABSENCE here is correct and never a finding.

Grounding check — validate anchors against `CONTEXT` and GOLDEN EXPECTED, not a
hard-coded allowlist. Together they already tell you which sources exist for this
case: the source_type map in the source packet names the project-context inbox
files that back it, the signed sources, tracker record, and baseline-spec blocks
are present inline, and GOLDEN EXPECTED's Source Inventory shows the concrete
anchor (inbox filename + line range) for each mapped source. The source_type map
may describe a source generically (e.g. "matching project-context inbox confluence
files") without naming the file; GOLDEN EXPECTED supplies the canonical filename
for exactly those cases. For every candidate `Reference / Anchor`, decide whether
it corresponds to a source that `CONTEXT` or GOLDEN EXPECTED supports:
- A citation that points at a source available here (a mapped inbox file — using
  the concrete filename GOLDEN EXPECTED cites for it — a signed source, the
  tracker record, or a baseline spec block), in a well-formed anchor shape, is
  valid, even if the exact line range differs slightly. Do NOT flag a candidate
  anchor as fabricated merely because its concrete inbox filename is not spelled
  out verbatim in the source_type map: if GOLDEN EXPECTED cites that same file, it
  is grounded.
- The user's stated delivery intent is itself a legitimate discovery input. A
  scope-decision reference of the form `user request` or the `/opsx:propose`
  command argument is schema-sanctioned and MUST NOT be flagged as a fabricated
  source, even when GOLDEN EXPECTED spells it slightly differently (e.g. `user
  request` vs the command form); treat these as equivalent. What is still a defect
  is inventing a project-context meeting-note or scope-decision FILE to represent a
  verbal decision.
- A citation that points at a file, path, ticket, spec, or URL that neither
  `CONTEXT` nor GOLDEN EXPECTED supports is fabricated; flag it (this is the "нет
  ли чего-то левого" check). An inbox path invented for a source that `CONTEXT`
  exposes only as a bare identifier (for example fabricating `inbox/jira/<KEY>.md`
  when the tracker is provided only as a key, and GOLDEN EXPECTED cites the
  identifier form) counts as fabricated.
- A required source that is available here but the candidate omits (see the
  minimum expectation above) is a missing-citation gap; flag it.

FSD/ISD source lookup results may appear here as availability evidence, but the
`NEW`/`EXTEND` publication decision belongs in `Delivery Scope & Targets`, not
in this section. Do not require exact wording from the golden, but prefer
candidate rows that explain how each source was used in discovery rather than
generic notes.

### 4.3 Requirements Trace

Purpose: preserve signed requirements and scope disposition.

All requirements in `CONTEXT` must be represented. Later-phase items explicitly
called out by the source should be `deferred`. New scope not in the signed BRD
should not be silently absorbed into this trace as in scope.

**Scope disposition is owner-based.** A requirement is `in scope` when THIS change
closes its acceptance criteria — WHETHER it builds the work itself OR consumes /
configures the selected service/vendor that already fulfils it. It is `out of
scope` only when the deliverable is owned by a SEPARATE change / WBS / team
(another feature FSD, a consumer integration, a platform-wide concern) or is
consumed by a different surface, with that owner recorded as a Dependency.
Consuming, rendering, reading, or configuring a vendor/service does NOT by itself
make a requirement `out of scope`: a vendor-hosted / no-code / externally-authored
artifact THIS change consumes to satisfy an in-scope requirement is correctly
`in scope`, with its authoring surface listed under Dependencies — do NOT flag
that as absorbed non-BRD scope or as a disposition defect. The inverse — a
requirement THIS change actually closes marked `out of scope` merely because it
consumes a vendor, or because it shares a WBS/vendor prefix — IS a
`disposition_correctness` defect. It conservatively UNDER-claims scope (in-scope
work marked out), so route it to `warnings` (§7.2): lower `disposition_correctness`
but do NOT cap the score. Judge WHETHER a deliverable is
owned elsewhere from `CONTEXT`, using GOLDEN EXPECTED only as a reference for
which rows land where; the specific owner SLUG a golden picks for an out-of-scope
row is normalized wording, not source truth. A valid `out of scope` owner is a NAMED
separate change / WBS / team / feature FSD that DELIVERS the row; a vendor console,
hosted authoring / no-code studio, or an operator / author named as the "owner" is
NOT a valid owner. An `out of scope` row justified ONLY by such a surface (e.g. a
map studio) — where GOLDEN / `CONTEXT` show the row is one THIS change
consumes / configures — is a `disposition_correctness` defect routed to `warnings`
(§7.2), not an allowed disposition: a conservative under-claim that lowers
`disposition_correctness` without capping the score.

**Delivery mode is a separate current-state FACT, not a scope signal.** In-scope
rows carry a `Delivery mode` of `build`, `consume`, or `configure`; out-of-scope
and deferred rows use `—`. Mode never changes disposition (a `consume` /
`configure` row is still `in scope`). Mode follows ONE actor test: `build` when
THIS change writes the code, `consume` when it references or renders an
already-published / externally-authored artifact AS-IS, `configure` when THIS
change ITSELF sets the vendor-hosted settings. When someone else authors and
publishes the artifact and this change only references it, the mode is `consume`,
not `configure`. Judge mode only lightly: an in-scope row
whose recorded mode plainly contradicts `CONTEXT` (e.g. `build` for an artifact
`CONTEXT` shows is authored and published elsewhere, or `consume`
for connector work the change clearly writes) is at most a `warnings` entry — it
is a factual-accuracy nuance, never a `critical_findings` item, and does not lower
`disposition_correctness` when the in/out disposition itself is right. An invalid
mode token (anything other than `build` / `consume` / `configure`) on an in-scope
row, or a non-`—` mode on an `out of scope` / `deferred` row, is likewise a
`warnings` entry (factual accuracy), never a `critical_findings` item. A missing
Delivery mode column or blank mode cells are structural and handled by the harness
gate; do not deduct here.

`Requirement` and `Requirement Details` are two separate columns that mirror the
source: `Requirement` carries the verbatim source requirement (e.g. the user
story); `Requirement Details` carries the verbatim source detail list, preserving
its numbering, and is `—` when the source row has no separate details. Paraphrasing
or dropping source-present details, merging both into one cell, or inventing details
the source does not carry is a `source_fidelity` / `no_fabrication` defect. Flattening
a multi-line detail list into a single cell (numbering preserved inline) is expected
and correct, not a defect.

The `affected capability` of an IN-SCOPE requirement MUST match the capability
`CONTEXT` assigns it to. Re-pointing such a requirement to a different capability
— e.g. mapping a "reachable from global navigation" requirement to
`store-locator` instead of `global-navigation` — is a materially altered source
fact, even when the requirement text is intact.

On `out of scope` / `deferred` rows the cell is a BEST-EFFORT attribution of the
owning workstream. Where GOLDEN names an owner slug that `CONTEXT` does not
carry, that slug is shape/coverage GUIDANCE, not source truth: any owner the
candidate can ground in `CONTEXT` is acceptable, and a differing slug is at most
a `warnings` entry. Grade these rows on the in/out disposition, not on matching
GOLDEN's owner wording.

The trace is scoped by the change's delivery scope (see §4.9 and the "Delivery
scope decision" in `CONTEXT`). No source requirement is dropped: requirements
whose deliverable is owned by a SEPARATE change / WBS / team are still listed,
dispositioned `out of scope` (delivery mode `—`) with a note naming that owner,
or mapped to an integration/functional obligation as GOLDEN EXPECTED shows. Do
not penalize a correctly-scoped trace for declining to mark another change's
owned requirements as IN SCOPE — recording them `out of scope` (delivery mode
`—`, owner named in the note) is the intended branching. This NEVER licenses
OMITTING the row: "declining to mark in scope" means dispositioning it
`out of scope`, not deleting it. A signed requirement that GOLDEN EXPECTED carries
(whether in scope OR out of scope — including the flow-appropriate sibling-owned
rows an integration trace keeps, e.g. the functional RQ-701..710 rows) but that is
ABSENT from the candidate Requirements Trace is a DROPPED signed row: route it to
`critical_findings` per §5 fact 4 and §7.1, never treat it as an allowed
exclusion. Conversely, a vendor-consumed requirement this change DOES close stays
`in scope` (mode `consume` / `configure`) per the owner-based rule above.

The `Notes` column is an optional context cache: a place to record material
findings gathered while sourcing (client wishes, workshop nuance, unresolved
ambiguity, technical constraints, or a cross-link to a `Q#`/`CR#`). Score it as
follows:

- An empty note (`—`) is expected and correct for routine rows; never penalize a
  blank note or treat it as missing content. Most rows legitimately have none.
- A note that is present must be grounded in `CONTEXT`; a fabricated,
  unsupported, or requirement-restating note is a `source_fidelity` /
  `no_fabrication` defect.
- Capturing a genuine, source-backed finding that the golden also notes (e.g. the
  geolocation privacy constraint, the fixed v1 service list, or the `/stores`
  redirect cross-link) is positive signal for `source_fidelity`, but do not
  require the candidate to match the golden's notes verbatim or to note every row
  the golden notes. Judge notes on grounding and materiality, not coverage.

### 4.4 Affected Capabilities

Purpose: show which OpenSpec capabilities the change touches.

Enumeration is expected to draw on the project-context skill (baseline spec
search) **and** direct inspection of `openspec/specs/` plus `openspec list`;
project-context is a lead, not the sole source. Do not penalize the artifact for
naming these tools, and do not require any specific tool to be cited. Judge
completeness against the capabilities implied by `CONTEXT` and the Requirements
Trace: a single-row table on a clearly multi-area feature is a miss.

Spec path for an `ADD` (new) capability: the spec does not exist yet, so **both
forms are equally valid and MUST NOT be penalized, warned on, or reported as
missing** — the planned canonical path (e.g. `openspec/specs/<capability>/spec.md`)
or an explicit not-yet-exists marker (e.g. `— (does not exist yet)`, `to be
created`). The `ADD` disposition already conveys non-existence; do not require the
golden's exact form here. Only treat the location as genuinely missing when the
cell is blank for a `MODIFY`/`REMOVE` capability, whose spec should be locatable in
`CONTEXT`.

Name of a NEW (`ADD`) integration capability: the candidate invents this name, so
do NOT require an exact match to the golden's spelling. Accept any reasonable
agnostic, service-oriented variant (e.g. `<service>-integration`, `<service>-maps`,
`<service>-maps-integration`). What matters is: (a) the name is agnostic and
service-scoped, not feature-owned (not named after the consumer feature); and
(b) it is used CONSISTENTLY across the Requirements Trace, Affected Capabilities,
and the Delivery Scope target path. Route an internal inconsistency (different
names in Trace vs Affected vs target) to `warnings`, and a feature-owned name to
`warnings`; do not treat a mere agnostic-variant spelling difference from the
golden as a `critical_findings` alteration of source truth.

Judge dispositions against baseline capability specs in `CONTEXT`:

- New capability absent from baseline -> `ADD`.
- Existing baseline capability changed by the feature -> `MODIFY`.
- Existing baseline capability retired or superseded -> `REMOVE`.

A touched `MODIFY` capability should normally have at least one Requirements
Trace row mapped to it. A `REMOVE` capability may instead be justified by
baseline/current-behaviour evidence and explicit retirement/supersession context;
do not require a signed BRD requirement to map to a retired legacy capability
when the source truth shows it is being replaced.

Owner-of-out-of-scope-rows guard: a capability named ONLY as the owning
capability of `out of scope` / `deferred` Requirements Trace rows is a
consumer/other-change surface this change does NOT touch. Under an integration
scope it is correctly absent from Affected Capabilities (which lists only the
capabilities this change owns); do NOT flag such a capability as a "missing
touched capability" (§7.1). Only capabilities this change ADDs / MODIFIEs /
REMOVEs are "touched" and required here.

### 4.5 Dependencies

Purpose: list external capabilities/services an in-scope requirement RELIES ON
that this change does NOT own (owned by another change/team and consumed here).

- A row belongs here when an in-scope requirement needs a service owned by a
  different change to work (a third-party integration, another service
  integration, or an already-integrated data/API service). It is correct to note
  which requirement(s) consume it and whether it is already integrated or still to
  be built as a separate integration change.
- A platform/OOTB-managed capability an in-scope requirement relies on — Business
  Manager config/attributes, Page Designer authoring, Campaigns/Promotions/Source
  Codes, Custom Site Preferences, or Shopper Context — is a legitimate Dependencies
  row (type `platform config`, with its availability status and the consuming
  requirement(s)). When `CONTEXT` clearly establishes such a reliance, omitting it
  is a `source_fidelity` / `cr_handling` gap and routes to `warnings`, never to
  `critical_findings`. Do NOT require a Notes backlink on the consuming requirement
  to credit it: the §4.3 Notes rule (grounding and materiality, not coverage) still
  governs, so a present-and-grounded backlink is positive signal but its absence is
  not a defect.
- When an in-scope requirement is satisfied by CONSUMING / CONFIGURING a
  vendor-hosted / no-code / externally-authored artifact (delivery mode `consume` /
  `configure`), the requirement stays `in scope` in the Requirements Trace AND the
  vendor's authoring/hosting surface is listed here as the dependency it relies on.
  This pairing (in-scope row + Dependencies row for the authoring surface) is the
  expected owner-based shape, not a double-count or a defect.
- A Dependency is something RELIED ON (exists/owned elsewhere). It is distinct
  from a Change Request (a scope-addition on this change's own surface) and from an
  Open Question (a HOW/refinement decision). Listing a consumed dependency as an
  Affected Capability, or a scope-addition as a Dependency, is a defect.
- "None." is correct and expected when the change consumes nothing external — for
  example an agnostic integration capability that other changes consume but that
  does not itself depend on another owned service. Do not treat a "None." here as a
  missing section or manufacture a dependency to fill it.
- Whether a consumed integration is invoked client-side or via a proxy, and its
  key/quota mechanics, are HOW-questions owned by that integration's own change —
  not required detail in this Dependencies list.

### 4.6 Change Request Candidates

Purpose: surface proposed scope changes not already in signed requirements, on
THIS change's own delivery surface.

New non-BRD scope for the surface this change owns belongs here. It is acceptable
for the CR to remain unresolved at discovery time as long as the artifact does not
absorb it into confirmed scope. A stakeholder ask about a DIFFERENT surface
(another feature, or a service integration this change merely consumes) is not a
CR for this change — expecting it here, or penalizing its absence here, is wrong;
it belongs to that other change. For a functional change, a genuinely new
client/workshop ask not traceable to the signed source (and on the feature's own
surface) is a valid CR; for an agnostic integration change, an unrelated
feature-surface ask is correctly absent (CR "None.").

### 4.7 Open Questions

Purpose: capture ambiguities or decisions needed to refine already-known scope.

Severity semantics:

- `CRITICAL`: the WHAT, scope, requirement, or acceptance criteria are unclear
  enough that a proposal built now could be wrong.
- `NICE-TO-HAVE`: a HOW-to-implement or refinement decision that can flow into
  grooming/design without changing what is being built.

Typical NICE-TO-HAVE examples: redirect type, exact redirect target,
holiday-hours display when the backend schema is not yet known.

**Scope filter for `functional` (FSD) discovery:** Open Questions must be about
THIS change's own functional surface — the WHAT, acceptance criteria, or
refinements of in-scope requirements. HOW mechanics owned by a separate
integration change are not valid Open Questions here:
- geocoding call approach (client vs server proxy) → belongs to the maps
  integration SA/ISD discovery, not the consumer FSD;
- address-autocomplete call approach → belongs to the autocomplete integration;
- appointment-booking embed/iframe vs API choice → belongs to that integration.
If a functional candidate lists these integration-HOW questions as Open
Questions, route each instance to `warnings` under `open_questions_quality`
("out-of-scope integration question for this functional change"). The presence
of such questions does not by itself block acceptance; route them to `warnings`.

Severity-calibration defects are quality defects. Route them to `warnings` by
default. Promote an Open Question issue to `critical_findings` only when the
candidate also claims a clean first Discovery Gate checkbox (`- [x]`) while a
`CRITICAL` Open Question remains `Open`.

A missing NICE-TO-HAVE / HOW Open Question (e.g. the redirect target/type for a
retired URL) is a `warnings` entry at most, never a `critical_findings`. It caps
`open_questions_quality` no lower than `0.75` and must not be scored under
`source_fidelity` — the dropped item is an optional refinement, not a signed
requirement.

A severity mis-calibration in the SAFE (conservative) direction is floored the
same way. Over-tagging a NICE-TO-HAVE / HOW refinement (e.g. a redirect type or
target) as `CRITICAL` — which correctly forces the first Discovery Gate to
`- [ ]` — loses no WHAT, drops no signed requirement, and holds the gate safely,
so it is a scope/severity-hygiene `warnings` entry that caps
`open_questions_quality` no lower than `0.75`. This is the same routing already
given to a missing NICE-TO-HAVE and to an out-of-scope integration-HOW row that
remains `Open` under a correct `- [x]` gate: a safe mis-calibration is never a
source-truth loss. Only the DANGEROUS (under-claiming) direction is scored low —
a real WHAT-ambiguity mis-tagged `NICE-TO-HAVE`, or a `- [x]` gate claimed while
a `CRITICAL` question is `Open` (§7.3) — because that one can let a wrong
requirement set through. Do not stack these hygiene warnings into a compounded
sub-0.75 score: a functional candidate whose only Open-Questions faults are a
safe over-tag and/or out-of-scope integration-HOW rows scores `open_questions_quality`
no lower than `0.75`.

### 4.8 Cross-cutting Requirements

Purpose: make cross-cutting concerns explicit, not accidental.

Expected areas:

- Accessibility
- SEO
- Consent / privacy
- i18n

Each area should be marked applicable or not applicable with a grounded reason.
Applicable areas should cite or summarize the source requirement or decision.

### 4.9 Delivery Scope & Targets

Purpose: declare the change's delivery scope (functional or integration) and the
documentation outputs (FSD/ISD) that this change actually produces or touches.

The `Scope` line and the FSD/ISD target rows must be consistent with the delivery
scope recorded in `CONTEXT` (see the "Delivery scope decision") and with GOLDEN
EXPECTED. Judge the targets against the DECLARED scope:

- `Scope: functional` -> list the FSD target(s) only. Integration dependencies
  should be recorded in the Dependencies section (§4.5), not represented as `N/A`
  ISD rows.
- `Scope: integration` -> list the ISD target(s) only. The ISD should be
  service-oriented and named/scoped for the integration capability (e.g.
  `cartora-maps-integration`), not for the first consumer feature.

Do not require excluded document types and do not prefer `N/A` rows; absence of a
document type means this change does not produce it. A `Scope`/target
combination that contradicts `CONTEXT` is a defect routed to `critical_findings`.
The agnostic-naming rule applies ONLY when `Scope: integration`. Under
`Scope: integration`, an ISD target whose NAME or working-copy PATH is
feature-owned (e.g. a path under the consumer feature's change folder such as
`openspec/changes/store-locator/isd/...`) instead of the agnostic service-owned
change is a defect routed to `warnings` (§7.2) — the same route §4.4 gives a
feature-owned capability name; it preserves source truth, so it is never a
`critical_findings` entry on its own.

Under `Scope: functional` the opposite holds: the FSD is OWNED by the feature, so
a working-copy path under this change's own folder (`openspec/changes/<change>/fsd/<feature>.md`)
is CORRECT. Never report a functional FSD path as feature-owned, and never ask a
functional change for a service-oriented name — that is the integration rule
applied outside its scope, and the GOLDEN itself carries the feature-owned path.

Separately, and for BOTH scopes: a working-copy path must use the change name
`CONTEXT` states. A path built on a different change slug is a GROUNDING defect
routed to `warnings` — report it as such, never as the agnostic-naming defect
above, which is about who OWNS the target, not how the change is spelled.

### 4.10 Document Consumption Guide

Purpose: the interface a downstream reader — an LLM first, a human second — uses to
consume this artifact without re-reading the whole body. It has ONE row per
consumed section, and each row answers WHAT that section settles and WHY it matters
downstream (the `What it settles & why` cell — navigation) plus the KEY DATA that
section exports (the `Key data` cell — the identifiers and sets a consumer joins
on). It sits directly after the Executive Summary — the two-layer interface before
the detail sections — and is a PROJECTION of the body sections it names, never a new
source of truth.

Judge it on two things only: completeness of the fixed per-section rows, and
agreement of each row's key data with the section it names.

**Fixed rows.** One row per consumed section, in this order: `Requirements Trace`,
`Affected Capabilities`, `Dependencies`, `Open Questions`, `Change Request
Candidates`, `Cross-cutting Requirements`, `Delivery Scope & Targets`. All seven
should be present, in that order, with an empty set written `None` rather than
dropped. A dropped, merged, renamed, or reordered row is a `handoff_fidelity`
defect. Do NOT require the golden's exact label wording where a candidate's variant
is unambiguous; do require the row to exist.

**Agreement.** Check each row's `Key data` against the section it names:

- `Requirements Trace` carries three labelled sets. In scope: the WBS codes of the
  trace rows dispositioned `in scope`. Deferred: those dispositioned `deferred` (or
  `None`). Out of scope: the WBS of EVERY trace row dispositioned `out of scope`,
  each with the owner its capability cell names — the WHOLE out-of-scope set, never
  the RQ id. Judge these as SETS of codes, not as ordering or formatting.
- `Affected Capabilities` key data equals Affected Capabilities with each
  capability's ADD / MODIFY / REMOVE direction.
- `Dependencies` key data equals the Dependencies rows (`None` when that section
  reads "None.").
- `Open Questions` key data equals the Open Questions rows still `Open`, with their
  severities. Carrying each still-`Open` question WITH the severity the Open
  Questions table assigns it is the REQUIRED projection: a severity here that
  matches the table (e.g. `Q1 CRITICAL + Open` in both) is agreement, never drift —
  do not deduct `handoff_fidelity` for it, and do not re-route an Open Questions
  severity or scope defect (§4.7) into a second deduction. A question the table
  marks `Resolved` should be ABSENT here, not carried with a Resolved marker.
- `Change Request Candidates` key data equals the CR candidates with their statuses
  (`None` when there are none).
- `Cross-cutting Requirements` key data lists the areas marked applicable and omits
  every area marked `N/A`. An `N/A` area is SUPPOSED to be absent: its absence is
  the correct projection and must NEVER be reported as drift or as missing. The two
  defects here are listing an `N/A` area and omitting an applicable one.
- `Delivery Scope & Targets` key data equals the resolved scope
  (functional / integration) plus the Delivery Scope & Targets rows — document
  type, disposition, and working-copy path. An unknown Confluence target stays
  `TBD`.

**The `What it settles & why` cell is short human-readable prose, and is judged
only for grounding, never for wording.** It orients a reader to the section; a
phrase that agrees with the body is correct however it is worded, and a
plainer or richer phrasing than the golden's is NOT a defect. It becomes a
`handoff_fidelity` defect only when it asserts something the section it names does
not support (e.g. calls a deferred requirement "out of scope", or claims a section
settles a decision the body leaves open).

**Key data is identifiers and sets**, not prose: WBS codes, capability names with
direction, `Q#`/`CR#` with status, paths. A `Key data` cell that restates a
requirement title or explains itself in a sentence is a `handoff_fidelity` defect.

**Drift is a disagreement about CONTENT, never about wording.** Judge each row's
key data as a SET compared with its section, and ignore presentation: ordering,
separators, backticks, an owner slug spelled differently from the golden's
normalized choice, a path written with or without backticks, a status phrased in
fewer words, or a dependency named more briefly than its Dependencies row. Report
drift only when the row and its section disagree about WHICH items are in the set
or what their state is. A row that matches its section but not the golden's phrasing
is correct.

**Routing.** Projection drift — key data here that disagrees with the section it
projects, or a `What it settles & why` phrase the section does not support — is a
`warnings` entry under `handoff_fidelity`. The artifact still carries the truth in
the section it drifted from, so drift alone is NOT a source-truth critical; do not
route it to `critical_findings`, and do not double-report the same drift under
`source_fidelity` or `disposition_correctness`. The one exception follows the
existing fabrication rule: an identifier here that NO section of the artifact and
no `CONTEXT` fact supports — an invented WBS code, capability, dependency,
`Q#`/`CR#`, or working path — is a `no_fabrication` critical like any other
invented fact.

Checking the fourth Discovery Gate box while the guide visibly disagrees with its
sections is a false self-check: report it once as a `warnings` entry under
`handoff_fidelity`. It is not a gate critical — the gate critical is reserved for
the first checkbox with a `CRITICAL` + `Open` question (§4.11, §7.3).

### 4.11 Discovery Gate

Purpose: expose a management interface, not a hidden inference.

The first gate checkbox means exactly: "No CRITICAL open questions remain Open."
Read it literally.

- `- [x] No CRITICAL...` means the candidate claims the gate is clean.
- `- [ ] No CRITICAL...` means the candidate does not claim the gate is clean.

Open NICE-TO-HAVE questions do not block this first gate line.
A candidate may correctly check the first line while leaving NICE-TO-HAVE
questions `Open` for grooming/design. That is the expected BA happy path in this
fixture and matches the golden. Do not reinterpret this line as "all open
questions are resolved"; doing so is a judge error.

## 5. Facts To Extract Before Judging

Before scoring, extract these facts from the candidate. Keep this extraction out
of the final output; only the brief `<reasoning>` routing pass and the JSON
verdict (see §9) are emitted.

<extraction_checklist>

1. List the section headings present in the candidate.
2. Summarize the Executive Summary in one line and record whether it includes
   business value, avoids detailed scope duplication, and matches the Discovery
   Gate readiness state.
3. List the Discovery Source Inventory rows by source/topic and their
   `Reference / Anchor`, including any explicit missing-source rows or source
   availability checks. For each anchor, mark whether it is grounded (points at a
   source `CONTEXT` or GOLDEN EXPECTED supports), fabricated (points at a source
   neither supports), or a required available source that is missing.
4. Reconcile the signed requirement set EXPLICITLY — this is a mechanical set
   operation, not a judgement, and you MUST perform it before scoring:
   a. `SIGNED_RQ_SET` = EVERY RQ id that appears anywhere in `CONTEXT` (the base
      requirements packet AND every per-flow overlay / additional source block)
      OR in `GOLDEN EXPECTED`'s Requirements Trace. This includes the
      flow-appropriate sibling-owned rows the golden carries `out of scope` for
      traceability — e.g. the functional RQ-701..710 rows an integration trace
      keeps. Enumerate the full list; do NOT pre-filter it by scope.
   b. `CANDIDATE_TRACE_RQ_SET` = the RQ ids that actually appear as rows in the
      candidate Requirements Trace.
   c. `DROPPED = SIGNED_RQ_SET − CANDIDATE_TRACE_RQ_SET`. State all three sets in
      your `<reasoning>` routing pass. EVERY id in `DROPPED` is a DROPPED SIGNED
      ROW (lost source truth) and MUST route to `critical_findings` per §7.1 — it
      is NOT a demotable absence, and "correctly excluded for this scope" is NEVER
      a valid reason to omit a row: another change's owned requirement is recorded
      `out of scope`, it is not deleted. A signed id PRESENT in the trace but
      dispositioned `out of scope` / `deferred` is NOT in `DROPPED` (it is
      present).
5. List touched capabilities and their dispositions. Then, for each requirement,
   record its `disposition`, `delivery mode`, and `affected capability`, and
   verify: (a) the in/out disposition is owner-based and matches GOLDEN EXPECTED
   (in-scope = this change closes the AC, including by consuming/configuring a
   vendor; out-of-scope = a separate change/WBS/team owns it); (b) the delivery
   mode is a plausible current-state fact (`—` for out/deferred rows); and (c) for
   IN-SCOPE rows, the affected capability matches `CONTEXT` and is consistent with
   the Affected Capabilities table — on out-of-scope / deferred rows the owner slug
   is best-effort attribution, so check only that an owner is named, not that it
   equals GOLDEN's. Flag any IN-SCOPE requirement re-pointed to a different
   capability than `CONTEXT` assigns, any `MODIFY` capability that no requirement maps to,
   and any `REMOVE` capability that lacks baseline/current behaviour evidence of
   retirement or supersession.
6. List CR candidates by CR ID and short topic.
7. List Open Questions with Q ID, severity, status, and short topic.
8. Record the exact first Discovery Gate checkbox token: `- [ ]`, `- [x]`, or
   missing.
9. Determine whether any Open Question row has `Severity = CRITICAL` and
   `Status = Open`.
10. Determine whether any topic appears both as a CR candidate and as an Open
   Question.
11. Record the Delivery Scope & Targets `Scope` (functional / integration) and
    the FSD/ISD target rows; verify the scope and targets match the delivery
    scope in `CONTEXT`, with no `N/A` rows required for excluded document types.
12. Reconcile the Document Consumption Guide projection (§4.10): list which of the
    seven fixed per-section rows are present, then compare each present row's key
    data against the section it names — in particular the `Requirements Trace`
    in-scope / deferred / out-of-scope WBS sets against the trace dispositions, the
    `Affected Capabilities` plan against Affected Capabilities, `Open Questions`
    against the questions still `Open`, and `Cross-cutting Requirements` against the
    areas marked applicable. Record each disagreement as drift, and separately mark
    any identifier here that no section and no `CONTEXT` fact supports.
13. List any invented, dropped, materially altered, or misclassified source facts
    (for example a Delivery Scope & Targets `NEW`/`EXTEND` decision recorded as if it
    were a source inventory fact).

</extraction_checklist>

Do not emit a critical gate finding unless facts 8 and 9 prove the dangerous gate
case.

## 6. Judging Pipeline

Apply this pipeline in order:

1. Parse the candidate structure and required tables.
2. Extract literal management-interface facts from section 5.
3. Compare source inventory and source fidelity against `CONTEXT`.
4. Compare capability and requirement dispositions against `CONTEXT`.
5. Classify CR candidates and Open Questions by topic and role.
6. Evaluate cross-cutting and Delivery Scope & Targets completeness.
7. Classify findings by severity.
8. Assign numeric rubric scores.
9. Compute overall score and pass/fail.

Do not start from a general impression. Findings must be traceable to a table row,
checkbox token, missing section, or source requirement.

## 7. Severity Policy

Classify each issue by routing it to one destination. Use the first matching
route below.

### 7.1 Route To `critical_findings`

**Route to `critical_findings`** when the issue changes source truth, loses source
truth, or claims unsafe scope can proceed:

- invented, dropped, or materially altered signed source requirement (an RQ/WBS
  row from `CONTEXT`), citation, capability, or behaviour. A DROPPED signed row is
  any id in `SIGNED_RQ_SET − CANDIDATE_TRACE_RQ_SET` (§5 fact 4): a signed
  requirement absent from the Requirements Trace. It is LOST source truth and
  routes here — it is NOT demoted as an ordinary absence by the shared severity
  gate. A signed row that is PRESENT but dispositioned `out of scope` / `deferred`
  is not a drop and does not route here. "Materially altered" means the row's id,
  text, details, or citation was changed — a `Delivery mode` cell is NOT source
  truth and never satisfies this bullet. A missing Open Question is NOT this;
  route it to `warnings` (§4.7);
- new non-BRD scope absorbed as confirmed in-scope instead of surfaced as a CR
  candidate;
- later-phase/deferred source item marked confirmed in-scope;
- required touched capability missing or materially wrong;
- an IN-SCOPE requirement's `affected capability` re-pointed to a different
  capability than `CONTEXT` assigns, a touched `MODIFY` capability left with no
  requirement mapping to it, or a touched `REMOVE` capability lacking
  baseline/current-behaviour evidence of retirement or supersession. A
  re-pointing finding MUST quote the candidate row's exact `Affected capability`
  cell as evidence; if that cell matches the source owner, the finding
  is invalid — do not record it. This route covers IN-SCOPE rows only: on an
  `out of scope` / `deferred` row the cell is a best-effort attribution of the
  owning workstream, so a slug that differs from GOLDEN's normalized choice is at
  most a `warnings` entry and NEVER a critical (§4.4);
- first Discovery Gate checkbox is `- [x]` while an Open Question is `CRITICAL`
  and `Open`;
- structure missing so severely that source traceability cannot be judged.

A wrong `Delivery mode` value (`build` / `consume` / `configure`) on a row whose
in/out disposition is correct is NOT a source-truth alteration and NEVER routes
here — delivery mode is a current-state fact, so route it to `warnings` per §4.3
(§7.2). Likewise a CONSERVATIVE disposition error — a signed row PRESENT (id,
text, citation intact) but marked `out of scope` / `deferred` when GOLDEN /
`CONTEXT` show THIS change closes it (e.g. a vendor-consumed row attributed to a
vendor console) — UNDER-claims scope and routes to `warnings` (§7.2), lowering
`disposition_correctness` but NOT capping the score. Only the requirement, RQ/WBS,
citation, capability, or behaviour — or an OVER-claiming disposition (out-of-scope
/ deferred / non-BRD work marked confirmed in-scope, per the bullets above) — can
be a source-truth critical.

A capability that the Requirements Trace names ONLY as the owner of `out of scope`
(or `deferred`) rows — a consumer feature/surface this change does not own, e.g.
`global-navigation`, `store-directory`, or `marketing-homepage` under an
integration scope — is NOT a "touched" capability. It is CORRECTLY absent from
Affected Capabilities (which for an integration change lists only capabilities
this change owns), and its absence NEVER routes here as a "required touched
capability missing" (§4.4). Only a capability this change actually ADDs,
MODIFIEs, or REMOVEs is touched.

When this route is used, cap overall `score` at `0.5`.

### 7.2 Route To `warnings`

**Route to `warnings`** when source truth is preserved but the artifact is
conservative, noisy, less polished, or less ready for downstream use:

- first Discovery Gate checkbox is `- [ ]` while all genuine Open Questions are
  NICE-TO-HAVE;
- Executive Summary is absent, not reader-oriented, lacks a business-value
  sentence, re-lists a table's rows instead of describing the file by subject and
  count (§4.1), or claims proposal readiness inconsistent with the Discovery Gate
  while the rest of the artifact remains judgeable. Naming subjects, stating
  counts, or pointing at the register a subject sits in is the intended reader-map
  shape and is NOT a finding;
- Document Consumption Guide drops, merges, renames, or reorders one of the seven
  fixed per-section rows, omits `None` for an empty set, or carries prose where
  key-data identifiers belong;
- a Document Consumption Guide row's key data disagrees with the section it projects
  (§4.10), or the fourth Discovery Gate box is checked while such a disagreement is
  visible, while the sections themselves remain correct and traceable;
- the Executive Summary's counts or subjects contradict the Document Consumption
  Guide (report once, under `handoff_fidelity`);
- redirect target/type, geocoding implementation, or holiday-hours display is
  tagged `CRITICAL` while the first gate checkbox is `- [ ]`;
- new-scope topic is already surfaced as a CR and is also listed as an Open
  Question while the first gate checkbox is `- [ ]`;
- HOW-to-implement decision appears as a CR Candidate instead of an Open Question;
- cross-cutting reason is missing, weak, or under-grounded;
- a NICE-TO-HAVE / HOW Open Question (redirect target/type, geocoding,
  holiday-hours) is missing or dropped while signed requirements remain traceable;
- the first section is document-only and omits relevant non-document evidence
  from `CONTEXT` (for example baseline specs, current implementation evidence,
  legacy/live URL evidence, or explicit missing design assets) while downstream
  traceability remains judgeable;
- source inventory rows cite only broad files or vague locations when `CONTEXT`
  provides precise line anchors, while downstream traceability remains judgeable;
- the first section mixes source inventory with downstream FSD/ISD publication
  decisions, such as treating `NEW` or `EXTEND` as a source inventory note instead
  of recording that decision in `Delivery Scope & Targets`;
- FSD/ISD target row is missing but requirements and scope remain traceable;
- under `Scope: integration` ONLY, an ISD target's name or working-copy path is
  feature-owned instead of the agnostic service-owned change (§4.9) while
  requirements and scope remain traceable — a functional change's FSD path under
  its own change folder is correct and is never this warning;
- wording is imprecise but not misleading, such as `out of scope (later phase)`
  for a source item that clearly remains deferred;
- a signed row is PRESENT (id / text / citation intact) but conservatively
  mis-dispositioned `out of scope` / `deferred` when GOLDEN / `CONTEXT` show THIS
  change closes it (an under-claim) — lower `disposition_correctness` (§4.3, §7.1).

Warnings reduce dimension scores. Warnings do not force `pass = false` by
themselves.

### 7.3 Route Open Question Issues

For Open Questions, always route by the first Discovery Gate checkbox:

Only rows in the `Open Questions` table count. CR candidates do not count as Open
Questions.

| First gate token | Any Open Question `CRITICAL` + `Open`? | Correct severity |
|---|---|---|
| `- [x]` | yes | route to `critical_findings` |
| `- [x]` | no | no gate issue, even when `NICE-TO-HAVE` rows remain `Open` |
| `- [ ]` | yes | route tagging/quality issue to `warnings` |
| `- [ ]` | no | route conservative gate issue to `warnings` at most |
| missing | any | structure warning or critical only if gate cannot be judged |

`- [ ]` means the candidate is holding the pipeline. Treat that as a safe
management state, then score whether the hold is useful or overly conservative.
If you put a `NICE-TO-HAVE + Open` gate issue in `critical_findings`, your
verdict is invalid; move it to `warnings` only if some other wording makes the
artifact noisy or inconsistent.

## 8. Rubric Dimensions

Score every dimension numerically from `0.0` to `1.0`. Do not use `null`, `n/a`,
strings, or omitted dimensions when `CONTEXT` is present.

- `source_fidelity`: Executive Summary is grounded in `CONTEXT`, Discovery Source
  Inventory cites the primary inputs and relevant non-document evidence from
  `CONTEXT`, and requirements are copied or faithfully represented with real RQ
  IDs, WBS/source grouping, and citations; nothing material is dropped.
- `no_fabrication`: no invented codes, requirements, citations, capabilities,
  behaviours, integrations, or unsupported decisions. A `Reference / Anchor` that
  points at a source neither `CONTEXT` nor GOLDEN EXPECTED supports (an invented
  file, path, ticket, spec, or URL — including an inbox path fabricated for a
  source exposed only as a bare identifier) is a `no_fabrication` defect. Do not
  penalize an anchor whose concrete inbox filename matches GOLDEN EXPECTED just
  because that filename is not spelled out verbatim in the source_type map (see
  §4.2 grounding check).
- `capability_decomposition`: all touched capabilities are listed with correct
  ADD/MODIFY/REMOVE dispositions against baseline specs. For `ADD` capabilities, an
  absent or placeholder spec path is not a defect (see §4.4); do not deduct for it.
- `disposition_correctness`: requirement scope dispositions match sources under
  the OWNER-BASED rule (§4.3) — a requirement THIS change closes is `in scope`
  (whether it BUILDS the work or CONSUMES / CONFIGURES the selected vendor that
  fulfils it), and one whose deliverable a SEPARATE change / WBS / team owns is
  `out of scope`; deferred and CR-candidate rows follow the source. Marking a
  consumed/configured in-scope requirement `out of scope` merely because it uses a
  vendor or shares a WBS/vendor prefix, or marking another change's owned
  requirement `in scope`, is the core defect here. Delivery mode is a separate
  FACT that never changes disposition: a wrong `build` / `consume` / `configure`
  value is at most a `warnings` nuance and does NOT lower this dimension when the
  in/out disposition itself is correct.
- `cr_handling`: non-BRD scope on THIS change's own surface is surfaced as CR
  candidates and not absorbed; the CR table is not polluted with ordinary HOW
  decisions, and an ask about a different surface (another feature, or an
  integration this change only consumes) is not listed as this change's CR. The
  Dependencies section correctly lists external capabilities the change consumes
  but does not own (with `None.` for an agnostic leaf integration), a consumed
  dependency is not miscast as an Affected Capability, and a scope-addition is not
  miscast as a Dependency.
- `cross_cutting_completeness`: required cross-cutting areas are consciously
  evaluated and grounded.
- `open_questions_quality`: real ambiguities are captured, severity is calibrated,
  status is clear, and the gate is interpreted correctly.
  **Gate scoring rule:** `- [x]` with all questions `NICE-TO-HAVE` + `Open` is
  the correct and expected BA happy path — the checkbox means "No CRITICAL
  questions remain Open", which is satisfied when all questions are NICE-TO-HAVE.
  This combination must NOT lower this dimension score; score it at `1.0` when
  ambiguities are well-captured and severity is correctly calibrated. A score of
  `0.0` here is only justified when real ambiguities are absent, severity is
  wrong in a material way, or the gate claim is invalid (i.e. `- [x]` while a
  CRITICAL question is Open). "Wrong in a material way" means the UNDER-claiming
  direction — a real WHAT-ambiguity mis-tagged `NICE-TO-HAVE` — not the safe
  over-tag direction. Out-of-scope or integration-HOW NICE-TO-HAVE rows that
  remain `Open`, and a NICE-TO-HAVE / HOW refinement over-tagged `CRITICAL` (which
  correctly holds the first gate `- [ ]`), are at most a scope/severity-hygiene
  `warnings` deduction floored at `0.75`; they never justify `0.0`, never make the
  gate claim invalid, and do not compound below `0.75` when they are the only
  Open-Questions faults.
- `publication_target_and_structure`: Delivery Scope & Targets declares a scope
  consistent with `CONTEXT`, and the FSD/ISD targets match that scope (required
  produced document type present and disposed; excluded types omitted) and are
  usable for downstream documentation. Section presence/order/columns is
  gate-enforced against the golden, not scored here.
- `handoff_fidelity`: Document Consumption Guide carries all seven fixed per-section
  rows with empty sets written `None`, its `Key data` is identifiers and sets rather
  than prose, its `What it settles & why` phrases are grounded in the sections they
  name, and every row's key data agrees with the section it projects (§4.10) — the
  `Requirements Trace` WBS sets against the trace dispositions, capability directions
  against Affected Capabilities, dependencies, `Open Questions` (questions still
  `Open` only), change requests, applicable cross-cutting areas, and delivery
  scope + publication targets. The Executive Summary's counts and subjects agreeing
  with this guide counts here too. Score `1.0` when the projection is complete and
  reconciles; deduct per drifted or missing row. Drift is a `warnings` route (§7.2),
  never a source-truth critical — the section it drifted from still carries the
  truth.

Suggested scoring guide:

- `1.0`: complete and grounded.
- `0.75`: minor omissions or weak wording, no material scope risk.
- `0.5`: partial, materially incomplete, or confusing but still judgeable.
- `0.0`: absent, contradicted, fabricated, or impossible to judge.

Compute `score` as the mean of the nine dimension scores, then cap it at `0.5`
if `critical_findings` is non-empty.

Set `pass = true` only when `score >= threshold` and `critical_findings` is empty.
Warnings and missing_evidence reduce dimension scores, but they do not force
`pass = false` if the final `score` is still above threshold and
`critical_findings` is empty.

Before returning JSON, run this consistency check:

- If `critical_findings` is empty and `score >= threshold`, `pass` MUST be `true`.
- If `critical_findings` is non-empty, `score` MUST be `<= 0.5` and `pass` MUST be
  `false`.
- A verdict with `pass = false`, `score >= threshold`, and empty
  `critical_findings` is invalid. Fix the JSON before returning it.
- A verdict with any `critical_findings` and `score > 0.5` is invalid. Fix the
  JSON before returning it.

Pass truth table:

| Score vs threshold | Critical findings empty? | pass |
|---|---|---|
| `score >= threshold` | yes | `true` |
| `score >= threshold` | no | `false` |
| `score < threshold` | yes | `false` |
| `score < threshold` | no | `false` |

## 8.6 Split-Gate Mode (ambiguous delivery scope)

Use this section INSTEAD of §§3–8.5 and the §9 verdict shape when Mode Detection
(§1.0) selected split-gate mode. Here the candidate should NOT be a discovery
artifact: it should be a short stop-and-ask that recognises the ask is a bundled
functional + integration request with no resolved delivery scope, and returns a
decomposition plus a request for the user to choose scope.

Judge the candidate on these dimensions (grounded in `CONTEXT`, with `GOLDEN
EXPECTED` as the shape/coverage reference):

- `stops_before_discovery` — the candidate holds before authoring: it does NOT
  emit a filled discovery artifact (no Requirements Trace, capability
  decomposition, Discovery Gate, etc.). Emitting a discovery artifact anyway is a
  critical finding and forces this dimension to `0.0`.
- `split_recommendation` — it explicitly recommends splitting the bundled ask and
  names WHY (a functional feature surface plus one or more distinct external
  integrations cannot share one delivery scope).
- `decomposition_completeness` — it enumerates the distinct deliverables implied
  by `CONTEXT`: the functional Store Locator change (FSD-bound) AND exactly one
  integration change PER DISTINCT external service (ISD-bound). `CONTEXT` here has
  three distinct third-party services — Cartora (maps + geocoding/proximity),
  Addressly (typed-address autocomplete), and SlotBook (appointment booking) — so a
  complete decomposition lists all three as SEPARATE ISDs. The commerce Stores API
  is an existing, already-in-use read service with no new data model in scope, so a
  correct decomposition treats it as a dependency of the functional change, NOT as a
  new integration change; proposing an ISD for it is a defect. The following each
  lower this dimension: omitting a distinct service (fewer than three ISDs);
  inventing a service not present in `CONTEXT`; promoting the already-integrated
  commerce Stores API to a new integration change; merging two or more distinct
  services into one combined integration change; or routing an ISD under a
  feature-owned path (e.g. a change nested under or named after the consumer
  feature, like `store-locator/isd/...`) instead of an agnostic service-capability
  path (e.g. `cartora-maps-integration/...`, `addressly-integration/...`,
  `slotbook-integration/...`).
- `user_choice_presented` — it asks the user to choose/confirm which single scope
  to proceed with (e.g. proceed as the functional change, as ONE specific
  integration, or confirm the full split) rather than silently picking one.
  Offering a bundled "one integration that owns multiple services together" option
  is a defect — a single change owns exactly one service — and lowers this dimension.
- `no_fabrication` — it does not invent requirements, capabilities, integrations,
  citations, or decisions beyond what `CONTEXT` supports. Fabrication is a
  critical finding.

Routing:

- `critical_findings`: emitting a full discovery artifact instead of stopping;
  recommending that authoring proceed under the unresolved scope without a split;
  or fabricating scope facts / integrations.
- `warnings`: an incomplete decomposition (omits a distinct change, merges two or
  more distinct services into one combined integration, or promotes the
  already-integrated commerce Stores API to a new integration change); offering a
  bundled "multiple services together" integration option; routing an ISD under a
  feature-owned path instead of an agnostic service-capability path; missing the
  explicit user-choice request; or weak/absent rationale for the split — each when
  the candidate still stops and does not fabricate.

Scoring: compute `score` as the mean of the five dimensions above, then cap it at
`0.5` if `critical_findings` is non-empty. Set `pass = true` only when `score >=
threshold` and `critical_findings` is empty. The same consistency check and pass
truth table from §8 apply.

Split-gate verdict shape:

```json
{
  "pass": true,
  "score": 1.0,
  "rubric_scores": {
    "stops_before_discovery": { "score": 1.0, "reason": "..." },
    "split_recommendation": { "score": 1.0, "reason": "..." },
    "decomposition_completeness": { "score": 1.0, "reason": "..." },
    "user_choice_presented": { "score": 1.0, "reason": "..." },
    "no_fabrication": { "score": 1.0, "reason": "..." }
  },
  "critical_findings": [
    { "issue": "...", "evidence": "..." }
  ],
  "warnings": [
    { "issue": "...", "evidence": "..." }
  ],
  "missing_evidence": [
    "..."
  ],
  "recommendation": "..."
}
```

Reasons and evidence must quote the candidate or name the missing `CONTEXT`
deliverable. Keep the JSON valid.

## 9. Output Contract

The OUTPUT PROCEDURE in the system message controls how you package your answer
(the `<reasoning>` routing pass and the fenced JSON verdict). This section defines
what the discovery routing pass must contain and the exact verdict shape.

Routing pass, for this artifact:

1. State the first Discovery Gate token you observed: `- [x]`, `- [ ]`, or
   missing.
2. For each candidate issue, write one line: name it, then route it using §1.1
   and §7.3. A severity/gate/duplicate issue while the first gate is `- [ ]` is a
   safe hold — route it to `warnings`, never to `critical_findings`. Reserve
   `critical_findings` for source-truth changes, absorbed non-BRD scope,
   later-phase work marked confirmed in scope, or first gate `- [x]` with a
   `CRITICAL` + `Open` question. Never create a critical gate finding from
   `NICE-TO-HAVE` questions that remain `Open`, even when those questions are
   out-of-scope, wrong-flow, or integration-HOW mechanics that do not belong in
   this discovery — that is a `warnings` scope-hygiene issue, not a gate critical.
3. Confirm the pass/score rule from §8: `critical_findings` non-empty forces
   `score <= 0.5` and `pass = false`; empty plus `score >= threshold` is `pass =
   true`.

Verdict shape:

```json
{
  "pass": true,
  "score": 1.0,
  "rubric_scores": {
    "source_fidelity": { "score": 1.0, "reason": "..." },
    "no_fabrication": { "score": 1.0, "reason": "..." },
    "capability_decomposition": { "score": 1.0, "reason": "..." },
    "disposition_correctness": { "score": 1.0, "reason": "..." },
    "cr_handling": { "score": 1.0, "reason": "..." },
    "cross_cutting_completeness": { "score": 1.0, "reason": "..." },
    "open_questions_quality": { "score": 1.0, "reason": "..." },
    "publication_target_and_structure": { "score": 1.0, "reason": "..." },
    "handoff_fidelity": { "score": 1.0, "reason": "..." }
  },
  "critical_findings": [
    { "issue": "...", "evidence": "..." }
  ],
  "warnings": [
    { "issue": "...", "evidence": "..." }
  ],
  "missing_evidence": [
    "..."
  ],
  "recommendation": "..."
}
```

Reasons and evidence must quote the candidate or name the missing source
requirement/capability. Keep the JSON valid.

</judge_contract>
