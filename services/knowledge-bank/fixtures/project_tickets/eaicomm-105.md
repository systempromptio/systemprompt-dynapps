---
key: EAICOMM-105
title: "Storefront | Checkout | Emit express checkout analytics events"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Kofi Mensah
project: EAICOMM
date: 2026-07-17
---

# EAICOMM-105 Storefront | Checkout | Emit express checkout analytics events

## Context

The pilot is measured by four events defined in "FSD - Express Checkout": shown, submitted, completed and failed. Each carries the pilot cohort flag so the analytics team can isolate the ExpressCheckoutBeta group, and completed carries the order number.

## Acceptance criteria

1. express_checkout_shown fires once per cart page render on which the panel is visible.
2. express_checkout_submitted fires when Place order is pressed and express_checkout_completed fires on the confirmation page with the order number.
3. express_checkout_failed carries the reason code from the fixed list.
4. Every event carries a boolean pilot cohort flag derived from ExpressCheckoutBeta membership.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Events go through the existing analytics adapter; the cohort flag is computed once in the cart loader and passed to the panel as a prop. Confirmed with the analytics team that the event names match their tracking plan.

## Comments

- **Decision** (Priya Raman, 2026-07-30): Accepted at sprint review.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Kofi Mensah.
