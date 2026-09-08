# Cartora Maps Integration — Technical Design

## Technical Design Overview

This technical design plans the Cartora Maps Integration — an agnostic capability that
publishes a server-side geocode interface and a governed, lazy-loaded map-rendering entrypoint
that consuming features call, keeping the vendor key server-side and consuming the brand team's
published map presentation by reference. It owns the developer-level HOW of the capability only:
consumer UI and store data stay out of scope, owned by the Store Locator change. Four primary
delivery units close through three published phases — a server-side geocode path and a client
map entrypoint that share no module, plus the consumed brand presentation the entrypoint renders
against — while shopper-coordinate privacy is carried inside each phase rather than scheduled as
one.

What constrains downstream (tasks): each phase closes only when a consumer can call its path
against the published interface; the geocode path is server-only with the Cartora key held in a
service credential and a quota-aware cache, the map SDK stays off the consumer's critical path
behind a lazy entrypoint initialised with a domain-restricted key, and presentation is consumed
from the published Map Style ID by reference so a re-publish reaches consumers with no release.
Each phase is proven by developer-owned verification only (mapping, cache, credential-handling,
and lazy-gate unit tests plus a bundle check); functional and QA testing are out of scope, and
shopper coordinates stay transient — never persisted, logged, or sent to analytics.

Two questions are passed forward to the planning approval session: the consumer-facing contract
of the geocode interface — its endpoint namespace/path and response/error schema (CRITICAL — it
blocks tasks) — and the bounded cache lifetime for geocode responses (NICE-TO-HAVE); key issuance
remains a DEFERRED-APPROVED release prerequisite. No planning decision is approved yet; the
questions below are open.

## Technical Design Source Inventory

| Artifact | Location | Planning use |
|---|---|---|
| ISD | `openspec/changes/add-cartora-maps-integration/isd/cartora-maps-integration.md` | Signed integration WHAT — primary interpretation of approved intent; carries the integration contract and spec-derived bounds |
| Specs | `openspec/changes/add-cartora-maps-integration/specs/**/*.md` | Structured executable projection of the signed WHAT — behaviour contract and RQ/WBS trace |
| Grooming | `openspec/changes/add-cartora-maps-integration/grooming.md` | APPROVED decision authority — locked decisions and prerequisites (P#) this plan honours and references, never re-decides |
| Architecture design | `openspec/changes/add-cartora-maps-integration/design.md` | HOW / architecture rationale — vendor-contract working material (endpoints, data mapping, data flow) and locked decisions; not a requirements source |
| Proposal | `openspec/changes/add-cartora-maps-integration/proposal.md` | Approved delivery offer — Requirements Lock, delivery slices, scope boundaries, and dependencies/access |
| Discovery | `openspec/changes/add-cartora-maps-integration/discovery.md` | Source/provenance history — requirement trace origins, affected capabilities, and dependency background |
| Stories | `openspec/changes/add-cartora-maps-integration/stories.md` | Jira delivery grouping (TBD-1..TBD-4); not a behaviour source |

## Delivery Inventory

| Unit | Source | Story | Spec trace | Classification | Implementation areas | Notes |
|---|---|---|---|---|---|---|
| Server-side geocode interface | `stories.md` + `specs/cartora-maps-integration/spec.md` | `TBD-1` | RQ-711 / 3PI.MAP.GEO — Postcode and city geocoding with proximity ranking | PRIMARY | Backend (SFCC) | Publishes the geocode contract per Q1 and owns vendor mapping, governance, credential handling, and cache. |
| Governed map rendering | `stories.md` + `specs/cartora-maps-integration/spec.md` | `TBD-2` | RQ-712 / 3PI.MAP.RND — Governed map rendering entrypoint | PRIMARY | Frontend (capability entrypoint) | Lazy client entrypoint; consumers configure no Cartora key. |
| Consumed map presentation | `stories.md` + `specs/cartora-maps-integration/spec.md` | `TBD-3` | RQ-713 / 3PI.MAP.STY — Consumed map presentation reference | PRIMARY | Frontend (capability entrypoint) | Joins the marker story in P3 because both close through the published presentation reference. |
| Consumed store markers | `stories.md` + `specs/cartora-maps-integration/spec.md` | `TBD-4` | RQ-714 / 3PI.MAP.MRK — Consumed store marker set | PRIMARY | Frontend (capability entrypoint) | Joins the style story in P3 because markers are part of the same published presentation mechanism. |
| Shopper coordinate privacy | `specs/cartora-maps-integration/spec.md` | — | RQ-711 / 3PI.MAP.GEO — Shopper coordinate privacy | CROSS-CUTTING | Backend (SFCC), Frontend (capability entrypoint) | Carried by geocode and map paths; proven inside each phase rather than scheduled as a phase. |
| Deferred / out-of-scope scope | approved Requirements Lock | — | RQ-715 — store-density heatmap; RQ-701 / SLS.SRCH and RQ-702 / SLS.MAP consumer functional surfaces | DEFERRED | — | Out of scope for the Cartora Maps Integration change; consumer UI/data remains owned by the Store Locator change. |

## Delivery Phases

**Decomposition criterion:** a phase is one consumable capability path delivered end to end behind its
stable interface — the vendor call, its governance (key handling, caching, quota), and the test evidence a
consuming feature can rely on. A phase is closed only when a consumer could call that path against the
published interface.

**Primary phase units:** three published increments across the four approved stories — the server-side
geocode interface (`TBD-1` / RQ-711), the client-side map entrypoint (`TBD-2` / RQ-712), and the consumed
brand presentation that entrypoint renders against (`TBD-3` / RQ-713 with `TBD-4` / RQ-714). Geocode and
map rendering share no in-change artifact, so they may run in either order or in parallel; the presentation
increment closes through the map surface P2 publishes, so it follows P2.

**Shared seams:** none — the geocode path and the maps entrypoint have no common module in this change, so
nothing has to be folded forward and no foundation phase is warranted at this size.

**Cross-cutting obligations:** shopper coordinate privacy (the RQ-711 privacy requirement) applies to both
paths and is proven inside each phase's own verification rather than scheduled as a phase.

**Rejected splits:** the geocode path is not split into separate service-definition, cache, SCAPI-contract,
and Business-Manager-metadata phases — none of those is callable by a consumer on its own, so they are
tasks inside P1, not phases. `TBD-3` and `TBD-4` are not separate phases either: both close through the one
published-style reference the entrypoint holds, so splitting style from markers would yield two phases over
a single mechanism with no independent consumer value. Key issuance (grooming P1, DEFERRED-APPROVED) is a
release prerequisite for every phase, not a phase of its own: each is built and verified mock-first and
re-verified against the real vendor once the key is issued.

| Phase | Name | Delivers (story / RQ · WBS) | Acceptance coverage | Areas | Depends on | Verification |
|---|---|---|---|---|---|---|
| P1 | Server-side geocode interface | `TBD-1` / RQ-711 · 3PI.MAP.GEO — server-side geocode interface, governance, cache, and credential handling | Valid postcode, valid city, and unresolvable input; vendor `lat`/`lng` maps to `coordinates`, `score` maps to `proximityRanking`, `label` is dropped; clear error result has no coordinates; Cartora key never reaches the browser | Server-side geocode interface; Geocode governance; Coordinate privacy controls | — | Mapping and error-mapping unit tests; cache hit/miss unit test; credential-handling unit test asserting the vendor key stays server-side |
| P2 | Governed map rendering entrypoint | `TBD-2` / RQ-712 · 3PI.MAP.RND — lazy SDK bootstrap and the domain-restricted key supplied at initialisation | SDK and vector tiles load only on consumer demand; consumer content stays usable before and while the map loads; consumers configure no key | Lazy map entrypoint; Coordinate privacy controls | — | Lazy-gate unit test; bundle check that the SDK stays off the consumer's critical path |
| P3 | Consumed brand presentation and markers | `TBD-3` / RQ-713 · 3PI.MAP.STY and `TBD-4` / RQ-714 · 3PI.MAP.MRK — the published Map Style ID held as a reference at initialisation, carrying both map type/theme and the marker set | Published presentation reaches the map surface; a re-published style, map type, or marker set reaches consumers with no release; an unresolvable reference still yields a usable map on the vendor default; brand-authored markers plot the stores and a presentation without custom markers falls back to the vendor default marker | Lazy map entrypoint; Published presentation reference | P2 | Presentation-reference and marker-set unit tests |
| Cross-cutting | Shopper coordinate privacy | RQ-711 · 3PI.MAP.GEO (privacy requirement, both paths) | Coordinates remain transient in both paths; never persisted, logged, or sent to analytics | Coordinate privacy controls across geocode and map blocks | — | Proven inside each phase: coordinates never persisted, never logged, never sent to analytics |

## Implementation Architecture

### Frontend / Storefront Surface

| Surface | Server responsibility | Client responsibility | Hydration / ownership |
|---|---|---|---|
| None — the capability ships no storefront routes or pages; it exposes a lazy-loadable map entrypoint module that consuming features import on demand | — | Cartora Maps SDK entrypoint loads only on consumer demand | Client-only SDK entrypoint; consumer surfaces own their own rendering |

### Backend / SFCC Surface

| Surface | Responsibility | Inputs / triggers | Outputs / side effects |
|---|---|---|---|
| Custom SCAPI geocode interface (namespace/path and schema per Q1) | Stable server-side interface that resolves a shopper-entered postcode or city through the contract approved for Q1 | Request fields per Q1 | Response schema and error shape per Q1 |
| Cartora Geocoding HTTP service | Server-side vendor call through the service framework with the credential-held key | Endpoint request on cache miss | Vendor results mapped per design: `results[].lat`/`results[].lng` → `coordinates`, `results[].score` → `proximityRanking`, `results[].label` dropped |

### CMS / Content Surface

| Surface | Responsibility | Owned identifiers | Notes |
|---|---|---|---|
| None — the capability creates no CMS or Page Designer content; consuming features own their surfaces | — | — | — |

### Implementation Building Blocks

| Block | Responsibility | Interfaces / contracts | Owned identifiers | Trace |
|---|---|---|---|---|
| Server-side geocode interface | Publishes the capability's server-side geocode seam, maps vendor results, returns clear error results, and keeps shopper coordinates transient | Custom SCAPI contract per Q1; vendor GET `…/v1/geocode`; request/response fields and error shape per Q1 | `int_cartora` geocode implementation, custom SCAPI API definition | RQ-711 |
| Geocode governance | Owns service definition, credential handling, quota-aware cache, and server-only key exposure | Business Manager service + credential; custom cache partition with TTL per Q2 | Cartora HTTP service definition, service credential, geocode cache partition | RQ-711 |
| Lazy map entrypoint | Loads the Cartora SDK bundle + vector tiles only when a consumer asks and initialises the map with the capability-governed key | Consumer render request; SDK init with domain-restricted key and presentation reference | `src/lib/cartora-maps/entrypoint.ts`, maps key configuration | RQ-712 |
| Published presentation reference | Consumes the brand team's published Map Style ID and marker set by reference, never by copying presentation assets | Published Map Style ID (the brand team's "Brand Street" style, owned by that team); vendor default fallback when reference is unresolvable | Platform configuration for the published Map Style ID | RQ-713, RQ-714 |
| Coordinate privacy controls | Keeps coordinates transient across both geocode and map paths | No coordinate persistence, logging, analytics emission, or browser-visible server key | Geocode path, map entrypoint | RQ-711 |

## Runtime Flows

**Geocode path (server-side)**

```mermaid
sequenceDiagram
    participant C as Consumer feature (server-side loader)
    participant EP as Geocode endpoint (custom SCAPI, contract per Q1)
    participant Cache as Geocode custom cache (TTL per Q2)
    participant V as Cartora Geocoding API

    C->>EP: geocode request (namespace/path and request fields per Q1)
    EP->>Cache: lookup
    alt hit within the bounded TTL
        Cache-->>EP: cached result
    else miss
        EP->>V: GET …/v1/geocode via the service framework (credential-held key)
        V-->>EP: results[] (lat, lng, score, label)
        EP->>EP: map vendor fields into the response contract per Q1; drop label unless Q1 approves it
        EP->>Cache: store (bounded TTL)
    end
    alt input resolved
        EP-->>C: resolved geocode response per Q1
    else unresolvable input
        EP-->>C: error response per Q1
    end
    Note over C,V: The Cartora key never appears in a browser-visible request
```

**Map path (lazy entrypoint)**

```mermaid
sequenceDiagram
    participant C as Consuming feature (client)
    participant E as Map entrypoint (lazy)
    participant SDK as Cartora Maps SDK + vector tiles

    C->>E: request map (on consumer demand)
    E->>SDK: load bundle, init with the domain-restricted key + published Map Style ID
    alt style reference resolves
        SDK-->>C: interactive map on the brand presentation, stores plotted with its marker set
    else style reference unresolvable
        SDK-->>C: interactive map on the vendor default presentation — the consumer's render does not fail
    end
    Note over C,SDK: Consumer content stays usable before and while loading; shopper coordinates stay transient — never persisted, never sent to analytics
```

## Implementation Constraints

| Concern | Planning decision | Verification |
|---|---|---|
| Rendering / hydration | The map is a client-only SDK behind a lazy-loadable entrypoint; nothing renders server-side and nothing loads until a consumer asks. Presentation — map type, brand theme, marker set — is consumed from the published Map Style ID by reference, so re-publishing changes the rendered map with no release | Entrypoint unit test: SDK loads only on demand; presentation test: init carries the configured Style ID and falls back to the vendor default when it cannot be resolved |
| Data fetching | Geocoding is served server-side only, through the capability's endpoint contract per Q1; consumers never call Cartora directly and hold no key | Mapping unit tests; credential-handling unit test (vendor key stays server-side) |
| Caching | Geocode responses cached in a custom cache with a bounded lifetime to respect quota (TTL per Q2) | Cache-hit unit test once Q2 is approved |
| Security | Geocode key server-side in the service credential; maps key domain-restricted and origin-locked at SDK init; quota and billing guardrails on the client's Cartora account; coordinates transient | Credential-handling unit test (key stays server-side); BM service/credential review |
| Performance | The SDK stays off the consumer's critical JS path via the lazy entrypoint | Bundle check on a consumer harness |
| i18n / translations | None — the capability has no shopper-facing copy (vendor display labels are dropped from the contract) | — |

## Verification Strategy

| Target | Evidence | Coverage / trace | Blockers | Notes |
|---|---|---|---|---|
| Server-side geocode interface | Unit tests for vendor mapping into the Q1-approved contract and error mapping; credential-handling unit test asserting the vendor key stays server-side | RQ-711 — valid postcode, valid city, unresolvable input; Cartora key stays out of the browser | Our-side request/response shape waits for Q1 | Vendor service mocked; fixtures follow the design data mapping |
| Geocode governance | Unit test for bounded-lifetime cache hit/miss | RQ-711 | TTL assertion waits for Q2 approval; real credential verification blocked by P1 | — |
| Lazy map entrypoint | Unit test for SDK lazy gate; bundle check | RQ-712 | SDK loader stubbed until real key exists | — |
| Published presentation reference | Unit tests for Style ID reference and marker set | RQ-713, RQ-714 | — | Verified against the brand team's published "Brand Street" style |
| Business Manager service, credential, and key restriction | Platform review of service, credential, and key restriction | RQ-711, RQ-712 | Blocked until the client's platform owner issues the domain-restricted key (P1) | — |
| Coordinate privacy controls | Unit / integration and review evidence that coordinates remain transient and never persisted, logged, or sent to analytics | RQ-711 privacy requirement | — | Proven inside both geocode and map-path verification |

## Risks

| Risk | Impact | Owner | Resolution |
|---|---|---|---|
| Key issuance timing (grooming P1, DEFERRED-APPROVED) | Real-vendor verification and first release blocked until the client's platform owner issues the domain-restricted key | SA / dev lead | Mock-first tests against the design data mapping; verify with the real key before release |
| Quota exhaustion under load | Failed geocode/map requests for consumers | dev lead | Bounded-TTL caching (Q2) and capability-owned throttling per the approved server-proxy decision |
| The brand team re-publishes the v1 presentation without a storefront release | The rendered map changes outside this change's release cycle, so a broken or off-brand publish reaches shoppers with no code deploy to roll back | SA / client brand-content team | Build against the reference and the vendor-default fallback; agree a publish check with the brand team that owns the style |

## Open Questions

| Q# | Severity | Question | Why unclear | Recommendation / Decision | Owner | Status | Approved by |
|---|---|---|---|---|---|---|---|
| Q1 | CRITICAL | What is the consumer-facing contract of the geocode interface — the custom SCAPI endpoint namespace/path and the response schema fields (coordinates, proximity ranking, error result shape)? | No source states the our-side endpoint contract: the ISD fixes the vendor mapping and names a stable consumer interface only; consumers and tests build against it, so a wrong assumption reworks the tasks (`b2c-scapi-custom`, `b2c-custom-api-development`) | Publish a custom SCAPI geocode endpoint with a documented namespace/path and a response schema (`coordinates`, `proximityRanking`, and a clear error result carrying no coordinates) that consumers and tests build against; confirm the exact fields and error shape at approval | dev lead / SA | OPEN | |
| Q2 | NICE-TO-HAVE | What bounded lifetime (TTL) should cached geocode responses use? | The ISD defers it: "the cache lifetime is settled in the technical design" — design and stories settle only that responses are cacheable server-side with a bounded lifetime to respect quota (`b2c-custom-caches`) | A short bounded TTL sized to respect the Cartora quota; confirm the exact value at approval (`b2c-custom-caches`) | dev lead | OPEN | |

## Review & Approval

- [ ] **Approved** — a human reviewed this technical design and approves proceeding to the next phase
- **Approved by:** —
- **Date:** —
- **Review notes:** —
