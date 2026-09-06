---
key: EAICOMM-117
title: "Salesforce Core | Reporting | Record engagement health weekly"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Jonas Kraft
project: EAICOMM
date: 2026-07-21
---

# EAICOMM-117 Salesforce Core | Reporting | Record engagement health weekly

## Context

The steering committee report reads Health__c and Health_Note__c on Engagement__c, as described in "Salesforce Core - engagement reporting schema". The field exists but nothing reminds the engagement manager to set it, and a stale Green is worse than no value. This story adds a weekly reminder and a validation that a non-Green health has a note.

## Acceptance criteria

1. Every Monday the engagement manager of each Active engagement receives a task to update Health__c.
2. Saving Health__c as Amber or Red without Health_Note__c is refused with a message.
3. The steering committee report shows the date Health__c was last changed.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Scheduled flow creating the task; validation rule on Engagement__c; Health_Last_Changed__c added as a field updated by a record-triggered flow. Deployed through the metadata pipeline on 2026-07-29 and announced in the delivery channel.

## Comments

- **Decision** (Miriam Okafor, 2026-07-29): Accepted; the reminder arrived on 2026-08-03 as expected.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Jonas Kraft.
