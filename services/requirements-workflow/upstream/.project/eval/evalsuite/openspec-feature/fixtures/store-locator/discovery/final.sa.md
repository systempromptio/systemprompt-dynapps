# add-cartora-maps-integration - Discovery & Requirements

## Executive Summary

This change introduces an agnostic Cartora maps integration capability for the storefront: map rendering with the Cartora maps SDK plus postcode/city geocoding and proximity through Cartora. Its business value is to give storefront features a reusable, governed way to present maps and location search without each feature reinventing API keys, billing constraints, privacy handling, or the geocoding call approach.

**What this file settles.** The delivery scope is integration: this change owns the Cartora maps/geocoding service capability itself, not a store-locator-owned implementation. Four obligations are in scope — map rendering and geocoding/proximity are built here, while map style/type and custom markers are closed by consuming the vendor-hosted Cartora Map Studio Style ID rather than by building a configuration surface. Requirements Trace holds each requirement's disposition and delivery mode, and names the owning workstream for every row this change does not own. Two vendor-side dependencies gate the capability: the published presentation style, and the account and restricted key the connector calls with.

**What stays out.** The functional Store Locator page, the separate Addressly address-autocomplete integration, the separate SlotBook appointment-booking integration, and the commerce Stores data service are separate changes, captured only as consumer context. The store-density heatmap serves the marketing homepage — a different consumer surface — and is listed for source fidelity only.

**What stays open.** Two refinements are registered in Open Questions — whether typed postcodes are geocoded client-side or through a server proxy, and which domain and quota guardrails the Cartora key uses. Both are NICE-TO-HAVE and routed to grooming. The map presentation boundary is no longer among them: it was settled at review, and the connector renders a published vendor-hosted style it does not author.

**Readiness.** No CRITICAL question is open, so discovery is ready to move into proposal. The Document Consumption Guide is the machine-readable form of everything the next artifacts consume.

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Requirements Trace | The signed scope boundary: the map obligations this change owns, and the store-locator / other workstreams it does not; downstream slices and spec headings key off it. | In scope: 3PI.MAP.GEO, 3PI.MAP.RND, 3PI.MAP.STY, 3PI.MAP.MRK; Deferred: None; Out of scope: SLS.SRCH, SLS.MAP, SLS.DTL, SLS.DIR, SLS.FLT, SLS.A11Y, SLS.I18N → store-locator; SLS.INV → inventory-integration; GH.UTL → global-navigation; SLS.AC → addressly-integration; 3PI.MAP.HEAT → marketing-homepage |
| Affected Capabilities | Which capabilities this change owns and how; fixes the specs file inventory. | cartora-maps-integration ADD |
| Dependencies | Vendor-side services consumed but not owned; become proposal external-dependency rows. | Cartora Map Studio presentation config (published Map Style ID), Cartora account + domain-restricted API key |
| Open Questions | Unresolved refinements gating readiness; the only source of Q# downstream. | Q1 — NICE-TO-HAVE, Open; Q2 — NICE-TO-HAVE, Open |
| Change Request Candidates | Asks against signed scope; seed proposal CR dispositions. | None |
| Cross-cutting Requirements | Applicable cross-cutting areas; required design config rows. | Consent / privacy |
| Delivery Scope & Targets | The resolved scope and what is published where. | integration; ISD NEW → `openspec/changes/add-cartora-maps-integration/isd/cartora-maps-integration.md` |

## Discovery Source Inventory

| Source | Reference / Anchor | Purpose |
|---|---|---|
| BRD / SRD | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` | Consumer feature context that motivates the maps/geocoding integration; do not turn Store Locator UI scope into this ISD |
| Technical notes / integration | `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md` | Cartora maps SDK, Cartora geocoding/proximity, lazy-load constraint, privacy constraint, and no persisted shopper coordinates |
| Meeting notes | `.cursor/.project/project-context/inbox/meeting_notes/2026-02-04_storefront-workshop_transcript.txt` | Possible existing Cartora account, domain-restricted key, client-vs-proxy geocoding question, and the privacy constraint |
| Delivery scope decision | `user request` (architect scope: build the Cartora maps integration) | Confirms integration-only scope for the selected Cartora service; Store Locator is a consumer, not the owner of the integration |
| Existing OpenSpec baseline specs | project-context `specs`: no existing `openspec/specs/cartora-maps-integration/spec.md` found | Confirms the Cartora maps integration capability is new (ADD), not an extension of an existing spec |
| Store Locator functional change | `user request` (functional storefront UI is a separate change owned by others) | Downstream consumer context only; the storefront UI/UX is delivered by a separate FSD change |
| Addressly / SlotBook / commerce Stores data | `.cursor/.project/project-context/inbox/meeting_notes/2026-02-04_storefront-workshop_transcript.txt` | Other services named in the sources; each is a separate change and explicitly NOT part of this Cartora ISD |
| Jira | project-context `jira`: SFX-4102 | Tracker context only; do not infer extra requirements beyond the cited source packet |
| ISD source lookup | project-context `confluence`: no matching Cartora maps integration ISD found | No existing integration source page to cite or extend; publication target decision is recorded separately |
| Prior-art — marketplace cartridge | project-context `web`: `https://marketplace.example/cartridges/cartora-commerce-maps` | Existing reuse candidate: a "Cartora Commerce Maps" cartridge advertising drop-in RSC map rendering + geocoding/proximity for Storefront Next (marketplace feature blurb). A FACT for design's build-vs-reuse decision; no approach chosen at discovery |
| Prior-art — vendor SDK / API docs | project-context `web`: `https://developers.cartora.example/docs/{maps-sdk,geocoding,keys}` | Cartora Maps SDK (client-side rendering) + Geocoding API (`…/v1/geocode` → `results[].{lat,lng,score}`) + key/quota policy — grounds the SDK and from-scratch options for design |
| Approach preference | `user request` (architect: build from scratch against Cartora's SDK/API, not a cartridge) | Recorded as a design input only; discovery notes it and chooses NO build-vs-reuse approach (that is design's Solution Approach decision) |
| Cartora Map Studio (vendor-hosted map presentation) | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` (SRD map-presentation rows RQ-713/714/715) | Signed vendor-hosted presentation config (map type, custom markers, brand theme) authored by the client brand/content team and published as a Map Style ID. The store-locator map style/markers (RQ-713/714) are consumed by RQ-712 and closed by this change via consume (mode consume), with Map Studio itself as a dependency; the RQ-715 heatmap serves the separate marketing homepage |

## Requirements Trace

<!-- Golden annotation: on `out of scope` / `deferred` rows the `Affected capability`
cell is a NORMALIZED, best-effort attribution of the owning workstream, not source
truth. Grade those rows on the in/out disposition; any owner a candidate can ground
in the context is acceptable. Only in-scope rows are graded against the context's
capability assignment. -->

Signed source requirements (15): `RQ-711`, `RQ-712`, `RQ-701`, `RQ-702`, `RQ-703`, `RQ-704`, `RQ-705`, `RQ-706`, `RQ-707`, `RQ-708`, `RQ-709`, `RQ-710`, `RQ-713`, `RQ-714`, `RQ-715`

| RQ ID | WBS | Title | Source requirement | Requirement | Requirement Details | Disposition | Delivery mode | Affected capability | Notes |
|---|---|---|---|---|---|---|---|---|---|
| RQ-711 | 3PI.MAP.GEO | Cartora geocoding & proximity | SRD RQ-711 | As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first. | 1. Resolve a typed postal code or city (or a user-initiated current location) to coordinates via Cartora. 2. Rank stores by proximity to the resolved location. 3. Shopper coordinates are never persisted or sent to analytics; client-side vs server-proxy call approach is undecided. | in scope | build | cartora-maps-integration | The integration obligation this change owns: Cartora geocoding + proximity ranking, consumed by the Store Locator search UI (SLS.SRCH). The connector builds the geocoding/proximity call (mode build). Client-vs-server call approach is open — see Q1. |
| RQ-712 | 3PI.MAP.RND | Cartora map rendering | SRD RQ-712 | As a shopper, I want to see nearby stores on an interactive map, so that I can locate a store to visit. | 1. Render store locations on an interactive map surface via the Cartora Maps SDK. 2. Lazy-load the map so it does not block the critical render path. 3. Domain-restricted key; quota and billing sit on the client Cartora account. | in scope | build | cartora-maps-integration | The integration obligation this change owns: the Cartora map-rendering surface, consumed by the Store Locator map UI (SLS.MAP). The synchronized list and map/list highlighting stay functional UI (Store Locator FSD) and are never specced here. Map presentation (map type, markers, brand theme) is delivered by consuming the vendor-hosted Cartora Map Studio Style ID (RQ-713, RQ-714, in scope via consume); the Map Studio authoring surface is the dependency — see Dependencies. The v1 style and its owner were settled at review. |
| RQ-701 | SLS.SRCH | Store search | SRD RQ-701 | As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location. | 1. Search inputs: postal code, city, or current location. 2. Current-location search uses device geolocation and is user-initiated. | out of scope | — | store-locator | Functional search UI owned by the Store Locator FSD; it CONSUMES the Cartora geocoding/proximity integration (3PI.MAP.GEO), which is the obligation this change owns. |
| RQ-702 | SLS.MAP | Map + synchronized list | SRD RQ-702 | Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other. | 1. Present results as an interactive map and a synchronized results list. 2. Selecting a store in the list highlights it on the map, and vice versa. | out of scope | — | store-locator | Functional map UI — including the synchronized list and map/list selection highlighting — owned by the Store Locator FSD; it CONSUMES the Cartora map-rendering integration (3PI.MAP.RND), which is the obligation this change owns. |
| RQ-703 | SLS.DTL | Store detail | SRD RQ-703 | Each store SHALL display its name, full address, phone number, opening hours and the services available at that store. | 1. Display store name, full address, phone number, opening hours, and available services. 2. Store data is read from the commerce backend; no new store data model for v1. | out of scope | — | store-locator | Functional store-detail surface owned by the Store Locator FSD; its store data is read from the existing, already-integrated commerce Stores API, so no integration change owns this row (attribution corrected at review). |
| RQ-704 | SLS.DIR | Get directions | SRD RQ-704 | Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination. | 1. Provide a "Get directions" action for each store. 2. Opens the shopper's map provider with the store set as the destination. | out of scope | — | store-locator | Functional UI action owned by the Store Locator FSD; opens the shopper's device/native map provider, not a Cartora integration obligation. |
| RQ-705 | SLS.FLT | Filter by service | SRD RQ-705 | The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping. | 1. Filter stores by available service. 2. v1 services: click-and-collect, returns, and gift wrapping. | out of scope | — | store-locator | Functional service-filter surface owned by the Store Locator FSD; the service attributes come from the existing, already-integrated commerce Stores API, not from Cartora and not from an integration change (attribution corrected at review). |
| RQ-706 | SLS.A11Y | Accessibility | SRD RQ-706 | The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map. | — | out of scope | — | store-locator | Functional accessibility requirement owned by the Store Locator FSD. |
| RQ-707 | SLS.I18N | Localizable copy | SRD RQ-707 | All shopper-facing text in the store locator SHALL be localizable. | — | out of scope | — | store-locator | Functional copy requirement owned by the Store Locator FSD. |
| RQ-708 | SLS.INV | Real-time in-store inventory | SRD RQ-708 | The store detail SHALL show real-time in-store product availability. | 1. Show real-time in-store product availability on the store detail. | out of scope | — | inventory-integration | Later-phase inventory integration; not part of the Cartora maps integration. |
| RQ-709 | GH.UTL | Reachable from navigation | SRD RQ-709 | The store locator SHALL be reachable from the global site navigation. | — | out of scope | — | global-navigation | Functional navigation requirement owned by the Store Locator FSD. |
| RQ-710 | SLS.AC | Address autocomplete | SRD RQ-710 | As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions. | 1. As the shopper types a postal code or partial address, offer address suggestions. 2. Suggestions come from the address type-ahead supplier (see workshop). | out of scope | — | addressly-integration | Address autocomplete is a separate third-party integration (Addressly) — a different supplier and a different change from this Cartora integration; do not fold it in. |
| RQ-713 | 3PI.MAP.STY | Map style & type | SRD RQ-713 | As a shopper, I see the store map in the brand's configured style and map type (street/satellite). | 1. Map type (street/satellite) and brand theme are configured in Cartora Map Studio (vendor-hosted, no-code) and published as a Map Style ID. 2. The connector renders by referencing the published Style ID; it builds no configuration surface. | in scope | consume | cartora-maps-integration | This change closes the map style/type obligation by CONSUMING the vendor-hosted Cartora Map Studio Style ID and rendering it through RQ-712 (delivery mode consume); no separate change owns this presentation for the store-locator map. The Map Studio authoring surface (client brand/content team) is the dependency — see Dependencies. Settled at review: v1 renders the brand team's published "Brand Street" style (street map type, brand palette, custom pin set), authored and owned by that team. |
| RQ-714 | 3PI.MAP.MRK | Custom store markers/pins | SRD RQ-714 | As a shopper, I see stores on the map with the brand's custom markers/pins. | 1. A custom store marker/pin set is authored in Cartora Map Studio and published in the same Map Style ID. 2. The connector renders the published markers; no marker-authoring UI is built here. | in scope | consume | cartora-maps-integration | This change closes the custom-marker presentation by CONSUMING the published Map Style ID and rendering it through RQ-712 (delivery mode consume). The Map Studio authoring surface is the dependency — see Dependencies. |
| RQ-715 | 3PI.MAP.HEAT | Store-density heatmap overlay | SRD RQ-715 | As a marketer, I want a store-density heatmap overlay for the marketing homepage. | 1. Cartora Map Studio can publish a store-density heatmap overlay used by the marketing homepage. | out of scope | — | marketing-homepage | Serves the marketing homepage — a DIFFERENT consumer surface owned by a separate change — and is not rendered by this store-locator maps integration. Listed for source fidelity only; it is not a dependency of this change. |

## Affected Capabilities

Touched capabilities (1): `cartora-maps-integration`

| Capability | Spec path | Disposition |
|---|---|---|
| cartora-maps-integration | — (does not exist yet) | ADD (new) — agnostic Cartora map-rendering + geocoding/proximity capability for any storefront consumer |

## Dependencies

| Dependency | Type | Status | Used by |
|---|---|---|---|
| Cartora Map Studio presentation config (map type, custom markers/pins, brand theme → published Map Style ID) | platform config | vendor-hosted; authored by the client brand/content team; v1 renders their published "Brand Street" style, settled at the discovery review by Dane Whitfield (SA lead) — available, the connector references the published ID and builds no configuration surface | RQ-712, RQ-713, RQ-714 |
| Cartora account + domain-restricted API key (quota and billing on the client's Cartora account) | credentials | client-administered; unconfirmed — the restriction and quota policy is Q2 | RQ-711, RQ-712 |

## Change Request Candidates

None. The appointment-booking idea belongs to the functional Store Locator surface and a separate appointment integration; it is not a change to this Cartora maps integration's scope.

## Open Questions

| Q# | Severity | Question | Why unclear | Tech note (if constrained) | Owner | Status |
|---|---|---|---|---|---|---|
| Q1 | NICE-TO-HAVE | Geocode typed postcodes client-side with Cartora or via a server proxy? | Affects Cartora API key exposure, billing, and where request throttling/error handling lives | Undecided; to be resolved in grooming | SA | Open |
| Q2 | NICE-TO-HAVE | Which domain restrictions and quota/billing guardrails should the Cartora key use? | A Cartora account may already exist, but exact key policy is not documented and set-up is unconfirmed | Confirm with the client's Cartora account owner during grooming | SA | Open |

## Cross-cutting Requirements

| Area | Applicable? | Requirement summary (if applicable) |
|---|---|---|
| Accessibility (WCAG 2.2) | N/A | No client-facing UI is produced by this integration; accessible map/list behaviour is owned by consumer FSDs |
| SEO (meta, canonical, structured data, sitemap, noindex) | N/A | No indexable page or route is produced by this integration |
| Consent / privacy (tracking gating, cookies, DNT) | yes | Shopper geolocation is user-initiated by consumers; coordinates are never persisted or sent to analytics by this integration |
| i18n (user-facing strings) | N/A | No shopper-facing copy is produced by this integration |

## Delivery Scope & Targets

Scope: integration — delivers an agnostic Cartora maps/geocoding service capability; functional pages consume it but do not own it.

| Type | Disposition | Confluence target | Working copy |
|---|---|---|---|
| ISD | NEW | TBD | `openspec/changes/add-cartora-maps-integration/isd/cartora-maps-integration.md` |

## Discovery Gate

- [x] No CRITICAL open questions remain Open (Q1 and Q2 are NICE-TO-HAVE and flow into grooming)
- [x] Resolved refinements captured in the register that owns them (the map-presentation boundary was settled at the discovery review with the SA lead's sign-off and recorded as a consumed Dependency, not an open question)
- [x] Executive Summary reflects discovery readiness without replacing the detailed sections
- [x] Document Consumption Guide matches the sections it projects (scope, WBS sets, capabilities, dependencies, open items)

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
