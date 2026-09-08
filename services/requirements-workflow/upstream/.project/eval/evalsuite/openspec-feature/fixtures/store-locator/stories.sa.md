# Cartora Maps Integration — User Stories

| Field                   | Value                   |
| ----------------------- | ----------------------- |
| **Code - Feature Name** | Cartora Maps Integration |
| **Change**              | `openspec/changes/add-cartora-maps-integration` |
| **Behaviour source**    | `openspec/changes/add-cartora-maps-integration/specs/**/*.md` with Trace/WBS metadata |

## Stories

### TBD-1 — 3PI.MAP.GEO — Geocoding with proximity ranking

**User story:** As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first.

**Acceptance criteria:** 3PI.MAP.GEO — Acceptance Criteria (see the Cartora Maps Integration ISD)

**Implementation notes:**

- Rendering & hydration: Client-only SDK, exposed as a lazy-loadable entrypoint — `sfnext-routing`
- Caching: If a server proxy is adopted (Open Question Q1), geocode responses are cacheable server-side with a bounded lifetime to respect quota; a client-side call relies on browser/CDN caching instead — `b2c-custom-caches`
- Security: Domain-restricted Cartora key governed centrally by the capability; consumers hold no Cartora key of their own. Whether the key stays server-side or is domain-restricted in the browser follows the call approach (Q1) — `b2c-webservices`
- Performance: Lazy-loadable entrypoint; proximity handled by Cartora — `sfnext-performance`

**References:**

- ISD: Cartora Maps Integration

### TBD-2 — 3PI.MAP.RND — Governed map rendering

**User story:** As a shopper, I want to see nearby stores on an interactive map, so that I can locate a store to visit.

**Acceptance criteria:** 3PI.MAP.RND — Acceptance Criteria (see the Cartora Maps Integration ISD)

**Implementation notes:**

- Rendering & hydration: Client-only SDK, exposed as a lazy-loadable entrypoint — `sfnext-routing`
- Caching: If a server proxy is adopted (Open Question Q1), geocode responses are cacheable server-side with a bounded lifetime to respect quota; a client-side call relies on browser/CDN caching instead — `b2c-custom-caches`
- Security: Domain-restricted Cartora key governed centrally by the capability; consumers hold no Cartora key of their own. Whether the key stays server-side or is domain-restricted in the browser follows the call approach (Q1) — `b2c-webservices`
- Performance: Lazy-loadable entrypoint; proximity handled by Cartora — `sfnext-performance`

**References:**

- ISD: Cartora Maps Integration

### TBD-3 — 3PI.MAP.STY — Consumed map presentation (style and map type)

**User story:** As a shopper, I see the store map in the brand's configured style and map type (street/satellite).

**Acceptance criteria:** 3PI.MAP.STY — Acceptance Criteria (see the Cartora Maps Integration ISD)

**Implementation notes:**

- Rendering & hydration: Client-only SDK, exposed as a lazy-loadable entrypoint — `sfnext-routing`
- Caching: If a server proxy is adopted (Open Question Q1), geocode responses are cacheable server-side with a bounded lifetime to respect quota; a client-side call relies on browser/CDN caching instead — `b2c-custom-caches`
- Security: Domain-restricted Cartora key governed centrally by the capability; consumers hold no Cartora key of their own. Whether the key stays server-side or is domain-restricted in the browser follows the call approach (Q1) — `b2c-webservices`
- Performance: Lazy-loadable entrypoint; proximity handled by Cartora — `sfnext-performance`

**References:**

- ISD: Cartora Maps Integration

### TBD-4 — 3PI.MAP.MRK — Consumed store markers

**User story:** As a shopper, I see stores on the map with the brand's custom markers/pins.

**Acceptance criteria:** 3PI.MAP.MRK — Acceptance Criteria (see the Cartora Maps Integration ISD)

**Implementation notes:**

- Rendering & hydration: Client-only SDK, exposed as a lazy-loadable entrypoint — `sfnext-routing`
- Caching: If a server proxy is adopted (Open Question Q1), geocode responses are cacheable server-side with a bounded lifetime to respect quota; a client-side call relies on browser/CDN caching instead — `b2c-custom-caches`
- Security: Domain-restricted Cartora key governed centrally by the capability; consumers hold no Cartora key of their own. Whether the key stays server-side or is domain-restricted in the browser follows the call approach (Q1) — `b2c-webservices`
- Performance: Lazy-loadable entrypoint; proximity handled by Cartora — `sfnext-performance`

**References:**

- ISD: Cartora Maps Integration
