# Plan — <slug>

Level: `simple` · Branch: `<branch>` · Goal: `goal.md`

Full suite: CI — `<.github/workflows/tests.yml>`, job `<unit>`
<!-- or, when no CI runs the whole suite on PRs: Full suite: `<command>` — run by close
     or, when the last phase's verify is the whole suite: Full suite: phase NN — <why> -->

<!-- This file is the plan's state. `run` updates the Status and Delivered columns when it closes
     a phase, in the same commit. Status is `pending`, `done` or `blocked`, nothing else.
     Strip this comment. -->

## Phases

| # | Phase | Executor | Verify | Status | Delivered |
|---|---|---|---|---|---|
| 01 | <name> | sonnet | `pytest -q tests/test_x.py` | pending | |

## Crosscutting skills

The conventions for the whole plan — read by every executor, not repeated in each phase. At most
four, each with its fallback:

- Use <plugin:skill> for <what>. If unavailable: <what instead>.

<!-- When a phase spends its rounds, add:

## Blocked

**Phase NN — <name>.** What was tried in each round, the last verify's exit and tail, and what a
person has to decide. Remove the section once it is resolved.
-->
