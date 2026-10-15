# Mode: close

Close a finished plan: settle the full suite, check the documentation holds together, keep what
was learned, publish, archive. Closing is publication, not writing — the documentation was written phase by phase.

**Refuse to start** when any row of the `plan.md` table is not `done`, or the plan is not in
`active/`.

## 1. The full suite

Read `plan.md`'s `Full suite:` line.

- **CI** — nothing runs here: the PR/MR carries the suite. Say so in the report, naming the job.
- **A command** — run it once with the verify recipe (`SKILL.md`), log `logs/full-suite.log`. Exit 0
  with tests counted → go on. Red → **the plan does not close**: add a `pending` row to the table
  for the fix, with the failing tests in its `phase.md`, and stop. A red the plan did not cause
  (it fails on the base branch too) is reported to the user, who decides; never skip the test.
- **The last phase** — it ran when that phase closed; nothing to repeat.
- **No line** — a plan from before this rule: ask which, once.

## 2. Coherence of the whole

Read `docs/` once as a whole: terminology that shifted, links to files that moved, statements a
later phase contradicted. Check every `-> phase NN` reservation landed somewhere. A document
contradicting `decisions.md` is wrong; if the decision itself was wrong, append a new entry, never
edit the old one. Fix what you find now, in the closing commit.

## 3. What was learned

For each `GOTCHAS.md` entry ask one question: would this happen on another project? If it would,
propose it to the user as one line for the skill that owns that stack or tool (the trigger a reader
recognises in a diff, and what to do instead). The user decides; the plan stays the record
of the rest. Say in the report how many were proposed.

From the `plan.md` table and each `result.md`'s `## Review`, one line per executor: phases, how
many went in as delivered, fixes made here, rounds returned, relaunches on a bigger model. That is
what calibrates `new`. **When more than one `haiku` phase in three needed a correction**, say so:
for this kind of work the briefs were not closed enough, and the user decides whether that
becomes a lesson. Tokens are not visible from inside the session; a headless run's JSON output
gives them per model (`modelUsage`).

## 4. Publish

Only documents whose frontmatter carries `publish: true` leave the plan — **never publish `docs/`
wholesale**: a plan holds attempts, reasoning and, in client work, customer data.

Hand each one to **`common-skills:dynbrain`** and write the identifier it returns into that
document's `remote_id`. A document is published when it has a `remote_id`, never otherwise.

If `common-skills:dynbrain` is not loaded in this session, or the call fails: tell the user the
documentation is ready and **unpublished**, and stop there. Do not invent the call and never write
to an external system by another route.

## 5. Archive

```bash
git mv plans/active/<slug> plans/done/<slug> && git commit -q -m "chore(plan): close <slug>"
```

Outside git, `mv` and say so. Check that `plans/` is not in `.gitignore`: an archived plan outside
version control is a contract nobody can restore.

## Report

```
Plan <slug> closed.
  Phases: 8/8 · haiku 3 (2 as delivered, 1 fixed here) · sonnet 4 (1 returned) · inline 1
  Full suite: left to CI (.github/workflows/tests.yml, job unit) · or: `pytest -q` exit 0, 412 tests
  Documentation: 3 documents published · or: ready, NOT published (<why>)
  Lessons: 1 proposed for <the skill that owns it>, 4 left with the plan
  Archived to: plans/done/<slug>
```

Never report a plan as published when it was only archived locally. Closing is the one step that
is hard to undo: when in doubt, stop.
