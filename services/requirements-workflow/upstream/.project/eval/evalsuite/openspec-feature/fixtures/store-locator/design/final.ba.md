# add-store-locator — Architecture / Solution Design

## Solution Overview

<!-- Reader map (not a summary): what this design settles, why it is shaped this way,
     what constrains downstream, what is passed forward. By subject and count; do not
     re-list the tables below — that is the Document Consumption Guide's job. -->

This design settles the functional store-locator storefront surface — an SSR/RSC
page shell for proximity search whose accessible result list is the primary surface
and whose interactive map is a lazy-loaded companion — and retires the legacy static
`/stores` directory at launch. It owns the storefront experience ONLY: every
third-party and commerce service it needs (maps/geocoding, address autocomplete,
store data) is consumed from capabilities other changes own, so the feature holds no
vendor key, quota, or billing. Ten capability facets (SLS.* + GH.UTL plus the
carried store-directory REMOVE) are classified by gap; seven architecture decisions
(D1–D7) carry the solution shape.

What constrains downstream (technical-design): the page shell is SSR/RSC with the
map lazy-loaded off the critical path; the accessible list is the canonical surface
and the no-JS fallback (D7); "Get directions" is a device hand-off that spends no
maps quota; store data comes from the existing Stores API with no new model; the
surface is served at `/store-locator`. Two questions are passed to grooming —
special/holiday opening hours (Q1) and the retired-URL redirect type/target (Q2);
nothing is routed to planning as a deferral. Two items are explicitly out of scope
and not decided here: real-time in-store inventory (owned by the inventory
integration workstream) and in-store appointment booking (a change request the
client declined). The design is warranted and not yet approved.

## Document Consumption Guide

<!-- Map of THIS document's current sections: one row per ## section in document
     order (excludes Solution Overview, this guide, and Design Gate). Key data is
     identifiers/sets, `None` for an empty set — re-derived from the body, never
     carried over. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Affected Capabilities | The capabilities this change owns and their direction; fixes the specs file inventory. | store-locator ADD; global-navigation MODIFY; store-directory REMOVE |
| Design Source Inventory | The already-collected inputs this design reasons from; lets downstream re-open a source without re-discovery. | discovery, BRD/SRD, technical notes, baseline specs, visual reference, review gate session 2026-02-12 |
| Gap Analysis | Per in-scope facet, the current-vs-target gap and its class; feeds technical-design's build effort. | SLS.SRCH rebuild; SLS.MAP rebuild; SLS.DTL rebuild; SLS.DIR rebuild; SLS.FLT rebuild; SLS.A11Y rebuild; SLS.I18N config; SLS.AC rebuild; GH.UTL config; store-directory remove |
| Architecture Decisions | The solution shape technical-design inherits as settled HOW background; each records rationale/alternatives/trade-offs. | D1–D7 |
| Base Configuration & Cross-Cutting | The cross-cutting constraints per concern the storefront/build must honour; the rows the stories projection reads. | Rendering, Caching, Security, Performance, i18n / Content, Merchant configuration, CMS |
| Integration Specification | Whether this change owns a third-party service contract. | N/A — functional change (consumed integrations owned elsewhere) |
| Risks / Mitigations | The delivery risks and how they are contained. | third-party call approach, Cartora key setup, retiring `/stores`, special/holiday hours (Q1) |
| Dependencies | Upstream deliveries this design consumes but does not own; become proposal external-dependency rows. | Cartora maps/geocoding; Addressly autocomplete; Commerce Stores API |
| Open Questions | Unresolved refinements gating readiness; the only source of design `Q#` downstream. | Q1 (special/holiday hours) — BA/SA, raised for grooming; Q2 (redirect type/target) — SA, raised for grooming |
| Planning Deferrals | Developer-HOW items design routes to technical-design planning; the only source of `PD#`. | PD1 (caching strategy) |
| Review & Approval | Whether a human approved proceeding to the next phase. | Approved: no; approver: — |

## Affected Capabilities

<!-- Capabilities this change owns and what it does to each, at capability level. -->

| Capability | Change | What changes (architecture level) | Spec |
|---|---|---|---|
| store-locator | ADD | New storefront locator surface: postcode/city/geolocation search, an interactive map synchronized with an accessible store list, store detail, a service filter, and a native "Get directions" hand-off | `openspec/specs/store-locator/spec.md` |
| global-navigation | MODIFY | Re-point the footer "Our stores" entry from the static `/stores` directory to the new locator at `/store-locator` | `openspec/specs/global-navigation/spec.md` |
| store-directory | REMOVE | Retire the static `/stores` Page Designer directory page superseded by the locator; `/stores` redirects to `/store-locator` (redirect type is Q2) | `openspec/specs/store-directory/spec.md` |

## Design Source Inventory

<!-- References into already-collected context; do not restate discovery here. -->

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-store-locator/discovery.md` | Resolved functional scope, RQ-701–RQ-710 trace, open questions Q1/Q2, CR1 |
| BRD / SRD | `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt` | Signed WHAT and dispositions |
| Technical notes | `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md` | Lazy-load and privacy constraints; consumed third-party/service list |
| Baseline specs | `openspec/specs/global-navigation/spec.md`; `openspec/specs/store-directory/spec.md` | `global-navigation` = MODIFY, `store-directory` = REMOVE |
| Visual reference | current live site (no committed UX prototype or design asset) | No committed design asset exists; the current site is the visual reference the storefront UI follows, and FSD Visualization describes the needed screens in words |
| Review gate session 2026-02-12 (BA Elena Sokolova) | `user request` (per-artifact review gate, Walk the questions) | Human decisions taken during review: the accessible-list-primacy stance recorded as an explicit architecture decision (D7), and the exact caching strategy for the read-only store data formally routed to technical-design planning as PD1 (rather than left in Caching-row prose) |

## Gap Analysis

<!-- Per in-scope requirement: what exists today and how far it is from the target. Facts with
     anchors; the section decides nothing. -->

| WBS | Current State | Desired State | Gap | Notes |
|---|---|---|---|---|
| SLS.SRCH | No store search exists on the storefront: the legacy `/stores` directory has no search field (`openspec/specs/store-directory/spec.md`), and postcode/city geocoding + proximity are not integrated (owned by the maps integration change, to be delivered) | Search by postcode, city, or user-initiated current location, returning stores ranked by proximity | rebuild | Nothing to configure or adapt — no OOTB or legacy search surface exists (RQ-701). Default radius/units and the use-my-location toggle are Business Manager merchant config, not code |
| SLS.MAP | No map exists: the legacy directory renders "no search field, service filter, or map" (`openspec/specs/store-directory/spec.md`); map rendering is consumed from the maps integration change (to be delivered) | Interactive map beside a synchronized, accessible result list | rebuild | Map surface consumed, list/sync owned here (RQ-702). The landing's promo region is Page-Designer-authored (`page.storelocator`, `assets.storeCard`) — consumed, not owned; v1 promo content settled at the discovery review as a single curated Featured-Store card |
| SLS.DTL | Legacy directory shows store name and postal address only, manually authored in Page Designer (`openspec/specs/store-directory/spec.md`); the authoritative fields (phone, hours, services) already exist in the commerce Stores API, already integrated | Store detail with name, full address, phone, opening hours, and available services | rebuild | Data layer needs no build — the Stores API already exposes the records (RQ-703); only the detail surface is new. Special/holiday hours availability is unconfirmed (Q1) |
| SLS.DIR | No directions action exists today; the hand-off targets the shopper's own device map app and needs no maps-vendor call | "Get directions" per store, opening the device map app with the store as destination | rebuild | Device hand-off, so it consumes no maps quota (RQ-704); no OOTB or legacy component surfaced to reuse |
| SLS.FLT | No filtering exists on the legacy directory (`openspec/specs/store-directory/spec.md`); service attributes already come from the commerce Stores API | Filter stores by click-and-collect, returns, and gift wrapping | rebuild | Attributes exist, the filter surface does not (RQ-705); the offered service options are seeded in Business Manager merchant config |
| SLS.A11Y | No locator surface exists to be accessible; the legacy static list is the only current store surface and offers no map alternative | Keyboard-operable, screen-reader-usable locator with an accessible list alternative to the map (WCAG 2.2 AA) | rebuild | Applies to the new surface, so there is no baseline behaviour to adapt (RQ-706); the accessible list doubles as the non-map fallback |
| SLS.I18N | The storefront is already multi-locale and routes all shopper-facing copy through translation keys; only locator copy does not exist yet | All locator copy localizable through the same mechanism | config | The existing platform i18n mechanism covers this — only the new keys are added, no new capability (RQ-707, `sfnext-i18n`) |
| SLS.AC | No type-ahead exists in any storefront search box; address suggestions come from the address-autocomplete integration change (to be delivered) | Address suggestions as the shopper types a postcode or partial address | rebuild | Suggestion UI owned here, suggestions consumed from the autocomplete capability (RQ-710) |
| GH.UTL | The footer "About" group already exposes an "Our stores" link, pointing at the static directory `/stores` (`openspec/specs/global-navigation/spec.md`) | The same footer entry reaches the new locator | config | The navigation capability itself is unchanged — only the target is re-pointed (RQ-709). The retired `/stores` URL still needs a redirect; its type stays Q2 |
| store-directory | The static `/stores` Page Designer directory page is live, footer-linked, bookmarked, and indexed (`openspec/specs/store-directory/spec.md`) | The directory is retired once the locator ships | remove | Removal is already SIGNED upstream (discovery Affected Capabilities REMOVE), so it is carried, not a design-proposed removal needing the class A/B classifier. Capability-level row: no signed WBS of its own. Redirect type is Q2 |

### Legacy Behaviour Overview

Today the only store-finding surface is the static Store Directory at `/stores`
(`openspec/specs/store-directory/spec.md`): a Page Designer content page, maintained by hand in
Business Manager, listing every store grouped alphabetically by city and showing the store name
and postal address only. It carries no search field, no service filter, no map, and no live store
data — the entries are authored content, not Stores API records. Shoppers reach it from the
footer "Our stores" link under "About" (`openspec/specs/global-navigation/spec.md`), and the URL
is bookmarked and indexed.

Functionally the legacy page therefore covers exactly one slice of the target behaviour — "show
me the shops and their addresses" — and none of proximity search, filtering, opening hours,
services, map presentation, or directions. No behaviour is portable from it: the locator replaces
the surface rather than re-implementing it, and the only obligation carried over is that the
retired URL must not 404 (redirect type is Q2).

## Architecture Decisions

### D1 — Lazy-load the map behind an SSR/RSC page shell
- **Decision:** Render the locator page shell server-side (SSR/RSC) with an accessible store list; load the map lazily on the client after initial render. (RQ-702, RQ-706)
- **Rationale:** The map must not block initial render or the critical JS path; the accessible list doubles as the non-map fallback.
- **Alternatives considered:** Eagerly bundling and rendering the map — rejected: blocks the critical path and degrades the no-JS / screen-reader experience.
- **Trade-offs:** A brief post-load map hydration; mitigated by rendering the list first so results are usable immediately.

### D2 — Responsive result layout differs by surface
- **Decision:** On desktop, show the interactive map beside the result list and keep it visible; on mobile and tablet, use a list-first layout with the map collapsed behind a toggle. (RQ-702, RQ-706)
- **Rationale:** A persistent side-by-side map suits large viewports; on small viewports a list-first view protects the primary experience and the accessible list alternative.
- **Alternatives considered:** One reflowing layout for all sizes — rejected: a shrunk side-by-side map crowds small viewports and buries the list.
- **Trade-offs:** Two layout modes to build and test; contained to the result view.

### D3 — "Get directions" is a native deep-link hand-off
- **Decision:** "Get directions" opens the shopper's own device/native map app with the store as destination via a deep link; it does not render through or call the maps capability. (RQ-704)
- **Rationale:** Directions are a device hand-off, not a map render/call; this also avoids extra map quota and billing.
- **Alternatives considered:** In-app routing via the maps vendor — rejected: unnecessary usage/cost with no requirement.
- **Trade-offs:** Directions UX depends on the shopper's device map app — expected and acceptable.

### D4 — Read store data from the existing Stores API, no new model
- **Decision:** Source all store fields (name, address, geo, hours, services) from the existing commerce Stores API; introduce no new store custom objects for v1. (RQ-703, RQ-705)
- **Rationale:** Store data is authoritative in the backend and already read via the Stores API — "no new store data model for v1".
- **Alternatives considered:** New store custom objects / data model — rejected: duplicates authoritative data and is out of v1 scope.
- **Trade-offs:** Bound by the fields the Stores API exposes; special/holiday-hours availability is unconfirmed (Q1).

### D5 — Consume third-party integrations across a stable feature boundary
- **Decision:** Consume maps/geocoding (Cartora) and address autocomplete (Addressly) through their own agnostic integration capabilities; the feature holds no vendor keys, quota, or billing and does not decide the call approach. (RQ-701, RQ-710)
- **Rationale:** Each supplier is a separate account/contract/key; ownership belongs to the integration changes, keeping the feature vendor-agnostic.
- **Alternatives considered:** Calling suppliers directly with embedded keys — rejected: leaks key/billing into the storefront and couples the feature to a vendor.
- **Trade-offs:** The feature depends on those integration capabilities being delivered; the client-vs-proxy call approach is decided there, not here.

### D6 — Serve the locator at a dedicated URL `/store-locator`
- **Decision:** The new store locator surface is served at the user-facing path `/store-locator`; the footer "Our stores" entry and the retired `/stores` redirect both target this path. The redirect TYPE (permanent vs temporary) stays Q2. (RQ-709)
- **Rationale:** A new storefront surface needs a stable, indexable URL that the navigation entry, the retired-URL redirect target, and SEO all depend on at design time; `/store-locator` matches the capability.
- **Alternatives considered:** Reusing `/stores` for the new page — rejected: `/stores` is the retired directory's URL and is redirected to the locator, so the new surface needs its own path. Leaving the path undecided until Dev — rejected: the redirect target and the navigation entry cannot be specified without it.
- **Trade-offs:** Fixes the public URL early; a later rename would need its own redirect.

### D7 — The accessible result list is the canonical surface; the map is progressive enhancement
- **Decision:** Treat the accessible, server-rendered result list as the canonical store-finding surface and the definitive no-JS / screen-reader path; the interactive map is a progressive enhancement layered on top, never a prerequisite for any locator function. (RQ-706, RQ-702)
- **Rationale:** Recorded explicitly at the review gate: the list-first stance is implied across D1 and the SLS.A11Y gap, but making it a first-class decision keeps technical-design from treating the map as load-bearing and protects the WCAG 2.2 AA target.
- **Alternatives considered:** Leaving list-primacy implicit in the rendering decision — rejected at review: it left the accessibility contract to inference. A map-first surface with a list fallback — rejected: inverts the no-JS/screen-reader guarantee.
- **Trade-offs:** Commits every locator function (search, detail, filter, directions) to work through the list alone; accepted as the accessibility floor.

## Base Configuration & Cross-Cutting

| Concern | Applies to | Constraint / decision | Rationale |
|---|---|---|---|
| Rendering | SLS.SRCH SLS.MAP | SSR/RSC page shell + accessible list; map lazy-loaded client-side (`sfnext-routing`) | Map must not block initial render or the critical JS path (RQ-706) |
| Caching | SLS.DTL SLS.FLT | Store data is read-only and stable, so it is a caching candidate at the storefront/edge (HTTP); no new caching layer is introduced (`b2c-ecdn`) | Read-only reference data already served by an integrated API; exact strategy deferred to technical-design planning (PD1) |
| Security | SLS.SRCH | Geolocation user-initiated and permission-gated; coordinates never persisted or sent to analytics; no vendor keys held by the feature (`sfnext-authentication`) | Privacy constraint; key/billing owned by the integration changes |
| Performance | SLS.MAP | Lazy-load the map; list-first render keeps results usable off the critical path (`sfnext-performance`) | Lazy-load constraint protects the initial render budget |
| i18n / Content | SLS.I18N | All shopper-facing copy via translation keys; multi-locale storefront (`sfnext-i18n`) | RQ-707 |
| Merchant configuration | SLS.SRCH SLS.MAP SLS.DTL SLS.FLT | Default search radius/units and use-my-location toggle, stores-per-page and load-more batch, and the offered filter services are managed by the client's team in Business Manager; store records (name/address/phone/hours/services) are maintained in the commerce backend | Client-owned admin controls; no vendor keys/quotas (owned by the integration changes) |
| CMS | GH.UTL | Legacy `/stores` Page Designer content page retired at launch (`sfnext-page-designer`) | The locator supersedes the static directory |

## Integration Specification

<!-- Integration-scope changes only: the third-party service contract. This is a functional change — N/A. -->

N/A — functional change. Maps/geocoding (Cartora) and address autocomplete (Addressly) are
**consumed** as dependencies; their service contracts (endpoints, data mapping, data flow,
keys/quota) are owned and specified by the separate integration changes, not here. See
Dependencies below.

## Risks / Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Third-party call approach undecided (client vs server proxy) | Key exposure, quota, and billing ownership unresolved | Decided in the integration changes' grooming/design; the feature consumes a stable capability either way |
| Cartora account / domain-restricted key setup unconfirmed | Map may fail to render if the account/key is not ready | Confirm account ownership and the key with the client before build (integration change) |
| Retiring `/stores` (indexed and bookmarked) | 404s and SEO / link loss at launch | Add a redirect from `/stores` to the locator; exact type/target is Q2, confirmed in grooming |
| Special/holiday hours may not be exposed by the Stores API (Q1) | Store detail may omit holiday hours | Confirm field availability with the store-data integration; treat holiday hours as conditional |

## Dependencies

<!-- Upstream deliveries this functional design depends on, at the technical/delivery level;
     the feature holds no vendor account/key/billing (owned by the integration changes). -->

| Dependency | Type | Status | Note |
|---|---|---|---|
| Cartora maps/geocoding capability | upstream integration change | to be designed & delivered (separate change) | Consumed by RQ-701, RQ-702 |
| Addressly address-autocomplete capability | upstream integration change | to be designed & delivered (separate change) | Consumed by RQ-710 |
| Commerce Stores API (store data) | existing service | already integrated | Consumed by RQ-703, RQ-705 |

## Open Questions

<!-- Design-level questions raised for grooming; inherits discovery's NICE-TO-HAVE items. -->

| Q# | Question | Owner | Status |
|---|---|---|---|
| Q1 | Should store detail surface special/holiday opening hours (depends on Stores API)? | BA / SA | Raised for grooming |
| Q2 | What redirect (type and target) should the retired `/stores` URL use? | SA | Raised for grooming |

## Planning Deferrals

<!-- Developer-HOW items the design intentionally routes to technical-design planning. Always
     present (like Open Questions / Dependencies); `None.` when the design defers nothing. -->

| PD# | Source (design section/decision) | Deferred item | Why not decided now | Target phase | Owner | Status |
|---|---|---|---|---|---|---|
| PD1 | Base Configuration & Cross-Cutting (Caching) | Exact caching strategy for the read-only store data — the storefront/edge (HTTP) cache dimensions, TTL, and invalidation | Read-only reference data is an edge/HTTP caching candidate, but the concrete cache key, TTL, and invalidation are a developer-HOW that does not change any capability contract; recorded at the review gate to route it to planning rather than leave it as Caching-row prose | planning | tech-design | routed to planning |

## Design Gate

- [x] Design is warranted (new storefront surface, privacy-sensitive geolocation, legacy URL retirement, three consumed integration boundaries)
- [x] Delivery scope and change identity match the discovery trace (functional store-locator surface)
- [x] Every architecture decision records rationale, alternatives, and trade-offs
- [x] Decisions needing sign-off are raised as Open Questions for grooming, not asserted as decided
- [x] No developer-implementation detail (concrete signatures/paths/codes, wiring) — deferred to technical-design

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
