# add-cartora-maps-integration — Proposal

## Executive Summary

<!-- Reader map (not a summary): what this proposal settles, what it hands
     downstream, what stays open. By subject and count; do not re-list the tables
     below — that is the Document Consumption Guide's job. -->

Storefront features need maps and location search — the Store Locator is the first consumer — and today there is no governed way to use Cartora: keys, quota, billing, and privacy handling would otherwise be reinvented per feature. This change delivers Cartora map rendering plus postcode/city geocoding and proximity as an agnostic, reusable integration capability, with key, quota, billing, and privacy governance centralized in one place. The business value is unblocking location-based features while keeping vendor cost and key exposure under control.

**What this proposal settles.** Locks the discovery-reconciled integration-only scope as four consumer-value slices (one per in-scope WBS — geocoding with proximity, governed map rendering, and the two presentation obligations closed by consuming the vendor-hosted style configuration), each carrying its covering requirement's verbatim user story and near-final acceptance criteria. Discovery carries no CR candidates; the unconfirmed Cartora account/key setup is recorded as the blocking prerequisite.

**What it hands downstream.** The Delivery Breakdown is the single source of truth the ISD and specs project read-only — coverage is a bijection: one slice per in-scope requirement. The offer is deliberately thin — the service contract (endpoints, data mapping, quota/TTL) lives in the design's Integration Specification, not here. Out of Scope & Boundaries names the owner of every requirement this capability does not own — the functional Store Locator surface, store data, address autocomplete, the marketing heatmap, and the vendor-owned geocoding place data itself (Cartora's, consumed as returned).

**What stays open / readiness.** No new proposal-stage questions; discovery's Q1 (geocoding call approach) and Q2 (key/quota guardrails) stay registered upstream for grooming. The offer is ready for grooming.

## Document Consumption Guide

<!-- Map of THIS document's current sections: one row per ## section in document
     order (excludes Executive Summary, this guide, and Proposal Gate). Key data is
     identifiers/sets, `None` for an empty set — re-derived from the body. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Tracker | The ticket providing context; the sources record no dedicated ticket for this integration. | JIRA SFX-4102 (consumer-feature context) |
| Proposal Source Inventory | The upstream artifacts and key sources behind the lock; lets downstream re-open a source without re-discovery. | discovery, design, technical notes, BRD/SRD, review gate session 2026-02-19 |
| Requirements Lock | The locked scope boundary grooming, specs, and the ISD key off; one row per discovery trace row. | In scope: 3PI.MAP.GEO, 3PI.MAP.RND, 3PI.MAP.STY, 3PI.MAP.MRK; Out: 3PI.MAP.HEAT, SLS.* (functional), RQ-710 autocomplete |
| Change Request Dispositions | CR dispositions PROPOSED for grooming (proposing is not approving). | None — discovery carries no CR candidates for this integration |
| Re-usability Legend | The fixed marker key the client reads the same on every document; scopes the Acceptance Criteria marker groups. | 6 markers ([SF Next OOTB] … [New]) |
| Delivery Breakdown | The offer core: near-final per-slice blocks (user story + acceptance criteria) the ISD and specs project read-only; coverage is a bijection. | S1–S4 (4 slices = 4 in-scope WBS) |
| Out of Scope & Boundaries | What this capability does not own and who does. | Functional SLS.* UI; store data / Addressly / appointment; the directions hand-off; presentation authoring; heatmap (RQ-715); Cartora geocoding place data (vendor-owned) |
| External Dependencies & Access | Operational prerequisites grooming turns into prerequisites; each row leads with the slice it blocks. | Cartora API key — domain-restricted (S1–S4, Q2); Cartora commercial account — billing/quota; published Map Style ID (S2–S4) |
| Approach | Compression of the design + the ISD working copy and the testing obligation; the service contract lives in design's Integration Specification. | ISD NEW → isd/cartora-maps-integration.md |
| Open Questions | New proposal-stage Q#s only. | None (Q1/Q2 remain upstream for grooming) |
| Review & Approval | Whether a human approved proceeding to grooming. | Approved: no; approver: — |

## Tracker

JIRA `SFX-4102` — "Store Locator (find a store)": consumer-feature tracker context; the sources record no dedicated ticket for this integration change.

## Proposal Source Inventory

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-cartora-maps-integration/discovery.md` | Integration-only scope, per-requirement obligations, Q1–Q2 |
| Design (this change) | `openspec/changes/add-cartora-maps-integration/design.md` | Service capability shape, Integration Specification, operational prerequisites |
| Technical notes | `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md` | Cartora scope (rendering + geocoding/proximity), lazy-load and privacy constraints |
| BRD / SRD | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` | Signed RQ/WBS codes behind the lock, including the map-presentation rows (RQ-713–RQ-715) |
| Review gate session 2026-02-19 (SA Dmitry Ivanov) | `user request` (per-artifact review gate, Request changes) | Human decision taken during review: the vendor-owned geocoding place data recorded as an explicit boundary in Out of Scope & Boundaries |

## Requirements Lock

| WBS | RQ ID | Requirement summary | Disposition | Notes |
|---|---|---|---|---|
| 3PI.MAP.GEO | RQ-711 | Cartora geocoding & proximity ranking | in scope | Integration obligation this change owns; consumed by the functional store search (SLS.SRCH); client-vs-proxy call approach is Q1 |
| 3PI.MAP.RND | RQ-712 | Governed Cartora map-rendering surface | in scope | Integration obligation this change owns; consumed by the functional map UI (SLS.MAP); domain-restricted key, quota/billing on client account (Q2); its presentation comes from the consumed vendor-hosted style config (3PI.MAP.STY, 3PI.MAP.MRK) |
| 3PI.MAP.STY | RQ-713 | Brand-configured map style and map type (street/satellite) | in scope | Closed by CONSUMING the vendor-hosted presentation configuration and rendering it through 3PI.MAP.RND (delivery mode `consume`); the client brand/content team's authoring surface is the dependency, not work owned here; v1 renders that team's published "Brand Street" style, settled at the discovery review |
| 3PI.MAP.MRK | RQ-714 | Custom brand store markers/pins | in scope | Closed by CONSUMING the same published presentation configuration (delivery mode `consume`); no marker-authoring surface is owned here |
| 3PI.MAP.HEAT | RQ-715 | Store-density heatmap overlay | out of scope | Serves the marketing homepage — a different consumer surface owned by a separate change; not rendered by this integration and not a dependency of it |
| SLS.SRCH | RQ-701 | Search stores by postcode, city, or current location | out of scope | Functional search UI (Store Locator FSD); consumes the 3PI.MAP.GEO geocoding integration |
| SLS.MAP | RQ-702 | Map + synchronized results list with selection highlighting | out of scope | Functional map UI (Store Locator FSD); consumes the 3PI.MAP.RND rendering integration; the synchronized list and highlighting are functional UI, never specced here |
| SLS.DTL | RQ-703 | Store detail: name, address, phone, opening hours, services | out of scope | Functional store-detail surface owned by the Store Locator FSD change; its data is read from the existing, already-integrated commerce Stores API, so no integration change owns it |
| SLS.DIR | RQ-704 | "Get directions" opens the shopper's map provider | out of scope | Functional UI action (native device deep-link hand-off) owned by the Store Locator FSD change; no Cartora call |
| SLS.FLT | RQ-705 | Filter stores by service (click-and-collect, returns, gift wrapping) | out of scope | Functional service-filter surface owned by the Store Locator FSD change; the service attributes come from the existing, already-integrated commerce Stores API |
| SLS.A11Y | RQ-706 | Fully keyboard operable and screen-reader usable, with an accessible list alternative | out of scope | Functional accessibility requirement owned by the Store Locator FSD change |
| SLS.I18N | RQ-707 | All shopper-facing text localizable | out of scope | Functional copy requirement owned by the Store Locator FSD change |
| SLS.INV | RQ-708 | Real-time in-store product availability | out of scope | Later-phase inventory integration, separate from this capability |
| GH.UTL | RQ-709 | Locator reachable from the global site navigation | out of scope | Functional navigation requirement owned by the Store Locator FSD change |
| SLS.AC | RQ-710 | Address autocomplete while typing in store search | out of scope | Separate Addressly integration (different supplier, account, and change) |

## Change Request Dispositions

| CR# | Description | Impact | Proposed disposition | Status |
|---|---|---|---|---|
| None | — | — | — | Discovery carries no CR candidates for this integration; the appointment-booking ask belongs to the functional surface and a separate appointment integration |

## Re-usability Legend

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

## Delivery Breakdown

### S1 — 3PI.MAP.GEO — Geocoding with proximity ranking

- **Covers (WBS):** 3PI.MAP.GEO
- **Capability:** cartora-maps-integration
- **ISD:** isd/cartora-maps-integration.md
- **Depends on:** Cartora API key — domain-restricted (unconfirmed, Q2)

#### User Story

As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first.

#### Acceptance Criteria

##### [New]

1. The capability SHALL expose a stable interface that resolves a postal code or city to coordinates and returns proximity-ranked store results.
2. Consumers SHALL integrate against that interface without holding any Cartora key, quota, or billing.
3. The capability SHALL keep Cartora key, quota, and billing governance centralized so consuming features hold none of it.
4. The geocoding call approach (client-side vs server proxy) remains undecided (Q1).

### S2 — 3PI.MAP.RND — Governed map rendering

- **Covers (WBS):** 3PI.MAP.RND
- **Capability:** cartora-maps-integration
- **ISD:** isd/cartora-maps-integration.md
- **Depends on:** Cartora API key — domain-restricted (unconfirmed, Q2); published Cartora Map Studio Style ID

#### User Story

As a shopper, I want to see nearby stores on an interactive map, so that I can locate a store to visit.

#### Acceptance Criteria

##### [3rd party OOTB]

1. The capability SHALL render through the vendor's own map runtime as delivered, owning no map-drawing behaviour of its own; what the runtime draws changes with the vendor, not with this change.

##### [New]

1. The capability SHALL expose map rendering as a lazy-loadable entrypoint so consuming features keep maps off their critical path.
2. The capability SHALL keep the Cartora key domain-restricted, with quota and billing governed on the client's Cartora account (guardrails per Q2).
3. The capability SHALL keep shopper coordinates transient — never persisted and never sent to analytics.

### S3 — 3PI.MAP.STY — Consumed map presentation (style and map type)

- **Covers (WBS):** 3PI.MAP.STY
- **Capability:** cartora-maps-integration
- **ISD:** isd/cartora-maps-integration.md
- **Depends on:** published Cartora Map Studio Style ID; Cartora API key — domain-restricted (unconfirmed, Q2)

#### User Story

As a shopper, I see the store map in the brand's configured style and map type (street/satellite).

#### Acceptance Criteria

The presentation itself — map type and brand theme — is authored and published OUTSIDE this change, on the vendor's hosted no-code surface, by the client's brand/content team. This slice delivers only how the capability references that published presentation and how it behaves when the presentation changes or cannot be resolved.

##### [3rd party OOTB]

1. The capability SHALL render its map surface against the published presentation configuration it is pointed at, so the configured map type and brand theme reach the storefront with no storefront release.
2. The capability SHALL hold a reference to that presentation and never a copy of it, so a re-published presentation reaches the storefront without a code change.
3. When the referenced presentation cannot be resolved, the capability SHALL still expose a usable map surface on the vendor's default presentation rather than failing the consuming feature's render.
4. The v1 presentation configuration SHALL be the brand/content team's published "Brand Street" style, which that team authors and owns.

### S4 — 3PI.MAP.MRK — Consumed store markers

- **Covers (WBS):** 3PI.MAP.MRK
- **Capability:** cartora-maps-integration
- **ISD:** isd/cartora-maps-integration.md
- **Depends on:** published Cartora Map Studio Style ID; Cartora API key — domain-restricted (unconfirmed, Q2)

#### User Story

As a shopper, I see stores on the map with the brand's custom markers/pins.

#### Acceptance Criteria

The marker artwork is authored and published OUTSIDE this change, inside the same vendor-hosted presentation the client's brand/content team owns. This slice delivers only how the capability renders the published marker set.

##### [3rd party OOTB]

1. The capability SHALL plot store locations with the marker set carried by the published presentation, so brand-authored markers reach the storefront with no storefront release.
2. The capability SHALL define no marker artwork of its own, so the published marker set stays the single source for how a store appears on the map.
3. When the published presentation carries no custom marker set, the capability SHALL still plot store locations using the vendor's default marker.

## Out of Scope & Boundaries

- Consumer UI/UX (owned by the functional Store Locator change, `SLS.*`): the store search UI (SLS.SRCH) and the map + synchronized results list with map/list selection highlighting (SLS.MAP), plus store detail, service filters, navigation, accessibility, and shopper-facing copy. These consume this integration; they are not part of it.
- Store data (commerce Stores API), address autocomplete (Addressly), and appointment booking (SlotBook) — separate services owned by separate changes.
- The "Get directions" hand-off: a native device deep-link owned by the functional surface; it does not call Cartora.
- Authoring the map presentation (map type, brand theme, marker artwork) and publishing it: a vendor-hosted, no-code surface the client's brand/content team owns. S3 and S4 consume its published result; this change never authors or re-implements it.
- The store-density heatmap overlay (RQ-715): it serves the marketing homepage, a different consumer surface owned by a separate change.
- The underlying geocoding place data and its accuracy: owned by Cartora as the vendor. This change consumes the geocoding result Cartora returns and owns no place-data curation, correction, or coverage surface; a wrong or missing place is a vendor-data matter, not a defect of this capability (distinct from the Q1 call approach, which stays open for grooming).

## External Dependencies & Access

| Dependency / Access | Needed for | Owner / Source | Status |
|---|---|---|---|
| Cartora API key — domain-restricted to the storefront origins | S1–S4 — the capability cannot call Cartora without a ready, restricted key | Client platform developer | unconfirmed (Q2) — confirm before build |
| Cartora commercial account — billing and usage quota for the Cartora calls | Quota and billing governance for every Cartora call the capability makes | Client account / procurement owner | client-owned; quota and billing guardrails pending (Q2) |
| Published Cartora Map Studio Style ID — the vendor-hosted presentation (map type, brand theme, marker set) the connector renders by reference | S2, S3, S4 (RQ-712, RQ-713, RQ-714 — the presentation the capability renders) | Client brand/content team, authored on the vendor's hosted surface | v1 "Brand Street" style is settled (discovery review); the published Style ID must be handed to the build before first render |

## Approach

The capability lands as designed: Cartora as the single vendor for both map rendering and geocoding/proximity, exposed as an agnostic, lazy-loadable capability that centralizes key, quota, billing, and privacy governance; consumers integrate against the stable interface only. Map presentation is not built here — the capability references the client-published, vendor-hosted configuration, so style and markers change without a storefront release. The service contract — endpoints, data mapping, data flow, and quota/TTL guidance — is specified in the design's Integration Specification and is not restated here. The geocoding call approach (Q1) and the key/quota guardrails (Q2) stay undecided and registered for grooming; the v1 presentation configuration and its owner were settled at the discovery review.

Delivery follows discovery's Delivery Scope & Targets: a NEW ISD working copy at `openspec/changes/add-cartora-maps-integration/isd/cartora-maps-integration.md`. Test coverage applicable to the capability is part of every slice; specs and publication artifacts follow grooming approval.

## Open Questions

| Q# | Question | Owner | Status |
|---|---|---|---|
| None | No new proposal-stage questions — the lock, the four-slice breakdown, and the dependency verification raised none; Q1 and Q2 from discovery remain registered for grooming | — | — |

## Proposal Gate

- [x] Requirements Lock mirrors every discovery trace row exactly once with inherited dispositions
- [x] Delivery Breakdown covers every in-scope requirement and only in-scope requirements; every slice traces to the lock
- [x] No scope absent from discovery was introduced (new asks are CR candidates in discovery first)
- [x] Every Change Request disposition is proposed for grooming, none approved here
- [x] Every External Dependencies & Access row is verified against the upstream artifacts or explicitly marked UNVERIFIED
- [x] Slices and Approach inherit the design without re-deciding architecture or adding implementation detail

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
