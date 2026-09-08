# Checkout payments — working notes

## Background

We're adding card payment to the Storefront Next checkout ("SF Next", our headless
React storefront on MRT). Today checkout ends at a "review" step with no real
payment; finance wants live card authorization before the holiday peak. This is the
ISD for the payments integration.

## Design decision (from the design doc)

- Provider: **Bolt** (chosen over Adyen/Stripe on the existing MSA and faster
  onboarding — see design "PSP selection", not repeated here).
- Integration style: a server-to-server call to Bolt's hosted **Authorize** API
  from SF Next's checkout action; no card fields on our own origin.
- **PCI posture: Bolt is the cardholder-data environment. We are SAQ-A — the card
  number never touches SF Next or the SFCC commerce backend; Bolt tokenizes and
  vaults the card on its side.** Security signed off on this, and it's the reason
  reviewers keep asking "do we store cards?" — we don't.

## Runtime behaviour

- The shopper submits the order on **SF Next**.
- SF Next persists the order in **SFCC Commerce** (the backend of record) and gets the
  created order back.
- Before calling Bolt, SF Next **re-checks the amount** it is about to authorize against
  the total on the order it just got back — a guard against a tampered client-side amount.
- SF Next asks **Bolt** to authorize that amount, and sends Bolt only the amount, never
  cart or pricing detail.
- **Bolt tokenizes and vaults the card** on its own side.
- The order is confirmed to the shopper only once Bolt returns an approval.
- A **decline** sends the shopper back to the payment step with an error.

The authorization is synchronous.

## Config / ops (implementation detail)

- Auth request timeout 8s; one retry on a network error, then treat as a decline.
- Idempotency key = order id, so a retry can't double-charge.
- Bolt's webhook for async capture/settlement is a **later** phase — not in this doc.
- Refund / void flow is a **separate ISD** (the returns team owns it).

## Meeting notes (2026-07-30 sync)

- Merch asked whether Apple Pay is in scope — deferred to a follow-up, out of scope here.
- Analytics: add a GA4 `purchase` event on confirmation (tracked by the analytics story).
- A11y: the decline error must be announced to screen readers (a frontend story).
- Open Q: which currencies at launch? finance to confirm — does not affect the auth flow.

## Glossary

SF Next = the Storefront Next app. SFCC = the B2C Commerce backend. Bolt = the payment provider.
