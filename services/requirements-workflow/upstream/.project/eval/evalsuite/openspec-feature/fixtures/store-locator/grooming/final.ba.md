# add-store-locator — Grooming

Status: **APPROVED — grooming session complete.** Every item below was decided by a named human in the grooming session on 2026-02-18; the Grooming Approval Gate passes and the pipeline may proceed to specs, FSD, and stories.

## Executive Summary

The grooming session settled the requirements questions and architecture decisions the pre-grooming pipeline raised. The client PO confirmed regular weekly opening hours only for v1 (Q1) and the SA lead confirmed a permanent redirect from the retired `/stores` URL to the locator page (Q2); the appointment-booking ask (CR1) arrived already declined from the discovery review and was recorded rather than re-decided. The convergence review surfaced the grounded merchant-configuration and responsive-layout decisions (Q3–Q6) and settled them with the client PO and SA lead. The three consumed capabilities were dispositioned: the commerce Stores API is available, and the Cartora and Addressly capabilities are deferred-approved — specs and stories proceed against their stable interfaces while implementation of the dependent paths waits on their delivery. Design routed one developer-HOW item to planning for this functional change — the exact caching strategy for the read-only store data (PD1) — which the SA lead acknowledged as routed to technical-design planning. The Requirements Lock and the nine-slice Delivery Breakdown were approved as offered, unchanged.

## Grooming Source Inventory

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-store-locator/discovery.md` | Q1–Q2 origin and severities, CR1 origin, source anchors |
| Design (this change) | `openspec/changes/add-store-locator/design.md` | Settled architecture decisions; risk context behind Q1/Q2; Planning Deferrals (PD1 — caching strategy) |
| Proposal (this change) | `openspec/changes/add-store-locator/proposal.md` | Requirements Lock, Delivery Breakdown S1–S9, CR1 proposed disposition, External Dependencies & Access |
| Tracker | JIRA `SFX-4102` — "Store Locator (find a store)" (project-context `jira`: SFX-4102) | Session reference; Jira stories follow approval |

## Prerequisites & Access

| P# | Prerequisite / access | Origin | Needed for | Owner / provider | Current status (from sources) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|
| P1 | Cartora maps/geocoding capability delivered | proposal External Dependencies & Access — Cartora maps/geocoding | S1, S3 (RQ-701, RQ-702) | Separate Cartora integration change | to be delivered (separate change) | Proceed against the capability's stable interface; the geocoding path of S1 and the map path of S3 release only after the Cartora change delivers | Dane (SA lead) | DEFERRED-APPROVED |
| P2 | Addressly address-autocomplete capability delivered | proposal External Dependencies & Access — Addressly autocomplete | S2 (RQ-710) | Separate Addressly integration change | to be delivered (separate change) | Proceed against the capability's stable interface; the autocomplete path of S2 releases only after the Addressly change delivers | Dane (SA lead) | DEFERRED-APPROVED |
| P3 | Commerce Stores API (store data) | proposal External Dependencies & Access — Commerce Stores API | S4, S5 (RQ-703, RQ-705) | Existing commerce backend service | have (already integrated) | Confirmed available — already integrated on the storefront | Dane (SA lead) | APPROVED |

## BA / Requirements Questions

| Q# | Origin | Severity | Question | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | discovery Open Questions Q1; design Open Questions Q1 | NICE-TO-HAVE | Should the store detail surface special/holiday opening hours? | Store detail (S4) reads hours from the commerce Stores API; whether holiday hours are exposed is unconfirmed, so the display behaviour cannot be specced yet | (a) show holiday hours when the Stores API exposes them — richer detail, conditional on the field; (b) regular hours only in v1 — no API dependency, holiday hours join a later phase | Confirm field availability with the store-data service first; if still unconfirmed at spec time, take (b) | (b) Store detail shows regular weekly opening hours only in v1; special/holiday hours join a later phase | Mara (client PO) | APPROVED |
| Q3 | design Base Configuration — Merchant configuration; discovery RQ-701 Notes | NICE-TO-HAVE | Default search radius, units, and use-my-location toggle | Search (S1) needs a default proximity and units, and the use-my-location option should be switchable per site | (a) hard-code a radius; (b) configure the default radius, units, and the use-my-location toggle in Business Manager | (b) — keep it merchant-configurable per locale | Default search radius, units, and the use-my-location option are configured by administrators in Business Manager | Mara (client PO) | APPROVED |
| Q4 | design Base Configuration — Merchant configuration; discovery RQ-702 Notes | NICE-TO-HAVE | Store list page size | The result list (S3) needs a page size and a load-more behaviour for large result sets | (a) fixed page size; (b) Business-Manager-configurable page size with load-more | (b) — merchant-tunable | The number of stores per page and the load-more batch are configured by administrators in Business Manager; the list paginates with a load-more control | Mara (client PO) | APPROVED |
| Q5 | design Base Configuration — Merchant configuration; discovery RQ-705 Notes | NICE-TO-HAVE | Which services are offered as filters | The service filter (S5) lists services; the offered set may change per merchandising | (a) fix the v1 three in code; (b) configure the offered filter services in Business Manager (v1 seed: click-and-collect, returns, gift wrapping) | (b) — merchant-configurable, seeded with the v1 three | The services offered as filter options are configured by administrators in Business Manager; v1 is seeded with click-and-collect, returns, and gift wrapping | Mara (client PO) | APPROVED |

## Architecture / Tech Decisions

| Q# | Origin | Severity | Decision needed | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q2 | discovery Open Questions Q2; design Open Questions Q2 (D6) | NICE-TO-HAVE | What redirect (type and target) should the retired `/stores` URL use? | `/stores` is bookmarked and indexed; retiring it without a redirect breaks links and hurts SEO (S9 carries the navigation re-point and retirement) | (a) permanent redirect to the new locator page — search standing transfers and bookmarks keep working; (b) temporary redirect — keeps `/stores` indexed and signals the move may revert, wrong for a retirement | (a) permanent redirect to the locator, per the SEO note in discovery | (a) Permanent redirect from `/stores` to the store locator page at launch | Dane (SA lead) | APPROVED |
| Q6 | design D2 (responsive result layout) | NICE-TO-HAVE | Responsive results layout across surfaces | The map+list result view (S3) renders on desktop and on mobile/tablet, where the same side-by-side layout does not fit | (a) one responsive layout that reflows; (b) desktop shows the map beside the list (persistent), mobile/tablet is list-first with the map behind a toggle | (b) — list-first protects the small-viewport experience and the accessible list | (b) Desktop shows the map beside the list and keeps it visible; mobile and tablet use a list-first layout with the map collapsed behind a toggle | Dane (SA lead) | APPROVED |

## Planning Routes

| PD# | Origin (design section/decision) | Deferred item | Target phase | Owner | Route acknowledgement | Approved by | Status |
|---|---|---|---|---|---|---|---|
| PD1 | design Planning Deferrals PD1 (Base Configuration & Cross-Cutting — Caching) | Exact caching strategy for the read-only store data (edge/HTTP cache dimensions, TTL, invalidation) | technical-design | tech-design | Acknowledged as routed to technical-design planning — the storefront caches read-only store data at the edge/HTTP and the concrete cache key, TTL, and invalidation are decided in technical-design | Dane (SA lead) | ROUTED-TO-PLANNING |

## Change Request Decisions

| CR# | Origin | Description | Impact | Proposed disposition (from proposal) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|
| CR1 | discovery Change Request Candidates CR1; declined at the discovery review 2026-02-11 | "Book an in-store appointment" from the locator — workshop ask, not in the signed SRD | scope+ (new UI flow here plus a separate appointment-booking integration) | none to propose — the client already answered | Declined for this release at the discovery review — stays out of scope; carried as a separate future change, may return together with the SlotBook integration | Mara (client PO), taken at the discovery review via Elena Sokolova (BA) | APPROVED |

## Scope & Offer Approval

| O# | Origin | Item | What the human approves | Decision | Approved by | Status |
|---|---|---|---|---|---|---|
| O1 | proposal Requirements Lock | Requirements Lock | The lock as proposed: RQ-701–RQ-707, RQ-709, RQ-710 in scope; RQ-708 out of scope (owned end to end by the inventory integration workstream); no other scope introduced | Approved as offered | Mara (client PO) | APPROVED |
| O2 | proposal Delivery Breakdown S1–S9 | Delivery Breakdown | Nine shopper-value slices S1–S9 (one per in-scope requirement) with their acceptance-criteria seeds as the approved slicing for specs and stories | Approved as offered — the S1–S9 slicing stands | Mara (client PO) | APPROVED |

## Grooming Approval Gate

All items above must be APPROVED or DEFERRED-APPROVED by a named human before specs, FSD/ISD, or stories are generated.

- [x] Every prerequisite (P#) is APPROVED, or DEFERRED-APPROVED with a named human approver and an explicit constraint
- [x] Every BA / requirements question (Q#) is APPROVED or DEFERRED-APPROVED by a named human
- [x] Every architecture / tech decision is APPROVED or DEFERRED-APPROVED by a named human, or a named human confirmed none is required
- [x] Every planning route (PD#) carried from design is acknowledged as ROUTED-TO-PLANNING by a named human, or the register is empty
- [x] Every Change Request (CR#) has a disposition decided by a named human
- [x] The Requirements Lock and Delivery Breakdown (O1, O2) are approved by a named human
- [x] Every migrated or raised item back-links to the upstream artifact section or decision that raised it
- [x] Approved decisions are written back into proposal.md / design.md where they change scope or approach
- [x] No approver is "Agent", "LLM", "Cursor", blank, or implied
