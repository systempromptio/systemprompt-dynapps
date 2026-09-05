---
key: EAICOMM-112
title: "Storefront | PDP | Enforce configurator constraints on the client and the server"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Ben Carver
project: EAICOMM
date: 2026-08-03
---

# EAICOMM-112 Storefront | PDP | Enforce configurator constraints on the client and the server

## Context

The constraints engine of "FSD - PDP Configurator" disables invalid option values as the customer changes a selection, resets an invalidated step to its default with a message, and re-validates on the server at add-to-basket, where the server is authoritative and returns a structured error listing violated rules.

## Acceptance criteria

1. Selecting a value that makes another step's current value invalid resets that step to its default and shows a message naming the rule in customer language.
2. Values that are invalid under the current selection are disabled with a tooltip naming the rule.
3. An add-to-basket request with an invalid combination is refused with a structured error listing the violated rules and the PDP re-renders with the corrected state.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

No comments yet.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Ben Carver.
