# Mode: run

Execute the plan in `plans/active/`. `run` advances one phase; `run --all` keeps going until the
plan is done or blocked. Every call you make here re-reads this whole window: batch commands, keep
outputs short, and do not re-read what is already in context.

## Before the first phase — one command

```bash
cd <repo> && git status --short | head -5; git rev-parse --abbrev-ref HEAD; \
  sed -n '/^## Phases/,/^## /p' <plan>/plan.md; cat <plan>/GOTCHAS.md
```

- **A draft named by run** is activated if the checklist in `modes/new.md`, "## Activating", holds;
  otherwise print what fails and stop.
- **Refuse to start** when a row is `blocked`: report its `## Blocked` section.
- **Branch**: on a branch that is not the default and carries its own commits, say so once and let
  the user decide. Never switch branches for them.
- **`--all`**: say how many phases are left and what stops it, and ask once. In a headless run, an
  explicit `--all` in the invocation is the answer. It stops on exactly what stops a single run, and
  at 30 phases.

The next phase is the first `pending` row. Run its `## Requires` commands; a failing one stops the
run before anything is touched — a missing prerequisite, not a failed phase.

## Per phase

```
executor (red first if asked → implement → verify → result.md) ─► review ─► fix / return / escalate ─► close + commit
```

### 1. Hand it to the executor

**Subagent** (`haiku`, `sonnet`, `opus`) — launch `plan-executor` with that `model` and a prompt of
paths, never pasted content: `goal.md`, `plan.md`, `GOTCHAS.md`, its `phases/NN-*/phase.md` and the
plan directory.
Then end your turn and wait for its report. Keep its id: a correction goes back to it.

**`inline`** — do it here from the same `phase.md`: the red first when `red_first`, then the work,
the verify (`SKILL.md`, "The verify") and `result.md`.

### 2. Review — always yours, in as few commands as possible

One command first:

```bash
git add -N . && git diff --stat && tail -n 20 <plan>/phases/NN-*/result.md
```

- **Light** (`simple`, `media` without `risk`): that output plus the executor's report. Open only
  the hunks that surprise — files outside `## Files touched`, a size out of proportion, a changed
  test — with `git diff -- <file>`. A `red_first` phase with no red in `result.md` is **unproven**,
  and a verify whose tail shows no tests ran, or fewer than its `expect`, is red (GOTCHA 1).
- **Both**: every "not tested", "not verified" or "unsure" in the executor's report about this
  phase's own code is a correction — fix it or send it back — never a reservation. Reservations
  are for what belongs elsewhere.
- **Full** (`alta`, or any `risk`): the whole diff against `phase.md` — promise vs delivery, scope
  creep, tests that check behaviour rather than the implementation, leftovers, and the `## Docs`
  actually updated, and the new code covered by the tests the verify runs. When a skill the phase
  names carries a review checklist for its stack, go through it on this diff.

### 3. Correct

| What the review found | What you do | Costs a round |
|---|---|---|
| nothing | close | — |
| small: ≤ ~20 lines in 1–2 files, no design change | fix it yourself, re-run the verify | no |
| big: incomplete or wrong in several places | a concrete list of corrections to **the same subagent** (SendMessage) | yes |
| the approach is wrong, or the model fell short | relaunch fresh, one model up, with only the correction list | yes |
| inline phase turned out bigger than planned | hand it to a subagent | no |

Moving a phase to another executor is one line in `decisions.md` with its reason.

**When `rounds` are spent** and the verify is still red or the review still fails: do not commit
the work. Set the row to `blocked`, write `## Blocked` in `plan.md` (what each round tried, the
last verify's exit and tail, what a person must decide), commit only `plan.md`, stop. Leave the
attempt on disk and say so. Do not lower the verify, do not redefine the objective.

### 4. Close — two calls

First, in one message: append `## Review` to `result.md` (one line per correction: fixed here,
returned or relaunched, and what for; `none`), then `## Reservations` (each with its destination; a
lesson later phases would trip over also goes into `GOTCHAS.md` now), and edit the phase's row in
`plan.md` to `done` with one line of what it delivered.

Then commit, staging only this phase's files and the plan:

```bash
git add <## Files touched> <docs it updated> <plan dir> && \
  git commit -q -m "<type>(<scope>): <phase name> — phase NN of <slug>" && git log --oneline -1
```

The commit is the phase's checkpoint. Outside a git repository, say there is none.

## When the last row is `done`

The phases are green one by one; the suite is not yet. Say what `plan.md`'s `Full suite:` line
asks for — the PR's CI, or `close` running the command — and point to `/dev-plan close`.

## Reporting

One line per phase, in the language of the work: what it delivered, who ran it, how many rounds.
Name the thing, not its id: "phase 4 (tax lines) gave up after two rounds; the verify still fails
on the rounding test", never "phase 4 blocked 2/2".

After about eight phases — inline ones count double — suggest continuing in a fresh session:
`/dev-plan brief` hands it over and nothing is lost.

## Headless

```bash
claude -p "/dev-plan run --all" --plugin-dir <plugin> --add-dir <plugin> \
  --allowedTools "Bash,Read,Edit,Write,Glob,Grep,Agent,SendMessage"
```

Judge the outcome from the plan, never from the log: every row of the `plan.md` table is `done`
(`! grep -Eq '\| (pending|blocked) \|' <plan>/plan.md`) and `git log` has one commit per phase.
