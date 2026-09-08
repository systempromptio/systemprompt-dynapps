## ADDED Requirements

### Requirement: 3PI.MAP.GEO — Postcode and city geocoding with proximity ranking

<!-- Trace: RQ-711 · Source: SRD RQ-711 (`store-locator-srd-requirements.csv.txt`); technical notes (Cartora geocoding/proximity); grooming Q1 (server proxy) -->

The capability SHALL resolve a postal code or a city to coordinates and return
proximity-ranked results to the consuming feature through its stable interface.
Per the approved geocoding approach, geocoding requests SHALL be served
server-side: the Cartora key SHALL NOT appear in any browser-visible request,
and throttling and quota governance SHALL be owned by the capability, not by
consumers.

#### Scenario: Valid postcode resolves to proximity-ranked results
- **GIVEN** a consumer requesting geocoding for a valid postal code
- **WHEN** the capability processes the request
- **THEN** the consumer receives coordinates and proximity-ranked results for that postal code

#### Scenario: Valid city resolves to proximity-ranked results
- **GIVEN** a consumer requesting geocoding for a city name
- **WHEN** the capability processes the request
- **THEN** the consumer receives coordinates and proximity-ranked results for that city

#### Scenario: Unresolvable input returns a clear error to the consumer
- **GIVEN** a consumer requesting geocoding for input Cartora cannot resolve
- **WHEN** the capability processes the request
- **THEN** the consumer receives a clear error result instead of coordinates

#### Scenario: Cartora key stays out of the browser
- **GIVEN** a consumer using the capability's geocoding
- **WHEN** a shopper's search triggers geocoding requests
- **THEN** no browser-visible request contains the Cartora key

### Requirement: 3PI.MAP.RND — Governed map rendering entrypoint

<!-- Trace: RQ-712 · Source: SRD RQ-712 (`store-locator-srd-requirements.csv.txt`); technical notes (lazy-load); grooming Q2 (domain-restricted key) -->

The capability SHALL expose interactive map rendering to consuming features as
a lazy-loadable entrypoint: the map SHALL load only when the consumer requests
it, and the consumer's content SHALL stay usable before and while the map is
loading. Consumers SHALL NOT hold or configure any Cartora key; map requests
SHALL carry the capability-governed key, which is domain-restricted to the
storefront origins with quota and billing on the client's Cartora account.

#### Scenario: Map loads on consumer demand without blocking content
- **GIVEN** a consuming feature whose page content has rendered
- **WHEN** the consumer requests the map entrypoint
- **THEN** an interactive map is rendered
- **AND** the consumer's content stays usable before and while the map loads

#### Scenario: Consumers render maps without holding a vendor key
- **GIVEN** a consuming feature integrating the map entrypoint
- **WHEN** the map renders
- **THEN** the consumer supplies no Cartora key of its own
- **AND** map requests carry the capability's domain-restricted key

### Requirement: 3PI.MAP.STY — Consumed map presentation reference

<!-- Trace: RQ-713 · Source: SRD RQ-713 (`store-locator-srd-requirements.csv.txt`); technical notes (vendor-hosted presentation, no-code surface); discovery review (v1 presentation and its owner) -->

The presentation — map type and brand theme — is authored and published on the
vendor's hosted surface by the client's brand/content team, outside this
capability. The capability SHALL render its map surface against the published
presentation it is pointed at, and SHALL hold a reference to that presentation
rather than a copy, so a re-published presentation reaches consumers with no
release. The v1 presentation is the brand/content team's published "Brand Street"
style, which that team authors and owns.

#### Scenario: Published presentation reaches the map surface
- **GIVEN** a published presentation carrying a map type and brand theme
- **WHEN** a consumer renders the capability's map surface
- **THEN** the map surface is rendered against that published presentation

#### Scenario: Re-published presentation needs no release
- **GIVEN** a consumer rendering the capability's map surface
- **WHEN** the brand team re-publishes the presentation on the vendor surface
- **THEN** the changed map type and theme reach the consumer without a code change

#### Scenario: Unresolvable presentation still yields a usable map
- **GIVEN** a referenced presentation the capability cannot resolve
- **WHEN** a consumer renders the capability's map surface
- **THEN** a usable map surface is rendered on the vendor's default presentation
- **AND** the consuming feature's render does not fail

### Requirement: 3PI.MAP.MRK — Consumed store marker set

<!-- Trace: RQ-714 · Source: SRD RQ-714 (`store-locator-srd-requirements.csv.txt`); technical notes (marker artwork inside the vendor-hosted presentation) -->

The marker artwork is authored and published inside the same vendor-hosted
presentation the client's brand/content team owns. The capability SHALL plot
store locations with the marker set that presentation carries and SHALL define
no marker artwork of its own, so the published set stays the single source for
how a store appears on the map.

#### Scenario: Brand-authored markers plot the stores
- **GIVEN** a published presentation carrying a custom marker set
- **WHEN** the capability plots store locations on the map surface
- **THEN** the stores are plotted with that published marker set

#### Scenario: Presentation without custom markers falls back to the default
- **GIVEN** a published presentation carrying no custom marker set
- **WHEN** the capability plots store locations on the map surface
- **THEN** the stores are plotted using the vendor's default marker

### Requirement: 3PI.MAP.GEO — Shopper coordinate privacy

<!-- Trace: RQ-711 · Source: technical notes (privacy: coordinates never persisted or sent to analytics) -->

The capability SHALL treat shopper coordinates as transient: coordinates passed
through geocoding or map rendering SHALL NOT be persisted and SHALL NOT be sent
to analytics.

#### Scenario: Coordinates are not persisted or sent to analytics
- **GIVEN** a shopper interaction that passed coordinates through the capability
- **WHEN** the geocoding or map request completes
- **THEN** the shopper's coordinates are not persisted by the capability
- **AND** the shopper's coordinates are not sent to analytics
