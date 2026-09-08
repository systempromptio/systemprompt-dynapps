# SLS - Store Locator FSD

## General FSD Information
|                  |                                             |
| ---------------- | ------------------------------------------- |
| WBS-Feature Name | SLS - Store Locator                         |
| Project Name     | Meridian Outfitters — Storefront (SFX-4102) |
| Package/Set/Batch |                                            |
| Author/Owner     | Ben Okafor (BA), Dane (SA lead)             |
| FSD Status       | draft                                       |
| Target Approval Date | dd/mm/yyyy                              |
### Astound Approval
|     |             |            |
| --- | ----------- | ---------- |
| SA  | not started | Dane       |
| FE  | not started | TBC        |
| BE  | not started | TBC        |
| QA  | not started | TBC        |

### Meridian Outfitters Approval
|     |             |      |
| --- | ----------- | ---- |
| PO  | not started | Mara |

## Reference Materials
| Material      | Link / reference           | Notes                                                    |
| ------------- | -------------------------- | -------------------------------------------------------- |
| SRD / BRD     | https://wiki.example.com/display/COM/Store+Locator+SRD | Source of SLS requirements                               |

---
## In Scope Functional Requirements

**Re-usability Legend**

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

### SLS.SRCH — Store search by postal code, city, or location

#### User Story

As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Default | Store search — desktop: search field with matching stores listed by proximity | Store search — mobile: search field with the proximity-ordered store list |
| Empty results | Empty results — desktop: no-matches message replacing the store list | Empty results — mobile: no-matches message replacing the store list |
#### Acceptance Criteria

##### [SF Next OOTB]

1. Business users SHALL be able to configure the default search radius and its units in Business Manager.
2. Business users SHALL be able to enable or disable the use-my-location option in Business Manager.

##### [New]

1. The store search SHALL let the shopper search for stores by postal code or city and SHALL show matching stores ordered by proximity.
2. Search by current location SHALL run only on an explicit shopper action and SHALL be gated on the browser location permission.
3. When the location permission is denied, the store search SHALL explain that the permission is required and SHALL still allow search by postal code or city.
4. When a search matches no stores, the store search SHALL show an empty-results message instead of the result list.
5. The store locator SHALL NOT persist the shopper's coordinates and SHALL NOT send them to analytics.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Store search** |  |  |  |  |
| 1 | Search by postal code returns nearby stores | a shopper on the store locator | they enter a valid postal code and submit the search | stores near that postal code are shown, ordered by proximity |
| 2 | Search by city returns nearby stores | a shopper on the store locator | they enter a city name and submit the search | stores in and around that city are shown, ordered by proximity |
| 3 | Search by current location with permission granted | a shopper who grants the browser location permission | they choose "use my location" | stores near their current location are shown, ordered by proximity |
| 4 | Location permission denied falls back to manual search | a shopper who denies the browser location permission | they choose "use my location" | a message explains that the permission is required<br>they can still search by postal code or city |
| 5 | Search with no matching stores shows an empty state | a shopper on the store locator | their search matches no stores | an empty-results message is shown instead of the result list |
| 6 | Shopper coordinates stay transient | a shopper who searched by current location | the search completes | the shopper's coordinates are not persisted<br>the shopper's coordinates are not sent to analytics |
| **Management** |  |  |  |  |
| 7 | Default search radius follows the Business Manager configuration | an administrator has set the default search radius and units in Business Manager | a shopper runs a store search | results are limited to the configured default radius |
| 8 | Use-my-location availability follows the Business Manager configuration | an administrator has disabled the use-my-location option in Business Manager | a shopper opens the store locator | only postal-code and city search are offered |

### SLS.AC — Address autocomplete suggestions

#### User Story

As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Suggestions open | Address autocomplete — desktop: suggestion list under the search field as the shopper types | Address autocomplete — mobile: suggestion list under the search field as the shopper types |
#### Acceptance Criteria

The suggestions themselves come from the address-autocomplete capability a separate change delivers; this slice delivers the type-ahead surface in the store search and how it behaves when suggestions do not arrive.

##### [New]

1. While the shopper types a postal code or partial address, the store search SHALL offer address suggestions; selecting a suggestion SHALL run the search for that address.
2. If suggestions are unavailable, the typed search SHALL still submit and return results.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Suggestions appear while typing | a shopper typing a postal code or partial address in the store search | they pause typing | address suggestions matching the typed input are offered |
| 2 | Selecting a suggestion runs the search | address suggestions shown for the typed input | the shopper selects a suggestion | the store search runs for the selected address |
| 3 | Search stays usable when no suggestions are available | a shopper typing in the store search while suggestions are unavailable | they submit the typed postal code or city | the search runs on the typed input without suggestions |

### SLS.MAP — Interactive map with a synchronized list

#### User Story

Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Map and list | Map and list — desktop: interactive map beside the synchronized result list, map persistently visible | Map and list — mobile: list-first results with the map collapsed behind a toggle |
| Store selected | Store selected — desktop: a store highlighted on both map and list | Store selected — mobile: a store highlighted on the list and on the opened map |
#### Acceptance Criteria

The map itself is rendered by the maps/geocoding capability a separate change delivers, and the landing's promo/editorial region is authored in the Page Designer library the PD workstream owns (RQ-751, RQ-752; v1 ships one curated Featured-Store card, settled at the discovery review). This slice delivers the result view around them — layout, list/map synchronization, and the accessible list that carries the results.

##### [SF Next OOTB]

1. Business users SHALL be able to configure the number of stores shown per page and the load-more batch in Business Manager.

##### [SF Next Modified]

1. The result view SHALL render server-side with the store list first and load the map as a companion afterwards, so the list stays usable while the map loads.

##### [New]

1. On desktop, the result view SHALL show the interactive map beside the result list and keep it visible.
2. On mobile and tablet, the result view SHALL be list-first with the map collapsed behind a toggle, so the result list is the primary view and usable before the map is opened.
3. Wherever the map is open, selecting a store in the list SHALL highlight it on the map and selecting a store on the map SHALL highlight it in the list.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Synchronized map and list** |  |  |  |  |
| 1 | Selecting a store in the list highlights it on the map | search results shown as a map and a list | the shopper selects a store in the list | the corresponding store is highlighted on the map |
| 2 | Selecting a store on the map highlights it in the list | search results shown as a map and a list | the shopper selects a store marker on the map | the corresponding entry is highlighted in the list |
| 3 | Result list is usable before the map has loaded | a shopper whose search results are ready while the map is still loading | the result list renders | the shopper can browse and select stores from the list without waiting for the map |
| **Side-by-side map and list on desktop** |  |  |  |  |
| 4 | Map is shown side by side with the list on desktop | a shopper on a desktop viewport | search results are shown | the interactive map is displayed side by side with the result list |
| 5 | Map stays visible while the shopper browses the list | a shopper on a desktop viewport with search results shown | they browse and select stores in the result list | the map stays visible next to the list |
| **List-first layout with map toggle on mobile and tablet** |  |  |  |  |
| 6 | Result list is shown first with the map behind a toggle on mobile and tablet | a shopper on a mobile or tablet viewport | search results are shown | the result list is the primary view and the map is collapsed behind a toggle the shopper can open |
| 7 | Result list is usable before the map toggle is opened | a shopper on a mobile or tablet viewport with the map still collapsed | they browse and select stores in the result list | the stores can be browsed and selected without opening the map<br>opening the toggle shows the same results on the map |
| **Management** |  |  |  |  |
| 8 | Results per page follow the Business Manager configuration | an administrator has set the number of stores per page in Business Manager | a shopper's search returns more stores than that number | the list shows one page of results and offers a load-more control for the rest |

### SLS.DTL — Store details

#### User Story

Each store SHALL display its name, full address, phone number, opening hours and the services available at that store.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Store detail | Store detail — desktop: store name, address, phone, opening hours, and services | Store detail — mobile: store name, address, phone, opening hours, and services |
#### Acceptance Criteria

The store records behind the detail (name, address, phone, opening hours, services) are maintained by the client's team in the commerce backend and read through the already-integrated Stores API — this change neither owns nor authors them. This slice delivers the detail surface that presents whatever each record exposes.

##### [New]

1. The store detail SHALL show each store's name, full address, phone number, regular weekly opening hours, and the services available at that store; holiday-hours display follows what the commerce Stores API exposes (Q1).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Viewing a store shows its detail | a store in the results | the shopper opens that store's detail | the name, full address, phone number, regular weekly opening hours, and available services are shown |
| 2 | Store detail reflects the administered store record | an administrator maintains a store's record in the commerce backend (Business Manager) | a shopper opens that store's detail | the detail shows the name, address, phone, regular weekly opening hours, and services from that record |

### SLS.FLT — Filter stores by service

#### User Story

The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Filters open | Filters open — desktop: service filter controls with the result list and map | Filters open — mobile: service filter controls with the result list and map |
| No matches | No matches — desktop: empty-results message for the filtered view | No matches — mobile: empty-results message for the filtered view |
#### Acceptance Criteria

##### [SF Next OOTB]

1. Business users SHALL be able to configure which services are offered as filter options in Business Manager; v1 is seeded with click-and-collect, returns, and gift wrapping.

##### [New]

1. The service filter SHALL let the shopper filter stores by the services offered (in v1: click-and-collect, returns, and gift wrapping).
2. Applying a filter SHALL update both the list and the map; clearing the filter SHALL restore the full search results.
3. When a filter matches no stores, the result view SHALL show an empty-results message for the filtered view.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Filter stores by service** |  |  |  |  |
| 1 | Filtering by a service narrows list and map | search results containing stores with and without click-and-collect | the shopper filters by click-and-collect | only stores offering click-and-collect remain in the list and on the map |
| 2 | Clearing the filter restores all stores | an active service filter | the shopper clears the filter | all stores from the search are shown again in the list and on the map |
| 3 | Filter with no matching stores shows an empty state | search results where no store offers the selected service | the shopper applies that service filter | an empty-results message is shown for the filtered view |
| **Management** |  |  |  |  |
| 4 | Offered filter services follow the Business Manager configuration | an administrator has set which services are offered as filters in Business Manager | a shopper opens the service filter | only the configured services are offered as filter options |

### SLS.DIR — Get directions

#### User Story

Each store SHALL provide a "Get directions" action that opens the shopper's map provider with the store as the destination.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Get directions | Get directions — desktop: the directions action on a store that opens the shopper's map app | Get directions — mobile: the directions action on a store that opens the shopper's map app |
#### Acceptance Criteria

##### [New]

1. Each store SHALL provide a "Get directions" action that opens the shopper's own map provider with the store as the destination.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Get directions opens the shopper's map provider | a store in the results | the shopper activates "Get directions" | the shopper's map provider opens with that store as the destination |

### SLS.A11Y — Accessible, keyboard-operable store locator

#### User Story

The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Keyboard and screen-reader flow | Accessible locator — desktop: visible keyboard focus moving through search, list, and map | Accessible locator — mobile: visible keyboard focus moving through search and list |
#### Acceptance Criteria

##### [New]

1. The store locator SHALL be fully keyboard operable and usable with a screen reader, with the list as an accessible alternative to the map (WCAG 2.2 AA).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Keyboard-only operation | a keyboard-only shopper on the store locator | they navigate the search, results, filters, and store actions | every control is reachable and operable without a pointer |
| 2 | Screen reader announces the result list | a shopper using a screen reader | search results are shown | the store list is announced and usable as an accessible alternative to the map |

### SLS.I18N — Localization

#### User Story

All shopper-facing text in the store locator SHALL be localizable.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Localized UI | Localized store locator — desktop: all locator text shown in the shopper's active locale | Localized store locator — mobile: all locator text shown in the shopper's active locale |
#### Acceptance Criteria

##### [SF Next OOTB]

1. All shopper-facing text in the store locator SHALL be localizable through the storefront's existing translation mechanism and SHALL be shown in the shopper's active locale.

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| 1 | Labels are shown in the active locale | a shopper using a non-default locale | they open the store locator | all labels and messages are shown in that locale |

### GH.UTL — Navigation entry and retired directory URL

#### User Story

The store locator SHALL be reachable from the global site navigation.

#### Visualization
| State | Desktop | Mobile |
| --- | --- | --- |
| Navigation entry | Footer navigation — desktop: the "Our stores" link under "About" leading to the store locator | Footer navigation — mobile: the "Our stores" link under "About" leading to the store locator |
#### Acceptance Criteria

##### [SF Next Modified]

1. The footer "Our stores" link under "About" SHALL point to the store locator page instead of the static directory.

##### [Legacy Modified]

1. The static store directory page SHALL be retired at launch, with no shopper-facing entry to it left in the storefront.
2. The retired `/stores` URL SHALL keep working for bookmarked and indexed links by redirecting to the locator instead of returning a not-found page (redirect type/target per Q2).

#### Key behaviours

| # | Scenario | GIVEN | WHEN | THEN |
| --- | --- | --- | --- | --- |
| **Footer utility links** |  |  |  |  |
| 1 | Footer exposes the Our stores link | a shopper on any storefront page | they open the footer | an "Our stores" link is present under "About"<br>it navigates to `/store-locator` |
| **Retired store directory URL redirects to the locator** |  |  |  |  |
| 2 | Opening the retired URL lands on the locator | a shopper opening the retired `/stores` URL from a bookmark or search result | the request resolves | the shopper is permanently redirected to `/store-locator` |

## Deferred Requirements

### SLS.INV — Real-time in-store availability
- **Status:** Not delivered by this change.
- **Reason:** Signed as "Could / Later phase" in the BRD; the discovery review settled that it belongs to the inventory integration workstream, which owns it end to end.
- **Planned delivery:** By the inventory integration workstream, on its own schedule.
- **Business Requirements:**
  1. The store detail shows real-time in-store product availability.

---
## Change Requests

### CR1 — Book an in-store appointment
- **Status:** Declined for this release at the discovery review.
- **Why a change request:** not in the signed BRD — introduced in the discovery workshop.
- **Tracker Ticket:** TBD.
- **Business Requirements:**
  1. The shopper can book an in-store appointment from the store locator. Declined for the initial release; may return as its own change.

## Open Questions

Use inline comments during review whenever possible; this table is only for questions in areas the document does not cover.

| Q# | Question | Owner | Notes | Decision |
| --- | --- | --- | --- | --- |
