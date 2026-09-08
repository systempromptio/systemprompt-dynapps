# add-cartora-maps-integration — Grooming

Status: **APPROVED — grooming session complete.** Every item below was decided by a named human in the grooming session on 2026-02-18; the Grooming Approval Gate passes and the pipeline may proceed to specs and the ISD.

## Executive Summary

The grooming session settled the requirements questions and architecture decisions the pre-grooming pipeline raised for this integration. The SA lead approved the server-proxy approach for typed-postcode geocoding (Q1): the Cartora key stays server-side and throttling, caching, and quota governance live in the capability. The client's platform owner confirmed the existing Cartora account and the key policy (Q2): the key is domain-restricted to the storefront origins with quota and billing guardrails on the client account; issuing the restricted key is a deferred-approved prerequisite gating release. The published Cartora Map Studio Style ID is a deferred-approved prerequisite (P3): the v1 style itself was settled at the discovery review — the brand/content team's published "Brand Street" style — but the published Style ID must still be handed to the build before first render, and the connector references that ID without building any configuration surface. The two developer-HOW items design routed to planning (PD1 geocode-cache strategy, PD2 SDK bundle load/version) were acknowledged as routed to technical-design planning — the session approves the route, not the developer-HOW. The Requirements Lock and the four-slice Delivery Breakdown were approved as offered, unchanged. No requirements questions and no change requests existed for this integration.

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
| P1 | Cartora API key — domain-restricted to the storefront origins | proposal External Dependencies & Access — Cartora API key | S1–S4 — the capability cannot call Cartora without a ready, restricted key | Client platform developer | unconfirmed (Q2) — blocks build | Existing client Cartora access confirmed in session; the domain-restricted key is issued by the client's platform developer before the capability's first release | Sam (client platform dev) | DEFERRED-APPROVED |
| P2 | Cartora commercial account — billing and usage quota | proposal External Dependencies & Access — Cartora commercial account | Quota and billing governance for every Cartora call | Client account / procurement owner | client-owned; quota and billing guardrails pending (Q2) | Quota and billing stay on the client's Cartora commercial account with the guardrails decided in Q2 | Sam (client platform dev) | APPROVED |
| P3 | Published Cartora Map Studio Style ID (the vendor-hosted presentation the connector renders by reference) | proposal External Dependencies & Access — published Map Style ID | S2, S3, S4 (RQ-712, RQ-713, RQ-714) | Client brand/content team (authored on the vendor's hosted surface) | v1 "Brand Street" style settled at the discovery review; the published Style ID handoff to the build is pending | The brand/content team hands the published "Brand Street" Style ID to the build before first render; the connector references the published ID and builds no configuration surface | Dane (SA lead) | DEFERRED-APPROVED |

## BA / Requirements Questions

| Q# | Origin | Severity | Question | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| None | — | — | No open requirements questions — Q1 and Q2 are technical/operational decisions carried below | — | — | — | — | — | — |

## Architecture / Tech Decisions

| Q# | Origin | Severity | Decision needed | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| Q1 | discovery Open Questions Q1; design Open Questions Q1 (D6) | NICE-TO-HAVE | Geocode typed postcodes client-side with Cartora or via a server proxy? | Determines Cartora key exposure, quota/billing ownership, and where throttling and error handling live (S1 seed keeps this undecided) | (a) direct client-side call — simpler, no extra hop, but the key is exposed in the browser and throttling sits client-side; (b) server proxy — hides the key and centralizes quota/throttling and caching, at the cost of a hop | (b) server proxy, the design's proposed direction — recommendation only, not approval | (b) Server proxy for typed-postcode geocoding: the Cartora key stays server-side; throttling, caching, and quota governance are centralized in the capability | Dane (SA lead) | APPROVED |
| Q2 | discovery Open Questions Q2; design Open Questions Q2 | NICE-TO-HAVE | Which domain restrictions and quota/billing guardrails should the Cartora key use? | A Cartora account may already exist, but the key policy is undocumented and setup unconfirmed — this also gates P1/P2 | (a) confirm the existing account with the client's Cartora owner and set domain restriction + quota/billing guardrails on it; (b) no existing account confirmed — client provisions one before build | Resolve together with P1: have the client's Cartora account owner confirm account, key policy, and guardrails in this session | (a) Existing account confirmed; the key is domain-restricted to the storefront origins with quota and billing guardrails set on the client account | Sam (client platform dev) | APPROVED |

## Planning Routes

| PD# | Origin (design section/decision) | Deferred item | Target phase | Owner | Route acknowledgement | Approved by | Status |
|---|---|---|---|---|---|---|---|
| PD1 | design Planning Deferrals — Base Configuration & Cross-Cutting (Caching) | Geocode-response cache strategy: TTL, cache key, and invalidation for server-proxied geocode results | technical-design planning | tech-design | Route to technical-design planning acknowledged; the cache strategy is decided at planning under the approved server-proxy branch (Q1), not here | Dane (SA lead) | ROUTED-TO-PLANNING |
| PD2 | design Planning Deferrals — D5 (lazy-loadable entrypoint) | Cartora Maps SDK bundle load + version-pinning strategy for the lazy entrypoint | technical-design planning | SA / tech-design | Route to technical-design planning acknowledged; bundle load and version-pinning are implementation-time HOW settled at planning, not here | Dane (SA lead) | ROUTED-TO-PLANNING |

## Change Request Decisions

| CR# | Origin | Description | Impact | Proposed disposition (from proposal) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|
| None | — | Discovery carries no CR candidates for this integration; the appointment-booking ask belongs to the functional surface and a separate appointment integration | — | — | — | — | — |

## Scope & Offer Approval

| O# | Origin | Item | What the human approves | Decision | Approved by | Status |
|---|---|---|---|---|---|---|
| O1 | proposal Requirements Lock | Requirements Lock | The lock as proposed: RQ-711, RQ-712, RQ-713, and RQ-714 in scope as the capability's own interface obligations only — postcode/city geocoding, a lazy-loadable map-rendering entrypoint, and the vendor-hosted presentation (map style/type and store markers) the capability consumes and renders through that entrypoint; map/list synchronization and all consumer UI stay in the functional Store Locator change; every other requirement out of scope with its owning change named | Approved as offered | Mara (client PO) | APPROVED |
| O2 | proposal Delivery Breakdown S1–S4 | Delivery Breakdown | Four consumer-value slices S1–S4 (one per in-scope requirement) with their acceptance-criteria seeds as the approved slicing for specs and the ISD | Approved as offered — the S1–S4 slicing stands | Mara (client PO) | APPROVED |

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
