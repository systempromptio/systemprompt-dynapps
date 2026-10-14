# Mode: brief

Produce the handoff message for someone who will execute this plan without the
conversation that produced it: another session, another teammate, another tool.

Read-only. Never modifies the plan.

## Steps

1. Locate the plan (named slug, or the single one in `plans/active/`).
2. Read `BRIEF.md`, `goal.md`, the `plan.md` table and `GOTCHAS.md`.
3. If `BRIEF.md` is missing, say so and stop: without it there is nothing to
   hand over that the receiver could act on. Offer to write one.
4. Emit the message below and nothing else after it.

## Output

```
Paste this into the other session:
──────────────────────────────────────────────
You are going to execute a phased plan at <absolute path>.

Start by reading BRIEF.md, then goal.md and GOTCHAS.md.
Run one phase with: /dev-plan run
(no dev-plan skill? BRIEF.md carries the protocol)

Current state: phase <n> of <total>, <status>.
<one line per open blocker, or "no blockers">

Hard rules: never edit tests, never weaken a verify, and if a phase
spends its correction rounds, stop and report rather than working around it.
──────────────────────────────────────────────
```

## Rules

- **Keep it short.** A handoff that does not fit in one message does not get
  pasted. Everything else lives in the plan; this points at it.
- **Absolute path**, always. The receiver is not in your working directory.
- **Never include secrets**, and never inline a `## Requires` command's secret. Name
  what access is needed and let `BRIEF.md` say how to get it.
- State the blockers plainly. Handing over a blocked plan without saying so
  wastes the receiver's first hour.
- Written in the user's language, like every other mode.
