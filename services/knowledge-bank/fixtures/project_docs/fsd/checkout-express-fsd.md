---
title: "FSD - Express Checkout"
space: EAICOMM
page_id: "4718601"
date: 2026-07-14
status: Approved
owner: Priya Raman (BA)
---

# FSD - Express Checkout

## Purpose and scope

Express Checkout lets a signed-in customer with a saved address and a saved payment method complete a purchase from the cart page in a single confirmation step, without walking the four-stage checkout. This functional specification covers the storefront behaviour, the eligibility rules, the feature flag that gates rollout, and the analytics events. It does not cover the PSP vaulting itself, which is specified in "FSD - Account Saved Payments", nor the order management system side, which is unchanged: an express order is an ordinary order by the time it reaches OMS. The scope is the Northwind Trading storefront on Salesforce Commerce Cloud with the Storefront Next (React Router) front end; the legacy SFRA site is out of scope and will not receive the feature.

## Eligibility rules

A customer is eligible for Express Checkout when all of the following hold at the moment the cart page renders: the customer is authenticated (not a guest and not a remembered-cookie session), the account holds at least one saved shipping address flagged as default, the account holds at least one saved payment method that has not expired and whose PSP token is in status ACTIVE, the cart contains no item that requires an age check, a gift message, or a scheduled delivery, and the cart's shipping destination country is one for which the default address is valid. A cart containing a pre-order line is eligible, because pre-order authorisation follows the normal capture-on-ship flow. If any rule fails, the cart page shows the standard "Proceed to checkout" call to action and never mentions express.

## Feature flag

The feature is gated by the site preference `expressCheckoutEnabled` (Boolean, custom preference group "Checkout"). The express checkout feature flag is read once per request on the server and injected into the page data, so a flag change takes effect on the next page load without a deploy or a cache clear of the static bundle; the cart page itself is never cached. In addition to the site-level flag, the customer group `ExpressCheckoutBeta` restricts the feature during the pilot: while `expressCheckoutPilotOnly` is true, only members of that group see the express path. Rollout is: pilot group on staging, pilot group on production for two weeks, then `expressCheckoutPilotOnly` false. Both preferences default to false in the metadata import so a fresh sandbox never exposes the feature by accident.

## User journey

On the cart page an eligible customer sees an "Express checkout" panel above the order summary showing the default address, the masked payment method (brand and last four digits), the shipping method that will be applied (the cheapest method valid for the address), and the order total including tax and shipping. A single "Place order" button submits. On success the customer lands on the order confirmation page exactly as they would from the normal flow, with the express order carrying the custom attribute `c_checkoutMode = EXPRESS` for reporting. The panel offers "Change" links for address and payment; either link opens the normal checkout at the relevant stage with the express selection pre-filled, and the customer completes from there. There is no separate express review page: the panel is the review.

## Error handling

If the payment authorisation is declined, the panel shows the PSP's customer-safe decline message inline and offers "Use a different payment method", which opens the normal checkout at the payment stage. The cart is preserved unchanged. If a basket validation fails at submit (stock, price change, promotion expiry), the customer is shown the standard basket-updated banner and the express panel re-renders with recalculated totals; it does not auto-submit. If the saved address fails address verification at submit, the express panel is hidden for that session and the customer is routed to the normal flow with a message explaining that the address needs review. Every failure writes an `express_checkout_failed` event with a reason code from the fixed list: DECLINED, BASKET_CHANGED, ADDRESS_INVALID, TOKEN_EXPIRED, UNKNOWN.

## Analytics events

Four events are emitted to the analytics layer: `express_checkout_shown` when the panel renders, `express_checkout_submitted` when "Place order" is pressed, `express_checkout_completed` with the order number on confirmation, and `express_checkout_failed` with the reason code above. Each carries the customer group membership flag so the pilot cohort can be isolated in reporting. The conversion metric for the pilot is completed divided by shown, compared against the same customers' historical four-stage conversion; the success threshold agreed with Northwind is a relative uplift of eight percent over the two-week production pilot, measured by the analytics team, not by the storefront.

## Open questions

Whether express orders should be excluded from the "abandoned checkout" email trigger is still with the marketing team; the current implementation excludes them, because a customer who saw the panel and did not press "Place order" never entered a checkout stage. Whether a pre-order line should be eligible was reviewed on 2026-07-09 and confirmed eligible. The behaviour for accounts with more than one default address in different countries is unresolved and is tracked as EAICOMM-118.
