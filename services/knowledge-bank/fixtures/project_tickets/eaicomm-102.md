---
key: EAICOMM-102
title: "Storefront | Checkout | Show the express checkout panel to eligible customers"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Ben Carver
project: EAICOMM
date: 2026-07-16
---

# EAICOMM-102 Storefront | Checkout | Show the express checkout panel to eligible customers

## Context

The cart page needs an express panel above the order summary for customers who meet every eligibility rule in "FSD - Express Checkout". The panel shows the default address, the masked saved card, the shipping method that will apply and the total, with a single Place order button and Change links for address and payment.

## Acceptance criteria

1. An authenticated customer with an active saved card and a default address sees the express panel on the cart page when expressCheckoutEnabled is true.
2. A guest, a remembered-cookie session, or a customer without an active saved card never sees the panel and sees the standard Proceed to checkout call to action instead.
3. The panel shows the masked card as brand and last four digits, the default address, the cheapest valid shipping method and the total including tax and shipping.
4. The Change links open the normal checkout at the address or payment stage with the express selection pre-filled.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Implemented as NwtExpressCheckoutPanel under app/components/checkout, reading eligibility from the cart loader, which calls the eligibility service added in EAICOMM-103. The preference is read through useSitePreference. No new dependency.

## Comments

- **Finding** (Lena Sørensen, 2026-07-29): Panel renders correctly for the six eligibility cases in the test plan. Accessibility scan clean.
- **Decision** (Priya Raman, 2026-07-30): Accepted at sprint review by Daniel Reyes.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Ben Carver.
