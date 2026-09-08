# Buy-online-pickup-in-store (BOPIS) — working notes

## Background

SF Next is adding **store pickup** at checkout. On the cart page the shopper can
choose "pick up in store"; before we let them commit we need to decide, per item and
store, whether pickup is actually available. This is the FSD for that behaviour.

## Behaviour (from grooming)

- The shopper picks a store.
- We check whether the item is **in stock at that store**.
- Store-level inventory is owned by **Fluent OMS**, a **third-party** system — we read
  availability from it; we consume it, we don't build or run it.
- **If it is available**, we reserve the unit and confirm the pickup option, with a
  "ready in ~2h" promise.
- **If it is not available**, we don't offer pickup for that store — we fall back to
  offering ship-to-home instead.

There is no partial pickup in v1: it is all-or-nothing per item.

## Where the data comes from

- Reservations, once the shopper commits, are also placed through Fluent.
- Store list and hours come from SFCC store objects.

## Config / ops (detail)

- Availability call p95 budget 300ms; store-inventory cached 60s.
- "Ready in 2h" is a fixed copy string for v1 (no real ETA calc).
- Curbside pickup is a later phase — not here.

## Meeting notes (2026-07-28)

- CS asked about cancelling a reservation — a separate flow, out of scope.
- Analytics: fire a `pickup_selected` event (a separate story).
- Open Q: max reservation hold time? ops to confirm — doesn't change the eligibility
  decision.

## Glossary

OMS = order management system (Fluent). BOPIS = buy online, pick up in store.
