---
key: EAICOMM-107
title: "Storefront | Account | Saved card list shows an expired card as active"
type: Bug
status: In review
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Ben Carver
project: EAICOMM
date: 2026-08-04
---

# EAICOMM-107 Storefront | Account | Saved card list shows an expired card as active

## Context

On staging, a card whose expiry month passed on 2026-07-31 still shows as active in the account area and, worse, makes the customer eligible for express checkout, which then fails with TOKEN_EXPIRED at authorisation. "FSD - Account Saved Payments" says the nightly job moves such cards to EXPIRED; the job ran but the status did not change.

## Acceptance criteria

1. A card whose expiry month has passed shows as expired in the account area after the nightly job.
2. A customer whose only saved card is expired is not eligible for express checkout.
3. The nightly job logs how many cards it moved to EXPIRED.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Root cause: the job compared expiry against the current month with a string comparison on MM/YY, so 07/26 sorted after 08/26 as text. Replaced with a date comparison on the last day of the expiry month. Regression test added with month boundaries in both directions.

## Comments

- **Finding** (Lena Sørensen, 2026-08-04): Reproduced on staging with the QA test account nwt-qa-04; blocks the production pilot start.
- **Finding** (Ben Carver, 2026-08-06): Fix on the review branch; needs a job run on staging tonight to confirm.

Status: In review. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Ben Carver.
