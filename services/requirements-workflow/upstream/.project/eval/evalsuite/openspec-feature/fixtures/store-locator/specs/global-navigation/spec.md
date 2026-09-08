# global-navigation

## Purpose

> Provenance: reverse-engineered from the Storefront Next baseline; code is source of truth.

The global navigation is the site-wide header and footer shell rendered on every
storefront page. It exposes primary shopping entry points (categories, search,
account, cart) in the header and a set of utility links in the footer.

## Requirements

### Requirement: Header primary entries

The global header SHALL present the primary shopping entries on every page: the
brand logo (links home), the top-level product categories, the search entry, the
account entry, and the mini-cart.

#### Scenario: Header is present on every page
- **GIVEN** a shopper on any storefront page
- **WHEN** the page renders
- **THEN** the header shows the logo, category entries, search, account, and mini-cart

### Requirement: GH.UTL — Footer utility links

The global footer SHALL present utility links grouped into "Customer service",
"About", and "Legal". The "About" group SHALL include an "Our stores" link that
points to the static store directory page at `/stores`.

#### Scenario: Footer exposes the Our stores link
- **GIVEN** a shopper on any storefront page
- **WHEN** they open the footer
- **THEN** an "Our stores" link is present under "About"
- **AND** it navigates to the static store directory at `/stores`

### Requirement: Locale and region switch

The global footer SHALL provide a locale/region switcher that reloads the current
page in the selected locale.

#### Scenario: Switching locale reloads in the chosen locale
- **GIVEN** a shopper on any storefront page
- **WHEN** they choose a different locale in the switcher
- **THEN** the current page reloads with all navigation labels in that locale
