# Result — phase NN

<!-- The executor writes the first three sections; the main conversation adds ## Review and
     ## Reservations after its review. Strip this comment. -->

## What was done

What was actually implemented, not what was planned. Executor: `<inline|haiku|sonnet|opus>`.

## Decisions taken

The ones not in phase.md that someone will need in six months. `none` if nothing.

## Verify

```
red before: exit=1 · logs/phase-NN-red.log        (only when red_first)
<the command>
exit=0 · logs/phase-NN-1.log
<the last lines of the output>
```

## Review

One line per correction the review made: `fixed here`, `returned` or `relaunched on <model>`, and
what for. `none` if the phase went in as delivered. `close` counts these per executor.

## Reservations

What the review raised and this phase did not resolve. `none` if nothing. Every entry has a
destination:

```
- the tax rounding is only covered for ES -> GOTCHAS.md
- the importer has no test for 10k rows   -> phase 07
- the naming does not match the old module -> discarded: the old one is going away
```

`-> phase 07` must point at a phase that exists and is still ahead.
