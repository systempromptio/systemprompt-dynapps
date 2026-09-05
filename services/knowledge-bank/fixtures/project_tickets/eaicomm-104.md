---
key: EAICOMM-104
title: "Storefront | Checkout | Place an express order from the cart page"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Ben Carver
project: EAICOMM
date: 2026-07-16
---

# EAICOMM-104 Storefront | Checkout | Place an express order from the cart page

## Context

Pressing Place order on the express panel submits the basket with the saved address, the saved card token and the chosen shipping method, authorises with the PSP, and lands the customer on the standard order confirmation page. The order carries c_checkoutMode = EXPRESS. Failures follow the error handling section of "FSD - Express Checkout".

## Acceptance criteria

1. A successful submission creates an order with c_checkoutMode set to EXPRESS and shows the standard confirmation page.
2. A PSP decline shows the customer-safe decline message inline and offers Use a different payment method, which opens the normal checkout at the payment stage with the cart unchanged.
3. A basket validation failure shows the basket-updated banner and re-renders the panel with recalculated totals without submitting.
4. Every failure emits express_checkout_failed with one of DECLINED, BASKET_CHANGED, ADDRESS_INVALID, TOKEN_EXPIRED or UNKNOWN.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Route action in app/routes/cart.tsx calling the checkout data functions in sequence; the PSP authorisation reuses the saved-method path from the account cartridge. A declined authorisation does not release the basket. Reason codes are a zod enum shared with analytics.

## Comments

- **Finding** (Lena Sørensen, 2026-07-31): Decline, basket change and token expiry paths verified on staging with the PSP sandbox. ADDRESS_INVALID could not be forced on the sandbox and was verified by unit test only.
- **Decision** (Priya Raman, 2026-07-30): Accepted at sprint review.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Ben Carver.
