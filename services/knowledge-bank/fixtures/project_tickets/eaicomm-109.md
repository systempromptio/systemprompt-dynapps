---
key: EAICOMM-109
title: "Storefront | Account | Retry token revocation for REVOKE_PENDING cards"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Kofi Mensah
project: EAICOMM
date: 2026-08-03
---

# EAICOMM-109 Storefront | Account | Retry token revocation for REVOKE_PENDING cards

## Context

When the PSP fails to revoke a token as the customer removes a card, the record stays in REVOKE_PENDING so a scheduled job can retry rather than leaving a live token at the provider without a customer record. The job does not exist yet; the status does. This story adds the job and the report metric that alarms on old REVOKE_PENDING records.

## Acceptance criteria

1. A scheduled job retries revocation for every REVOKE_PENDING record at least daily and moves successful ones to REVOKED.
2. A record that fails revocation three days running is reported in the job log with the customer number masked.
3. The count of REVOKE_PENDING records older than a day is exposed as a report metric.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

- **Question** (Kofi Mensah, 2026-08-05): Does the PSP return a distinct error for a token that is already revoked on their side? If so we should treat it as success. Asked the PSP support desk.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Kofi Mensah.
