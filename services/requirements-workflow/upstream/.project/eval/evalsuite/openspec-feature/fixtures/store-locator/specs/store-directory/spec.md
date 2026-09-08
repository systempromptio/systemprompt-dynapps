# store-directory

## Purpose

> Provenance: reverse-engineered from the Storefront Next baseline; code is source of truth.

The store directory is the storefront's current way of showing physical store
locations: a single static content page at `/stores`, authored in Page Designer
and maintained manually in Business Manager. It is not backed by the Shopper
Stores API — it has no search, no map, and no live store data.

## Requirements

### Requirement: Static store directory page

The storefront SHALL serve a store directory page at `/stores` that lists all
physical stores as a static, manually maintained list grouped alphabetically by
city. Each entry SHALL show the store name and postal address only.

#### Scenario: Viewing the store directory
- **GIVEN** a shopper who opens `/stores`
- **WHEN** the page renders
- **THEN** all stores are shown as a static list grouped alphabetically by city
- **AND** each entry shows the store name and postal address

### Requirement: Manually authored content

The store directory content SHALL be authored and updated in Page Designer; the
storefront SHALL render it as static content with no search, filtering, map, or
live availability data.

#### Scenario: Directory has no search or map
- **GIVEN** a shopper on the `/stores` directory
- **WHEN** the page renders
- **THEN** no search field, service filter, or map is present
- **AND** the content matches what was authored in Page Designer
