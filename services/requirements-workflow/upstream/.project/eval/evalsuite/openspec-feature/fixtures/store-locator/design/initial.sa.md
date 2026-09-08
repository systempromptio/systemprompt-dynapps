# add-cartora-maps-integration — Architecture / Solution Design

## Solution Overview

<!-- Reader map (not a summary): what this design settles, why it is shaped this way,
     what constrains downstream, what is passed forward. By subject and count; do not
     re-list the tables below — that is the Document Consumption Guide's job. -->

This design settles ONE agnostic capability — Cartora map rendering plus
geocoding/proximity — exposed behind a stable, lazy-loadable interface that any
storefront feature consumes without holding a Cartora key or calling Cartora
directly; key, quota, billing, and privacy are governed centrally. It is shaped by
one load-bearing approach: build DIRECTLY against Cartora's own SDK (rendering) and
Geocoding API (server-side), NOT a marketplace cartridge — the architect's explicit
signal, recorded against the build-vs-reuse ladder. Six architecture decisions
(D1–D6) carry that shape; four capability facets (3PI.MAP.GEO/RND/STY/MRK) are
classified by gap.

What constrains downstream (technical-design): the rendering entrypoint is
lazy-loaded and client-only; geocoding runs server-side with the key off the
browser; shopper coordinates stay transient. Two questions are passed to grooming —
the geocoding call approach (Q1, linked to D6) and the domain-restricted key policy
(Q2). The capability is warranted (new external dependency, key/quota/privacy
governance) and not yet approved.

## Document Consumption Guide

<!-- Map of THIS document's current sections: one row per ## section in document
     order (excludes Solution Overview, this guide, and Design Gate). Key data is
     identifiers/sets, `None` for an empty set — re-derived from the body, never
     carried over. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Affected Capabilities | The one capability this change owns and its direction; fixes the specs file inventory. | cartora-maps-integration ADD |
| Design Source Inventory | The already-collected inputs this design reasons from; lets downstream re-open a source without re-discovery. | discovery, technical notes, meeting notes, Cartora vendor docs (SDK + Geocoding API) |
| Gap Analysis | Per capability facet, what exists to build on and how far from the obligation; feeds the solution approach. | 3PI.MAP.GEO rebuild; 3PI.MAP.RND adapt; 3PI.MAP.STY config; 3PI.MAP.MRK config |
| Architecture Decisions | The solution shape technical-design inherits as settled HOW background; each records rationale/alternatives/trade-offs. | D1–D6 (D6 proposed, links Q1) |
| Base Configuration & Cross-Cutting | The cross-cutting constraints per concern the storefront/build must honour; the rows the stories projection reads. | Rendering, Caching, Security, Performance, Privacy |
| Integration Specification | The third-party service contract downstream ISD/technical-design consume. | endpoints: geocoding GET `…/v1/geocode`, Maps SDK bundle+tiles; request q/key/country/limit; response lat/lng/score |
| Risks / Mitigations | The delivery risks and how they are contained. | Q1 call approach, Q2 key setup, quota exhaustion |
| Dependencies | Operational prerequisites (accounts, credentials) this design relies on; become proposal external-dependency rows. | Cartora account + domain-restricted key (unconfirmed, Q2); client Cartora billing account |
| Open Questions | Unresolved refinements gating readiness; the only source of design `Q#` downstream. | Q1 (call approach) — SA, raised for grooming; Q2 (key policy) — SA, raised for grooming |
| Planning Deferrals | Developer-HOW items design routes to technical-design planning; the only source of `PD#`. | None |
| Review & Approval | Whether a human approved proceeding to the next phase. | Approved: no; approver: — |

## Affected Capabilities

<!-- Capabilities this change owns and what it does to each, at capability level. -->

| Capability | Change | What changes (architecture level) | Spec |
|---|---|---|---|
| cartora-maps-integration | ADD | New agnostic Cartora capability: map rendering plus geocoding/proximity behind a stable, lazy-loadable interface, with key/quota/billing/privacy governed centrally; consumers hold no Cartora key and never call Cartora directly | `openspec/specs/cartora-maps-integration/spec.md` |

## Design Source Inventory

<!-- References into already-collected context; do not restate discovery here. -->

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-cartora-maps-integration/discovery.md` | Resolved integration-only scope, obligations for RQ-711/RQ-712, open questions |
| Technical notes | `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md` | Cartora SDK, geocoding/proximity, lazy-load and privacy constraints |
| Meeting notes | `.cursor/.project/project-context/inbox/meeting_notes/2026-02-04_storefront-workshop_transcript.txt` | Possible existing Cartora account, domain-restricted key, client-vs-proxy question |
| Vendor documentation | Cartora Maps SDK + Geocoding API docs (see Integration Specification → Integration Documentation) | Interface, endpoints, and quota/billing reference |

## Gap Analysis

<!-- Capability-level for an integration change: what already exists to build on (cartridge /
     module / SDK / API / vendor-hosted config / nothing) and how far it is from the obligation.
     Feeds the Solution Approach decision below; the section itself decides nothing. -->

| WBS | Current State | Desired State | Gap | Notes |
|---|---|---|---|---|
| 3PI.MAP.GEO | No geocoding capability exists (project-context `specs`: no `openspec/specs/cartora-maps-integration/spec.md`). Prior art available: a partner "Cartora Commerce Maps" cartridge advertising drop-in geocoding/proximity, and Cartora's documented Geocoding API (`…/v1/geocode`) | Governed geocoding + proximity behind an agnostic interface; consumers hold no key | rebuild | Nothing exists to configure or adapt, so only build-shaped rungs close it. The cartridge fits on paper but its RSC / CSP-nonce fit is unverified, and the architect signalled owning the connector — see D1 |
| 3PI.MAP.RND | No rendering capability exists; the Cartora Maps SDK is a documented client-side, lazy-loadable bundle the platform can load | Lazy-loadable map-rendering entrypoint consumers call without holding a key | adapt | The vendor SDK is the rendering surface and is used as-is; only the agnostic entrypoint and key governance are ours |
| 3PI.MAP.STY | Map type and brand theme are already authorable in Cartora Map Studio (vendor-hosted, no-code) and published as a Map Style ID; no code surface exists here | The rendered map carries the brand's configured style and map type | config | Vendor-managed presentation consumed by reference (delivery mode `consume`); the Map Studio authoring surface is the dependency. The v1 Style ID and its owner were settled at the discovery review: the brand team's published "Brand Street" style, owned by that team |
| 3PI.MAP.MRK | Custom store markers/pins are authored in the same Map Studio style and published within that Style ID | Stores render with the brand's custom markers | config | Same published Style ID as 3PI.MAP.STY; no marker-authoring UI is built here |

## Architecture Decisions

### D1 — Solution approach: build directly against Cartora's SDK/API, not a cartridge
- **Decision:** Deliver this capability by integrating DIRECTLY against Cartora's own SDK + API — the Maps SDK for rendering and the Geocoding API (server-side) for geocoding/proximity — rather than reusing a marketplace cartridge. This honours the architect's explicit approach signal ("build from scratch against Cartora's SDK/API, not a cartridge"). The signal overrides the build-vs-reuse ladder; the per-rung fit is recorded (Enumerate-then-Justify):
  - **Reuse cartridge / module (priority 1):** discovery surfaced a "Cartora Commerce Maps" marketplace cartridge advertising drop-in RSC map + geocoding. Fits on paper, but DECLINED — the architect signalled owning the connector, and the cartridge's RSC / CSP-nonce compatibility and governance seams are unverified against this stack.
  - **Vendor SDK (priority 2):** the Cartora Maps SDK is a client-side, lazy-loadable bundle the platform can load — fits for map rendering; used.
  - **From scratch (priority 3):** the Geocoding API (`…/v1/geocode`) has no drop-in and is served server-side (key off the browser) — used for geocoding/proximity.
- **Rationale:** Owning the connector keeps key/quota/billing/privacy governance and the server-proxy call placement under this capability's control (RQ-711/712), which a third-party cartridge would obscure; the SDK is the vendor's own supported rendering surface.
- **Alternatives considered:** Reuse the Cartora Commerce Maps cartridge — rejected per the architect's build-from-scratch signal and unverified stack fit. A single mechanism for both obligations — rejected: rendering is a client SDK, geocoding is a server-side API call, so they take different shapes.
- **Trade-offs:** Owning the connector is more build than a cartridge but yields deterministic control over governance, CSP, and call placement; if the cartridge's fit were later verified, revisiting reuse would cut build.

### D2 — Deliver Cartora as an agnostic, reusable capability
- **Decision:** Own map rendering and geocoding/proximity as a service capability; consumers integrate through a stable interface and hold no Cartora keys. (RQ-711, RQ-712)
- **Rationale:** Discovery scope is integration-only; centralizing governs key, quota, billing, and privacy once instead of per feature.
- **Alternatives considered:** Each feature integrates Cartora directly — rejected: duplicates key/billing/privacy handling and couples every feature to the vendor.
- **Trade-offs:** Adds an indirection layer consumers depend on; justified by reuse and single-point governance.

### D3 — Single vendor (Cartora) for both rendering and geocoding/proximity
- **Decision:** Use the Cartora maps SDK for rendering and Cartora geocoding for postcode/city search and proximity.
- **Rationale:** The technical notes name Cartora for both; one vendor covers the capability.
- **Alternatives considered:** Separate geocoding vendor — rejected: extra vendor with no requirement.
- **Trade-offs:** Vendor lock to Cartora; acceptable per the sources.

### D4 — Domain-restricted key with billing on the client account
- **Decision:** The Cartora key is domain-restricted and quota/billing sit on the client's Cartora account, governed by this capability.
- **Rationale:** Meeting-notes constraint — the key must be domain-restricted and billing sits on the client account.
- **Raised for grooming (Q2):** Exact key policy and account setup are unconfirmed — carried as an open question, not decided here.

### D5 — Lazy-loadable capability entrypoint
- **Decision:** Expose the map capability as a lazy-loadable entrypoint so consumers keep it off the critical path.
- **Rationale:** Lazy-load constraint from the technical notes; the SDK is client-only.
- **Trade-offs:** Consumers manage a loading state; standard for third-party map SDKs.

### D6 (proposed, raised for grooming — Q1) — geocoding call approach
- **Proposed direction:** A server proxy for typed-postcode geocoding hides the key and centralizes quota/throttling; a direct client call is simpler but exposes the key.
- **Status:** Undecided — raised for grooming with rationale/alternatives; not asserted as decided.

## Base Configuration & Cross-Cutting

| Concern | Applies to | Constraint / decision | Rationale |
|---|---|---|---|
| Rendering | all | Client-only SDK, exposed as a lazy-loadable entrypoint (`sfnext-routing`) | Off critical path; SDK is client-side |
| Caching | all | If a server proxy is adopted (Open Question Q1), geocode responses are cacheable server-side with a bounded lifetime to respect quota; a client-side call relies on browser/CDN caching instead (`b2c-custom-caches`) | Reduce third-party calls; approach + TTL depend on Q1, settled in technical-design |
| Security | all | Domain-restricted Cartora key governed centrally by the capability; consumers hold no Cartora key of their own. Whether the key stays server-side or is domain-restricted in the browser follows the call approach (Q1) (`b2c-webservices`) | Key-exposure + cost-ownership risk |
| Performance | all | Lazy-loadable entrypoint; proximity handled by Cartora (`sfnext-performance`) | Meets lazy-load constraint |
| Privacy | all | Shopper coordinates are transient — never persisted or sent to analytics by this capability | Privacy constraint from sources |

## Integration Specification

<!-- Integration-scope changes only: the third-party service contract. Omit for functional changes. -->

### Overview

This section carries Cartora's third-party contract at the shape the Solution Approach fixed — a
mixed SDK + API integration — so it splits into a rendering channel (the Maps SDK) and a geocoding
endpoint with request/response mapping (the Geocoding API). It states the vendor contract only: the
key stays capability-held, the call placement is deferred (Q1), and our own build defers to
technical-design.

Read in order: the cited vendor docs (Integration Documentation) → the wire contract (Endpoints +
Data Mapping — the geocoding request/response and the SDK bundle/tiles channel) → the boundary
sequence (Data Flow).

### Integration Documentation

<!-- Official vendor docs used to derive endpoints and data mapping. Grounded in the source packet. -->

| Document | Reference |
|---|---|
| Cartora Geocoding API reference | project-context `web`: `https://developers.cartora.example/docs/geocoding` |
| Cartora Maps SDK guide | project-context `web`: `https://developers.cartora.example/docs/maps-sdk` |
| Key domain-restriction & quota policy | project-context `web`: `https://developers.cartora.example/docs/keys` |

### Endpoints

<!-- One row per endpoint/channel this integration uses. -->

| Endpoint / Channel | Protocol / Method | Data Format | Authorization |
|---|---|---|---|
| Geocoding / proximity | HTTPS / GET `…/v1/geocode` | JSON | Domain-restricted API key sent as the `key` query parameter (origin-locked) |
| Maps rendering (SDK bundle + tiles) | HTTPS (SDK) | JS bundle + vector tiles | Same domain-restricted key, supplied at SDK init and attached by the SDK |

### Data Mapping

<!-- What we send and what we receive, field-level. Grounded in the vendor documentation above. -->

**Request → Cartora Geocoding**

| Field | Source | Note |
|---|---|---|
| `q` | shopper postcode or city | Verbatim search query |
| `key` | capability-held Cartora key | Domain-restricted API key, sent as a query parameter (origin-locked) |
| `country` | storefront locale region | Scopes results to the market |
| `limit` | capability config | Caps proximity results |

**Response ← Cartora Geocoding**

| Cartora field | Maps to | Note |
|---|---|---|
| `results[].lat`, `results[].lng` | `coordinates` | Transient; never persisted |
| `results[].score` | `proximityRanking` | Result ordering |
| `results[].label` | — | Vendor display label; this agnostic capability returns coordinates + ranking, so it is not part of the consumer contract |

### Data Flow

```mermaid
sequenceDiagram
    participant C as Consumer feature
    participant Cap as Cartora capability
    participant Cartora as Cartora API
    C->>Cap: geocode request (postcode / city)
    Cap->>Cartora: GET …/v1/geocode?q=…&key=… (key as query param; call placement per Q1)
    Cartora-->>Cap: results (lat, lng, proximity score)
    Cap-->>C: coordinates + proximity ranking
    C->>Cap: render-map request
    Cap->>Cartora: load Maps SDK + tiles (domain-restricted key)
    Cartora-->>C: interactive map (rendered via SDK)
```

## Risks / Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Geocoding call approach undecided (Q1) | Determines key exposure, quota ownership, and where throttling lives | Resolve in grooming; a server proxy hides the key at the cost of a hop |
| Cartora account / domain-restricted key setup unconfirmed (Q2) | Capability cannot function without a ready, restricted key | Confirm account ownership and key policy with the client's Cartora owner in grooming |
| Quota exhaustion under load | Failed map/geocode requests | Cacheable geocode responses (server-proxy branch, Q1) + quota governance at the capability |

## Dependencies

<!-- Operational prerequisites (accounts, credentials, sandboxes, environment) this design relies on. -->

| Dependency | Type | Status | Note |
|---|---|---|---|
| Cartora account + domain-restricted key | credentials | unconfirmed (Q2) | Blocks build until confirmed |
| Client Cartora billing account | account | client-owned | Billing sits here, not agency |

## Open Questions

<!-- Design-level questions raised for grooming; inherits discovery's NICE-TO-HAVE items. -->

| Q# | Question | Owner | Status |
|---|---|---|---|
| Q1 | Geocode typed postcodes client-side or via a server proxy? | SA | Raised for grooming |
| Q2 | Domain restrictions and quota/billing guardrails for the Cartora key? | SA | Raised for grooming |

## Planning Deferrals

<!-- Developer-HOW items the design intentionally routes to technical-design planning. Always
     present (like Open Questions / Dependencies); `None.` when the design defers nothing. -->

None.

## Design Gate

- [x] Design is warranted (new external dependency; integration/security/performance complexity)
- [x] Delivery scope and change identity match the discovery trace (agnostic Cartora capability)
- [x] Every architecture decision records rationale, alternatives, and trade-offs
- [x] Decisions needing sign-off are raised as Open Questions for grooming, not asserted as decided
- [x] No developer-implementation detail (concrete signatures/paths/codes, wiring) — deferred to technical-design

## Review & Approval

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
