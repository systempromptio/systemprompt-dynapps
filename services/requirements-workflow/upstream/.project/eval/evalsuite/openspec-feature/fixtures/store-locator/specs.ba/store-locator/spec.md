## ADDED Requirements

### Requirement: SLS.SRCH — Store search

<!-- Trace: RQ-701 · Source: SRD RQ-701 (`store-locator-srd-requirements.csv.txt`); technical notes (privacy) -->

The store locator SHALL allow a shopper to search for stores by entering a
postal code or a city, or by using their current location. Search by current
location SHALL run only on an explicit shopper action and SHALL be gated on the
browser location permission. Matching stores SHALL be ordered by proximity, and
a search with no matches SHALL show an empty-results message. The shopper's
coordinates SHALL NOT be persisted and SHALL NOT be sent to analytics.

#### Scenario: Search by postal code returns nearby stores
- **GIVEN** a shopper on the store locator
- **WHEN** they enter a valid postal code and submit the search
- **THEN** stores near that postal code are shown, ordered by proximity

#### Scenario: Search by city returns nearby stores
- **GIVEN** a shopper on the store locator
- **WHEN** they enter a city name and submit the search
- **THEN** stores in and around that city are shown, ordered by proximity

#### Scenario: Search by current location with permission granted
- **GIVEN** a shopper who grants the browser location permission
- **WHEN** they choose "use my location"
- **THEN** stores near their current location are shown, ordered by proximity

#### Scenario: Location permission denied falls back to manual search
- **GIVEN** a shopper who denies the browser location permission
- **WHEN** they choose "use my location"
- **THEN** a message explains that the permission is required
- **AND** they can still search by postal code or city

#### Scenario: Search with no matching stores shows an empty state
- **GIVEN** a shopper on the store locator
- **WHEN** their search matches no stores
- **THEN** an empty-results message is shown instead of the result list

#### Scenario: Shopper coordinates stay transient
- **GIVEN** a shopper who searched by current location
- **WHEN** the search completes
- **THEN** the shopper's coordinates are not persisted
- **AND** the shopper's coordinates are not sent to analytics

### Requirement: SLS.SRCH — Management

<!-- Trace: RQ-701 · Source: SRD RQ-701 (`store-locator-srd-requirements.csv.txt`) -->

Administrators SHALL be able to configure the default search radius and its
units, and whether the use-my-location option is offered to shoppers, in
Business Manager.

#### Scenario: Default search radius follows the Business Manager configuration
- **GIVEN** an administrator has set the default search radius and units in Business Manager
- **WHEN** a shopper runs a store search
- **THEN** results are limited to the configured default radius

#### Scenario: Use-my-location availability follows the Business Manager configuration
- **GIVEN** an administrator has disabled the use-my-location option in Business Manager
- **WHEN** a shopper opens the store locator
- **THEN** only postal-code and city search are offered

### Requirement: SLS.AC — Address autocomplete in store search

<!-- Trace: RQ-710 · Source: SRD RQ-710 (`store-locator-srd-requirements.csv.txt`) -->

As the shopper types a postal code or partial address in the store search, the
store locator SHALL offer address autocomplete suggestions. Selecting a
suggestion SHALL run the store search for that address.

#### Scenario: Suggestions appear while typing
- **GIVEN** a shopper typing a postal code or partial address in the store search
- **WHEN** they pause typing
- **THEN** address suggestions matching the typed input are offered

#### Scenario: Selecting a suggestion runs the search
- **GIVEN** address suggestions shown for the typed input
- **WHEN** the shopper selects a suggestion
- **THEN** the store search runs for the selected address

#### Scenario: Search stays usable when no suggestions are available
- **GIVEN** a shopper typing in the store search while suggestions are unavailable
- **WHEN** they submit the typed postal code or city
- **THEN** the search runs on the typed input without suggestions

### Requirement: SLS.MAP — Synchronized map and list

<!-- Trace: RQ-702 · Source: SRD RQ-702 (`store-locator-srd-requirements.csv.txt`); technical notes (lazy-loaded map) -->

The store locator SHALL present search results as an interactive map and a
synchronized list: selecting a store in the list SHALL highlight it on the map,
and selecting a store on the map SHALL highlight it in the list. The result
list SHALL render and stay usable before and while the map is loading.

#### Scenario: Selecting a store in the list highlights it on the map
- **GIVEN** search results shown as a map and a list
- **WHEN** the shopper selects a store in the list
- **THEN** the corresponding store is highlighted on the map

#### Scenario: Selecting a store on the map highlights it in the list
- **GIVEN** search results shown as a map and a list
- **WHEN** the shopper selects a store marker on the map
- **THEN** the corresponding entry is highlighted in the list

#### Scenario: Result list is usable before the map has loaded
- **GIVEN** a shopper whose search results are ready while the map is still loading
- **WHEN** the result list renders
- **THEN** the shopper can browse and select stores from the list without waiting for the map

### Requirement: SLS.MAP — Side-by-side map and list on desktop

<!-- Trace: RQ-702 · Source: SRD RQ-702 (`store-locator-srd-requirements.csv.txt`); grooming Q6 (desktop keeps the map side by side with the list) -->

On desktop the store locator SHALL show the interactive map side by side with
the result list and SHALL keep it visible while the shopper browses the results.

#### Scenario: Map is shown side by side with the list on desktop
- **GIVEN** a shopper on a desktop viewport
- **WHEN** search results are shown
- **THEN** the interactive map is displayed side by side with the result list

#### Scenario: Map stays visible while the shopper browses the list
- **GIVEN** a shopper on a desktop viewport with search results shown
- **WHEN** they browse and select stores in the result list
- **THEN** the map stays visible next to the list

### Requirement: SLS.MAP — List-first layout with map toggle on mobile and tablet

<!-- Trace: RQ-702 · Source: SRD RQ-702 (`store-locator-srd-requirements.csv.txt`); grooming Q6 (mobile and tablet are list-first with the map behind a toggle) -->

On mobile and tablet the store locator SHALL present the result list first with
the map collapsed behind a toggle, so the list is the primary view and stays
usable before the map is opened.

#### Scenario: Result list is shown first with the map behind a toggle on mobile and tablet
- **GIVEN** a shopper on a mobile or tablet viewport
- **WHEN** search results are shown
- **THEN** the result list is the primary view and the map is collapsed behind a toggle the shopper can open

#### Scenario: Result list is usable before the map toggle is opened
- **GIVEN** a shopper on a mobile or tablet viewport with the map still collapsed
- **WHEN** they browse and select stores in the result list
- **THEN** the stores can be browsed and selected without opening the map
- **AND** opening the toggle shows the same results on the map

### Requirement: SLS.MAP — Management

<!-- Trace: RQ-702 · Source: SRD RQ-702 (`store-locator-srd-requirements.csv.txt`) -->

Administrators SHALL be able to configure the number of stores shown per page,
and the load-more batch, in Business Manager.

#### Scenario: Results per page follow the Business Manager configuration
- **GIVEN** an administrator has set the number of stores per page in Business Manager
- **WHEN** a shopper's search returns more stores than that number
- **THEN** the list shows one page of results and offers a load-more control for the rest

### Requirement: SLS.DTL — Store detail

<!-- Trace: RQ-703 · Source: SRD RQ-703 (`store-locator-srd-requirements.csv.txt`); grooming Q1 (regular hours only in v1) -->

Each store SHALL display its name, full address, phone number, regular weekly
opening hours, and the services available at that store. Special/holiday
opening hours are not displayed in v1. The detail SHALL show the values the
store's maintained record exposes, so a record maintained in the commerce
backend is reflected in the detail.

#### Scenario: Viewing a store shows its detail
- **GIVEN** a store in the results
- **WHEN** the shopper opens that store's detail
- **THEN** the name, full address, phone number, regular weekly opening hours, and available services are shown

#### Scenario: Store detail reflects the administered store record
- **GIVEN** an administrator maintains a store's record in the commerce backend (Business Manager)
- **WHEN** a shopper opens that store's detail
- **THEN** the detail shows the name, address, phone, regular weekly opening hours, and services from that record

### Requirement: SLS.FLT — Filter stores by service

<!-- Trace: RQ-705 · Source: SRD RQ-705 (`store-locator-srd-requirements.csv.txt`) -->

The shopper SHALL be able to filter stores by available service:
click-and-collect, returns, and gift wrapping. Applying a filter SHALL update
both the list and the map; clearing the filter SHALL restore the full search
results. A filter matching no stores SHALL show an empty-results message for
the filtered view.

#### Scenario: Filtering by a service narrows list and map
- **GIVEN** search results containing stores with and without click-and-collect
- **WHEN** the shopper filters by click-and-collect
- **THEN** only stores offering click-and-collect remain in the list and on the map

#### Scenario: Clearing the filter restores all stores
- **GIVEN** an active service filter
- **WHEN** the shopper clears the filter
- **THEN** all stores from the search are shown again in the list and on the map

#### Scenario: Filter with no matching stores shows an empty state
- **GIVEN** search results where no store offers the selected service
- **WHEN** the shopper applies that service filter
- **THEN** an empty-results message is shown for the filtered view

### Requirement: SLS.FLT — Management

<!-- Trace: RQ-705 · Source: SRD RQ-705 (`store-locator-srd-requirements.csv.txt`) -->

Administrators SHALL be able to configure which services are offered as filter
options in Business Manager; v1 is seeded with click-and-collect, returns, and
gift wrapping.

#### Scenario: Offered filter services follow the Business Manager configuration
- **GIVEN** an administrator has set which services are offered as filters in Business Manager
- **WHEN** a shopper opens the service filter
- **THEN** only the configured services are offered as filter options

### Requirement: SLS.DIR — Get directions

<!-- Trace: RQ-704 · Source: SRD RQ-704 (`store-locator-srd-requirements.csv.txt`); technical notes (device map hand-off) -->

Each store SHALL provide a "Get directions" action that opens the shopper's own
map provider with the store as the destination.

#### Scenario: Get directions opens the shopper's map provider
- **GIVEN** a store in the results
- **WHEN** the shopper activates "Get directions"
- **THEN** the shopper's map provider opens with that store as the destination

### Requirement: SLS.A11Y — Accessible store locator

<!-- Trace: RQ-706 · Source: SRD RQ-706 (`store-locator-srd-requirements.csv.txt`); technical notes (WCAG 2.2 AA) -->

The store locator SHALL be fully keyboard operable and usable with a screen
reader, and SHALL provide an accessible list alternative to the map. The
accessibility target is WCAG 2.2 AA.

#### Scenario: Keyboard-only operation
- **GIVEN** a keyboard-only shopper on the store locator
- **WHEN** they navigate the search, results, filters, and store actions
- **THEN** every control is reachable and operable without a pointer

#### Scenario: Screen reader announces the result list
- **GIVEN** a shopper using a screen reader
- **WHEN** search results are shown
- **THEN** the store list is announced and usable as an accessible alternative to the map

### Requirement: SLS.I18N — Localizable content

<!-- Trace: RQ-707 · Source: SRD RQ-707 (`store-locator-srd-requirements.csv.txt`) -->

All shopper-facing text in the store locator SHALL be localizable and SHALL be
shown in the shopper's active locale.

#### Scenario: Labels are shown in the active locale
- **GIVEN** a shopper using a non-default locale
- **WHEN** they open the store locator
- **THEN** all labels and messages are shown in that locale

### Requirement: GH.UTL — Retired store directory URL redirects to the locator

<!-- Trace: RQ-709 · Source: grooming Q2 (permanent redirect); design (locator URL `/store-locator`); discovery cross-cutting SEO -->

The store locator SHALL be reachable via the retired store directory URL: a
request to `/stores` SHALL be permanently redirected to the store locator page
at `/store-locator`, so bookmarks and indexed links keep working after the
static directory is retired.

#### Scenario: Opening the retired URL lands on the locator
- **GIVEN** a shopper opening the retired `/stores` URL from a bookmark or search result
- **WHEN** the request resolves
- **THEN** the shopper is permanently redirected to `/store-locator`
