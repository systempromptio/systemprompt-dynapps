---
key: EAICOMM-116
title: "Ops | Orders | Add the express order count to the deploy verification"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Lena Sørensen
project: EAICOMM
date: 2026-07-23
---

# EAICOMM-116 Ops | Orders | Add the express order count to the deploy verification

## Context

"Runbook - SFCC storefront deploy" verifies order placement after activation by comparing the order count in the last fifteen minutes with the normal range. Once express is live, an express-specific failure could hide inside a normal total, so the verification needs the express count separately during the pilot.

## Acceptance criteria

1. The deploy verification checklist includes the count of orders with c_checkoutMode EXPRESS in the last fifteen minutes during the pilot.
2. The report used for the count is linked from the runbook.
3. The runbook page is updated and its date bumped.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Report added to the operations dashboard as Orders by checkout mode, last 15 minutes. Runbook updated on 2026-08-12 with the release-week changes.

## Comments

- **Decision** (Ben Carver, 2026-08-12): Runbook updated together with the rollback section changes from the release readiness meeting.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Lena Sørensen.
