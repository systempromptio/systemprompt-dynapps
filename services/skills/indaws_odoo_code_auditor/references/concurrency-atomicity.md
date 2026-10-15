# Concurrency & Atomicity — Detection Guide

Audit lens for **two simultaneous callers** and **crash-in-the-middle**, the
failure mode that single-execution tests never reveal. Critical for any
single-use/limited resource (tokens, quotas, counters), batch crons, and
non-idempotent side effects (external calls, mails, payments).

> The bug is structurally invisible to a happy-path test: it only appears under
> interleaving or a mid-run crash. Audit by *reasoning about two callers*, not by
> running the code once.

## The invariant (the one rule)

> A check and the write it guards must be **atomic** (one locked statement or one
> transaction). A batch must **commit per task**, not per batch. A
> non-idempotent effect must be **claimed before** it runs.

## Detection greps

```bash
# check-then-write (TOCTOU): a read/flag check followed later by the write
grep -rn 'if .*\.\(used\|consumed\|count\|state\)\b' --include="*.py" .   # inspect each: is the write atomic?
grep -rn 'use_count\|max_uses\|single_use\|one_time\|\.used\b' --include="*.py" .

# row locking present? (good sign in claim paths)
grep -rn 'FOR UPDATE\|_for_update\|with_lock\|select_for_update' --include="*.py" .

# commit cadence in crons/batches
grep -rn 'env.cr.commit\|\.commit()\|cr.savepoint' --include="*.py" .

# non-idempotent effects inside loops/batches
grep -rn 'requests\.\(post\|put\)\|send_mail\|message_post\|_send\b' --include="*.py" .
```

## Checklist

| # | Check | FAIL signal |
|---|-------|-------------|
| 1 | **Single-use / max-uses claimed atomically.** Token/quota consumed with `SELECT … FOR UPDATE` on the row, or `UPDATE … WHERE use_count < max_uses RETURNING` (atomic claim). | `if self.used: raise; … self.used = True` — TOCTOU window: two callers both pass the check. |
| 2 | **Batch commits per task, not per batch.** Each task wrapped in its own commit/savepoint so a crash re-runs only the unfinished ones. | One `commit()` after the whole loop, or none → crash re-processes completed tasks. |
| 3 | **Non-idempotent effect claimed before firing.** Mark "in progress / done" (committed) before the external call/mail/payment, so a retry doesn't duplicate it. | Effect fires, THEN state is written → crash between them duplicates the effect on retry. |
| 4 | **State transitions guard against concurrent switch.** Status/stage changes that gate logic re-read under lock or use a conditional UPDATE. | Two callers read state A, both transition → double side effect. |
| 5 | **No silent reliance on request serialization.** Don't assume two requests can't overlap; Odoo runs concurrent workers. | "Can't happen twice" comments with no lock. |

## Anti-patterns (PASS / FAIL)

**A. Single-use token (TOCTOU)**
```python
# FAIL — check and write are separate; two callers both see used=False
if token.used:
    raise UserError(_("Token already used"))
# ... do work ...
token.used = True

# PASS — atomic claim: only one UPDATE wins
self.env.cr.execute(
    "UPDATE mcp_token SET used = TRUE WHERE id = %s AND used = FALSE RETURNING id",
    [token.id])
if not self.env.cr.fetchone():
    raise UserError(_("Token already used"))
```

**B. Batch cron commit cadence**
```python
# FAIL — one transaction for the whole batch; crash at task 90/100 re-runs all 100
for task in tasks:
    task._process()
self.env.cr.commit()

# PASS — commit per task; crash re-runs only the unfinished
for task in tasks:
    try:
        task._process()
        self.env.cr.commit()
    except Exception:
        self.env.cr.rollback()
        task._mark_failed()
        self.env.cr.commit()
```
> `cr.commit()` is otherwise discouraged in business logic — the legitimate uses
> are per-task batch boundaries and best-effort writes on a separate cursor.

## ISO mapping
A.8.28 (secure coding — race conditions), A.8.26 (application security
requirements), A.5.14 (information transfer integrity).
