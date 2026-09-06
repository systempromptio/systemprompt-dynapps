---
key: EAICOMM-101
title: "Storefront | Checkout | Express Checkout pilot"
type: Epic
status: In progress
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Priya Raman
project: EAICOMM
date: 2026-07-15
---

# EAICOMM-101 Storefront | Checkout | Express Checkout pilot

## Context

Umbrella epic for the Express Checkout pilot specified in "FSD - Express Checkout". The pilot lets an authenticated customer with a saved card and a default address place an order from the cart page in one step, gated by the site preference expressCheckoutEnabled and the ExpressCheckoutBeta customer group. The epic tracks the storefront panel, the eligibility service, the analytics events, the OMS export attribute and the pilot rollout across staging and production.

## Acceptance criteria

1. Every child story is accepted by Daniel Reyes at a sprint review before the pilot group is enabled on production.
2. The pilot runs for two weeks on production with the ExpressCheckoutBeta group only and the conversion uplift is reported by the analytics team against the eight percent threshold.
3. The rollout decision is recorded on this epic with the steering committee date.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Left blank by the BA; the epic has no technical notes of its own.

## Comments

- **Decision** (Daniel Reyes, 2026-07-16): Pre-order lines are eligible for express checkout, as reviewed on 2026-07-09. No further discussion needed.
- **Finding** (Miriam Okafor, 2026-08-01): Steering committee confirmed the production pilot start for 2026-08-24, subject to EAICOMM-107 closing.

Status: In progress. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Priya Raman.
