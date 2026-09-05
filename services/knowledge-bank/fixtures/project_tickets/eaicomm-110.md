---
key: EAICOMM-110
title: "Integration | Orders | Exclude express orders from the abandoned checkout email"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Kofi Mensah
project: EAICOMM
date: 2026-08-03
---

# EAICOMM-110 Integration | Orders | Exclude express orders from the abandoned checkout email

## Context

The abandoned checkout trigger in the marketing platform fires for carts that entered checkout and did not convert. A customer who saw the express panel and did not press Place order never entered a checkout stage, so the storefront currently excludes them, but the trigger reads a basket flag that express does not set consistently. "FSD - Express Checkout" lists this as an open question with marketing.

## Acceptance criteria

1. A cart that showed the express panel and was not submitted does not trigger the abandoned checkout email.
2. A cart whose customer used a Change link and then abandoned the normal checkout does trigger it, as today.
3. The behaviour is confirmed with the marketing team and recorded on this ticket.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

- **Question** (Priya Raman, 2026-08-04): Waiting on the marketing team's answer from the 2026-08-06 call before this is groomed.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Kofi Mensah.
