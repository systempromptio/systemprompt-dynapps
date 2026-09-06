---
key: EAICOMM-115
title: "Storefront | Basket | Shipping method list is empty after changing the address country"
type: Bug
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Ben Carver
project: EAICOMM
date: 2026-07-22
---

# EAICOMM-115 Storefront | Basket | Shipping method list is empty after changing the address country

## Context

Changing the shipping address country on the checkout address stage sometimes leaves the shipping method list empty until the page is reloaded. Reproduced on staging by switching from Germany to Austria on a basket with two items. Reported by Sunita Bhatt's team from a customer call.

## Acceptance criteria

1. Changing the shipping country repopulates the shipping method list with the methods valid for the new country without a reload.
2. The cheapest valid method is preselected after the change.
3. The regression test for the address stage covers a country change.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

The address action returned before the shipment was updated, so the loader read stale methods. Awaited the shipment update and revalidated the route; added a Playwright test for the country switch.

## Comments

- **Finding** (Lena Sørensen, 2026-07-25): Verified on staging; also checked that the express panel picks the cheapest method for Austria correctly.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Ben Carver.
