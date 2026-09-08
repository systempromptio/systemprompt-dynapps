# add-store-locator — Proposal

## Executive Summary

<!-- Reader map (not a summary): what this proposal settles, what it hands
     downstream, what stays open. By subject and count; do not re-list the tables
     below — that is the Document Consumption Guide's job. -->

The storefront's only store-finding surface today is a legacy static directory at `/stores` — no search, no map, no live store data. This change delivers a modern store locator: shoppers search by postcode, city, or their current location, explore results on an interactive map synchronized with an accessible list, see each store's details and services, filter by service, and hand off to their own map app for directions. The static directory is retired at launch. The business value is reduced store-finding friction, stronger support for click-and-collect journeys, and more qualified visits to physical stores.

**What this proposal settles.** Locks the discovery-reconciled functional scope as nine shopper-value slices (one per in-scope requirement — store search, address autocomplete, the interactive map, store detail, service filter, directions hand-off, accessibility, localized copy, and the navigation entry with legacy retirement), each carrying its inherited user story and near-final acceptance criteria. CR1 (in-store appointment booking) is carried as a proposed disposition for grooming; three consumed capabilities are recorded as external dependencies.

**What it hands downstream.** The Delivery Breakdown is the single source of truth the FSD and specs project read-only — coverage is a bijection: one slice per in-scope requirement, no grouping or splitting. Out of Scope & Boundaries names the owner of everything no slice delivers — the inventory integration workstream (RQ-708), the Page Designer workstream (RQ-751/752/753), the maps/autocomplete/store-data integration changes, and store-record authoring (the client's store-ops team via the commerce Stores API).

**What stays open / readiness.** No new proposal-stage questions; discovery's Q1 (holiday-hours display) and design's Q2 (retired-URL redirect) stay registered upstream for grooming. The offer is ready for grooming.

## Document Consumption Guide

<!-- Map of THIS document's current sections: one row per ## section in document
     order (excludes Executive Summary, this guide, and Proposal Gate). Key data is
     identifiers/sets, `None` for an empty set — re-derived from the body. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Tracker | The ticket this offer is booked under; downstream links its work items to it. | JIRA SFX-4102 |
| Proposal Source Inventory | The upstream artifacts and key sources behind the lock; lets downstream re-open a source without re-discovery. | discovery, design, BRD/SRD, review gate session 2026-02-19 |
| Requirements Lock | The locked scope boundary grooming, specs, and the FSD key off; one row per discovery trace row. | In scope: SLS.SRCH, SLS.MAP, SLS.DTL, SLS.DIR, SLS.FLT, SLS.A11Y, SLS.I18N, GH.UTL, SLS.AC; Out: SLS.INV, PD.PT.SLP, PD.CMP.STC, PD.CMP.HMH |
| Change Request Dispositions | CR dispositions PROPOSED for grooming (proposing is not approving). | CR1 — declined this release, carried as a separate future change |
| Re-usability Legend | The fixed marker key the client reads the same on every document; scopes the Acceptance Criteria marker groups. | 6 markers ([SF Next OOTB] … [New]) |
| Delivery Breakdown | The offer core: near-final per-slice blocks (user story + acceptance criteria) the FSD and specs project read-only; coverage is a bijection. | S1–S9 (9 slices = 9 in-scope WBS) |
| Out of Scope & Boundaries | What no slice delivers and who owns it; nothing here may appear in a slice. | RQ-708 inventory; CR1 appointment; consumed vendor contracts (Cartora, Addressly, Stores API); PD promo (RQ-751/752/753); store-record authoring (client store-ops) |
| External Dependencies & Access | Build prerequisites grooming turns into prerequisites; each row leads with the slice it blocks. | Cartora maps/geocoding (S1, S3); Addressly autocomplete (S2); Commerce Stores API (S4, S5) |
| Approach | Compression of the design + the FSD working copy and the testing obligation; no re-decided architecture. | FSD NEW → fsd/store-locator.md |
| Open Questions | New proposal-stage Q#s only. | None (Q1/Q2 remain upstream for grooming) |
| Review & Approval | Whether a human approved proceeding to grooming. | Approved: no; approver: — |

## Tracker

JIRA `SFX-4102` — "Store Locator (find a store)" (project-context `jira`: SFX-4102).

## Proposal Source Inventory

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-store-locator/discovery.md` | Requirements trace and dispositions, dependencies, CR1, Q1–Q2, delivery targets |
| Design (this change) | `openspec/changes/add-store-locator/design.md` | Solution shape, consumed-capability boundaries, risks |
| BRD / SRD | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` | Signed WHAT and RQ/WBS codes behind the Requirements Lock |
| Review gate session 2026-02-19 (BA Elena Sokolova) | `user request` (per-artifact review gate, Request changes) | Human decision taken during review: store-record ownership recorded as an explicit boundary in Out of Scope & Boundaries |

## Requirements Lock

| WBS | RQ ID | Requirement summary | Disposition | Notes |
|---|---|---|---|---|
| SLS.SRCH | RQ-701 | Search stores by postcode, city, or current location | in scope | Geolocation user-initiated and permission-gated; geocoding consumed from the maps/geocoding dependency |
| SLS.MAP | RQ-702 | Interactive map synchronized with the store list | in scope | Map rendering consumed from the maps/geocoding dependency; lazy-loaded per design |
| SLS.DTL | RQ-703 | Store detail: name, address, phone, opening hours, services | in scope | Store data from the commerce Stores API; holiday-hours display is Q1 |
| SLS.DIR | RQ-704 | "Get directions" opens the shopper's map provider | in scope | Native device deep-link hand-off per design; no maps-vendor call |
| SLS.FLT | RQ-705 | Filter stores by service (click-and-collect, returns, gift wrapping) | in scope | v1 service list fixed; service attributes from the commerce Stores API |
| SLS.A11Y | RQ-706 | Fully keyboard operable and screen-reader usable, with an accessible list alternative | in scope | WCAG 2.2 AA target |
| SLS.I18N | RQ-707 | All shopper-facing text localizable | in scope | — |
| SLS.INV | RQ-708 | Real-time in-store product availability | out of scope | Owned end to end by the inventory integration workstream — not a later phase of this change (disposition corrected at the discovery review) |
| GH.UTL | RQ-709 | Locator reachable from the global site navigation | in scope | Footer "Our stores" re-pointed from `/stores`; retired-URL redirect is Q2 |
| SLS.AC | RQ-710 | Address autocomplete while typing in store search | in scope | Suggestions consumed from the address-autocomplete dependency (separate supplier from maps) |
| PD.PT.SLP | RQ-751 | Store Locator content fragment (Page Designer) | out of scope | Owned by the Page Designer FSD (`PD` WBS family, a separate workstream); the locator landing RENDERS that PD-authored promo region — consumed, not owned; v1 promo content was settled at the discovery review as a single curated Featured-Store card, no editorial fragment |
| PD.CMP.STC | RQ-752 | Featured-Store promo card (Page Designer) | out of scope | Owned by the Page Designer FSD; rendered by the locator landing — consumed, not owned |
| PD.CMP.HMH | RQ-753 | Homepage hero banner (Page Designer) | out of scope | Owned by the Page Designer FSD; homepage merchandising, not used by the locator |

## Change Request Dispositions

| CR# | Description | Impact | Proposed disposition | Status |
|---|---|---|---|---|
| CR1 | "Book an in-store appointment" from the locator — workshop ask, not in the signed SRD | scope+ (new UI flow here plus a separate appointment-booking integration) | none to propose — the client already answered | declined for this release at the discovery review; carried as a separate future change |

## Re-usability Legend

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

## Delivery Breakdown

### S1 — SLS.SRCH — Store search by postal code, city, or location

- **Covers (WBS):** SLS.SRCH
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** Cartora maps/geocoding capability

#### User Story

As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.

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

### S2 — SLS.AC — Address autocomplete suggestions

- **Covers (WBS):** SLS.AC
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** Addressly address-autocomplete capability

#### User Story

As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions.

#### Acceptance Criteria

The suggestions themselves come from the address-autocomplete capability a separate change delivers; this slice delivers the type-ahead surface in the store search and how it behaves when suggestions do not arrive.

##### [New]

1. While the shopper types a postal code or partial address, the store search SHALL offer address suggestions; selecting a suggestion SHALL run the search for that address.
2. If suggestions are unavailable, the typed search SHALL still submit and return results.

### S3 — SLS.MAP — Interactive map with a synchronized list

- **Covers (WBS):** SLS.MAP
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** Cartora maps/geocoding capability

#### User Story

Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other.

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

### S4 — SLS.DTL — Store details

- **Covers (WBS):** SLS.DTL
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** Commerce Stores API (store data)

#### User Story

Each store SHALL display its name, full address, phone number, opening hours and the services available at that store.

#### Acceptance Criteria

The store records behind the detail (name, address, phone, opening hours, services) are maintained by the client's team in the commerce backend and read through the already-integrated Stores API — this change neither owns nor authors them. This slice delivers the detail surface that presents whatever each record exposes.

##### [New]

1. The store detail SHALL show each store's name, full address, phone number, regular weekly opening hours, and the services available at that store; holiday-hours display follows what the commerce Stores API exposes (Q1).

### S5 — SLS.FLT — Filter stores by service

- **Covers (WBS):** SLS.FLT
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** Commerce Stores API (service attributes)

#### User Story

The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping.

#### Acceptance Criteria

##### [SF Next OOTB]

1. Business users SHALL be able to configure which services are offered as filter options in Business Manager; v1 is seeded with click-and-collect, returns, and gift wrapping.

##### [New]

1. The service filter SHALL let the shopper filter stores by the services offered (in v1: click-and-collect, returns, and gift wrapping).
2. Applying a filter SHALL update both the list and the map; clearing the filter SHALL restore the full search results.
3. When a filter matches no stores, the result view SHALL show an empty-results message for the filtered view.

### S6 — SLS.DIR — Get directions

- **Covers (WBS):** SLS.DIR
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** —

#### User Story

Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination.

#### Acceptance Criteria

##### [New]

1. Each store SHALL provide a "Get directions" action that opens the shopper's own map provider with the store as the destination.

### S7 — SLS.A11Y — Accessible, keyboard-operable store locator

- **Covers (WBS):** SLS.A11Y
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** —

#### User Story

The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map.

#### Acceptance Criteria

##### [New]

1. The store locator SHALL be fully keyboard operable and usable with a screen reader, with the list as an accessible alternative to the map (WCAG 2.2 AA).

### S8 — SLS.I18N — Localization

- **Covers (WBS):** SLS.I18N
- **Capability:** store-locator
- **FSD:** fsd/store-locator.md
- **Depends on:** —

#### User Story

All shopper-facing text in the store locator SHALL be localizable.

#### Acceptance Criteria

##### [SF Next OOTB]

1. All shopper-facing text in the store locator SHALL be localizable through the storefront's existing translation mechanism and SHALL be shown in the shopper's active locale.

### S9 — GH.UTL — Navigation entry and retired directory URL

- **Covers (WBS):** GH.UTL
- **Capability:** global-navigation; store-directory
- **FSD:** fsd/store-locator.md
- **Depends on:** —

#### User Story

The store locator SHALL be reachable from the global site navigation.

#### Acceptance Criteria

##### [SF Next Modified]

1. The footer "Our stores" link under "About" SHALL point to the store locator page instead of the static directory.

##### [Legacy Modified]

1. The static store directory page SHALL be retired at launch, with no shopper-facing entry to it left in the storefront.
2. The retired `/stores` URL SHALL keep working for bookmarked and indexed links by redirecting to the locator instead of returning a not-found page (redirect type/target per Q2).

## Out of Scope & Boundaries

- Real-time in-store product availability (RQ-708 — owned end to end by the inventory integration workstream; no slice delivers it).
- In-store appointment booking (CR1 — unsigned, declined for this release and carried as a separate future change).
- The maps/geocoding (Cartora), address-autocomplete (Addressly), and commerce store-data service contracts: consumed as dependencies; their keys, quotas, billing, and call approach are owned by the separate integration changes.
- The Page Designer components the locator landing renders (RQ-751 Store Locator content fragment, RQ-752 Featured-Store promo card) — authored and delivered by the Page Designer FSD workstream; the locator renders the slot and owns no promo authoring. RQ-753's homepage hero is that workstream's too and the locator does not use it.
- Store record authoring and maintenance (store name, address, phone, opening hours, services): owned by the client's store-ops team in the commerce backend and read through the already-integrated Stores API. The locator reads and presents whatever each record exposes; it never authors, edits, or maintains the records themselves (distinct from the Stores API service contract above, which the separate integration owns).

## External Dependencies & Access

| Dependency / Access | Needed for | Owner / Source | Status |
|---|---|---|---|
| Cartora maps/geocoding capability | S1, S3 (RQ-701, RQ-702 — geocoding, proximity, map rendering) | Separate Cartora integration change | to be delivered (separate change) |
| Addressly address-autocomplete capability | S2 (RQ-710 — search suggestions) | Separate Addressly integration change | to be delivered (separate change) |
| Commerce Stores API (store data) | S4, S5 (RQ-703, RQ-705 — store detail and service attributes) | Existing commerce backend service | have (already integrated) |

## Approach

The locator lands as designed: a server-rendered page shell with an accessible, list-first result view and a lazily loaded map, store data read from the existing commerce Stores API, directions as a native hand-off, and the maps/geocoding and address-autocomplete capabilities consumed across stable feature boundaries — the feature holds no vendor keys, quota, or billing. Decisions and rationale live in `design.md` and are not re-decided here; holiday hours (Q1) and the retired-URL redirect (Q2) stay open for grooming.

Delivery follows discovery's Delivery Scope & Targets: a NEW FSD working copy at `openspec/changes/add-store-locator/fsd/store-locator.md`. Unit and E2E tests are part of every slice; specs and publication artifacts follow grooming approval.

## Open Questions

| Q# | Question | Owner | Status |
|---|---|---|---|
| None | No new proposal-stage questions — the lock, the slicing, and the dependency verification raised none; Q1 and Q2 from discovery remain registered for grooming | — | — |

## Proposal Gate

- [x] Requirements Lock mirrors every discovery trace row exactly once with inherited dispositions
- [x] Delivery Breakdown is 1:1 with the in-scope requirements — each in-scope requirement delivered by exactly one slice and each slice delivers exactly one in-scope requirement (no grouping, no splitting); every slice traces to the lock
- [x] No scope absent from discovery was introduced (new asks are CR candidates in discovery first)
- [x] Every Change Request disposition is proposed for grooming, none approved here
- [x] Every External Dependencies & Access row is verified against the upstream artifacts or explicitly marked UNVERIFIED
- [x] Slices and Approach inherit the design without re-deciding architecture or adding implementation detail

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
