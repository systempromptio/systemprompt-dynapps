# Fsd contract

Pinned upstream: 2209c7ecd294fe728141d6e4fe55c74d5cacc968

PURPOSE:
The FSD is the client-facing projection of the approved offer — the
document the client reviews and formally approves. It lives as a
working copy at `openspec/changes/<name>/fsd/<feature>.md`;
`/opsx:propose-submit` later publishes it to Confluence for client
sign-off. It is the ONLY document of this change the client reads:
internal machinery (unresolved internal decisions, artifact paths,
agent process, consumed-integration mechanics) stays out. The FSD adds
no behaviour and decides nothing — the behaviour canon stays in specs.
But once the client signs it, the FSD is the PRIMARY functional source
of truth for the downstream developer phase (technical-design, tasks,
build): it is the signed WHAT they read first, ranked above design and
grooming for functional intent. specs stays the structured executable
projection of that same WHAT and the only artifact archived into the
timeless baseline — the two never conflict: the FSD is the signed client
contract, specs its machine-checkable behaviour canon.

AUTHORITY ORDER:
1. The APPROVED grooming decisions and the reconciled upstream
   artifacts — discovery.md (Requirements Trace: verbatim text and
   RQ/WBS codes; Delivery Scope & Targets), proposal.md (Requirements
   Lock and Delivery Breakdown), this change's delta specs (the
   behaviour), design.md (only where it shapes client-visible
   behaviour) — are the factual source of truth. Consume each through
   those NAMED sections — the upstream's machine-readable interface —
   not by mining other prose.
2. This `fsd` instruction is the artifact contract: follow it for
   coverage, client hygiene, and the quality bar.
3. The `fsd.md` template defines the section order, tables, heading
   levels, and placeholders.
4. Publishing and status transitions are owned here: a fresh FSD is
   `draft`; `/opsx:propose-submit` sets `astound review` at submit
   (delivering-org review), then `<client> review` when the doc goes to
   the client, and `approved` is set manually in Confluence after client
   sign-off — tooling never sets `approved`. The active working copy is
   the only source for publish metadata, approval, and status; never
   read `openspec/changes/archive/**` for current lookup. Project
   `config.rules.fsd` provides supporting context. Neither overrides
   this contract. Criterion phrasing is not this artifact's concern: the
   User Story / Acceptance Criteria blocks are projected from the
   proposal verbatim.

DECISION KERNEL:
- SCAFFOLD-FIRST (read-only projection): the per-requirement
  `#### User Story`, `#### Acceptance Criteria`, and `#### Key behaviours`
  sub-sections are filled DETERMINISTICALLY by the scaffold script
  (`generate-functional-artifacts.mjs`) — Story + Acceptance Criteria projected VERBATIM
  from the covering proposal slice, Key behaviours projected as a TABLE
  from the covering spec requirement(s)' full scenario set. RUN THE
  SCAFFOLD SCRIPT FIRST; then enrich ONLY the non-owned sections (General
  FSD Information, approval roster, Reference Materials, per-block
  Visualization, framing). EXTEND (a published doc fetched into the working
  copy) runs the SAME script WITH `--extend`: it updates/adds ONLY this
  change's WBS blocks and PRESERVES every prior in-scope block verbatim —
  WITHOUT `--extend` the reconcile treats this change's proposal as the whole
  in-scope set and REMOVES the prior change's blocks, silently stripping
  published scope. stories.md is always a full re-render from THIS change's
  slices (per-change; correct on both NEW and EXTEND). NEVER hand-author or edit the three
  script-owned sub-sections — to change a Story or a criterion edit the
  covering proposal slice, to change a Key behaviour edit the covering
  spec requirement, then re-run the script. That includes the table's bold
  sub-header rows: each one is a covering requirement's NAME, so a label
  changes only by renaming that spec requirement. The script also VALIDATES
  the join first; fix any reported error UPSTREAM (proposal / specs), never
  in the FSD.
- The FSD projects; it never decides. Scope is the lock, slicing and
  user stories were approved in the Delivery Breakdown, behaviour is
  canon in specs, decisions are grooming law. If a section cannot be
  filled from those without inventing, STOP and push the gap back to
  grooming — never fill it in a client document.
- Scope law: the FSD exists because the delivery scope resolved
  `functional` (discovery's Delivery Scope & Targets carries the FSD
  row). An `integration` change publishes an ISD instead — one change
  never produces both document types.
- A client document hides internal machinery: no unresolved internal
  decision, no internal paths or artifact names, no agent commentary,
  no code tokens, and no vendor/service names of consumed dependencies —
  behaviour is stated from the shopper's seat. RQ/WBS/CR codes are
  the exception: they are the client-shared trace vocabulary from
  the BRD and appear throughout.
- Approved decisions are law: encode each approved bound exactly
  ("regular weekly opening hours … not displayed in v1", "permanently
  redirected"), record each CR under its DECIDED status, and never
  resurrect a declined or rejected option.
- The working copy is fresh: FSD Status `draft`, every approval
  row `not started`, Open Questions EMPTY (everything pre-build was
  decided at grooming), no Confluence URL or Page ID. Publishing and
  status progression belong to `/opsx:propose-submit`, never here.

PIPELINE (follow each referenced section; do not improvise content):
1. Clear the ENTRY GATE (grooming approval + functional scope +
   client-facing surface).
2. For each FSD row in discovery's Delivery Scope & Targets, start
   the working copy per its disposition (NEW vs EXTEND — see FILE
   RULES).
3. Run the scaffold script (`generate-functional-artifacts.mjs`) to project the In Scope
   functional blocks — one per in-scope RQ, keyed by its clean WBS code,
   with the read-only User Story / Acceptance Criteria / Key behaviours
   filled (FUNCTIONAL BLOCK RULES). Fix any validation error UPSTREAM
   (proposal / specs) and re-run; never hand-fill those sub-sections.
   HOW TO RUN — (a) ONE-TIME, install the tooling deps: `npm install` in the
   deployed `.cursor/` directory (the script needs `nunjucks`); (b) invoke it
   with the change directory as its argument, e.g.
   `node .cursor/skills/plan/openspec-workflow/scripts/generate-functional-artifacts.mjs <path-to>/openspec/changes/<change> --dry`,
   inspect the `--dry` output, then re-run without `--dry` to write. For an
   EXTEND row (the disposition in discovery's Delivery Scope & Targets), add
   `--extend` so the script preserves the fetched doc's prior in-scope blocks
   and only updates/adds this change's WBS blocks; omit it only for a NEW doc.
   The script auto-locates the OpenSpec project root (the folder containing
   `openspec/`) to resolve the template via the `openspec` CLI, so cwd does
   not matter; pass `--template <path>` only as a dev/offline fallback.
4. Enrich the non-owned sections: the General FSD Information card,
   approval groups, and Reference Materials (GLOBAL RULES), and each
   block's Visualization (LLM-owned) — leaving the three script-owned
   sub-sections untouched.
5. Fill Deferred Requirements and Change Requests from the lock and
   the grooming decisions (OUT OF SCOPE RULES).
6. Run the SELF-CHECK before emitting.

ENTRY GATE (blocking — evaluate BEFORE authoring anything):
- discovery.md, design.md, proposal.md, grooming.md, and this
  change's delta specs must exist as completed upstream artifacts,
  and grooming's Grooming Approval Gate must have PASSED (every item
  APPROVED or DEFERRED-APPROVED by a named human, every gate box
  checked). If the gate has not passed, STOP — a client document must
  not be generated from unapproved decisions, open questions, missing
  prerequisites, or implied CR dispositions.
- Scope check: discovery's Delivery Scope & Targets must carry FSD
  row(s) (functional delivery scope). If the scope is integration,
  STOP — that change publishes an ISD, not an FSD.
- CONDITIONAL: produce an FSD only when the change has a
  client-facing surface a client or end user would review. For pure
  refactors, internal tech, or infra-only changes, skip the FSD
  entirely — do not create fsd/ files. Nothing downstream requires
  it, so skipping never blocks stories/tasks/apply.

FILE RULES:
- One working-copy file per FSD row in Delivery Scope & Targets
  (a cross-cutting change may produce several files, e.g. PDP + PLP +
  cart). Name each with a lowercase kebab feature code:
  `openspec/changes/<name>/fsd/<feature>.md`.
- Disposition per row: NEW -> create the working copy fresh from this
  change's approved artifacts. EXTEND -> fetch the live Confluence
  page into the working copy FIRST (no suffix), then update/add ONLY
  the section(s) this change touches, preserving every other section
  verbatim; never use archived OpenSpec changes as the baseline. On
  EXTEND, run the scaffold script WITH `--extend` (additive): it
  updates/adds ONLY this change's WBS blocks and PRESERVES every prior
  in-scope block verbatim. Running it WITHOUT `--extend` over a fetched
  published doc REMOVES the prior change's In Scope blocks (the reconcile
  treats this change's proposal as the whole in-scope set).
- If discovery carries no Delivery Scope & Targets decision, resolve
  it now with project-context `source_type: ["confluence"]` for each
  affected area (see the `project-context` skill): extend a match,
  else start new — ask the user when ambiguous.

GLOBAL RULES:
- The header card H2 is "## General FSD Information" (type-specific — NOT
  a generic "Document Card"): the explicit label keeps the reader/reviewer
  anchored in the functional-specification scope. It is a headerless
  2-column card (keep the empty header row; no "Field | Value"):
  WBS-Feature Name, Project Name, Package/Set/Batch, Author/Owner,
  FSD Status, Target Approval Date. FSD Status =
  `draft` on a fresh working copy. Package/Set/Batch is OPTIONAL: fill it
  VERBATIM only when a source states a package / set / batch number for
  this scope; otherwise leave the cell blank — NEVER invent or infer a
  number. Target Approval Date is the client's target sign-off date in
  `dd/mm/yyyy`; leave the `dd/mm/yyyy` placeholder on a fresh working copy
  (the real date is prompted and filled by `/opsx:propose-submit` at
  publish) — NEVER invent a date.
- Approval groups are H3 sub-sections under the General FSD Information
  card, one per approving org (the delivering org FIRST, then the
  client). The delivering-org group is ALWAYS titled "Astound Approval" —
  this header is STATIC; Astound is always the delivering agency, so never
  take it from CONTEXT and never substitute another agency name (e.g. a
  client's internal team). Title the client group "<Client Org> Approval"
  using the client ORG NAME from CONTEXT — do NOT invent a generic label
  like "Client Approval" when CONTEXT names the client org. Each is a
  headerless 3-column roster: role | status | name — the ROLE leads because
  column 1 becomes the wiki's Content Properties key that the published
  approval matrix aggregates on, so it stays a short stable token and the
  person moves to column 3. The Astound roster carries the DEFAULT FSD
  roles SA, FE, BE, QA (one row each) — seed the four rows with `TBC` names
  and substitute real names from CONTEXT when present (a real name for the
  matching role, `TBC` for the rest); never invent a person's NAME.
- ORG FIREWALL (commit BEFORE seeding any roster or Author, like the
  closed-membership ROWS/OUT): enumerate every CONTEXT-named person and
  commit one plan line `<name> -> <org = delivering (Astound) | client |
  other> -> <role>`, then seat STRICTLY by org. An Astound roster row is
  filled ONLY by a DELIVERING-ORG (Astound) person whose role matches
  SA/FE/BE/QA; a CLIENT (or any non-Astound) person NEVER fills an Astound
  row EVEN when their title sounds like a delivery role — a client "FE
  lead", "platform dev", or "architect" is the CLIENT's own staff, not
  Astound's engineer, so that Astound row stays `TBC`. Client-org people
  seat ONLY in the client group. Seating a client / non-Astound person in
  the Astound roster is a mis-attribution, not enrichment. The client
  group carries a single `TBC | not started | TBC` row unless CONTEXT
  names a CLIENT-ORG approver/role (e.g. the client PO). Every status is
  `not started` on a fresh copy. Status badges: `not started` |
  `in progress` | `approved`.
- Title and "WBS-Feature Name": the FEATURE-level WBS code and name
  the sources carry for this Delivery Scope & Targets row (e.g. the
  BRD feature block "CHK - Checkout Payments") — the document covers the
  row's ENTIRE scope, never a single requirement group. The H1 page
  title ALWAYS ends with " FSD" (e.g. "CHK - Checkout Payments FSD") so the
  document type is unmistakable in the Confluence page tree and to any
  reviewer; the "WBS-Feature Name" card value carries the same code+name
  WITHOUT the suffix.
- Author/Owner: the DELIVERING-ORG author(s) of THIS document — the BA who
  owns the functional side and, when CONTEXT names them as co-owner, the
  SA lead — VERBATIM with their role, and NOTHING else. It is NOT the
  approval roster (never dump the SA/FE/BE/QA rows into this cell) and
  NEVER a spreadsheet-/tracker-provenance name (a CSV `Created By` /
  `Updated By` / `Approved By` value is metadata about a row, not the
  document's author) nor a client-org person. Leave `TBC` (never blank,
  never invented) when CONTEXT names no document author. Never embellish a
  person's name.
- Project Name is the PROJECT/CLIENT identity the source packet names
  (e.g. "<Client> — <Programme> (<tracker key>)"), written in FULL — NOT
  the change slug (`add-<feature>`), NOT a single feature name, NOT the
  tech stack, and never abbreviated. If the sources name no project, ASK —
  do not assume or invent a project name.
- Reference Materials: determine which client-facing references THIS
  change cites — the BRD/SRD, existing published FSD/ISD pages, and any
  other client-facing reference the sources carry — and record each by
  its TITLE or client-facing URL. An internal repo path or
  `project-context` inbox/anchor (e.g. `.cursor/…`, `…/inbox/…`) or a
  working-copy link is internal plumbing, never a row — cite the
  client-facing title/URL the source carries instead.
- CHROME CELL FORMATTING: the card values and the Reference Materials
  Material/Notes cells carry ordinary inline markdown, exactly like the
  body. Write EVERY link as `[label](url)` — in the "Link / reference"
  column a bare URL is also allowed when the row needs no label (it
  publishes as an inline card). NEVER wrap a URL in backticks: that
  makes it dead monospace text instead of a link. Publishing turns this
  markdown into real Confluence links, code and mentions, and the
  reverse pull restores it, so the round-trip is lossless.
- SOURCE LINK RESOLUTION: before emitting a source document as a bare
  title, resolve it to the canonical client-facing link we already hold.
  Check, in order, the project link registry (the project's Key links),
  project-context (search the matching source_type — it returns the
  indexed file plus its origin), and the project wiki/Drive. Cite the
  resolved link wherever it lives (Confluence page, Google Doc/Sheet,
  wiki, or other canonical location), not a bare title. Leave a bare
  title ONLY when no link is on record, and say so in that row (e.g.
  Notes "no canonical link on record") so a reviewer sees it was checked.
  Never fabricate or guess a URL; if project-context holds only a local
  ingested copy with no origin, surface that rather than inventing a link.
- No summary/List-of-All-Requirements table and no legacy sections
  (Important Notes, Versions Log, Scope, Open Items Summary) and no
  Known-constraints section: a client FSD hides internal uncertainty —
  resolve it before generating, or record it as a Deferred Requirement
  or a Change Request.
- Open Questions: the section belongs to the CLIENT REVIEW cycle, not
  to authoring. Emit the template's review sentence and its bare table
  header with NO rows — reviewers add rows on the published page.
  Never write a row, and never carry an unresolved internal item into
  a generated FSD; every pre-build decision was settled at grooming.

FUNCTIONAL BLOCK RULES (the block SET is SCRIPT-OWNED; you ENRICH each block):
- The scaffold script creates exactly one "### <WBS> — <name>" block (H3 under
  the "## In Scope Functional Requirements" H2) per in-scope proposal slice, keyed
  by the slice's clean WBS code, and its VALIDATION enforces the coverage law: one
  in-scope RQ = one block, no consolidation / split / omission, no block for
  deferred / declined / out-of-scope work, and no in-scope WBS reused by a Deferred
  / Change Request block (reuse breaks the stories join). Do NOT hand-add, merge,
  split, rename, or re-key blocks, and do NOT hand-write the WBS-code heading — if
  the block set looks wrong, the fix is UPSTREAM (the proposal Delivery Breakdown or
  the spec WBS); re-run the script, which names the exact fix on a validation error.
  FEEDBACK ROUND-TRIP: on a client request to change / add / remove a requirement,
  edit the proposal slice and/or the spec, then re-run — the script RECONCILES the
  existing FSD in place (ADDs a block for a newly-covered WBS, REMOVEs a block whose
  WBS is no longer covered, UPDATEs a changed block's read-only bodies), PRESERVING
  each surviving block's Visualization and the rest of the document verbatim, and
  re-syncs stories.md. Never hand-edit the FSD to add/remove/change a requirement.
- The section OPENS with the SCRIPT-OWNED "**Re-usability Legend**" block — the
  proposal's `## Re-usability Legend` projected VERBATIM, once, before the first
  block, so the reader can decode the `#####` marker groups below it. It is a bold
  label, NOT a heading (a heading there would parse as a WBS-less block). The legend
  is authored ONLY in the proposal: never write, extend, trim, re-order, or re-word
  it here; to change it, edit the proposal's legend and re-run the script.
- Sub-sections are H4 (####); Acceptance-criteria groups and scenarios
  are H5 (#####). Lean and human-readable, not a test spec. Block order
  (User Story, Visualization, Acceptance Criteria, Key behaviours):
  * User Story (SCRIPT-OWNED, read-only): the scaffold script projects the
    covering slice's Story VERBATIM — the SAME sentence the approved
    Delivery Breakdown recorded (which is the discovery/SRD user story
    word-for-word). Do NOT hand-author, copy-edit, re-persona, paraphrase,
    or shorten it; to change it, edit the covering proposal slice and
    re-run the script.
  * Visualization (REQUIRED, LLM-owned — enrich this): a State | Desktop |
    Mobile table in EVERY requirement block — a functional block without
    it is incomplete, the same as one missing its Acceptance Criteria.
    Determine the meaningful states (rows) and surfaces (Desktop/Mobile;
    Tablet only when the behaviour differs) from the requirement's
    behaviour — one row per state, at least one row. Each cell is a short
    TEXT description of the screen to capture; the image is attached later,
    directly in the published page. Never invent an image file, a
    design-tool link, or a state the behaviour does not imply. On EXTEND /
    re-generation, preserve verbatim any cell that already carries an
    image — write the text description only into cells with no image, and
    never overwrite a supplied image with a description.
  * Acceptance Criteria (SCRIPT-OWNED, read-only): the scaffold script
    projects the covering proposal slice's Acceptance Criteria VERBATIM,
    including its `#####` re-usability marker group headings and any
    opening boundary note, exactly as authored in the proposal. The AC
    contract, the markers, and their grounding are decided IN THE
    PROPOSAL — never re-shaped, re-grouped, re-marked, added to, or trimmed
    here. To change a criterion, edit the covering proposal slice and
    re-run the script.
  * Key behaviours (SCRIPT-OWNED, read-only): the scaffold script projects
    the FULL scenario set of the covering spec requirement(s) as ONE TABLE
    (`# | Scenario | GIVEN | WHEN | THEN`), not a curated subset — several
    requirements sharing one WBS aggregate into this one block, in spec order
    (a `Management` group always closing the table) and numbered
    continuously. When the block covers 2+ requirements the
    script emits a bold sub-header row per requirement — the row is that
    requirement's NAME from its spec heading (the WBS prefix stripped) — and
    the rows beneath it are that requirement's scenarios; a single-requirement
    block stays flat with no sub-header. Multiple AND outcomes collapse into
    the THEN cell. The LLM never curates, copies, sanitises, prose-ifies, or
    omits it, and never edits or adds a sub-header row; to change a behaviour
    or a group label, edit the covering spec requirement (its heading name is
    the label) and re-run the script.

OUT OF SCOPE RULES (two bottom sections, never one generic list):
- "## Deferred Requirements": same signed scope delivered later
  (nothing about the agreed scope changes — only the timing). One
  block per deferred RQ with Status / Reason / Planned delivery /
  Business Requirements.
- "## Change Requests": any change to the signed BRD — added,
  removed, or altered scope (removing a BRD feature is a CR, not a
  deferral). One block per CR with the code, the DECIDED status from
  grooming (`Agreed` | `Declined` | `Pending client agreement`), why
  it is a CR vs the BRD, tracker ticket (the CR's OWN ticket when a
  source names one, else TBD — never the change's own tracker key),
  and its business requirements. A declined CR is recorded as
  declined — never resurrected as pending or absorbed into
  functional scope.
- Sources: the proposal's CR dispositions as DECIDED at grooming,
  plus discovery's Change Request Candidates. Use "None." when a
  section is empty.

OUTPUT CONTRACT:
- `openspec/changes/<name>/fsd/<feature>.md` working copies only —
  one per Delivery Scope & Targets FSD row, following the template
  section order and heading levels. No Confluence publishing, no
  Page IDs, no status progression, no commentary around the document.

SELF-CHECK BEFORE OUTPUT (fix any "no" before emitting):
- Did I verify the Grooming Approval Gate passed, the delivery scope
  is functional (FSD row present), and the change warrants a client
  document?
- Is every in-scope RQ from the discovery trace covered by exactly one
  functional block — one RQ, one block (no consolidation, no split) —
  with NO block for deferred, declined, or unlocked scope, and no
  summary requirements table?
- Did I run the scaffold script and leave the projected Re-usability Legend
  and the three script-owned sub-sections (User Story, Acceptance Criteria,
  Key behaviours) EXACTLY as projected — not hand-authored, re-shaped,
  re-grouped, curated, or edited, with every bold sub-header row in a Key
  behaviours table left as the script wrote it (it is a covering
  requirement's name) — each block keyed by the requirement's clean WBS code
  (no `RQ<n>_` prefix, no RQ-ID, no composite)?
- Did the scaffold script report a clean validation (no critical): every
  in-scope slice joined to a covering spec requirement, every WBS matched,
  no orphan or duplicate — with any reported error fixed UPSTREAM
  (proposal / specs) and the script re-run, never patched in the FSD?
- Is every approved decision encoded with its bound intact, every CR
  recorded under its decided status, and no declined/rejected option
  resurrected?
- Hygiene sweep of the ENRICHED (LLM-owned) content — scan every line and
  table cell I authored (General FSD Information, Reference Materials,
  Visualization, framing) for internal repo/artifact paths or names,
  open-question markers, agent commentary, code tokens (incl. HTTP status
  codes), and consumed-dependency vendor/service names; rewrite or remove
  each hit (RQ/WBS/CR codes stay — client vocabulary).
- Is every cited source document resolved to the canonical client-facing
  link we hold (link registry / project-context origin / wiki/Drive),
  with a bare title left ONLY where no link is on record and flagged as
  such, and no URL fabricated?
- Does EVERY functional block carry a Visualization (State | Desktop |
  Mobile) alongside its script-owned sub-sections, with no invented image
  file or state the behaviour does not imply?
- Is the working copy fresh (FSD Status `draft`, Target Approval Date the
  `dd/mm/yyyy` placeholder, every approval
  row `not started`, Open Questions carrying the review sentence and
  its empty table header only, no Confluence URL/Page ID)?
- Does the H1 page title end with " FSD", and is the header card H2
  "## General FSD Information" (not a generic "Document Card")?
- General FSD Information: is Project Name the FULL project/client
  identity from the sources (not the change slug or a bare feature
  name), is Package/Set/Batch blank when no source states a number
  (never invented), and is each approval group titled by its org name
  from CONTEXT?
- ORG FIREWALL: did I seat each roster row by the person's committed org —
  only delivering-org (Astound) people in the Astound SA/FE/BE/QA rows
  (a client "FE lead"/"platform dev"/"architect" stays `TBC`), client-org
  people only in the client group — and is Author/Owner the delivering-org
  document author(s) with role (not the roster, not a CSV provenance name,
  `TBC` when none, never blank/invented)?
