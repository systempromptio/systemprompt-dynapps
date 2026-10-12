# [DEMO] Linen-polo browsing — Discovery & Requirements

Fictional Linden & Rowe migration; dataset `demo-spa-migration`. This is a draft derived from the frozen Knowledge Bank export, not a statement about Charles Tyrwhitt.

## Executive Summary

The fictional retailer is replacing its legacy storefront frontend with an SPA while retaining Salesforce Commerce Cloud as the commerce system of record. This functional slice preserves linen-polo browsing and adds shareable category selections. The Requirements Trace identifies five browsing requirements to build and keeps the other programme workstreams with their named owners. Affected Capabilities identifies navigation and catalog modifications plus new category-state behaviour. Dependencies records the existing catalog adapter and separately owned route/content capabilities.

The baseline resolves the earlier failure-versus-empty conflict in favour of a distinct failure state and retry. Fabric-weight filtering and reserve-in-store remain unapproved change requests. Loading visuals, retry wording and focus treatment remain design refinements in Open Questions. No unresolved critical WHAT decision blocks preparation of the next draft; human approval remains pending, and this document does not authorize publication.

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
| --- | --- | --- |
| Requirements Trace | Separates the browsing build from programme-owned and deferred work | In scope: 1.1, 1.2, 1.3, 1.4, 1.5; Deferred: 3.5 → Search; Out of scope: 2.1, 2.2, 2.3, 2.4, 2.5 → Product detail; 3.1, 3.2, 3.3, 3.4 → Search; 4.1, 4.2, 4.3, 4.4, 4.5 → Cart; 5.1, 5.2, 5.3, 5.4 → Checkout; 6.1, 6.2, 6.3, 6.4 → Accounts; 7.1, 7.2, 7.3, 7.4 → Content and experience; 8.1, 8.2, 8.3, 8.4 → Migration readiness |
| Affected Capabilities | Names the capabilities changed by this slice | global-navigation MODIFY; product-catalog MODIFY; category-state ADD |
| Dependencies | Names capabilities consumed without taking their implementation ownership | Catalog adapter; centrally managed navigation; reviewed legacy route map; consent gating |
| Open Questions | Carries design refinements into design and grooming | Q1 — NICE-TO-HAVE, Open; Q2 — NICE-TO-HAVE, Open; Q3 — NICE-TO-HAVE, Open |
| Change Request Candidates | Keeps additions outside the baseline | CR-DEMO-01 — proposed, not approved; CR-DEMO-02 — proposed, not approved |
| Cross-cutting Requirements | Records applicable inherited storefront constraints | Accessibility; SEO; Consent / privacy |
| Delivery Scope & Targets | Identifies the draft functional output | functional; FSD NEW → openspec/changes/demo-linen-polo-browsing/fsd/linen-polo-browsing.md |


## Discovery Source Inventory

| Source | Reference / Anchor | Purpose |
| --- | --- | --- |
| [DEMO] Filter behaviour notes | [1374093465](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374093465) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Browsing baseline specification | [1374126144](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126144) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Navigation and URL notes | [1374126163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126163) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Baseline requirements register | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Content and analytics boundaries | [1374224885](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374224885) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Programme overview | [1374322821](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374322821) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Target boundaries | [1374355593](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374355593) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Linen-polo discovery workshop | [1374453777](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374453777) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Linen-polo merchandising examples | [1374453793](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374453793) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Conflicting stakeholder notes | [1374650394](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374650394) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Accessibility review notes | [1374683163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374683163) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Workstream ownership | [1374748676](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374748676) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Design and grooming agenda | [1374748692](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374748692) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Checkout and payment dependencies | [1374814210](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374814210) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Catalog integration contract | [1374846981](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374846981) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Acceptance and evidence index | [1374879762](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374879762) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Legacy estate inventory | [1374912513](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374912513) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Customer identity dependencies | [1374912529](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374912529) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Change request log | [1374912545](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374912545) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [DEMO] Migration cutover runbook | [1374945281](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374945281) | Programme baseline, authority, boundaries or supporting workshop evidence |
| [EAICOMM-35] [DEMO] Category browsing | [EAICOMM-35](https://dynapps.atlassian.net/browse/EAICOMM-35) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-36] [DEMO] RQ-SPA-001 Category identity and breadcrumbs | [EAICOMM-36](https://dynapps.atlassian.net/browse/EAICOMM-36) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-37] [DEMO] RQ-SPA-002 Product cards and prices | [EAICOMM-37](https://dynapps.atlassian.net/browse/EAICOMM-37) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-38] [DEMO] RQ-SPA-003 Filter and sort state | [EAICOMM-38](https://dynapps.atlassian.net/browse/EAICOMM-38) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-39] [DEMO] RQ-SPA-004 Shareable category navigation | [EAICOMM-39](https://dynapps.atlassian.net/browse/EAICOMM-39) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-40] [DEMO] RQ-SPA-005 Loading, empty and failure states | [EAICOMM-40](https://dynapps.atlassian.net/browse/EAICOMM-40) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-41] [DEMO] Legacy regression evidence — Category browsing | [EAICOMM-41](https://dynapps.atlassian.net/browse/EAICOMM-41) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-42] [DEMO] Evidence and ownership review — Category browsing | [EAICOMM-42](https://dynapps.atlassian.net/browse/EAICOMM-42) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-43] [DEMO] Product detail | [EAICOMM-43](https://dynapps.atlassian.net/browse/EAICOMM-43) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-44] [DEMO] RQ-SPA-006 Variant selection | [EAICOMM-44](https://dynapps.atlassian.net/browse/EAICOMM-44) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-45] [DEMO] RQ-SPA-007 Product content | [EAICOMM-45](https://dynapps.atlassian.net/browse/EAICOMM-45) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-46] [DEMO] RQ-SPA-008 Stock messaging | [EAICOMM-46](https://dynapps.atlassian.net/browse/EAICOMM-46) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-47] [DEMO] RQ-SPA-009 Image gallery | [EAICOMM-47](https://dynapps.atlassian.net/browse/EAICOMM-47) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-48] [DEMO] RQ-SPA-010 Product recommendations | [EAICOMM-48](https://dynapps.atlassian.net/browse/EAICOMM-48) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-49] [DEMO] Legacy regression evidence — Product detail | [EAICOMM-49](https://dynapps.atlassian.net/browse/EAICOMM-49) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-50] [DEMO] Evidence and ownership review — Product detail | [EAICOMM-50](https://dynapps.atlassian.net/browse/EAICOMM-50) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-51] [DEMO] Search | [EAICOMM-51](https://dynapps.atlassian.net/browse/EAICOMM-51) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-52] [DEMO] RQ-SPA-011 Search submission | [EAICOMM-52](https://dynapps.atlassian.net/browse/EAICOMM-52) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-53] [DEMO] RQ-SPA-012 Search result refinement | [EAICOMM-53](https://dynapps.atlassian.net/browse/EAICOMM-53) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-54] [DEMO] RQ-SPA-013 No-result guidance | [EAICOMM-54](https://dynapps.atlassian.net/browse/EAICOMM-54) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-55] [DEMO] RQ-SPA-014 Search service failure | [EAICOMM-55](https://dynapps.atlassian.net/browse/EAICOMM-55) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-56] [DEMO] RQ-SPA-015 Search suggestions | [EAICOMM-56](https://dynapps.atlassian.net/browse/EAICOMM-56) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-57] [DEMO] Legacy regression evidence — Search | [EAICOMM-57](https://dynapps.atlassian.net/browse/EAICOMM-57) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-58] [DEMO] Evidence and ownership review — Search | [EAICOMM-58](https://dynapps.atlassian.net/browse/EAICOMM-58) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-59] [DEMO] Cart | [EAICOMM-59](https://dynapps.atlassian.net/browse/EAICOMM-59) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-60] [DEMO] RQ-SPA-016 Add to bag | [EAICOMM-60](https://dynapps.atlassian.net/browse/EAICOMM-60) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-61] [DEMO] RQ-SPA-017 Update quantities | [EAICOMM-61](https://dynapps.atlassian.net/browse/EAICOMM-61) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-62] [DEMO] RQ-SPA-018 Cart totals | [EAICOMM-62](https://dynapps.atlassian.net/browse/EAICOMM-62) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-63] [DEMO] RQ-SPA-019 Guest cart persistence | [EAICOMM-63](https://dynapps.atlassian.net/browse/EAICOMM-63) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-64] [DEMO] RQ-SPA-020 Cart stock conflict | [EAICOMM-64](https://dynapps.atlassian.net/browse/EAICOMM-64) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-65] [DEMO] Legacy regression evidence — Cart | [EAICOMM-65](https://dynapps.atlassian.net/browse/EAICOMM-65) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-66] [DEMO] Evidence and ownership review — Cart | [EAICOMM-66](https://dynapps.atlassian.net/browse/EAICOMM-66) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-67] [DEMO] Checkout | [EAICOMM-67](https://dynapps.atlassian.net/browse/EAICOMM-67) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-68] [DEMO] RQ-SPA-021 Guest checkout | [EAICOMM-68](https://dynapps.atlassian.net/browse/EAICOMM-68) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-69] [DEMO] RQ-SPA-022 Delivery selection | [EAICOMM-69](https://dynapps.atlassian.net/browse/EAICOMM-69) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-70] [DEMO] RQ-SPA-023 Payment handoff | [EAICOMM-70](https://dynapps.atlassian.net/browse/EAICOMM-70) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-71] [DEMO] RQ-SPA-024 Order confirmation | [EAICOMM-71](https://dynapps.atlassian.net/browse/EAICOMM-71) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-72] [DEMO] Legacy regression evidence — Checkout | [EAICOMM-72](https://dynapps.atlassian.net/browse/EAICOMM-72) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-73] [DEMO] Evidence and ownership review — Checkout | [EAICOMM-73](https://dynapps.atlassian.net/browse/EAICOMM-73) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-74] [DEMO] Accounts | [EAICOMM-74](https://dynapps.atlassian.net/browse/EAICOMM-74) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-75] [DEMO] RQ-SPA-025 Customer sign-in | [EAICOMM-75](https://dynapps.atlassian.net/browse/EAICOMM-75) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-76] [DEMO] RQ-SPA-026 Order history | [EAICOMM-76](https://dynapps.atlassian.net/browse/EAICOMM-76) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-77] [DEMO] RQ-SPA-027 Address book | [EAICOMM-77](https://dynapps.atlassian.net/browse/EAICOMM-77) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-78] [DEMO] RQ-SPA-028 Password recovery | [EAICOMM-78](https://dynapps.atlassian.net/browse/EAICOMM-78) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-79] [DEMO] Legacy regression evidence — Accounts | [EAICOMM-79](https://dynapps.atlassian.net/browse/EAICOMM-79) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-80] [DEMO] Evidence and ownership review — Accounts | [EAICOMM-80](https://dynapps.atlassian.net/browse/EAICOMM-80) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-81] [DEMO] Content and experience | [EAICOMM-81](https://dynapps.atlassian.net/browse/EAICOMM-81) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-82] [DEMO] RQ-SPA-029 Global navigation | [EAICOMM-82](https://dynapps.atlassian.net/browse/EAICOMM-82) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-83] [DEMO] RQ-SPA-030 Campaign placement | [EAICOMM-83](https://dynapps.atlassian.net/browse/EAICOMM-83) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-84] [DEMO] RQ-SPA-031 Accessibility baseline | [EAICOMM-84](https://dynapps.atlassian.net/browse/EAICOMM-84) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-85] [DEMO] RQ-SPA-032 Analytics consent | [EAICOMM-85](https://dynapps.atlassian.net/browse/EAICOMM-85) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-86] [DEMO] Legacy regression evidence — Content and experience | [EAICOMM-86](https://dynapps.atlassian.net/browse/EAICOMM-86) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-87] [DEMO] Evidence and ownership review — Content and experience | [EAICOMM-87](https://dynapps.atlassian.net/browse/EAICOMM-87) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-88] [DEMO] Migration readiness | [EAICOMM-88](https://dynapps.atlassian.net/browse/EAICOMM-88) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-89] [DEMO] RQ-SPA-033 Legacy route redirects | [EAICOMM-89](https://dynapps.atlassian.net/browse/EAICOMM-89) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-90] [DEMO] RQ-SPA-034 Catalog reconciliation | [EAICOMM-90](https://dynapps.atlassian.net/browse/EAICOMM-90) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-91] [DEMO] RQ-SPA-035 Release rollback | [EAICOMM-91](https://dynapps.atlassian.net/browse/EAICOMM-91) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-92] [DEMO] RQ-SPA-036 Operational monitoring | [EAICOMM-92](https://dynapps.atlassian.net/browse/EAICOMM-92) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-93] [DEMO] Legacy regression evidence — Migration readiness | [EAICOMM-93](https://dynapps.atlassian.net/browse/EAICOMM-93) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| [EAICOMM-94] [DEMO] Evidence and ownership review — Migration readiness | [EAICOMM-94](https://dynapps.atlassian.net/browse/EAICOMM-94) | Tracker context and acceptance evidence; an Open issue is not artifact approval |
| Existing OpenSpec baseline specs | Not supplied as repository files | The Confluence browsing baseline is the available current-state evidence; no filesystem paths or line ranges are asserted |
| Legacy implementation / live reference | https://www.charlestyrwhitt.com/us/shirts-polo-shirts/linen-polo-shirts/ | Public visual reference only; no private source code or actual implementation inspection is asserted |
| Design assets / Figma / committed screenshots | Not supplied | Visual choices remain draft design refinements |
| FSD/ISD/TSD/API/config/source lookup | No existing FSD supplied in this programme snapshot | NEW FSD working copy; the existing adapter boundary is documented in the catalog integration contract |


## Requirements Trace

| RQ ID | WBS | Title | Source requirement | Requirement | Requirement Details | Disposition | Delivery mode | Affected capability | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RQ-SPA-001 | 1.1 | Category identity and breadcrumbs | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-36](https://dynapps.atlassian.net/browse/EAICOMM-36) | Show the linen-polo category title and a home-to-polos breadcrumb; each ancestor opens its category. | A direct category URL and navigation from Polos show the same heading and breadcrumb. | in scope | build | global-navigation | Baseline browsing build. |
| RQ-SPA-002 | 1.2 | Product cards and prices | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-37](https://dynapps.atlassian.net/browse/EAICOMM-37) | Show image, product name, colour and current price on every available product card; show a previous price only when a genuine promotion applies. | Selecting a card opens the matching product and colour; unavailable price data never renders as zero. | in scope | build | product-catalog | Baseline browsing build. |
| RQ-SPA-003 | 1.3 | Filter and sort state | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-38](https://dynapps.atlassian.net/browse/EAICOMM-38) | Support size and colour filters plus price ascending, price descending and recommended sorting; show selected filters and a clear-all control. | Combining filters narrows results; sorting preserves selected filters; clearing resets filters without leaving the category. | in scope | build | product-catalog | Baseline browsing build. |
| RQ-SPA-004 | 1.4 | Shareable category navigation | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-39](https://dynapps.atlassian.net/browse/EAICOMM-39) | Persist filter and sort selections in the URL and restore them on refresh and browser back/forward. | A shared category URL restores the same selections; unknown values are ignored with the remaining valid selections retained. | in scope | build | category-state | Baseline browsing build. |
| RQ-SPA-005 | 1.5 | Loading, empty and failure states | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-40](https://dynapps.atlassian.net/browse/EAICOMM-40) | Show a loading state during catalog requests, a helpful empty state when no products match, and a retry action when catalog loading fails. | A failed request never appears as a successful zero-results response; retry retains the current selections. | in scope | build | product-catalog | Baseline browsing build. Earlier empty-on-failure note is superseded by this later authoritative baseline; see [1374650394](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374650394). |
| RQ-SPA-006 | 2.1 | Variant selection | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-44](https://dynapps.atlassian.net/browse/EAICOMM-44) | Require a valid size and colour before adding the item to the bag. | Changing colour updates the product image and availability; unavailable variants cannot be added. | out of scope | — | — | Owned by Product detail; implementation stays in that workstream. |
| RQ-SPA-007 | 2.2 | Product content | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-45](https://dynapps.atlassian.net/browse/EAICOMM-45) | Render fabric composition, care guidance and a size guide from commerce content. | Missing optional care content does not hide the purchasable product. | out of scope | — | — | Owned by Product detail; implementation stays in that workstream. |
| RQ-SPA-008 | 2.3 | Stock messaging | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-46](https://dynapps.atlassian.net/browse/EAICOMM-46) | Show available, low-stock or unavailable messaging using the commerce availability response. | A stale availability response is rechecked when adding to the bag. | out of scope | — | — | Owned by Product detail; implementation stays in that workstream. |
| RQ-SPA-009 | 2.4 | Image gallery | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-47](https://dynapps.atlassian.net/browse/EAICOMM-47) | Provide keyboard-operable product imagery and meaningful alternative text. | The selected image can be changed without a pointer and focus remains visible. | out of scope | — | — | Owned by Product detail; implementation stays in that workstream. |
| RQ-SPA-010 | 2.5 | Product recommendations | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-48](https://dynapps.atlassian.net/browse/EAICOMM-48) | Show recommendations as a separately loaded optional section. | Recommendation failures do not block product detail or add-to-bag. | out of scope | — | — | Owned by Product detail; implementation stays in that workstream. |
| RQ-SPA-011 | 3.1 | Search submission | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-52](https://dynapps.atlassian.net/browse/EAICOMM-52) | Allow a shopper to submit a textual catalog query from global navigation. | Whitespace-only input is rejected; a valid query opens a shareable results URL. | out of scope | — | — | Owned by Search; implementation stays in that workstream. |
| RQ-SPA-012 | 3.2 | Search result refinement | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-53](https://dynapps.atlassian.net/browse/EAICOMM-53) | Support size and colour filters on search results. | Selected filters are visible and can be removed independently. | out of scope | — | — | Owned by Search; implementation stays in that workstream. |
| RQ-SPA-013 | 3.3 | No-result guidance | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-54](https://dynapps.atlassian.net/browse/EAICOMM-54) | Show the original query and a clear path to browse categories when search returns no matches. | The empty screen preserves the query and offers a new search. | out of scope | — | — | Owned by Search; implementation stays in that workstream. |
| RQ-SPA-014 | 3.4 | Search service failure | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-55](https://dynapps.atlassian.net/browse/EAICOMM-55) | Distinguish search service errors from no matches. | Retry keeps the query and filters. | out of scope | — | — | Owned by Search; implementation stays in that workstream. |
| RQ-SPA-015 | 3.5 | Search suggestions | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-56](https://dynapps.atlassian.net/browse/EAICOMM-56) | Defer type-ahead suggestions to phase two. | Phase-one search works without suggestions and does not call a suggestions service. | deferred | — | — | Deferred phase-two baseline owned by Search; not a new change request. |
| RQ-SPA-016 | 4.1 | Add to bag | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-60](https://dynapps.atlassian.net/browse/EAICOMM-60) | Add the chosen variant and quantity to the active shopper bag. | A confirmed add displays the correct variant; a failed add does not increment the bag count. | out of scope | — | — | Owned by Cart; implementation stays in that workstream. |
| RQ-SPA-017 | 4.2 | Update quantities | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-61](https://dynapps.atlassian.net/browse/EAICOMM-61) | Support valid quantity changes and remove-item actions. | Server-rejected quantities restore the previous accepted quantity and show a useful message. | out of scope | — | — | Owned by Cart; implementation stays in that workstream. |
| RQ-SPA-018 | 4.3 | Cart totals | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-62](https://dynapps.atlassian.net/browse/EAICOMM-62) | Display subtotal, discounts and estimated total from the commerce basket response. | The UI never computes a conflicting promotional total independently. | out of scope | — | — | Owned by Cart; implementation stays in that workstream. |
| RQ-SPA-019 | 4.4 | Guest cart persistence | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-63](https://dynapps.atlassian.net/browse/EAICOMM-63) | Restore the guest basket after refresh within the agreed basket lifetime. | An expired basket produces an empty-bag explanation rather than a server error. | out of scope | — | — | Owned by Cart; implementation stays in that workstream. |
| RQ-SPA-020 | 4.5 | Cart stock conflict | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-64](https://dynapps.atlassian.net/browse/EAICOMM-64) | Explain items that become unavailable before checkout. | An unavailable line is clearly identified and cannot proceed unchanged. | out of scope | — | — | Owned by Cart; implementation stays in that workstream. |
| RQ-SPA-021 | 5.1 | Guest checkout | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-68](https://dynapps.atlassian.net/browse/EAICOMM-68) | Allow checkout without account creation. | A guest can provide contact and delivery details without a forced sign-in. | out of scope | — | — | Owned by Checkout; implementation stays in that workstream. |
| RQ-SPA-022 | 5.2 | Delivery selection | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-69](https://dynapps.atlassian.net/browse/EAICOMM-69) | Present available delivery methods and quoted charges. | Changing address refreshes eligible methods and totals before payment. | out of scope | — | — | Owned by Checkout; implementation stays in that workstream. |
| RQ-SPA-023 | 5.3 | Payment handoff | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-70](https://dynapps.atlassian.net/browse/EAICOMM-70) | Consume the separately owned payment integration and handle its success, cancellation and failure results. | Cancelled payment returns to checkout without presenting a successful order. | out of scope | — | — | Owned by Checkout; implementation stays in that workstream. |
| RQ-SPA-024 | 5.4 | Order confirmation | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-71](https://dynapps.atlassian.net/browse/EAICOMM-71) | Show the server-confirmed order reference and summary after successful placement. | Refreshing confirmation does not place a second order. | out of scope | — | — | Owned by Checkout; implementation stays in that workstream. |
| RQ-SPA-025 | 6.1 | Customer sign-in | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-75](https://dynapps.atlassian.net/browse/EAICOMM-75) | Consume the existing identity service for sign-in and sign-out. | An expired session returns the shopper to sign-in without leaking cached account details. | out of scope | — | — | Owned by Accounts; implementation stays in that workstream. |
| RQ-SPA-026 | 6.2 | Order history | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-76](https://dynapps.atlassian.net/browse/EAICOMM-76) | Display the signed-in customer order list and details. | Unauthenticated requests cannot view another customer order. | out of scope | — | — | Owned by Accounts; implementation stays in that workstream. |
| RQ-SPA-027 | 6.3 | Address book | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-77](https://dynapps.atlassian.net/browse/EAICOMM-77) | Support creating, editing and selecting account delivery addresses. | A rejected address change retains the previous saved address. | out of scope | — | — | Owned by Accounts; implementation stays in that workstream. |
| RQ-SPA-028 | 6.4 | Password recovery | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-78](https://dynapps.atlassian.net/browse/EAICOMM-78) | Offer the existing identity service password-recovery journey. | The response does not reveal whether an email address exists. | out of scope | — | — | Owned by Accounts; implementation stays in that workstream. |
| RQ-SPA-029 | 7.1 | Global navigation | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-82](https://dynapps.atlassian.net/browse/EAICOMM-82) | Render centrally managed category navigation across storefront routes. | The current category remains navigable when optional campaign content fails. | out of scope | — | — | Owned by Content and experience; implementation stays in that workstream. |
| RQ-SPA-030 | 7.2 | Campaign placement | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-83](https://dynapps.atlassian.net/browse/EAICOMM-83) | Consume approved campaign content slots without hard-coding dated promotions. | Expired campaigns disappear according to the content schedule. | out of scope | — | — | Owned by Content and experience; implementation stays in that workstream. |
| RQ-SPA-031 | 7.3 | Accessibility baseline | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-84](https://dynapps.atlassian.net/browse/EAICOMM-84) | Make the phase-one journeys keyboard usable with labelled controls and announced errors. | Focus order remains logical and dynamic catalog errors are announced. | out of scope | — | — | Owned by Content and experience; implementation stays in that workstream. |
| RQ-SPA-032 | 7.4 | Analytics consent | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-85](https://dynapps.atlassian.net/browse/EAICOMM-85) | Emit optional analytics only after the relevant consent choice. | Declining analytics leaves shopping functionality usable. | out of scope | — | — | Owned by Content and experience; implementation stays in that workstream. |
| RQ-SPA-033 | 8.1 | Legacy route redirects | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-89](https://dynapps.atlassian.net/browse/EAICOMM-89) | Maintain a reviewed mapping from legacy category and product URLs to SPA routes. | Every approved mapping resolves without a redirect loop. | out of scope | — | — | Owned by Migration readiness; implementation stays in that workstream. |
| RQ-SPA-034 | 8.2 | Catalog reconciliation | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-90](https://dynapps.atlassian.net/browse/EAICOMM-90) | Compare representative migrated product and variant counts against SFCC. | Missing variants and price discrepancies are reported before release. | out of scope | — | — | Owned by Migration readiness; implementation stays in that workstream. |
| RQ-SPA-035 | 8.3 | Release rollback | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-91](https://dynapps.atlassian.net/browse/EAICOMM-91) | Maintain an operator-reviewed rollback plan for the storefront cutover. | A rehearsal records the restored legacy route and ownership of unresolved incidents. | out of scope | — | — | Owned by Migration readiness; implementation stays in that workstream. |
| RQ-SPA-036 | 8.4 | Operational monitoring | [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836); [EAICOMM-92](https://dynapps.atlassian.net/browse/EAICOMM-92) | Record failed commerce calls and route errors with trace identifiers. | Operators can correlate a failed shopper request without logging payment secrets. | out of scope | — | — | Owned by Migration readiness; implementation stays in that workstream. |

Requirement Details preserves the source register’s Acceptance column verbatim. The register and matching Jira story are cited per row. Dispositions apply the source workstream ownership to the selected browsing slice; they do not remove requirements from the programme.

## Affected Capabilities

| Capability | Spec path | Disposition |
| --- | --- | --- |
| global-navigation | [1374126144](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126144) — repository baseline path not supplied | MODIFY |
| product-catalog | [1374126144](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126144) — repository baseline path not supplied | MODIFY |
| category-state | — (new capability) | ADD (new) |

The browsing baseline explicitly identifies these modifications and the added state behaviour. No separately named retired capability is identified in the supplied sources; replacement of the legacy frontend does not justify inventing a REMOVE capability.

## Dependencies

| Dependency | Type | Status | Used by |
| --- | --- | --- | --- |
| Catalog adapter | data service | already integrated; SFCC connectivity, credentials and rate limits remain with its platform owner; [1374846981](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374846981) | RQ-SPA-001–005 |
| Centrally managed navigation | external capability | owned by Content and experience (7.1); consume its approved content; [1374191836](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374191836) | RQ-SPA-001 |
| Reviewed legacy route map | external capability | owned by Migration readiness (8.1); route review remains with that workstream; [1374126163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126163) | RQ-SPA-004 |
| Consent gating | external capability | owned by Content and experience (7.4); do not invent a tracking contract; [1374224885](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374224885) | Browsing cross-cutting privacy boundary |

Payment and identity implementations belong to Checkout and Accounts respectively and are not dependencies of this browsing slice. Browser APIs and services needed only by the unapproved reserve-in-store request are not listed as dependencies.

## Change Request Candidates

| CR# | Trigger | Description | Impact | Proposed disposition | Status |
| --- | --- | --- | --- | --- | --- |
| CR-DEMO-01 | [1374093465](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374093465); [1374912545](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374912545) | Add fabric-weight filtering beyond the size/colour baseline | Additional WHAT; facet data and UI scope would expand | Keep outside this baseline; commerce lead to decide through change control | proposed, not approved |
| CR-DEMO-02 | [1374912545](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374912545) | Add reserve-in-store behaviour | Additional WHAT and separately scoped service dependencies | Keep outside this baseline; commerce lead to decide through change control | proposed, not approved |


## Open Questions

| Q# | Severity | Question | Why unclear | Tech note (if constrained) | Owner | Status |
| --- | --- | --- | --- | --- | --- | --- |
| Q1 | NICE-TO-HAVE | Which loading indicator should the draft design propose? | Agenda leaves visual treatment open; [1374748692](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374748692) | Loading behaviour itself is required by 1.5 | Category browsing frontend/design | Open |
| Q2 | NICE-TO-HAVE | What retry wording should the draft design propose? | Agenda leaves exact copy open; [1374748692](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374748692) | Retry must preserve selections; failure must not look like zero matches | Category browsing frontend/design | Open |
| Q3 | NICE-TO-HAVE | What focus-restoration treatment should the draft design propose? | Agenda leaves focus treatment open; [1374748692](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374748692) | Keyboard use, visible focus and announced failures remain required; [1374683163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374683163) | Category browsing frontend/design | Open |

These are HOW refinements, not unresolved baseline behaviours. The earlier failure/empty disagreement is already settled by source authority, not by an invented reviewer decision. No closed question or human sign-off is fabricated.

## Cross-cutting Requirements

| Area | Applicable? | Requirement summary (if applicable) |
| --- | --- | --- |
| Accessibility (WCAG 2.2) | yes | Preserve keyboard-operable controls, visible focus, labelled sort selection and announced loading failures from [1374683163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374683163). Formal audit remains pending; no conformance result is claimed. |
| SEO (meta, canonical, structured data, sitemap, noindex) | yes | Consume the reviewed category route mapping owned by Migration readiness; [1374126163](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374126163). No new canonical, structured-data or sitemap rule is supplied by these sources. |
| Consent / privacy (tracking gating, cookies, DNT) | yes | Shopping remains usable when analytics is declined; consent/tracking contracts stay with Content and experience (7.4); [1374224885](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374224885). |
| i18n (user-facing strings) | N/A | No additional locale or translation requirement is supplied for this slice; do not invent supported locales or a translation architecture. Exact source-language retry copy remains Q2. |


## Delivery Scope & Targets

Scope: functional — the selected slice owns shopper interaction and client routing, consumes the already integrated catalog adapter, and retains SFCC. Any new connector boundary requires a separate integration discovery and ISD.

| Type | Disposition | Confluence target | Working copy |
| --- | --- | --- | --- |
| FSD | NEW | TBD — new page below [1374322821](https://dynapps.atlassian.net/wiki/spaces/EAICOMM/pages/1374322821) after actual final browser approval | openspec/changes/demo-linen-polo-browsing/fsd/linen-polo-browsing.md |


## Discovery Gate

- [x] No CRITICAL open questions remain Open; the three open rows are design refinements.
- [x] No unresolved WHAT was silently resolved; the documented baseline, not an invented human decision, settles the earlier failure conflict.
- [x] Executive Summary reflects draft discovery readiness.
- [x] Document Consumption Guide projects the body’s WBS sets, capabilities, dependencies and open items.

These are author checks, not human approval or permission to publish.

## Review & Approval

- [ ] **Approved** — human review remains pending.
- **Approved by:** —
- **Date:** —
- **Review notes:** Draft prepared from the frozen 80-source demo snapshot. The workflow requires actual browser grooming approval and a separate final publication approval. No mock stakeholder note grants either approval.
