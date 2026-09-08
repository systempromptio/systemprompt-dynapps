## What the user asked for

The user said:

> "I'm the architect on the Store Locator. I want to build the Cartora maps
> integration it needs — map rendering plus postcode/city geocoding and proximity
> — as an agnostic, reusable service capability. The functional storefront UI, the
> Addressly address-autocomplete integration, the SlotBook appointment integration,
> and the commerce store-data service are separate changes owned by others; treat
> those as consumers or separate integrations, not part of my change."

## SRD — Third-Party Integrations (`3PI`) section

> project-context result metadata — source_type: `confluence` (SRD export) · date: `2026-02-06` · date_source: `frontmatter`

The signed SRD groups third-party integrations under their own high-level WBS
(`3PI - Third-Party Integrations`), tracked separately from the functional Store
Locator UI rows (`SLS.*`). Integration rows are still written from the shopper's
perspective, but each one names the vendor obligation the integration owns and the
functional `SLS.*` row that consumes it. The `3PI` rows this Cartora change owns:

```
Highlevel WBS,Lowlevel WBS,Assigned WBS Code,ID,Requirement,Requirement Details,Notes/Dependencies,Re-usability,Release,"MoSCoW Rating",Client Approval,Approved By,Package,Current Release?,Req No.
3PI - Third-Party Integrations,3PI.MAP.GEO - Cartora geocoding & proximity,3PI.MAP.GEO,RQ-711,"As a shopper, I want to search for stores by postal code, city, or my current location, so that I see the nearest stores first.","1. Resolve a typed postal code or city (or a user-initiated current location) to coordinates via Cartora.
2. Rank stores by proximity to the resolved location.
3. Shopper coordinates are never persisted or sent to analytics; client-side vs server-proxy call approach is undecided.","- Cartora geocoding/proximity. Consumed by the Store Locator search UI (SLS.SRCH). Domain-restricted key; billing on the client Cartora account.",New,,Must have,Approved,Ben Okafor,3,Y,11
3PI - Third-Party Integrations,3PI.MAP.RND - Cartora map rendering,3PI.MAP.RND,RQ-712,"As a shopper, I want to see nearby stores on an interactive map, so that I can locate a store to visit.","1. Render store locations on an interactive map surface via the Cartora Maps SDK.
2. Lazy-load the map so it does not block the critical render path.
3. Domain-restricted key; quota and billing sit on the client Cartora account.","- Cartora Maps SDK. Consumed by the Store Locator map UI (SLS.MAP); the synchronized results list and the map/list selection highlighting are functional UI owned by the Store Locator change, not by this integration.",New,,Must have,Approved,Ben Okafor,3,Y,12
```

The functional `SLS.*` rows in Source A (search UI, map + synchronized list, store
detail, directions, filters, a11y, i18n, navigation, address autocomplete) are the
Store Locator functional change and other integrations (Addressly, SlotBook, the
commerce Stores API) — consumers or separate changes, NOT part of this Cartora
integration. In particular, the `SLS.MAP` synchronized list and map/list selection
highlighting are functional consumer UI and stay out of this change.

## SRD — Map presentation configuration (vendor-hosted Cartora Map Studio)

> project-context result metadata — source_type: `confluence` (SRD export) · date: `2026-02-06` · date_source: `frontmatter`

The signed SRD also carries map-presentation configuration rows — the map type,
custom store markers/pins, and brand theme the shopper sees on the store-locator
map. These are authored by the client's brand/content team in **Cartora Map
Studio**, the vendor's hosted, no-code styling console, and published as a **Map
Style ID**. When it renders the store-locator map (RQ-712), THIS connector is what
consumes and renders that published Style ID; there is no separate change, WBS, or
team that owns delivering the store-locator map's style and markers — the Map
Studio console is simply the vendor-hosted authoring surface the connector depends
on. The store-density heatmap (RQ-715) is different in ownership: it is published
for and consumed by the **marketing homepage**, a separate storefront surface owned
by another change, not the store-locator map this integration serves.

```
Highlevel WBS,Lowlevel WBS,Assigned WBS Code,ID,Requirement,Requirement Details,Notes/Dependencies,Re-usability,Release,"MoSCoW Rating",Client Approval,Approved By,Package,Current Release?,Req No.
3PI - Third-Party Integrations,3PI.MAP.STY - Map style & type,3PI.MAP.STY,RQ-713,"As a shopper, I see the store map in the brand's configured style and map type (street/satellite).","1. Map type (street/satellite) and brand theme are configured in Cartora Map Studio (vendor-hosted, no-code) and published as a Map Style ID.
2. The connector renders by referencing the published Style ID; it builds no configuration surface.","- Vendor-hosted config (Cartora Map Studio); authored by the client brand/content team.",Existing (vendor-hosted),,Should have,Approved,Ben Okafor,3,Y,13
3PI - Third-Party Integrations,3PI.MAP.MRK - Custom store markers/pins,3PI.MAP.MRK,RQ-714,"As a shopper, I see stores on the map with the brand's custom markers/pins.","1. A custom store marker/pin set is authored in Cartora Map Studio and published in the same Map Style ID.
2. The connector renders the published markers; no marker-authoring UI is built here.","- Vendor-hosted config (Cartora Map Studio).",Existing (vendor-hosted),,Could have,Approved,Ben Okafor,3,Y,14
3PI - Third-Party Integrations,3PI.MAP.HEAT - Store-density heatmap overlay,3PI.MAP.HEAT,RQ-715,"As a marketer, I want a store-density heatmap overlay for the marketing homepage.","1. Cartora Map Studio can publish a store-density heatmap overlay used by the marketing homepage.","- Vendor-hosted Map Studio capability; marketing-homepage merchandising.",Existing (vendor-hosted),,Could have,Approved,Ben Okafor,3,Y,15
```

## SF Next platform baseline — what the stack already provides

> project-context result metadata — source_type: `confluence` (platform/architecture notes) · date: `2026-01-20` · date_source: `frontmatter`

The Storefront Next baseline this integration sits on, stated as capability facts —
enough to judge, per obligation, whether it **builds on an existing platform
primitive** or is **net-new**. This is standing platform capability, not this
feature's code; cite the platform-notes anchor
(`.cursor/.project/project-context/inbox/confluence/sfnext-platform-baseline.md:<lines>`),
not this eval-only file.

- **Maps / geocoding / proximity:** none out of the box. Storefront Next ships no
  map rendering and no geocoding or proximity capability — so that core is net-new
  (consistent with the SRD `Re-usability: New` on these rows).
- **Rendering (React SSR/RSC):** client components can be lazy-loaded / dynamically
  imported on demand; deferring a heavy client widget past first render is an
  existing platform convention, not new infrastructure.
- **Server tier:** the storefront has a server layer (route/handler tier) that can
  make outbound calls and hold secrets, so serving a vendor call from the server
  rather than the browser reuses an existing platform capability (it does not have
  to be invented).
- **Secrets / config:** vendor keys are supplied through the platform's
  environment-based secret configuration; there is no separate managed key vault
  beyond that.
- **Analytics:** a storefront analytics event pipeline already exists — so
  "coordinates are never sent to analytics" is a governance constraint against an
  existing sink, not a new one.
- **Feature ↔ capability wiring:** features consume shared capabilities through
  stable module interfaces; exposing a capability behind such an interface is the
  established pattern.
- **Legacy storefront:** only a static Page Designer `/stores` directory (no map,
  no geocoding) — nothing map- or geocoding-related exists to port as-is.

## Prior-art search — existing solutions (web + marketplace, emulated)

> project-context result metadata — source_type: `web` (search results captured 2026-02-06)

What already exists that could satisfy the Cartora maps/geocoding need on Storefront
Next, returned as FACTS (no recommendation attached):

- **Cartridge / module (reuse candidate):** a partner listing — "Cartora Commerce
  Maps Cartridge" on the Storefront Next marketplace
  (`https://marketplace.example/cartridges/cartora-commerce-maps`) — advertises a
  drop-in, RSC-compatible integration providing map rendering + postcode/city
  geocoding + proximity for Storefront Next: a map component, a geocode helper,
  domain-restricted key config, and lazy-load. (Marketplace feature blurb only.)
- **Vendor SDK / API (integrate candidate):** Cartora publishes an official Maps
  SDK (client-side rendering) and a Geocoding API (postcode/city → coordinates +
  proximity) — see the Vendor documentation block below.
- **From scratch (build candidate):** the two obligations can also be built
  directly against Cartora's SDK/API with no cartridge, as the platform baseline
  above allows (server tier for a geocode proxy; client lazy-load for the SDK).

## Vendor documentation — Cartora (web, emulated)

> project-context result metadata — source_type: `web` (vendor docs)

- **Cartora Geocoding API** (`https://developers.cartora.example/docs/geocoding`):
  HTTPS GET `…/v1/geocode?q=<postcode|city>&key=<domain-restricted key>&country=<region>&limit=<n>`;
  JSON response `results[].{lat,lng,score,label}` (`score` = proximity ranking).
  Key domain-restricted; billing on the client account.
- **Cartora Maps SDK** (`https://developers.cartora.example/docs/maps-sdk`):
  client-side JS bundle + vector tiles; lazy-loadable; initialized with the
  domain-restricted key; renders an interactive map. Browser SDK — no server
  endpoint of its own.
- **Key domain-restriction & quota policy** (`https://developers.cartora.example/docs/keys`).

## Architect's stated approach preference (approach signal)

> project-context result metadata — source_type: `user request`

Beyond the integration-scope decision above, the architect stated an APPROACH
preference: **build the integration directly against Cartora's own SDK/API (from
scratch), not via a third-party cartridge** — the team wants to own the connector
rather than depend on a marketplace cartridge. (Recorded as a design input;
discovery only notes it and does not act on it.)
