---
key: EAICOMM-114
title: "Storefront | PDP | Show the live configured price"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Ben Carver
project: EAICOMM
date: 2026-08-04
---

# EAICOMM-114 Storefront | PDP | Show the live configured price

## Context

The configured price is the base unit price plus the selected option prices, computed on the client from the prices delivered with the page and confirmed by the server at add-to-basket, per "FSD - PDP Configurator". A mismatch of more than one minor unit fails the add and refreshes prices.

## Acceptance criteria

1. The displayed price updates on every option change without a server round trip.
2. At add-to-basket the server recomputes the price and a mismatch of more than one minor currency unit fails the add with a price refresh.
3. A customer in a group with an active category promotion sees the promotions applied in basket note.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

No comments yet.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Ben Carver.
