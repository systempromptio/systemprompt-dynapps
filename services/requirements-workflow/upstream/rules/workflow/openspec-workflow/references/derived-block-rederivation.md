# Derived-Block Re-derivation (discovery / design / proposal)

Read this from the [Artifact Review & Approval Gate](mdc:.cursor/rules/workflow/openspec-workflow/references/artifact-review-gate.md)
(step 3 / step 4) when you are about to re-present a `discovery`, `design`, or `proposal` artifact:
before it goes back to a human, rebuild its derived blocks from the CHANGED body. Grooming, ISD, and the
other artifacts are routed elsewhere by the gate — this reference covers only the three artifacts that
carry these derived blocks.

## DECISION KERNEL

Some blocks are not authored, they are **projections of the artifact's own body** — a reader-map summary,
a per-section consumption guide, and (on an integration design) a local reader-map opening the Integration
Specification. Any edit to the body can silently invalidate them, so before the artifact goes back to a
human you rebuild them from the CHANGED body.

**This is a rebuild, not an edit.** The previous pass's text has NO authority: a claim it makes that the
body no longer supports is wrong, and rewording that claim instead of correcting it leaves it just as
wrong — the test is agreement with the body, not distance from the old text. Equally, do not invent a
change the body does not carry: a row the session never touched keeps its value. Do the work below from
the body's tables; do not read the old block to "update" it.

### The derived block kinds

- **Executive Summary / Solution Overview — a reader map.** Orients a reader before the detailed
  sections: the problem/change context and business value in one short paragraph, then WHAT THIS FILE
  SETTLES (delivery scope, the shape of the requirement set, which capabilities move, which section
  holds each), WHAT STAYS OPEN (the subjects still undecided and the register each sits in), and
  READINESS stated consistently with the artifact's gate. Name **subjects, never identifiers** ("the
  redirect for the retired `/stores` URL", not "Q2"); counts are fine ("three refinements", "nine in
  scope"), re-listing a table's rows is not. Identifiers and sets live in the Document Consumption Guide
  — the two split the same facts along the prose/identifier line and never overlap.
- **Document Consumption Guide — a per-section interface table.** The map a downstream reader (an
  LLM first, a human second) uses to consume the file without re-reading the whole body. It has ONE row
  per consumed section, and each row answers WHAT that section settles and WHY it matters downstream (a
  short human-readable phrase) plus the KEY DATA that section exports (the identifiers and sets a
  consumer joins on). It is a PROJECTION, never a new source of truth: every value must already be
  readable in the section the row names. An empty set is written `None`; a row is never merged, renamed,
  or reordered. Key data carries identifiers and sets only (WBS codes, capability names with direction,
  `Q#`/`CR#`/`D#`/`PD#` with status, paths) — no restated requirement titles.
- **Integration Specification `### Overview` — a local reader-map (integration design only).** An
  integration design's `## Integration Specification` opens with a short `### Overview` that orients a
  reader WITHIN that one section: 1-2 sentences on the service + capability and the contract SHAPE, then
  a single `Read in order:` line signposting the subsections present, in document order. Like the two
  blocks above it is a PROJECTION of its section's body — its claimed shape and its signposted
  subsections must match the subsections the body ACTUALLY carries (an SDK-only contract with no
  Data-Mapping table is not signposted as carrying one), and it neither restates the Solution Overview
  nor re-lists Document Consumption Guide rows. A FUNCTIONAL design renders the section
  `N/A — functional change` and has no Overview to re-derive. Rebuild it from the section's CURRENT
  subsections, never from the stale text.

### Set-integrity laws that hold for every block

Each law is a TASK about a section's exported set — answer it by WALKING the section as it stands
NOW, never by recalling a plausible-looking cell.

- **Every consumed section earns its row — the terminal ones included.** The guide's row set is the
  body's `##` sections in document order, minus only the ones the block excludes. A section that sits
  LAST and states its value in a marker or a single sentence (Review & Approval, Approach) is a row
  like any other: dropping it because it is short, terminal, or "already known" is a row-omission
  defect, and the easiest miss to make. **The gate is NOT the last section.** Every artifact ends
  `… <its Gate> (excluded), ## Review & Approval (kept)` — the excluded gate is the second-to-last
  section and Review & Approval trails it, so ask "what sits AFTER the gate?" and keep that row:
  excluding the gate must never carry away the Review & Approval row behind it. (This does not license
  inventing a row: a `##` heading you cannot point to in the body — or the gate itself, which stays
  excluded even though the Review & Approval that follows it is kept — still gets none.)
- **A cell that packs members from more than one place carries them ALL, each with its qualifier —
  and NOTHING its section does not export.** Walk EVERY source the cell draws from and record every
  member together with the qualifier it carries — a `(proposed, links Q#)` on a decision, an
  `(unconfirmed, Q#)` on a dependency, a `→ <owner>` on an out-of-scope code, the slice-blocking tag
  on a dependency, a request/response field as well as its endpoint, an in-scope set as well as its
  out-of-scope set. A cell whose section exports TWO sets (endpoints AND data-mapping fields; in-scope
  AND out-of-scope WBS) is half-walked, not done, until BOTH sets are in it. Dropping a member — OR
  stripping a member's qualifier down to the bare id — is the same set-omission defect as dropping a
  row. Conversely, each cell carries ONLY the set ITS OWN section publishes: an owner, or a prose
  boundary item, is a member of Out of Scope & Boundaries, not of the Requirements Lock WBS set —
  importing another section's members into a cell is fabrication, not a fuller walk.
- **A single-value marker/prose cell proves its one value against the section as it stands NOW.** The
  Approach section names one FSD/ISD working-copy target in a sentence — that target is the working-copy
  PATH the section states (`openspec/changes/<change>/fsd|isd/<file>.md`), NOT a topical mention of
  another artifact the same prose also names (a design section, a spec, or an upstream doc it defers
  detail to); copy the path, not the topic. The Review &
  Approval section shows one approval state in its marker — REPORT that current state verbatim (e.g.
  `Approved: no; approver: —`). You are PROJECTING the marker, never setting it: an un-approved marker
  is reported un-approved and is never upgraded to approved, and `None`, `Pending`, or an invented
  state is a DROP. `None` for such a row is a defect — the section always states a value.

## RE-DERIVATION PROCEDURE

Do this as written work BEFORE you draft each block: the blocks are a copy of its result, not a fresh
pass over the old text. Skipping the walk is how a buried row survives stale.

1. **Walk the body and write the membership out.** Go through each driving table row by row and record,
   for every set the blocks project, its FULL member list as the body stands NOW — never from the stale
   block and never from memory of what the session changed. This is the step that stops a buried row from
   being dropped or left stale: you enumerate the whole set, not the items the session happened to name.
   The per-artifact walk detail — discovery's Requirements Trace bucketing, and the document-following
   walk plus the key-data map for design and proposal — lives in **PER-ARTIFACT RULES** below; apply the
   section for the artifact you are re-presenting.
2. **Diff the walk against the stale block.** For each value the old block asserts, mark KEEP (a body row
   still supports it) or FIX (the body moved it), and add any body fact the old block omits. A "KEEP" no
   body row supports is not a KEEP — it is a FIX. Every FIX and ADD must appear in the rebuilt blocks;
   this is what forces a disposition change or an owner correction buried in one trace row to surface.
3. **Emit the Document Consumption Guide from the walk.** The guide's ROWS are exactly the sections you
   walked — for a fixed-interface guide (discovery) the seven named rows; for a document-following guide
   (design, proposal) one row per current `##` section in document order, minus the excluded ones. Write
   NO row whose exact `##` heading you cannot point to in the body: a row for a section the body does not
   carry (a "Legacy Behaviour Overview" or any heading absent from the body) is a fabrication, not a
   projection — delete it. Fill each row's `Key data` cell as a VERBATIM COPY of that row's full set/value
   from step 1 — identifiers and sets only, never copied from the old table — then check the cell against
   the section it names: every member the section exports appears in the cell (BOTH halves of a two-part
   cell, EVERY mapping field of an Integration Specification cell), and a prose/marker section's single
   value is present. `None` is only ever a genuinely empty set — never a prose/marker cell whose section
   states a value, and never a half-walked set. Write the `What it settles & why` cell as a short
   body-grounded phrase. A row the body did not move keeps its value; changing it is fabrication, not
   refreshing.
4. **Emit the reader map, each claim body-checked.** Write it by subject and count; for EVERY sentence
   confirm a body row supports it and drop any that none does; reflect every FIX from step 2; never
   roll-call identifiers (those live in the Document Consumption Guide). Any count the reader map states
   (in-scope, deferred, out-of-scope, open) MUST equal the size of that same set from the step-1 walk —
   the summary and the guide report one set of numbers, so an out-of-scope count derived any other way
   (only the rows this session moved, only dependency rows, dropping the source-fidelity-only rows)
   contradicts the body and is a defect, not a rounding.
5. **Emit the Integration Specification `### Overview` (integration design only).** For an integration
   design, walk that section's CURRENT `###` subsections and rebuild its `### Overview`: 1-2 sentences on
   the service + capability and the contract shape, then a `Read in order:` line naming the subsections
   present in document order. The `Read in order:` line signposts ONLY subsections you can point to under
   `## Integration Specification`, each by its exact `###` heading; the `### Overview` is the map, not one
   of the mapped subsections, so it never lists itself, and a subsection the body does not carry is a
   fabrication, not a signpost. Every claim must be readable in the subsections it maps; do not restate
   the Solution Overview, re-list Document Consumption Guide rows, or dump contract detail. A functional
   design has no Overview (the section reads `N/A — functional change`) — skip this step.
6. **Re-set the gate boxes** from the refreshed blocks (e.g. discovery's Discovery Gate: no CRITICAL
   question Open, the guide matches the sections it projects).

## PER-ARTIFACT RULES

For a document-following guide (design, proposal) the walk FIRST re-derives the row set from the body's
current `##` sections, THEN fills each row's key data — the row set is never carried from the stale
block.

### discovery

- **Blocks to re-derive:** `## Executive Summary` (reader map) and `## Document Consumption Guide`, placed
  right after the Executive Summary; then re-set the Discovery Gate boxes.
- **Guide membership — a FIXED INTERFACE:** the same seven rows, one per consumed section, in order, every
  time, never dropped.
- **Walk rules (step 1 for discovery).** For discovery's Requirements Trace, assign EVERY row to exactly
  one bucket by its `Disposition` cell, then LIFT each member VERBATIM from that row's own cells — never
  derive, translate, or re-bucket it: the member code is copied from the row's WBS column, and for an
  out-of-scope row the owner is copied from that SAME row's Affected-capability cell. So `In-scope
  requirements` = the WBS-column value of every `in scope` row; `Deferred requirements` = the WBS-column
  value of every `deferred` row or `None`; `Out-of-scope` = every `out of scope` row as `<its WBS-column
  value> → <its Affected-capability-cell value>` (the WHOLE out-of-scope set, larger than the codes this
  session moved).
  **A member is only ever a value that literally appears in some row's WBS column.** The
  Affected-capability / workstream name is an OWNER, never a member — it may appear only to the RIGHT of
  an out-of-scope `→`, never as an in-scope or deferred member and never to the left of the `→`.
  In-scope and deferred members are bare WBS codes; out-of-scope members are `<WBS> → <owner>`, never
  `<owner> → <owner>` and never the RQ id, each out-of-scope row emitted once (no duplicate owner rows).
  **Disposition and dependency are different axes — decide out-of-scope membership from the `Disposition`
  cell ALONE.** A row stays in the out-of-scope set even when its `Notes` hedge it as "source fidelity
  only", "not a dependency", or "serves a different surface", and even when it is a homepage/other-consumer
  or vendor-family row this change never consumes: those remarks pick nothing off the set.
  Now walk the OTHER set rows the same way, and hold the rule that **`None` means the table has ZERO
  rows — never that its rows were decided or hedged away:**
  - Change Request Candidates: carry every `CR#` with its current status. A CR the review DECLINED (or
    accepted, or deferred) is still a member — write `CR# — <status>`, NOT `None`. `None` belongs only
    to a genuinely empty Change Request Candidates table.
  - Dependencies: carry every row of the Dependencies table. A consumed CMS / platform / content feature
    is a member exactly as much as a service or credential is; dropping one is the same set-omission
    defect as dropping an out-of-scope WBS.
  - Open Questions: carry every `Q#` still Open with its severity; a question the session RESOLVED leaves
    the set (it is not written as Open), and the count of still-open questions must match the body.
  Then finish the walk for the remaining set rows (cross-cutting, capability plan, targets).

### design

- **Blocks to re-derive:** `## Solution Overview` (reader map) and `## Document Consumption Guide`, placed
  right after the Solution Overview; for an integration design also the Integration Specification
  `### Overview` (local reader-map projecting that section — `N/A` for a functional design); then re-set
  the Design Gate boxes.
- **Guide membership — FOLLOWS THE DOCUMENT:** one row per `##` section the body carries, in document
  order, EXCLUDING only the Solution Overview, the guide itself, and the Design Gate. A section a real
  review ADDS earns a NEW row in place; a section the body does not carry has NO row and its absence is
  correct. A row's key data still moves when the review changes items inside its section (e.g. the
  Planning Deferrals `PD#` set filling from `None`).
- **Key-data map — each row's key data is fixed by the section it maps — fill the FULL exported set,
  never a type-token (`all`, `credentials`) and never `None` where the section has rows:** Affected
  Capabilities → the owned capabilities with their `ADD` / `MODIFY` / `REMOVE` direction; Design
  Source Inventory → the input source set (the sources, never their content); Gap Analysis → the
  `WBS → gap-class` set (`none/config/adapt/rebuild/remove` — the distance, never a chosen approach);
  Architecture Decisions → the `D#` set (a proposed `D#` notes its linked `Q#` — ids, never a restated
  title); Base Configuration & Cross-Cutting → the cross-cutting concerns carried (with their
  `Applies to`); Integration Specification → the contract keys (endpoints + data-mapping fields for an
  API shape; the SDK interface for an SDK shape), or `N/A — functional change` (`None`) for a
  functional change; Risks / Mitigations → the risk labels; Dependencies → the consumed-not-owned set;
  Open Questions → the `Q#` set with owner/status; Planning Deferrals → the `PD#` set (`None` only when
  the design defers nothing); Review & Approval → the approval state (`Approved` yes/no + approver).

### proposal

- **Blocks to re-derive:** `## Executive Summary` (reader map) and `## Document Consumption Guide`, placed
  right after the Executive Summary; then re-set the Proposal Gate boxes.
- **Guide membership — FOLLOWS THE DOCUMENT the same way:** one row per `##` section, in document order,
  EXCLUDING only the Executive Summary, the guide itself, and the Proposal Gate. The offer's section
  set is fixed, so a review typically MOVES a row's key data (a new Out of Scope & Boundaries owner,
  or the Proposal Source Inventory gaining the review-gate session) rather than adding a row.
- **Key-data map — each row's key data is likewise fixed by the section it maps — determine the FULL
  set/value the section publishes and record it; never a type-token and never `None` where the section
  carries a value or set:** Tracker → the tracker id; Proposal Source Inventory → the input source set
  (a review-gate session becomes a source row when one ran); Requirements Lock → the in-scope WBS set
  and the out-of-scope WBS set (dispositions, never the RQ id); Change Request Dispositions → the `CR#`
  set with proposed disposition/status (`None` for a genuinely empty table); Re-usability Legend → the
  marker set the legend defines (the count + the markers); Delivery Breakdown → the slice set `S1…Sn`
  (one per in-scope WBS); Out of Scope & Boundaries → the set of items no slice delivers, each with its
  owner; External Dependencies & Access → the dependency set, each with the slice it blocks; Approach →
  the FSD/ISD working-copy target the section names; Open Questions → the proposal-stage `Q#` set
  (`None` when the offer raised none — discovery/design `Q#` stay registered upstream); Review &
  Approval → the approval state. Not every section publishes its key data as a table: when a section
  carries its exported value in PROSE (the Approach section names the FSD/ISD working-copy target in a
  sentence, even though the per-slice `FSD:`/`ISD:` lines repeat the path) or in a GLOSSARY (the
  Re-usability Legend defines its markers as a list), read that value from the section and record it as
  the identifier/target/count it exports — `None` is reserved for a section that is genuinely empty,
  never for one whose value is merely narrated or listed.

## SELF-CHECK BEFORE PRESENTING

A summary that still describes the pre-edit file, or a Document Consumption Guide whose key data no
longer matches the sections it projects, is a defect the human is being asked to approve. Present the
artifact to the reviewer only once its derived blocks agree with its body — confirm each:

- [ ] Every set was walked from the body as it stands NOW (not the stale block, not memory of the
  session's changes), and every FIX and ADD from the diff appears in the rebuilt blocks.
- [ ] `None` appears only for a genuinely empty set — never on a table that has rows, and never on a
  prose/marker section that states a value.
- [ ] Every count the reader map states equals the size of that same set from the walk.
- [ ] The Document Consumption Guide has no row whose exact `##` heading you cannot point to in the body,
  and drops none the body carries — including the terminal marker/prose rows (Review & Approval,
  Approach), which are the easiest to drop.
- [ ] Every two-part cell carries BOTH halves, every mapping field of an Integration Specification cell
  is present, and every member keeps its qualifier (`(proposed)`, `(unconfirmed, Q#)`, `→ <owner>`) — no
  member stripped to a bare id.
- [ ] Review & Approval reports the marker's CURRENT state (e.g. `Approved: no`), never a set, upgraded,
  or invented (`Pending`) one.
- [ ] The Integration Specification `### Overview` (integration design) signposts only `###` subsections
  the body carries and never lists itself.
- [ ] The reader map names subjects, never identifiers (identifiers and sets live only in the Document
  Consumption Guide).
- [ ] The gate boxes are re-set from the refreshed blocks.
