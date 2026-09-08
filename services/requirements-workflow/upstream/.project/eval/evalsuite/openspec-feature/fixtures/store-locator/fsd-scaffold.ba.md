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

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

### SLS.SRCH — Store search by postal code, city, or location

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.

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

##### [SF Next OOTB]

1. Business users SHALL be able to configure the default search radius and its units in Business Manager.
2. Business users SHALL be able to enable or disable the use-my-location option in Business Manager.

##### [New]

1. The store search SHALL let the shopper search for stores by postal code or city and SHALL show matching stores ordered by proximity.
2. Search by current location SHALL run only on an explicit shopper action and SHALL be gated on the browser location permission.
3. When the location permission is denied, the store search SHALL explain that the permission is required and SHALL still allow search by postal code or city.
4. When a search matches no stores, the store search SHALL show an empty-results message instead of the result list.
5. The store locator SHALL NOT persist the shopper's coordinates and SHALL NOT send them to analytics.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Store search** |  |  |  |  |
| 1 | Search by postal code returns nearby stores | a shopper on the store locator | they enter a valid postal code and submit the search | stores near that postal code are shown, ordered by proximity |
| 2 | Search by city returns nearby stores | a shopper on the store locator | they enter a city name and submit the search | stores in and around that city are shown, ordered by proximity |
| 3 | Search by current location with permission granted | a shopper who grants the browser location permission | they choose "use my location" | stores near their current location are shown, ordered by proximity |
| 4 | Location permission denied falls back to manual search | a shopper who denies the browser location permission | they choose "use my location" | a message explains that the permission is required<br>they can still search by postal code or city |
| 5 | Search with no matching stores shows an empty state | a shopper on the store locator | their search matches no stores | an empty-results message is shown instead of the result list |
| 6 | Shopper coordinates stay transient | a shopper who searched by current location | the search completes | the shopper's coordinates are not persisted<br>the shopper's coordinates are not sent to analytics |
| **Management** |  |  |  |  |
| 7 | Default search radius follows the Business Manager configuration | an administrator has set the default search radius and units in Business Manager | a shopper runs a store search | results are limited to the configured default radius |
| 8 | Use-my-location availability follows the Business Manager configuration | an administrator has disabled the use-my-location option in Business Manager | a shopper opens the store locator | only postal-code and city search are offered |

### SLS.AC — Address autocomplete suggestions

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions.

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

The suggestions themselves come from the address-autocomplete capability a separate change delivers; this slice delivers the type-ahead surface in the store search and how it behaves when suggestions do not arrive.

##### [New]

1. While the shopper types a postal code or partial address, the store search SHALL offer address suggestions; selecting a suggestion SHALL run the search for that address.
2. If suggestions are unavailable, the typed search SHALL still submit and return results.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Suggestions appear while typing | a shopper typing a postal code or partial address in the store search | they pause typing | address suggestions matching the typed input are offered |
| 2 | Selecting a suggestion runs the search | address suggestions shown for the typed input | the shopper selects a suggestion | the store search runs for the selected address |
| 3 | Search stays usable when no suggestions are available | a shopper typing in the store search while suggestions are unavailable | they submit the typed postal code or city | the search runs on the typed input without suggestions |

### SLS.MAP — Interactive map with a synchronized list

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other.

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

The map itself is rendered by the maps/geocoding capability a separate change delivers, and the landing's promo/editorial region is authored in the Page Designer library the PD workstream owns (RQ-751, RQ-752; v1 ships one curated Featured-Store card, settled at the discovery review). This slice delivers the result view around them — layout, list/map synchronization, and the accessible list that carries the results.

##### [SF Next OOTB]

1. Business users SHALL be able to configure the number of stores shown per page and the load-more batch in Business Manager.

##### [SF Next Modified]

1. The result view SHALL render server-side with the store list first and load the map as a companion afterwards, so the list stays usable while the map loads.

##### [New]

1. On desktop, the result view SHALL show the interactive map beside the result list and keep it visible.
2. On mobile and tablet, the result view SHALL be list-first with the map collapsed behind a toggle, so the result list is the primary view and usable before the map is opened.
3. Wherever the map is open, selecting a store in the list SHALL highlight it on the map and selecting a store on the map SHALL highlight it in the list.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Synchronized map and list** |  |  |  |  |
| 1 | Selecting a store in the list highlights it on the map | search results shown as a map and a list | the shopper selects a store in the list | the corresponding store is highlighted on the map |
| 2 | Selecting a store on the map highlights it in the list | search results shown as a map and a list | the shopper selects a store marker on the map | the corresponding entry is highlighted in the list |
| 3 | Result list is usable before the map has loaded | a shopper whose search results are ready while the map is still loading | the result list renders | the shopper can browse and select stores from the list without waiting for the map |
| **Side-by-side map and list on desktop** |  |  |  |  |
| 4 | Map is shown side by side with the list on desktop | a shopper on a desktop viewport | search results are shown | the interactive map is displayed side by side with the result list |
| 5 | Map stays visible while the shopper browses the list | a shopper on a desktop viewport with search results shown | they browse and select stores in the result list | the map stays visible next to the list |
| **List-first layout with map toggle on mobile and tablet** |  |  |  |  |
| 6 | Result list is shown first with the map behind a toggle on mobile and tablet | a shopper on a mobile or tablet viewport | search results are shown | the result list is the primary view and the map is collapsed behind a toggle the shopper can open |
| 7 | Result list is usable before the map toggle is opened | a shopper on a mobile or tablet viewport with the map still collapsed | they browse and select stores in the result list | the stores can be browsed and selected without opening the map<br>opening the toggle shows the same results on the map |
| **Management** |  |  |  |  |
| 8 | Results per page follow the Business Manager configuration | an administrator has set the number of stores per page in Business Manager | a shopper's search returns more stores than that number | the list shows one page of results and offers a load-more control for the rest |

### SLS.DTL — Store details

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

Each store SHALL display its name, full address, phone number, opening hours and the services available at that store.

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

The store records behind the detail (name, address, phone, opening hours, services) are maintained by the client's team in the commerce backend and read through the already-integrated Stores API — this change neither owns nor authors them. This slice delivers the detail surface that presents whatever each record exposes.

##### [New]

1. The store detail SHALL show each store's name, full address, phone number, regular weekly opening hours, and the services available at that store; holiday-hours display follows what the commerce Stores API exposes (Q1).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Viewing a store shows its detail | a store in the results | the shopper opens that store's detail | the name, full address, phone number, regular weekly opening hours, and available services are shown |
| 2 | Store detail reflects the administered store record | an administrator maintains a store's record in the commerce backend (Business Manager) | a shopper opens that store's detail | the detail shows the name, address, phone, regular weekly opening hours, and services from that record |

### SLS.FLT — Filter stores by service

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping.

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

##### [SF Next OOTB]

1. Business users SHALL be able to configure which services are offered as filter options in Business Manager; v1 is seeded with click-and-collect, returns, and gift wrapping.

##### [New]

1. The service filter SHALL let the shopper filter stores by the services offered (in v1: click-and-collect, returns, and gift wrapping).
2. Applying a filter SHALL update both the list and the map; clearing the filter SHALL restore the full search results.
3. When a filter matches no stores, the result view SHALL show an empty-results message for the filtered view.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Filter stores by service** |  |  |  |  |
| 1 | Filtering by a service narrows list and map | search results containing stores with and without click-and-collect | the shopper filters by click-and-collect | only stores offering click-and-collect remain in the list and on the map |
| 2 | Clearing the filter restores all stores | an active service filter | the shopper clears the filter | all stores from the search are shown again in the list and on the map |
| 3 | Filter with no matching stores shows an empty state | search results where no store offers the selected service | the shopper applies that service filter | an empty-results message is shown for the filtered view |
| **Management** |  |  |  |  |
| 4 | Offered filter services follow the Business Manager configuration | an administrator has set which services are offered as filters in Business Manager | a shopper opens the service filter | only the configured services are offered as filter options |

### SLS.DIR — Get directions

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination.

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

##### [New]

1. Each store SHALL provide a "Get directions" action that opens the shopper's own map provider with the store as the destination.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Get directions opens the shopper's map provider | a store in the results | the shopper activates "Get directions" | the shopper's map provider opens with that store as the destination |

### SLS.A11Y — Accessible, keyboard-operable store locator

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map.

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

##### [New]

1. The store locator SHALL be fully keyboard operable and usable with a screen reader, with the list as an accessible alternative to the map (WCAG 2.2 AA).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Keyboard-only operation | a keyboard-only shopper on the store locator | they navigate the search, results, filters, and store actions | every control is reachable and operable without a pointer |
| 2 | Screen reader announces the result list | a shopper using a screen reader | search results are shown | the store list is announced and usable as an accessible alternative to the map |

### SLS.I18N — Localization

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

All shopper-facing text in the store locator SHALL be localizable.

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

##### [SF Next OOTB]

1. All shopper-facing text in the store locator SHALL be localizable through the storefront's existing translation mechanism and SHALL be shown in the shopper's active locale.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Labels are shown in the active locale | a shopper using a non-default locale | they open the store locator | all labels and messages are shown in that locale |

### GH.UTL — Navigation entry and retired directory URL

<!-- SCRIPT-OWNED (generate-functional-artifacts.mjs): the User Story, Acceptance Criteria, and Key behaviours
     sub-sections below are filled by the scaffold script and are READ-ONLY. To change a User Story or
     an acceptance criterion, edit the covering proposal slice; to change a Key behaviour, edit the
     covering spec requirement; then re-run the script. Do NOT hand-edit them here.
     The heading is the requirement's clean WBS code (e.g. `CHK.PAY`), set by the script from the
     covering slice — NO `RQ<n>_` prefix, no RQ-ID, no composite; a clean WBS is UNIQUE per document
     (never reuse an in-scope WBS for a Deferred / Change Request block — reuse breaks the join). -->

#### User Story

The store locator SHALL be reachable from the global site navigation.

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

##### [SF Next Modified]

1. The footer "Our stores" link under "About" SHALL point to the store locator page instead of the static directory.

##### [Legacy Modified]

1. The static store directory page SHALL be retired at launch, with no shopper-facing entry to it left in the storefront.
2. The retired `/stores` URL SHALL keep working for bookmarked and indexed links by redirecting to the locator instead of returning a not-found page (redirect type/target per Q2).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Footer utility links** |  |  |  |  |
| 1 | Footer exposes the Our stores link | a shopper on any storefront page | they open the footer | an "Our stores" link is present under "About"<br>it navigates to `/store-locator` |
| **Retired store directory URL redirects to the locator** |  |  |  |  |
| 2 | Opening the retired URL lands on the locator | a shopper opening the retired `/stores` URL from a bookmark or search result | the request resolves | the shopper is permanently redirected to `/store-locator` |

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
