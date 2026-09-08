## REMOVED Requirements

### Requirement: Static store directory page

**Reason:** Superseded by the new `store-locator` capability: shoppers find
stores through search, an interactive map, and live store data instead of a
static, manually maintained list.
**Migration:** The `/stores` URL is permanently redirected to the store locator
page at `/store-locator` (grooming Q2), and the footer "Our stores" entry
re-points to the locator (`global-navigation` delta). No shopper-facing entry to
the static directory remains.

### Requirement: Manually authored content

**Reason:** The store locator reads live store data (name, address, hours,
services) from the commerce Stores API; manually authored Page Designer store
content is retired together with the directory page.
**Migration:** No content migration — the static Page Designer store directory
content is retired; store data is served from the commerce Stores API.
