---
key: EAICOMM-118
title: "Storefront | Checkout | Decide express eligibility for accounts with default addresses in several countries"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Priya Raman
project: EAICOMM
date: 2026-08-05
---

# EAICOMM-118 Storefront | Checkout | Decide express eligibility for accounts with default addresses in several countries

## Context

"FSD - Express Checkout" leaves open what happens when an account holds more than one default address in different countries, which the address book allows because a default is per country. The eligibility rule reads the default address as if there were one. This story is the analysis and decision; implementation follows in a separate story once decided.

## Acceptance criteria

1. The behaviour for accounts with default addresses in more than one country is decided with Daniel Reyes and recorded on this ticket and in the specification.
2. The eligibility rules in the specification are updated to say which address is used.
3. An implementation story is created if the decision changes the current behaviour.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

- **Question** (Priya Raman, 2026-08-05): Proposal to discuss on 2026-08-11: use the default address for the cart's shipping country when a shipment exists, otherwise the most recently used default address.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Priya Raman.
