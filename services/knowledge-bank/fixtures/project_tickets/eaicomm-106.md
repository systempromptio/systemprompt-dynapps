---
key: EAICOMM-106
title: "Integration | Orders | Export the checkout mode attribute to OMS"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Kofi Mensah
project: EAICOMM
date: 2026-07-17
---

# EAICOMM-106 Integration | Orders | Export the checkout mode attribute to OMS

## Context

OMS needs to know which orders came through express so that Northwind's order reporting can split them. The order export job already sends custom attributes in an extension block; this story adds c_checkoutMode to that block and confirms OMS accepts it without a mapping change on their side.

## Acceptance criteria

1. The order export includes c_checkoutMode for every order, with EXPRESS for express orders and STANDARD for every other order.
2. OMS acknowledges an order carrying the attribute without a rejection, confirmed on the OMS test environment with Marcus Feld.
3. Orders placed before the release export STANDARD, not an empty value.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Added to the export mapping in the integration cartridge; default STANDARD applied in the mapping so old orders are covered. Marcus confirmed the OMS test environment accepts the field on 2026-07-28.

## Comments

- **Finding** (Kofi Mensah, 2026-07-28): OMS test accepted the extension attribute. Production OMS mapping change ticket raised by Marcus for the release.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Kofi Mensah.
