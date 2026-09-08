# add-store-locator - Discovery & Requirements

## Executive Summary

This change introduces a modern store locator for the storefront, replacing a legacy static directory with a more useful way for shoppers to find nearby stores and understand the services available to them. Its business value is to reduce store-finding friction, support click-and-collect journeys, and drive more qualified visits to physical stores.

**What this file settles.** The delivery scope is functional: it owns the storefront experience and the navigation entry, and consumes maps/geocoding, address autocomplete, and commerce store data as dependencies owned by separate changes. Nine signed requirements are in scope, one is deferred to a later phase, and the Page Designer promo components the landing renders belong to a separate workstream. Requirements Trace holds the per-requirement disposition and delivery mode; Affected Capabilities holds what is added, modified, and retired.

**What stays open.** Three refinements are registered in Open Questions — special/holiday opening hours, the redirect for the retired `/stores` URL, and the Page-Designer promo boundary — all NICE-TO-HAVE and routed to grooming. One new-scope idea, in-store appointment booking, sits in Change Request Candidates as a pending candidate, not as agreed scope.

**Readiness.** No CRITICAL question is open, so discovery is ready to move into proposal. The Document Consumption Guide is the machine-readable form of everything the next artifacts consume.

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Requirements Trace | The signed scope boundary: what this change builds, what is deferred, and what is left to another owner; downstream slices and spec headings key off it. | In scope: SLS.SRCH, SLS.MAP, SLS.DTL, SLS.DIR, SLS.FLT, SLS.A11Y, SLS.I18N, GH.UTL, SLS.AC; Deferred: SLS.INV; Out of scope: PD.PT.SLP, PD.CMP.STC, PD.CMP.HMH → page-designer |
| Affected Capabilities | Which capabilities this change owns and how; fixes the specs file inventory. | store-locator ADD, global-navigation MODIFY, store-directory REMOVE |
| Dependencies | External services and CMS features consumed but not owned; become proposal external-dependency rows. | Cartora maps/geocoding, Addressly address autocomplete, Commerce Stores API, SFCC Business Manager locator config, Page Designer content components |
| Open Questions | Unresolved refinements gating readiness; the only source of Q# downstream. | Q1 — NICE-TO-HAVE, Open; Q2 — NICE-TO-HAVE, Open; Q3 — NICE-TO-HAVE, Open |
| Change Request Candidates | Asks against signed scope; seed proposal CR dispositions. | CR1 — pending |
| Cross-cutting Requirements | Applicable cross-cutting areas; required design config rows. | Accessibility, SEO, Consent / privacy, i18n |
| Delivery Scope & Targets | The resolved scope and what is published where. | functional; FSD NEW → `openspec/changes/add-store-locator/fsd/store-locator.md` |

## Discovery Source Inventory

| Source | Reference / Anchor | Purpose |
|---|---|---|
| BRD / SRD | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` | Canonical signed WHAT, official RQ/section codes, and initial priority/disposition |
| Existing OpenSpec baseline specs | `openspec/specs/global-navigation/spec.md`; `openspec/specs/store-directory/spec.md` | Current behaviour and capability ownership for the footer "Our stores" link and legacy static `/stores` directory |
| Legacy implementation / current storefront | `openspec/specs/global-navigation/spec.md`; `openspec/specs/store-directory/spec.md` | Confirms the existing implementation being modified or retired; classifies `global-navigation` as MODIFY and `store-directory` as REMOVE |
| Legacy URL / live storefront page | `/stores` | Existing bookmarked/indexed storefront surface that needs a redirect decision and makes SEO applicable |
| Meeting notes | `.cursor/.project/project-context/inbox/meeting_notes/2026-02-04_storefront-workshop_transcript.txt` | Launch scope confirmation, third-party service list, appointment-booking new ask, and functional refinement questions |
| Technical notes | `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md` | Storefront constraints (lazy-load, privacy) and the consumed third-party/service list |
| Delivery scope decision | `user request` (BA preparing the functional Store Locator) | Confirms functional-only scope; integrations are consumed here and owned by separate agnostic integration changes |
| Jira | project-context `jira`: SFX-4102 | Tracker context only; do not infer extra requirements beyond the cited source packet |
| Visual reference | current live site (no committed UX prototype or design asset) | The client agreed to use the current site as the visual reference; FSD Visualization describes the needed screens in words and the BA attaches images to the published page later — do not fabricate an image file, a design-tool link, or a screen the behaviour does not imply |
| FSD source lookup | project-context `confluence`: no matching Store Locator FSD found | No existing functional source page to cite or extend; publication target decision is recorded separately |
| Page Designer authoring (signed PD family) | project-context `confluence`: `.cursor/.project/project-context/inbox/confluence/page-designer-srd-requirements.csv.txt` | Signed Page Designer components the locator landing renders — `RQ-751 PD.PT.SLP` (Store Locator content fragment), `RQ-752 PD.CMP.STC` (Featured-Store promo card) — owned by the Page Designer FSD and consumed here; the same list also carries a `RQ-753` homepage hero the locator does not use |

## Requirements Trace

<!-- Golden annotation: on `out of scope` / `deferred` rows the `Affected capability`
cell is a NORMALIZED, best-effort attribution of the owning workstream, not source
truth. Grade those rows on the in/out disposition; any owner a candidate can ground
in the context is acceptable. Only in-scope rows are graded against the context's
capability assignment. -->

Signed source requirements (13): `RQ-701`, `RQ-702`, `RQ-703`, `RQ-704`, `RQ-705`, `RQ-706`, `RQ-707`, `RQ-708`, `RQ-709`, `RQ-710`, `RQ-751`, `RQ-752`, `RQ-753`

| RQ ID | WBS | Title | Source requirement | Requirement | Requirement Details | Disposition | Delivery mode | Affected capability | Notes |
|---|---|---|---|---|---|---|---|---|---|
| RQ-701 | SLS.SRCH | Store search | SRD RQ-701 | As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location. | 1. Search inputs: postal code, city, or current location. 2. Current-location search uses device geolocation and is user-initiated. | in scope | build | store-locator | Geolocation is user-initiated and permission-gated. Postal/city geocoding and proximity consume the maps/geocoding dependency. Default search radius/units and whether use-my-location is offered are Business Manager merchant config. |
| RQ-702 | SLS.MAP | Map + synchronized list | SRD RQ-702 | Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other. | 1. Present results as an interactive map and a synchronized results list. 2. Selecting a store in the list highlights it on the map, and vice versa. | in scope | build | store-locator | The functional page owns map/list synchronization and lazy loading; map rendering consumes the maps dependency. Results page size and load-more batch are Business Manager merchant config. The landing also renders a Page-Designer-authored promo/editorial region (RQ-751, RQ-752) — consumed, not owned; see Dependencies and Q3. |
| RQ-703 | SLS.DTL | Store detail | SRD RQ-703 | Each store SHALL display its name, full address, phone number, opening hours and the services available at that store. | 1. Display store name, full address, phone number, opening hours, and available services. 2. Store data is read from the commerce backend; no new store data model for v1. | in scope | build | store-locator | Store data is consumed from the commerce Stores API dependency; the store records it draws (name/address/phone/hours/services) are Business Manager merchant config; special/holiday opening-hours display is open — see Q1. |
| RQ-704 | SLS.DIR | Get directions | SRD RQ-704 | Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination. | 1. Provide a "Get directions" action for each store. 2. Opens the shopper's map provider with the store set as the destination. | in scope | build | store-locator | Opens the shopper's device/native map provider; functional action owned here. |
| RQ-705 | SLS.FLT | Filter by service | SRD RQ-705 | The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping. | 1. Filter stores by available service. 2. v1 services: click-and-collect, returns, and gift wrapping. | in scope | build | store-locator | v1 service list is fixed to click-and-collect, returns, and gift wrapping; the offered filter options are seeded in Business Manager merchant config; service attributes come from the commerce Stores API dependency. |
| RQ-706 | SLS.A11Y | Accessibility | SRD RQ-706 | The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map. | — | in scope | build | store-locator | Target is WCAG 2.2 AA. |
| RQ-707 | SLS.I18N | Localizable copy | SRD RQ-707 | All shopper-facing text in the store locator SHALL be localizable. | — | in scope | build | store-locator | — |
| RQ-708 | SLS.INV | Real-time in-store inventory | SRD RQ-708 | The store detail SHALL show real-time in-store product availability. | 1. Show real-time in-store product availability on the store detail. | deferred | — | store-locator | Signed as "Could / Later phase"; client confirmed out of the initial release — do not build in v1. |
| RQ-709 | GH.UTL | Reachable from navigation | SRD RQ-709 | The store locator SHALL be reachable from the global site navigation. | — | in scope | build | global-navigation | Footer "Our stores" link re-points from the legacy `/stores` page to the locator; the retired `/stores` URL needs a redirect — see Q2. |
| RQ-710 | SLS.AC | Address autocomplete | SRD RQ-710 | As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions. | 1. As the shopper types a postal code or partial address, offer address suggestions. 2. Suggestions come from the address type-ahead supplier (see workshop). | in scope | build | store-locator | Autocomplete UI is owned here; suggestions consume the address-autocomplete dependency (a separate supplier, distinct from the maps service). |
| RQ-751 | PD.PT.SLP | Store Locator content fragment | SRD RQ-751 | As a Content Manager, I want a Store Locator content fragment so that the content team can author the promotional/editorial region shown on the store locator landing. | 1. A Page Designer page-fragment type that authors the locator landing's promo/editorial region. 2. Current Component ID `page.storelocator`. | out of scope | — | page-designer | Owned by the Page Designer FSD (`PD` WBS family, a separate workstream); the locator landing RENDERS this PD-authored promo region — consumed, not owned (see Dependencies). v1 promo authoring/timing is a Page Designer / content-team concern — see Q3. |
| RQ-752 | PD.CMP.STC | Featured-Store promo card | SRD RQ-752 | As a Content Manager, I want a Featured-Store promo card so that the content team can highlight selected stores or offers on the store locator. | 1. A Page Designer asset component rendering a featured-store / offer promo card. 2. Current Component ID `assets.storeCard`. | out of scope | — | page-designer | Owned by the Page Designer FSD; rendered (consumed) by the locator landing/detail — consumed, not owned (see Dependencies). |
| RQ-753 | PD.CMP.HMH | Homepage hero banner | SRD RQ-753 | As a Content Manager, I want a Homepage hero banner so that I can promote seasonal campaigns on the storefront homepage. | 1. A Page Designer asset component rendering the storefront homepage hero. 2. Current Component ID `assets.heroBanner`. | out of scope | — | page-designer | Owned by the Page Designer FSD; homepage merchandising — NOT used by the store locator. Listed for source fidelity only; it is not a locator dependency and does not appear in Dependencies. |

## Affected Capabilities

Touched capabilities (3): `store-locator`, `global-navigation`, `store-directory`

| Capability | Spec path | Disposition |
|---|---|---|
| store-locator | `openspec/specs/store-locator/spec.md` | ADD (new) |
| global-navigation | `openspec/specs/global-navigation/spec.md` | MODIFY (re-point "Our stores" from `/stores` to the locator) |
| store-directory | `openspec/specs/store-directory/spec.md` | REMOVE (legacy static `/stores` page superseded by the locator) |

## Dependencies

| Dependency | Type | Status | Used by |
|---|---|---|---|
| Cartora maps/geocoding | upstream integration change | separate integration change (to build) | RQ-701, RQ-702 |
| Addressly address autocomplete | upstream integration change | separate integration change (to build) | RQ-710 |
| Commerce Stores API (store data) | existing service | already integrated | RQ-703, RQ-705 |
| SFCC Business Manager — locator merchant config (default search radius/units + whether use-my-location is offered, stores-per-page + load-more batch, filter service options seed, store records name/address/phone/hours/services) | platform config | available (platform) — client merchandising/admin controls, not hard-coded | RQ-701, RQ-702, RQ-703, RQ-705 |
| Page Designer content components — Store Locator content fragment (`page.storelocator`) + Featured-Store promo card (`assets.storeCard`), the locator landing's promo/editorial authoring substrate | platform CMS feature — owned by the Page Designer FSD (`PD` family); itemized as the out-of-scope `PD.*` rows in the Requirements Trace | signed (Approved); consumed here, not owned; v1 promo authoring/timing → Q3 | RQ-702, RQ-703 |

## Change Request Candidates

| CR# | Trigger | Description | Impact | Proposed disposition | Status |
|---|---|---|---|---|---|
| CR1 | Discovery workshop 2026-02-04 (Mara) | "Book an in-store appointment" from the locator — not in the signed SRD | scope+ (new UI flow here, plus a separate appointment integration) | confirm with client | pending |

## Open Questions

| Q# | Severity | Question | Why unclear | Tech note (if constrained) | Owner | Status |
|---|---|---|---|---|---|---|
| Q1 | NICE-TO-HAVE | Should the store detail surface special/holiday opening hours? | Depends on whether the commerce Stores API exposes holiday hours | Confirm hours availability with the store-data integration | BA / SA | Open |
| Q2 | NICE-TO-HAVE | What redirect (type and target) should the retired `/stores` URL use? | `/stores` is bookmarked/indexed; a 404 on retirement hurts SEO and shoppers | Redirect type/target to be decided in grooming | SA | Open |
| Q3 | NICE-TO-HAVE | PD↔SLS delivery boundary — are the Store Locator content fragment (`page.storelocator`) and Featured-Store promo card (`assets.storeCard`) delivered by the Page Designer FSD and consumed by the locator, and which promo content ships in the locator slot at v1? | The locator landing renders a PD-authored promo region, but the components are signed under the `PD` WBS family (a separate workstream); the ownership seam and the v1 promo content must be explicit for planning | The Page Designer library authors the slot; the locator only renders it — no locator-owned authoring | SA / client | Open |

## Cross-cutting Requirements

| Area | Applicable? | Requirement summary (if applicable) |
|---|---|---|
| Accessibility (WCAG 2.2) | yes | Keyboard operable, screen-reader friendly, accessible list alternative to the map (RQ-706) |
| SEO (meta, canonical, structured data, sitemap, noindex) | yes | Retiring `/stores` requires a redirect so the indexed/bookmarked URL does not 404 (Q2); per-store indexable landing pages remain a later phase |
| Consent / privacy (tracking gating, cookies, DNT) | yes | Geolocation is user-initiated and gated on the browser permission prompt; the storefront does not persist or send coordinates to analytics |
| i18n (user-facing strings) | yes | All copy via translation keys (RQ-707) |

## Delivery Scope & Targets

Scope: functional — delivers the storefront store-locator surface; the maps/geocoding, address-autocomplete, and commerce store-data capabilities are consumed as dependencies, not owned by this change.

| Type | Disposition | Confluence target | Working copy |
|---|---|---|---|
| FSD | NEW | TBD | `openspec/changes/add-store-locator/fsd/store-locator.md` |

## Discovery Gate

- [x] No CRITICAL open questions remain Open (Q1-Q3 are NICE-TO-HAVE and flow into grooming)
- [x] Resolved answers captured back into the Open Questions table (no CRITICAL items to resolve)
- [x] Executive Summary reflects discovery readiness without replacing the detailed sections
- [x] Document Consumption Guide matches the sections it projects (scope, WBS sets, capabilities, dependencies, open items)

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
