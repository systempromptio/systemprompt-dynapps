---
key: EAICOMM-108
title: "Storefront | Account | Ask for the password again before showing payment methods"
type: Story
status: Done
sprint: "Sprint 14 (2026-07-20 to 2026-08-02)"
assignee: Ben Carver
project: EAICOMM
date: 2026-07-20
---

# EAICOMM-108 Storefront | Account | Ask for the password again before showing payment methods

## Context

"FSD - Account Saved Payments" requires a fresh login before the Payment methods page renders: a session authenticated more than fifteen minutes ago must re-enter the password, matching the Change email page. This story reuses that mechanism rather than building a second one.

## Acceptance criteria

1. A customer whose authentication is older than fifteen minutes is asked for their password before the Payment methods page renders, and lands on the page after a correct password.
2. A customer who authenticated within fifteen minutes sees the page directly.
3. The re-authentication uses the same component and route guard as the Change email page.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

Extracted the guard from the change-email route into a shared route helper in app/data/account/reauth.ts and applied it to both routes. No new component.

## Comments

- **Decision** (Priya Raman, 2026-07-30): Accepted at sprint review.

Status: Done. Sprint: Sprint 14 (2026-07-20 to 2026-08-02). Assignee: Ben Carver.
