<!--
  Client-facing Functional Specification Document (DRAFT).
  Generated in the BA/SA phase by /opsx:propose AFTER the Grooming Approval Gate passed
  (see the fsd schema instruction). Scope law: an FSD exists only for a FUNCTIONAL delivery
  scope (discovery's Delivery Scope & Targets carries the FSD row); an integration change
  publishes an ISD instead — never both.

  Publishing & status lifecycle (owned by the fsd schema instruction; /opsx:propose-submit runs the
  publish): draft -> astound review -> <client> review -> approved.

  The FSD is the client-facing projection of the APPROVED offer: the requirements table mirrors
  discovery's Requirements Trace, user stories come from the proposal's approved Delivery
  Breakdown slices, acceptance criteria restate this change's delta specs plus approved grooming
  decisions. Invent nothing; decide nothing here. An FSD is a client document at feature/area
  granularity — a single change may touch several areas and therefore produce several FSD files.
  Set FSD Status = `draft` on first generation. The page title (H1) ALWAYS ends with " FSD" so a
  reader/reviewer can tell the document type at a glance (e.g. "CHK - Checkout Payments FSD").

  This file is a WORKING COPY in the change folder at openspec/changes/<name>/fsd/<feature>.md.
  Its Confluence publish is prepared by /opsx:propose-submit (the agent hands off the command; the human
  runs it). When extending an existing feature, fetch the live Confluence page into this working copy
  first and keep untouched sections verbatim.

  SCAFFOLD-FIRST: the per-requirement User Story, Acceptance Criteria, and Key behaviours sub-sections
  are filled DETERMINISTICALLY by the scaffold script (generate-functional-artifacts.mjs), projected from the proposal
  slice (Story + Acceptance Criteria) and this change's specs (Key behaviours). RUN THE SCAFFOLD SCRIPT
  FIRST, then enrich ONLY the non-owned sections (General FSD Information, approval roster, Reference
  Materials, per-block Visualization, framing). Never hand-edit the three script-owned sub-sections.
-->

# <WBS> - <Feature Name> FSD

<!-- The H1 (= Confluence page title) ALWAYS ends with " FSD" — it marks the document type and its
     functional-specification boundary. Never drop the suffix (e.g. "CHK - Checkout Payments FSD").

     On a fresh draft this stays empty (no Confluence URL / Page ID yet). The publish
     script (publish.mjs, run by /opsx:propose-submit) stamps the two lines below right
     after this H1 on a successful publish — the same format a later pull re-stamps:
- Confluence: <URL>
- Page ID: <ID>
-->

## General FSD Information

<!-- Headerless 2-column card: keep the empty header row exactly (do NOT add "Field | Value").
     The card H2 is "General FSD Information" (NOT a generic "Document Card") — the type-specific label
     keeps the reader anchored in the functional-specification scope.
     FSD Status is `draft` on a fresh working copy.
     Project Name = the FULL project/client identity the sources name (e.g. "<Client> — <Programme>
     (<tracker key>)") — never the change slug or a bare feature name, never abbreviated.
     Package/Set/Batch is OPTIONAL: fill it only when a source states a package/set/batch number for
     this scope; otherwise leave the cell blank — never invent a number.
     Target Approval Date is the client's target sign-off date in `dd/mm/yyyy`; on a fresh working copy
     leave the `dd/mm/yyyy` placeholder — `/opsx:propose-submit` prompts for and fills the real date at
     publish. Never invent a date. -->

|                      |                                        |
| -------------------- | -------------------------------------- |
| WBS-Feature Name     | <WBS> - <Feature Name>                 |
| Project Name         | <Client — Programme (tracker key)>     |
| Package/Set/Batch    |                                        |
| Author/Owner         | <Author (role)>, <Owner (role)>        |
| FSD Status           | draft                                  |
| Target Approval Date | dd/mm/yyyy                             |

### Astound Approval

<!-- Two approval groups, each an H3 under the General FSD Information card: the delivering org FIRST,
     then the client. The delivering-org header is ALWAYS "Astound Approval" — it is STATIC; Astound is
     always the delivering agency, so never take this name from CONTEXT and never swap in another team.
     The client group is "### <Client Org> Approval" using the client org name from CONTEXT. Headerless
     3-column roster: role | status | name — the ROLE leads because, once published, column 1 becomes the
     wiki's Content Properties key that the approval matrix aggregates on, so it must stay a short stable
     token. The Astound roster carries the DEFAULT FSD roles SA, FE, BE, QA — seed the four rows with
     `TBC` names and substitute real names from CONTEXT when present. On a fresh working copy EVERY status
     is `not started`. Status badges: `not started` | `in progress` | `approved`. -->

|    |             |     |
| -- | ----------- | --- |
| SA | not started | TBC |
| FE | not started | TBC |
| BE | not started | TBC |
| QA | not started | TBC |

### <Client Org> Approval

|     |             |     |
| --- | ----------- | --- |
| TBC | not started | TBC |

## Reference Materials

<!-- Client-facing references only, each by its TITLE or client-facing URL. NEVER an internal repo path
     or `project-context` inbox/anchor (`.cursor/…`, `…/inbox/…`), discovery/proposal/grooming, specs,
     design, technical-design, tasks, or a local working-copy link — cite the client-facing title/URL
     the source carries instead. Resolve a cited source to the canonical link we hold (link registry /
     project-context origin / wiki/Drive) before emitting a bare title; bare title only if none exists
     (say so); never fabricate a URL. -->

| Material | Link / reference | Notes |
|---|---|---|
| SRD / BRD | <client-facing URL or document title> | <source of RQ/WBS rows> |

## In Scope Functional Requirements

<!-- ONE block per in-scope requirement — one RQ, one block. Do NOT consolidate several RQs into a
     single block, and do NOT split one RQ across blocks. Every in-scope lock RQ gets its own block;
     deferred and CR items get NO block (they go to the Deferred Requirements / Change Requests
     sections). Keep it lean and human-readable — a client document, not a test spec. Every statement
     MUST trace to this change's delta specs, the approved grooming decisions, or the
     discovery/proposal trace — invent nothing.

     HEADING LEVELS: each requirement is an H3 (###) under this H2, keyed by its clean WBS code; its
     sub-sections are H4 (####); Acceptance-criteria groups and scenarios are H5 (#####). -->

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs) — the proposal's `## Re-usability Legend`,
     projected VERBATIM once here, ahead of the first requirement block, so the reader can decode the
     `#####` marker groups below. It is a bold label (NOT a heading), so it never parses as a
     requirement block. The legend is authored ONLY in the proposal: never written, extended, trimmed,
     or re-worded here. The script removes it when the proposal carries no legend. -->

**Re-usability Legend**

- `[<Marker>]` — <the marker's fixed gloss, from the proposal>

### <WBS> — <Requirement name>

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

<!-- SCRIPT-OWNED — the covering Delivery Breakdown slice's User Story, projected verbatim. -->

As a <role>, I want <goal>, so that <benefit>.

#### Visualization

<!-- REQUIRED in every requirement block (like User Story and Acceptance Criteria). Determine the
     meaningful states (rows) and surfaces (Desktop/Mobile; add Tablet only when the behaviour differs
     there) from the requirement's behaviour — one row per state, at least one row. Each cell is a short
     TEXT description of the screen to capture — the image is added later, directly in the published page.
     Do NOT invent an image file, a design-tool link, or a state the behaviour does not imply; do NOT
     overwrite an image already present in a cell. -->

| State | Desktop | Mobile |
| ----- | ------- | ------ |
| <state> | <what the desktop screen shows> | <what the mobile screen shows> |

#### Acceptance Criteria

<!-- SCRIPT-OWNED — projected VERBATIM from the covering proposal slice's Acceptance Criteria,
     including its `#####` re-usability marker groups and any opening boundary note, unchanged. The
     AC shape and its markers are decided in the proposal, never re-shaped or re-marked here. -->

1. The system SHALL <mandatory behaviour>.


#### Key behaviours

<!-- SCRIPT-OWNED — the FULL scenario set of the covering spec requirement(s), projected as ONE
     TABLE (not curated): scenarios in spec order, numbered continuously across the requirements
     that share this WBS. When 2+ requirements share it, each contributes a bold sub-header row
     carrying that requirement's NAME followed by its own scenarios (shown below); a single
     requirement renders flat, with no sub-header row. Multiple AND outcomes collapse into the
     THEN cell. Never authored or edited here — change a behaviour, or a sub-header label, by
     editing the covering spec requirement, then re-run the script. -->

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **<first covering requirement's name>** |  |  |  |  |
| 1 | <scenario name> | <precondition> | <actor action or system event> | <observable outcome> |
| **<next covering requirement's name>** |  |  |  |  |
| 2 | <scenario name> | <precondition> | <actor action or system event> | <observable outcome> |

<!-- Repeat requirement blocks above for each additional requirement. -->

## Deferred Requirements

<!-- Signed BRD scope, UNCHANGED, delivered in a later phase. NOT change requests (nothing about the
     agreed scope changes — only the timing). If the agreed scope itself changes (added, removed, or
     altered), it belongs under Change Requests — removing a BRD feature is a CR, not a deferral.
     Use "None." if nothing is deferred. -->

### <Code> — <Requirement name>

- **Status:** Descoped from <this phase>.
- **Reason:** <why it was pulled out of this phase>.
- **Planned delivery:** <future dedicated spec / later phase / pending condition>.
- **Business Requirements:**
  1. <deferred requirement>

## Change Requests

<!-- Any CHANGE to the signed BRD — new scope, REMOVED scope, or altered behaviour — that needs client
     agreement. Removing a signed BRD feature counts here (not under Deferred). Status is the DECIDED
     disposition from the approved grooming session — a declined CR is recorded as Declined, never
     resurrected as pending. Say WHY vs the BRD (new / removed / altered). Source: the proposal's CR
     dispositions as decided at grooming + discovery.md "Change Request Candidates".
     Use "None." if there are none. -->

### <Code or CR#> — <Short Name>

- **Status:** <Agreed | Declined | Pending client agreement>.
- **Why a change request:** not in the signed BRD — <new capability / workshop-introduced / altered>.
- **Tracker Ticket:** <KEY or TBD>.
- **Business Requirements:**
  1. <main requirement>

## Open Questions

<!-- This section exists for the Confluence review cycle. On generation it is EMPTY — emit the
     sentence and the bare table header with NO rows; every pre-build decision was settled at
     grooming, so never carry an open item into a generated FSD. Reviewers add rows on the
     published page: Q# numbers them, Owner names who must answer, Decision records the answer. -->

Use inline comments during review whenever possible; this table is only for questions in areas
the document does not cover.

| Q# | Question | Owner | Notes | Decision |
| --- | --- | --- | --- | --- |
