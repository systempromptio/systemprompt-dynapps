# The daily pass: deadlines and escalation

Runs once a day. `deadline_pass.due_transitions()` works out what falls due and
**why**; the recipe below applies it. Every transition carries its reason, so the
chatter entry says something better than "the system did this".

Nothing here deletes, archives, or touches a body. Every transition is reversible
with one `action_verify`, and all of them go through `assert_transition()`.

## The four rules

| Situation | What happens | Indexed after? |
| --- | --- | --- |
| Review `pending`, deadline passed | article → `expiring` | **yes** — it warns |
| `expiring` past its grace period | article → `unverified` | **no** — it leaves |
| Review overdue > 60 days | territory flagged, backup notified | unchanged |
| `decided` past its expiry, no open review | annual-floor review opened | unchanged |

**The asymmetry between the first two is the design.** Missing a deadline does not
cost anyone retrieval — the document stays in the index and its owner gets told.
Only the grace period running out takes it out. Get that backwards and one late week
silently degrades search for the whole company.

**Escalation never reassigns.** Past two cycles the owner is not late, they are
absent — but who takes over is a human decision. The pass names the backup to notify
and stops there.

**Derived truth never appears in rule 4.** Its importer maintains it; a review
deadline would put a person on the hook for something they do not control.

## Where the grace period is measured from

`canon_review_deadline`. That is the date the article started expiring, and it is
already one of the eleven keys — no twelfth field needed.

If an `expiring` article has no `canon_review_deadline`, the pass **does not guess**.
It reports the article as a problem and leaves it alone: de-indexing a document on no
evidence is worse than leaving it one more day.

## The problem report

There are no database constraints behind this model. No `write()` override, no
`_sql_constraints`, no ACL. So the pass runs `validate_properties()` over everything
managed and returns `problems` **always** — an empty list is a result.

**Put it in the daily output even when it is empty.** A report that only appears on
failure is a report nobody notices has stopped being produced, and this one is the
only thing standing in for the constraints a module would have given us.

Typical entries: a `verified` article with no owner, `canon_managed` set with no
state (which silently retires it from the index), `derived` truth carrying an expiry
date.

## Applying it

For each transition: write `canon_state`, **re-read to confirm**, then post a note in
the article's chatter with the reason. `orm_post_message` with real HTML — never
pre-escaped, or the note comes out unreadable.

The note is the record, **not the notification**: `orm_post_message` takes no
`partner_ids`, so it neither mentions nor notifies. The owner finds out through the
activity, which is the weekly queue's job.

If the ORM proxy answers with a session or authentication error in the middle of a pass,
`whoami` to reopen it and retry that call **once**; then report the retry. If `whoami` itself
fails, the connector's authorisation is gone for the session: stop the pass and say so. It
expired mid-run on 2026-09-11, and later `whoami` failed too until the connector was re-authorised. A transition whose write failed is not applied: do not write its note.

Order matters on a big pass: transition, re-read, then note. A note written before a
write that did not land is a lie in the audit trail.
