---
key: EAICOMM-120
title: "Ops | Content | Warm the cache after a full invalidation from the deploy checklist"
type: Story
status: Done
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Kofi Mensah
project: EAICOMM
date: 2026-08-03
---

# EAICOMM-120 Ops | Content | Warm the cache after a full invalidation from the deploy checklist

## Context

"Runbook - SFCC cache invalidation" describes warming the top two hundred pages after a full invalidation. The warming script exists in the tools repository but is not part of the deploy checklist, and the last release skipped it. This story makes warming a checklist step and adds the page list export.

## Acceptance criteria

1. The deploy checklist includes warming after any full invalidation, with the command and the expected duration.
2. The page list is exported from the previous week's analytics automatically before each release.
3. The warming script logs every page returning a non-200 status and the deploy channel receives the summary.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Added the export step to the release recipe and the warming step to the checklist in the runbook. Summary posted by the script through the release channel webhook.

## Comments

- **Decision** (Ben Carver, 2026-08-12): Included in the runbook update of 2026-08-12.

Status: Done. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Kofi Mensah.
