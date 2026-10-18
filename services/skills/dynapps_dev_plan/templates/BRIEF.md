# Brief — <slug>

> Written for someone arriving with no context: another session, another
> teammate, another tool. Keep it under 40 lines. It points at the plan files,
> it does not repeat them — a brief that duplicates the goal drifts and ends up
> lying.

## What this is

Three lines. What the plan achieves and for whom.

## How to run it

```
With the dev-plan skill:   /dev-plan run          (one phase)
                           /dev-plan run --all    (chain to the end)
Without it:                follow the Protocol below
```

Plan directory: `<path>`

## Assumptions you cannot see from the files

Things that are true and nowhere written: the client's version, work happening
in parallel, constraints agreed verbally. This is the section that most often
saves a wasted afternoon.

## What not to do

The two or three things that would be actively harmful here.

## Known gotchas

See `GOTCHAS.md`. Read it before starting a phase, it grows as the plan runs.

## When you get stuck

Who decides, and what must never be done to get unblocked: never weaken a
verify, never edit tests to make them pass.

## Protocol — only if your tool has no dev-plan skill

The plan is plain markdown and git, so any agent can execute it:

1. Read `goal.md`, `plan.md` and `GOTCHAS.md`. The first `pending` row of the `plan.md` table is
   the next phase; a `blocked` row means stop and read `## Blocked`.
2. Read **only** that phase's `phases/NN-*/phase.md`. Never other phases.
3. If it says `red_first: true`, run its `## Verify` before touching anything and note the red.
4. Do what that phase declares, nothing belonging to a later one, and update the documents it lists.
5. Run its `## Verify` and review your own diff against the phase.
6. Exit 0 and the review holds → write `result.md`, set the row to `done` with one line of what it
   delivered, commit.
7. Otherwise correct it, up to the phase's `rounds`. Then set the row to `blocked`, write
   `## Blocked` in `plan.md`, stop.
8. Never edit tests or weaken a verify to get a phase to pass.
9. When every row is `done`, do what `plan.md`'s `Full suite:` line says: nothing if it is CI or
   the last phase, run its command otherwise. A red suite means the plan is not finished.
