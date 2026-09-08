<!--
  Integration Specification Document (DRAFT) — the engineering contract of an INTEGRATION change.
  Generated in the BA/SA phase by /opsx:propose AFTER the Grooming Approval Gate passed
  (see the isd schema instruction). Scope law: an ISD exists only for an INTEGRATION delivery
  scope (discovery's Delivery Scope & Targets carries the ISD row); a functional change publishes
  an FSD instead — never both.

  Publishing & status lifecycle (owned by the isd schema instruction; /opsx:propose-submit runs the
  publish): draft -> astound review -> <client> review -> approved.

  The ISD describes the AGNOSTIC service capability this change delivers — integration points,
  contract, and governance — for developers to implement, QA to test, and business to sign off.
  It is a projection of the approved offer: requirement areas from the lock/breakdown, the
  contract from design's Integration Specification, behaviour bounds from the delta specs,
  governance from the approved grooming decisions. Invent nothing; decide nothing here.
  Consumer features appear only as context and under Out Of Scope — never as owned scope or
  user stories. Set ISD Status = `draft` on first generation. The page title (H1) ALWAYS ends with
  " ISD" so a reader/reviewer can tell the document type at a glance (e.g. "Acme Payments Integration ISD").

  This file is a WORKING COPY in the change folder at openspec/changes/<name>/isd/<service-integration>.md.
  Its Confluence publish is prepared by /opsx:propose-submit (the agent hands off the command; the human
  runs it). Publishing renders the shared FSD/ISD chrome (see the atlassian-doc-templating skill): the
  card + H3 approval groups + "## Reference Materials" become the page header, and a Document Change Log
  footer is appended automatically — never author a Change History section here. When extending an
  existing integration, fetch the live Confluence page into this working copy first and keep untouched
  sections verbatim.

  ISD Status lifecycle (badges rendered at publish): draft -> astound review -> <client> review -> approved.
-->

# <Service / Capability Name> ISD

<!-- The H1 (= Confluence page title) ALWAYS ends with " ISD" — it marks the document type and its
     integration-specification boundary. Never drop the suffix (e.g. "Acme Payments Integration ISD").

     On a fresh draft this stays empty (no Confluence URL / Page ID yet). The publish
     script (publish.mjs, run by /opsx:propose-submit) stamps the two lines below right
     after this H1 on a successful publish — the same format a later pull re-stamps:
- Confluence: <URL>
- Page ID: <ID>
-->

## General ISD Information

<!-- Headerless 2-column card (keep the empty placeholder header row; do NOT add "Field | Value").
     This is the header card the chrome renders; its label ("General ISD Information") becomes the grey
     header row on the wiki. ISD Status = `draft` on a fresh working copy. WBS Code: only a code the
     sources carry for this capability; leave "—" when none exists (NEVER invent).
     Target Approval Date is the client's target sign-off date in `dd/mm/yyyy`; on a fresh working copy
     leave the `dd/mm/yyyy` placeholder — `/opsx:propose-submit` prompts for and fills the real date at
     publish. Never invent a date. -->

|   |   |
| - | - |
| WBS Code | <code from sources, or "—" when none exists — NEVER invent one> |
| Project Name | <Service / Capability Name> |
| Jira Reference | <ticket from sources; a consumer-feature tracker is allowed with an explicit note when no dedicated ticket exists> |
| Author/Owner | <Author (role)> |
| ISD Status | draft |
| Target Approval Date | dd/mm/yyyy |

### Astound Approval

<!-- Two approval groups, each an H3 under the card: the delivering org FIRST, then the client. The
     delivering-org header is ALWAYS "Astound Approval" — it is STATIC; Astound is always the delivering
     agency, so never take this name from CONTEXT and never swap in another team. The client group is
     "### <Client Org> Approval" using the client org name from CONTEXT. Headerless 3-column roster:
     role | status | name — the ROLE leads because, once published, column 1 becomes the wiki's Content
     Properties key that the approval matrix aggregates on, so it must stay a short stable token. The
     Astound roster carries the default ISD roles FA, FE, BE, QA (FA = Functional Analyst, this org's name
     for the BA role); seed names as `TBC` and substitute real names from CONTEXT when present. On a fresh
     working copy EVERY status is `not started`. Status badges: `not started` | `in progress` |
     `pending answers` | `approved`. -->

|    |             |     |
| -- | ----------- | --- |
| FA | not started | TBC |
| FE | not started | TBC |
| BE | not started | TBC |
| QA | not started | TBC |

### <Client Org> Approval

|     |             |     |
| --- | ----------- | --- |
| TBC | not started | TBC |

## Reference Materials

<!-- Vendor documentation (plain URLs as sourced — no project-context wrappers), tracker tickets, and
     source documents by title or client-facing URL. Never internal repo paths, discovery/proposal/
     grooming/specs/design references, or local working-copy links. Resolve a cited source to the
     canonical link we hold (link registry / project-context origin / wiki/Drive) before emitting a bare
     title; bare title only if none exists (say so); never fabricate a URL. -->

| Material | Link / reference | Notes |
|---|---|---|
| <vendor documentation> | <vendor documentation URL> | <what it grounds: endpoints, SDK, key policy> |
| <BRD / SRD title or URL> | <ref> | <consumer-feature context, if relevant> |

## Overview

<!-- A short executive-summary narrative (~4-8 sentences): what the capability delivers and to whom,
     the vendor named as what is built, and the first consumer as context. Orientation only, NOT a
     contract: no SHALL / obligation language, no table, no consumer user stories, no new scope, no
     provenance / process refs (meetings, Key Decisions, SRD / RQ / WBS ids), and no "see section X"
     pointers. Every in-scope obligation is single-homed elsewhere — the near-final Acceptance Criteria
     in In Scope Functional Requirements, the technical contract in Integration Specification, the key /
     quota / billing / privacy law in Key Governance & Privacy, and boundaries in Out Of Scope — so the
     Overview restates none of them; it only orients the reader. -->

<One-paragraph executive summary of the integration capability and its first consumer.>

## Integration Specification

<!-- The heart of the document: the technical contract, in data-flow order. Every endpoint,
     parameter, field, mapping, and environment MUST trace to design's Integration Specification
     or another cited source. Include vendor snippets/curl/payload examples ONLY when the sources
     carry them; when the sources provide tables, use tables — never fabricate example payloads,
     hosts, or values. Elide what the sources elide. Never assert an undecided magnitude (quota,
     TTL, limit): say where it is settled (e.g. "settled in the technical design"). -->

<One-paragraph intro: what this section specifies.>

### Integration Flow Overview

<!-- Render design's data flow as a SEQUENCE diagram with the drawio-diagrams skill
     (.cursor/skills/plan/drawio-diagrams): author a ```drawio:sequence:integration-flow-overview
     block, run the skill's generate script, then insert
     ![Integration Flow Overview](./assets/integration-flow-overview.png) on the next line.
     Boundary-neutral like design's data flow — arrows carry the logical call, never a
     vendor endpoint/path/key; the third-party vendor sits at the periphery. Use `sequence`
     (ordered call/return over time), not `flow`. -->

### <Integration point>

<!-- One H3 per integration point, in data-flow order. Prose first (what it does + the approved
     placement, e.g. served server-side per the approved decision), then the contract.
     OPTIONAL diagram (the exception, not the norm): a sub-section MAY carry a ```drawio:flow:<slug>
     block + ![…](./assets/<slug>.png) image ONLY when the diagram-opportunity analysis clears the
     hard bar (activity for a real decision with distinct outcomes; plain for a non-linear
     multi-node structure) — most points stay prose + tables and carry no diagram; boundary-neutral,
     never a vendor endpoint/path/param/key in an arrow. -->

**Endpoint:** <method + path / channel as the design states it>

**Request → <Vendor / service>**

<!-- Mirror the design's Data Mapping table: same columns, same rows, same order. Add
     Type/Required/In columns ONLY when vendor documentation in the sources states them. -->

| Field | Source | Note |
| --- | --- | --- |
| <field> | <where the value comes from> | <sourced meaning / constraint> |

**Response ← <Vendor / service>**

| <Vendor> field | Maps to | Note |
| --- | --- | --- |
| <field> | <consumer-contract field> | <sourced note> |

**Error handling:** <failure behaviour from the delta specs>.

### Key Governance & Privacy

<!-- Consolidate the approved governance law: key policy (restrictions, where the key lives per
     path), quota/billing ownership, release prerequisites, privacy bounds. This is an H3
     SUB-SECTION of Integration Specification — never a top-level section. -->

- **Key:** <approved key policy>.
- **Quota & billing:** <approved ownership and guardrails>.
- **Release prerequisite:** <grooming-approved gating, if any>.
- **Privacy:** <approved privacy bounds>.

<!-- A "## How To Test" section MAY be added here ONLY when the sources ground test accounts,
     sandbox hosts, or test data. By default there is NO such section — never fabricate test
     fixtures or credentials to fill one. -->

## In Scope Functional Requirements

<!-- SCAFFOLD-FIRST, SCRIPT-OWNED: this whole section is a DETERMINISTIC projection filled by the
     integration-artifacts scaffold script (generate-integration-artifacts.mjs), one block per in-scope
     integration requirement — one RQ, one block, keyed by its clean WBS code (e.g. `3PI.PAY.CHG`). It
     restates, for the developer and QA reader, the shopper-facing intent (User Story) and the near-final
     Acceptance Criteria the client signed, plus the behaviour scenarios of the covering spec — WITHOUT a
     Visualization (an ISD is a contract, not a UI document). It sits directly after the Integration
     Specification. The rest of the ISD (Overview, Integration Specification, governance) stays
     LLM-authored and stays the agnostic contract; this section is the ONLY place a shopper User Story
     appears. Do NOT hand-add, merge, split, rename, or re-key blocks, and do NOT hand-write the read-only
     sub-sections; to change a Story or a criterion edit the covering proposal slice, to change a
     behaviour edit the covering spec requirement, then re-run the script (it VALIDATES the join first and
     names the exact upstream fix on any error). -->

<!-- SCRIPT-OWNED — the proposal's `## Re-usability Legend`, projected VERBATIM once here, ahead of the
     first requirement block, so the reader can decode the `#####` marker groups below. It is a bold
     label (NOT a heading), so it never parses as a requirement block. The legend is authored ONLY in
     the proposal: never written, extended, trimmed, or re-worded here. The script removes it when the
     proposal carries no legend. -->

**Re-usability Legend**

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

### 3PI.MAP.GEO — Geocoding with proximity ranking

<!-- SCRIPT-OWNED (generate-integration-artifacts.mjs): the User Story, Acceptance Criteria, and Key
     behaviours sub-sections below are filled by the scaffold script and are READ-ONLY. The heading is
     the requirement's clean WBS code, set by the script from the covering slice — NO `RQ<n>_` prefix, no
     RQ-ID, no composite; a clean WBS is UNIQUE per document (never reuse an in-scope WBS for a Deferred /
     Change Request block — reuse breaks the join). To change a Story or an acceptance criterion, edit the
     covering proposal slice; to change a Key behaviour, edit the covering spec requirement; then re-run
     the script. Do NOT hand-edit them here.

     The Key behaviours table carries the FULL scenario set of the covering spec requirement(s), numbered
     continuously. When 2+ requirements share the WBS, each contributes a bold sub-header row carrying
     that requirement's NAME followed by its own scenarios (shown below); a single requirement renders
     flat, with no sub-header row. A label changes only by renaming that spec requirement. -->

#### User Story

As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first.

#### Acceptance Criteria

##### [New]

1. The capability SHALL expose a stable interface that resolves a postal code or city to coordinates and returns proximity-ranked store results.
2. Consumers SHALL integrate against that interface without holding any Cartora key, quota, or billing.
3. The capability SHALL keep Cartora key, quota, and billing governance centralized so consuming features hold none of it.
4. The geocoding call approach (client-side vs server proxy) remains undecided (Q1).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Postcode and city geocoding with proximity ranking** |  |  |  |  |
| 1 | Valid postcode resolves to proximity-ranked results | a consumer requesting geocoding for a valid postal code | the capability processes the request | the consumer receives coordinates and proximity-ranked results for that postal code |
| 2 | Valid city resolves to proximity-ranked results | a consumer requesting geocoding for a city name | the capability processes the request | the consumer receives coordinates and proximity-ranked results for that city |
| 3 | Unresolvable input returns a clear error to the consumer | a consumer requesting geocoding for input Cartora cannot resolve | the capability processes the request | the consumer receives a clear error result instead of coordinates |
| 4 | Cartora key stays out of the browser | a consumer using the capability's geocoding | a shopper's search triggers geocoding requests | no browser-visible request contains the Cartora key |
| **Shopper coordinate privacy** |  |  |  |  |
| 5 | Coordinates are not persisted or sent to analytics | a shopper interaction that passed coordinates through the capability | the geocoding or map request completes | the shopper's coordinates are not persisted by the capability<br>the shopper's coordinates are not sent to analytics |

### 3PI.MAP.RND — Governed map rendering

<!-- SCRIPT-OWNED (generate-integration-artifacts.mjs): the User Story, Acceptance Criteria, and Key
     behaviours sub-sections below are filled by the scaffold script and are READ-ONLY. The heading is
     the requirement's clean WBS code, set by the script from the covering slice — NO `RQ<n>_` prefix, no
     RQ-ID, no composite; a clean WBS is UNIQUE per document (never reuse an in-scope WBS for a Deferred /
     Change Request block — reuse breaks the join). To change a Story or an acceptance criterion, edit the
     covering proposal slice; to change a Key behaviour, edit the covering spec requirement; then re-run
     the script. Do NOT hand-edit them here.

     The Key behaviours table carries the FULL scenario set of the covering spec requirement(s), numbered
     continuously. When 2+ requirements share the WBS, each contributes a bold sub-header row carrying
     that requirement's NAME followed by its own scenarios (shown below); a single requirement renders
     flat, with no sub-header row. A label changes only by renaming that spec requirement. -->

#### User Story

As a shopper, I want to see nearby stores on an interactive map, so that I can locate a store to visit.

#### Acceptance Criteria

##### [3rd party OOTB]

1. The capability SHALL render through the vendor's own map runtime as delivered, owning no map-drawing behaviour of its own; what the runtime draws changes with the vendor, not with this change.

##### [New]

1. The capability SHALL expose map rendering as a lazy-loadable entrypoint so consuming features keep maps off their critical path.
2. The capability SHALL keep the Cartora key domain-restricted, with quota and billing governed on the client's Cartora account (guardrails per Q2).
3. The capability SHALL keep shopper coordinates transient — never persisted and never sent to analytics.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Map loads on consumer demand without blocking content | a consuming feature whose page content has rendered | the consumer requests the map entrypoint | an interactive map is rendered<br>the consumer's content stays usable before and while the map loads |
| 2 | Consumers render maps without holding a vendor key | a consuming feature integrating the map entrypoint | the map renders | the consumer supplies no Cartora key of its own<br>map requests carry the capability's domain-restricted key |

### 3PI.MAP.STY — Consumed map presentation (style and map type)

<!-- SCRIPT-OWNED (generate-integration-artifacts.mjs): the User Story, Acceptance Criteria, and Key
     behaviours sub-sections below are filled by the scaffold script and are READ-ONLY. The heading is
     the requirement's clean WBS code, set by the script from the covering slice — NO `RQ<n>_` prefix, no
     RQ-ID, no composite; a clean WBS is UNIQUE per document (never reuse an in-scope WBS for a Deferred /
     Change Request block — reuse breaks the join). To change a Story or an acceptance criterion, edit the
     covering proposal slice; to change a Key behaviour, edit the covering spec requirement; then re-run
     the script. Do NOT hand-edit them here.

     The Key behaviours table carries the FULL scenario set of the covering spec requirement(s), numbered
     continuously. When 2+ requirements share the WBS, each contributes a bold sub-header row carrying
     that requirement's NAME followed by its own scenarios (shown below); a single requirement renders
     flat, with no sub-header row. A label changes only by renaming that spec requirement. -->

#### User Story

As a shopper, I see the store map in the brand's configured style and map type (street/satellite).

#### Acceptance Criteria

The presentation itself — map type and brand theme — is authored and published OUTSIDE this change, on the vendor's hosted no-code surface, by the client's brand/content team. This slice delivers only how the capability references that published presentation and how it behaves when the presentation changes or cannot be resolved.

##### [3rd party OOTB]

1. The capability SHALL render its map surface against the published presentation configuration it is pointed at, so the configured map type and brand theme reach the storefront with no storefront release.
2. The capability SHALL hold a reference to that presentation and never a copy of it, so a re-published presentation reaches the storefront without a code change.
3. When the referenced presentation cannot be resolved, the capability SHALL still expose a usable map surface on the vendor's default presentation rather than failing the consuming feature's render.
4. The v1 presentation configuration SHALL be the brand/content team's published "Brand Street" style, which that team authors and owns.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Published presentation reaches the map surface | a published presentation carrying a map type and brand theme | a consumer renders the capability's map surface | the map surface is rendered against that published presentation |
| 2 | Re-published presentation needs no release | a consumer rendering the capability's map surface | the brand team re-publishes the presentation on the vendor surface | the changed map type and theme reach the consumer without a code change |
| 3 | Unresolvable presentation still yields a usable map | a referenced presentation the capability cannot resolve | a consumer renders the capability's map surface | a usable map surface is rendered on the vendor's default presentation<br>the consuming feature's render does not fail |

### 3PI.MAP.MRK — Consumed store markers

<!-- SCRIPT-OWNED (generate-integration-artifacts.mjs): the User Story, Acceptance Criteria, and Key
     behaviours sub-sections below are filled by the scaffold script and are READ-ONLY. The heading is
     the requirement's clean WBS code, set by the script from the covering slice — NO `RQ<n>_` prefix, no
     RQ-ID, no composite; a clean WBS is UNIQUE per document (never reuse an in-scope WBS for a Deferred /
     Change Request block — reuse breaks the join). To change a Story or an acceptance criterion, edit the
     covering proposal slice; to change a Key behaviour, edit the covering spec requirement; then re-run
     the script. Do NOT hand-edit them here.

     The Key behaviours table carries the FULL scenario set of the covering spec requirement(s), numbered
     continuously. When 2+ requirements share the WBS, each contributes a bold sub-header row carrying
     that requirement's NAME followed by its own scenarios (shown below); a single requirement renders
     flat, with no sub-header row. A label changes only by renaming that spec requirement. -->

#### User Story

As a shopper, I see stores on the map with the brand's custom markers/pins.

#### Acceptance Criteria

The marker artwork is authored and published OUTSIDE this change, inside the same vendor-hosted presentation the client's brand/content team owns. This slice delivers only how the capability renders the published marker set.

##### [3rd party OOTB]

1. The capability SHALL plot store locations with the marker set carried by the published presentation, so brand-authored markers reach the storefront with no storefront release.
2. The capability SHALL define no marker artwork of its own, so the published marker set stays the single source for how a store appears on the map.
3. When the published presentation carries no custom marker set, the capability SHALL still plot store locations using the vendor's default marker.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Brand-authored markers plot the stores | a published presentation carrying a custom marker set | the capability plots store locations on the map surface | the stores are plotted with that published marker set |
| 2 | Presentation without custom markers falls back to the default | a published presentation carrying no custom marker set | the capability plots store locations on the map surface | the stores are plotted using the vendor's default marker |

## Out Of Scope / Limitations

<!-- The boundary list: consumer UI/UX owned by the functional change, sibling services owned by
     separate changes, hand-offs this capability does not perform. Name the owning change/service
     for each item. -->

- <boundary item — owning change/service named>

## Deferred Requirements

<!-- Signed scope, UNCHANGED, delivered later (timing only). Use "None." if nothing is deferred. -->

None.

## Change Requests

<!-- Any CHANGE to the signed BRD needing client agreement, each under its DECIDED grooming
     status (`Agreed` | `Declined` | `Pending client agreement`). Use "None." if there are none. -->

None.

## Open Questions

<!-- This section exists for the Confluence review cycle. On generation it is EMPTY — emit the
     sentence and the bare table header with NO rows; every pre-build decision was settled at
     grooming, so never carry an open item into a generated ISD. Reviewers add rows on the
     published page: Q# numbers them, Owner names who must answer, Decision records the answer. -->

Use inline comments during review whenever possible; this table is only for questions in areas
the document does not cover.

| Q# | Question | Owner | Notes | Decision |
| --- | --- | --- | --- | --- |
