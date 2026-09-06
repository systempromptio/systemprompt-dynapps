---
key: EAICOMM-103
title: "Storefront | Checkout | Evaluate express eligibility on the server"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Ben Carver
project: EAICOMM
date: 2026-07-16
---

# EAICOMM-103 Storefront | Checkout | Evaluate express eligibility on the server

## Context

Eligibility for express checkout is a server-side decision that the cart loader injects into page data. It combines the authentication state, the saved card status from "FSD - Account Saved Payments", the default address, the cart contents rules and the two site preferences, and it must be cheap enough to run on every cart render because the cart page is never cached.

## Acceptance criteria

1. The eligibility service returns eligible true only when every rule in the specification holds, and returns the first failing rule's code otherwise.
2. A cart containing an age-checked item, a gift message or a scheduled delivery is not eligible.
3. A cart containing a pre-order line is eligible.
4. When expressCheckoutPilotOnly is true, only members of ExpressCheckoutBeta are eligible.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Service in app/data/checkout/eligibility.ts with a pure rules function and a thin loader wrapper. Rules are unit tested with a table of sixteen cases. Saved card status is read from the profile's saved methods, not recomputed from expiry.

## Comments

- **Finding** (Ben Carver, 2026-07-24): The rules function needed the shipping country of the cart, which the basket API returns only once a shipment exists; we now read it from the default address when the shipment is empty.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Ben Carver.
