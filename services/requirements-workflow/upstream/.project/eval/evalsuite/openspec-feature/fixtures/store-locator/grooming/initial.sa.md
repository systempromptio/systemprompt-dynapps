# add-cartora-maps-integration — Grooming

Status: **DRAFT — human approval required.** The agent prepared this ledger; no item is decided until a named human approves it in the grooming session.

## Executive Summary

The pre-grooming pipeline is complete: discovery pinned the integration-only scope (Cartora owns map rendering and geocoding/proximity; everything else stays with its owning change), design specified the service capability and its contract, and the proposal offers the scope as four consumer-value slices. This is the first point discovery, design, and proposal converge, so the session's decision surface is: two technical decisions (the geocoding call approach and the Cartora key/quota guardrails), readiness of the three operational prerequisites (the unconfirmed domain-restricted Cartora API key — the build blocker — the client-owned commercial account for billing and quota, and the published Cartora Map Studio Style ID whose handoff to the build is still pending), acknowledgement of the two developer-HOW items design routed to planning (PD1 geocode-cache strategy, PD2 SDK bundle load/version), and sign-off of the Requirements Lock and Delivery Breakdown as offered. No requirements questions and no change requests are pending for this integration.

Nothing below is approved yet — every decision cell is empty, every route unacknowledged, and every gate box unchecked by design. The pipeline stays blocked until the Grooming Approval Gate passes with named human approvers.

## Grooming Source Inventory

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/add-cartora-maps-integration/discovery.md` | Q1–Q2 origin and severities, integration-only scope, source anchors |
| Design (this change) | `openspec/changes/add-cartora-maps-integration/design.md` | Settled capability decisions, Integration Specification, operational prerequisites, Planning Deferrals PD1–PD2 |
| Proposal (this change) | `openspec/changes/add-cartora-maps-integration/proposal.md` | Requirements Lock, Delivery Breakdown S1–S4, External Dependencies & Access |
| Tracker | JIRA `SFX-4102` — "Store Locator (find a store)" (project-context `jira`: SFX-4102) | Consumer-feature tracker context; the sources record no dedicated ticket for this integration |

## Prerequisites & Access

| P# | Prerequisite / access | Origin | Needed for | Owner / provider | Current status (from sources) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|
| P1 | Cartora API key — domain-restricted to the storefront origins | proposal External Dependencies & Access — Cartora API key | S1–S4 — the capability cannot call Cartora without a ready, restricted key | Client platform developer | unconfirmed (Q2) — blocks build | — | — | BLOCKED |
| P2 | Cartora commercial account — billing and usage quota | proposal External Dependencies & Access — Cartora commercial account | Quota and billing governance for every Cartora call | Client account / procurement owner | client-owned; quota and billing guardrails pending (Q2) | — | — | OPEN |
| P3 | Published Cartora Map Studio Style ID (the vendor-hosted presentation the connector renders by reference) | proposal External Dependencies & Access — published Map Style ID | S2, S3, S4 (RQ-712, RQ-713, RQ-714) | Client brand/content team (authored on the vendor's hosted surface) | v1 "Brand Street" style settled at the discovery review; the published Style ID handoff to the build is pending | — | — | OPEN |

## BA / Requirements Questions

| Q# | Origin | Severity | Question | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| None | — | — | No open requirements questions — Q1 and Q2 are technical/operational decisions carried below | — | — | — | — | — | — |

## Architecture / Tech Decisions

| Q# | Origin | Severity | Decision needed | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | discovery Open Questions Q1; design Open Questions Q1 (D6) | NICE-TO-HAVE | Geocode typed postcodes client-side with Cartora or via a server proxy? | Determines Cartora key exposure, quota/billing ownership, and where throttling and error handling live (S1 seed keeps this undecided) | (a) direct client-side call — simpler, no extra hop, but the key is exposed in the browser and throttling sits client-side; (b) server proxy — hides the key and centralizes quota/throttling and caching, at the cost of a hop | (b) server proxy, the design's proposed direction — recommendation only, not approval | — | — | OPEN |
| Q2 | discovery Open Questions Q2; design Open Questions Q2 | NICE-TO-HAVE | Which domain restrictions and quota/billing guardrails should the Cartora key use? | A Cartora account may already exist, but the key policy is undocumented and setup unconfirmed — this also gates P1/P2 | (a) confirm the existing account with the client's Cartora owner and set domain restriction + quota/billing guardrails on it; (b) no existing account confirmed — client provisions one before build | Resolve together with P1: have the client's Cartora account owner confirm account, key policy, and guardrails in this session | — | — | OPEN |

## Planning Routes

| PD# | Origin (design section/decision) | Deferred item | Target phase | Owner | Route acknowledgement | Approved by | Status |
|---|---|---|---|---|---|---|---|
| PD1 | design Planning Deferrals — Base Configuration & Cross-Cutting (Caching) | Geocode-response cache strategy: TTL, cache key, and invalidation for server-proxied geocode results | technical-design planning | tech-design | — | — | OPEN |
| PD2 | design Planning Deferrals — D5 (lazy-loadable entrypoint) | Cartora Maps SDK bundle load + version-pinning strategy for the lazy entrypoint | technical-design planning | SA / tech-design | — | — | OPEN |

## Change Request Decisions

| CR# | Origin | Description | Impact | Proposed disposition (from proposal) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|
| None | — | Discovery carries no CR candidates for this integration; the appointment-booking ask belongs to the functional surface and a separate appointment integration | — | — | — | — | — |

## Scope & Offer Approval

| O# | Origin | Item | What the human approves | Decision | Approved by | Status |
|---|---|---|---|---|---|---|
| O1 | proposal Requirements Lock | Requirements Lock | The lock as proposed: RQ-711, RQ-712, RQ-713, and RQ-714 in scope as Cartora obligations; every other requirement out of scope with its owning change named | — | — | OPEN |
| O2 | proposal Delivery Breakdown S1–S4 | Delivery Breakdown | Four consumer-value slices S1–S4 (one per in-scope requirement) with their acceptance-criteria seeds as the approved slicing for specs and the ISD | — | — | OPEN |

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
