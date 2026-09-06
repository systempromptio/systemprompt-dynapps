---
key: EAICOMM-119
title: "Storefront | PDP | Configurator preview shows the previous render after a fast change"
type: Bug
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Ben Carver
project: EAICOMM
date: 2026-08-11
---

# EAICOMM-119 Storefront | PDP | Configurator preview shows the previous render after a fast change

## Context

Changing two options quickly on the configurator prototype shows the render for the first change after the second has completed, because the responses arrive out of order and the last one wins. "FSD - PDP Configurator" requires the last good image to stay while a render is pending and the preview to reflect the latest selection.

## Acceptance criteria

1. The preview always shows the render for the latest selection, regardless of the order in which render responses arrive.
2. While a render is pending the last good image stays with a progress indicator.
3. A render that fails leaves the last good image and logs the failure with the option values.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

- **Finding** (Lena Sørensen, 2026-08-11): Found on the prototype build on staging while testing EAICOMM-111; reproducible with throttled network.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Ben Carver.
