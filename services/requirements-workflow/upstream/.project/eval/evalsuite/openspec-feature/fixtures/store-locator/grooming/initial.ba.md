# add-store-locator — Grooming

Status: **DRAFT — human approval required.** The agent prepared this ledger; no item is decided until a named human approves it in the grooming session.

## Executive Summary

The pre-grooming pipeline is complete: discovery reconciled the signed requirement set, design settled the solution shape, and the proposal offers the scope as nine shopper-value slices. This is the first point discovery, design, and proposal converge, so the session's decision surface is: one requirements question (holiday-hours display), one technical decision (the retired `/stores` redirect), readiness of the three consumed capabilities the build depends on, and sign-off of the Requirements Lock and Delivery Breakdown as offered. Design routes one developer-HOW item to planning for this functional change — the exact caching strategy for the read-only store data (PD1) — which the session must acknowledge as routed-to-planning. The one change request (in-store appointment booking) arrives already answered — the client declined it at the discovery review — so it is migrated for the record, not re-opened.

Nothing is settled in this session yet: every decision cell an in-session item owns is empty and every gate box is unchecked by design. The single filled decision is the change request the client already closed upstream, migrated with its named approver and back-link and never re-opened. The pipeline stays blocked until the Grooming Approval Gate passes with named human approvers.

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
| P1 | Cartora maps/geocoding capability delivered | proposal External Dependencies & Access — Cartora maps/geocoding | S1, S3 (RQ-701, RQ-702) | Separate Cartora integration change | to be delivered (separate change) | — | — | OPEN |
| P2 | Addressly address-autocomplete capability delivered | proposal External Dependencies & Access — Addressly autocomplete | S2 (RQ-710) | Separate Addressly integration change | to be delivered (separate change) | — | — | OPEN |
| P3 | Commerce Stores API (store data) | proposal External Dependencies & Access — Commerce Stores API | S4, S5 (RQ-703, RQ-705) | Existing commerce backend service | have (already integrated) | — | — | OPEN |

## BA / Requirements Questions

| Q# | Origin | Severity | Question | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | discovery Open Questions Q1; design Open Questions Q1 | NICE-TO-HAVE | Should the store detail surface special/holiday opening hours? | Store detail (S4) reads hours from the commerce Stores API; whether holiday hours are exposed is unconfirmed, so the display behaviour cannot be specced yet | (a) show holiday hours when the Stores API exposes them — richer detail, conditional on the field; (b) regular hours only in v1 — no API dependency, holiday hours join a later phase | Confirm field availability with the store-data service first; if still unconfirmed at spec time, take (b) | — | — | OPEN |

## Architecture / Tech Decisions

| Q# | Origin | Severity | Decision needed | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q2 | discovery Open Questions Q2; design Open Questions Q2 (D6) | NICE-TO-HAVE | What redirect (type and target) should the retired `/stores` URL use? | `/stores` is bookmarked and indexed; retiring it without a redirect breaks links and hurts SEO (S9 carries the navigation re-point and retirement) | (a) permanent redirect to the new locator page — search standing transfers and bookmarks keep working; (b) temporary redirect — keeps `/stores` indexed and signals the move may revert, wrong for a retirement | (a) permanent redirect to the locator, per the SEO note in discovery | — | — | OPEN |

## Planning Routes

| PD# | Origin (design section/decision) | Deferred item | Target phase | Owner | Route acknowledgement | Approved by | Status |
|---|---|---|---|---|---|---|---|
| PD1 | design Planning Deferrals PD1 (Base Configuration & Cross-Cutting — Caching) | Exact caching strategy for the read-only store data (edge/HTTP cache dimensions, TTL, invalidation) | technical-design | tech-design | — | — | OPEN |

## Change Request Decisions

| CR# | Origin | Description | Impact | Proposed disposition (from proposal) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|
| CR1 | discovery Change Request Candidates CR1; declined at the discovery review 2026-02-11 | "Book an in-store appointment" from the locator — workshop ask, not in the signed SRD | scope+ (new UI flow here plus a separate appointment-booking integration) | none to propose — the client already answered | Declined for this release; carried as a separate future change | Mara (client PO), taken at the discovery review via Elena Sokolova (BA) | APPROVED |

## Scope & Offer Approval

| O# | Origin | Item | What the human approves | Decision | Approved by | Status |
|---|---|---|---|---|---|---|
| O1 | proposal Requirements Lock | Requirements Lock | The lock as proposed: RQ-701–RQ-707, RQ-709, RQ-710 in scope; RQ-708 out of scope (owned end to end by the inventory integration workstream); no other scope introduced | — | — | OPEN |
| O2 | proposal Delivery Breakdown S1–S9 | Delivery Breakdown | Nine shopper-value slices S1–S9 (one per in-scope requirement) with their acceptance-criteria seeds as the approved slicing for specs and stories | — | — | OPEN |

## Grooming Approval Gate

All items above must be APPROVED or DEFERRED-APPROVED by a named human before specs, FSD/ISD, or stories are generated.

- [ ] Every prerequisite (P#) is APPROVED, or DEFERRED-APPROVED with a named human approver and an explicit constraint
- [ ] Every BA / requirements question (Q#) is APPROVED or DEFERRED-APPROVED by a named human
- [ ] Every architecture / tech decision is APPROVED or DEFERRED-APPROVED by a named human, or a named human confirmed none is required
- [ ] Every planning route (PD#) carried from design is acknowledged as ROUTED-TO-PLANNING by a named human, or the register is empty
- [ ] Every Change Request (CR#) has a disposition decided by a named human
- [ ] The Requirements Lock and Delivery Breakdown (O1, O2) are approved by a named human
- [ ] Every migrated or raised item back-links to the upstream artifact section or decision that raised it
- [ ] Approved decisions are written back into proposal.md / design.md where they change scope or approach
- [ ] No approver is "Agent", "LLM", "Cursor", blank, or implied
