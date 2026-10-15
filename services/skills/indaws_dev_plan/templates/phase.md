---
executor: "sonnet"  # inline | haiku | sonnet | opus — per phase, never from the level
                    # haiku only with a closed brief: every section below filled, nothing open
risk: []            # core-model | customer-data | security | migration | irreversible
                    # non-empty → full review, red_first, rounds 2, and never inline
rounds: 1           # correction rounds before a person is needed (1 or 2)
red_first: false    # see the verify fail before the work starts
---

# Phase NN — <name>

## Objective

One or two sentences. What is true at the end that was not true at the start.

## Depends on

Previous phase numbers, or "nothing".

## What gets built

A concrete list, naming the tests to add and the existing ones to update. Over 150 lines of
detail means the phase must be split.

## Design

The decisions already taken, numbered. For `haiku`, none left open. For `sonnet` or `opus`, what is
open is written as such: "open: <what the executor decides>".

## Not in this phase

What stays out, including the options considered and rejected, with why. `none` only when the
phase is too small to have alternatives.

## Files touched

Every place to change as `file:line` (`models/fqdn.py:142`), from the reconnaissance; a bare path
for a new file. The review checks the diff against this list.

## Docs

The documents this phase writes or updates, in its own commit. `none` when nothing.

## Requires

Access only this phase needs: a read-only command that proves it, and how to get it when it fails.
`none` when nothing.

## Skills

At most two, confirmed by the user, only ones this session has. Instructions, each with a fallback:

```
Use <plugin:skill> to <what, exactly>. Do not <the manual alternative>.
If unavailable: <what to do instead>.
```

`none` when no installed skill applies.

## Verify

A shell command with an exit code that runs the tests of the code this phase touches — not the
whole suite — and the minimum number of tests it must report (`expect: ≥ 12 tests`). It is the gate: the review may send the phase back, never approve what the verify
failed.
