# SLS - Store Locator — User Stories

| Field                   | Value                   |
| ----------------------- | ----------------------- |
| **Code - Feature Name** | SLS - Store Locator |
| **Change**              | `openspec/changes/add-store-locator` |
| **Behaviour source**    | `openspec/changes/add-store-locator/specs/**/*.md` with Trace/WBS metadata |

## Stories

### TBD-1 — SLS.SRCH — Store search by postal code, city, or location

**User story:** As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.

**Acceptance criteria:** [SLS.SRCH — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria)

**Implementation notes:**

- Rendering & hydration: SSR/RSC page shell + accessible list; map lazy-loaded client-side — `sfnext-routing`
- Security: Geolocation user-initiated and permission-gated; coordinates never persisted or sent to analytics; no vendor keys held by the feature — `sfnext-authentication`

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-2 — SLS.AC — Address autocomplete suggestions

**User story:** As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions.

**Acceptance criteria:** [SLS.AC — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.1)

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-3 — SLS.MAP — Interactive map with a synchronized list

**User story:** Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other.

**Acceptance criteria:** [SLS.MAP — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.2)

**Implementation notes:**

- Rendering & hydration: SSR/RSC page shell + accessible list; map lazy-loaded client-side — `sfnext-routing`
- Performance: Lazy-load the map; list-first render keeps results usable off the critical path — `sfnext-performance`

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-4 — SLS.DTL — Store details

**User story:** Each store SHALL display its name, full address, phone number, opening hours and the services available at that store.

**Acceptance criteria:** [SLS.DTL — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.3)

**Implementation notes:**

- Caching: Store data is read-only and stable, so it is a caching candidate at the storefront/edge (HTTP); no new caching layer is introduced — `b2c-ecdn`

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-5 — SLS.FLT — Filter stores by service

**User story:** The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping.

**Acceptance criteria:** [SLS.FLT — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.4)

**Implementation notes:**

- Caching: Store data is read-only and stable, so it is a caching candidate at the storefront/edge (HTTP); no new caching layer is introduced — `b2c-ecdn`

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-6 — SLS.DIR — Get directions

**User story:** Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination.

**Acceptance criteria:** [SLS.DIR — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.5)

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-7 — SLS.A11Y — Accessible, keyboard-operable store locator

**User story:** The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map.

**Acceptance criteria:** [SLS.A11Y — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.6)

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-8 — SLS.I18N — Localization

**User story:** All shopper-facing text in the store locator SHALL be localizable.

**Acceptance criteria:** [SLS.I18N — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.7)

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)

### TBD-9 — GH.UTL — Navigation entry and retired directory URL

**User story:** The store locator SHALL be reachable from the global site navigation.

**Acceptance criteria:** [GH.UTL — Acceptance Criteria](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator#Acceptance-Criteria.8)

**References:**

- FSD: [Store Locator](https://astound.atlassian.net/wiki/spaces/SFX/pages/4102987654/SLS+-+Store+Locator)
