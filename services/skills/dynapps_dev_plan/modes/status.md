# Mode: status

Read-only. Never modifies the plan, never advances a phase.

## Steps

1. Locate the plan: if `$ARGUMENTS` names a slug, use it; otherwise the single directory under
   `plans/active/`. If there are several and none was named, list them and stop.
2. One command: the `plan.md` table and `## Blocked` if present, plus `git log --oneline -3`.
3. Report.

## Output

```
Plan: <slug>   ·   level: <level>   ·   <draft|active|done>

  Next: phase <n>/<total> — <title>   (executor: <executor>)
  Progress: <bar> <done>/<total> phases
```

Then, only when they apply:

- **A `blocked` row** — state it first, with the `## Blocked` text. This is the most important
  thing the user can learn from `status`.
- **In `draft/`** — nothing runs until it is in `active/`; `/dev-plan run` activates a draft whose
  checklist holds.
- **A `done` row with no commit for it in `git log`** — report the disagreement; do not guess
  which is right.

Report what the plan says, not what it intended.
