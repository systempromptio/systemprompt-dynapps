## MODIFIED Requirements

### Requirement: GH.UTL — Footer utility links

<!-- Trace: RQ-709 · Source: SRD RQ-709 (`store-locator-srd-requirements.csv.txt`); design (locator URL `/store-locator`); baseline `openspec/specs/global-navigation/spec.md` -->

The global footer SHALL present utility links grouped into "Customer service",
"About", and "Legal". The "About" group SHALL include an "Our stores" link that
points to the store locator page at `/store-locator`.

#### Scenario: Footer exposes the Our stores link
- **GIVEN** a shopper on any storefront page
- **WHEN** they open the footer
- **THEN** an "Our stores" link is present under "About"
- **AND** it navigates to `/store-locator`
