---
name: dynapps-dev-plan
version: 1.0.0
model: opus
effort: medium
description: >-
  Runs a development objective as a phased plan: each phase goes to a fresh haiku, sonnet or opus subagent, or inline, is gated by the tests of the code it touches and reviewed by the main conversation, with iterations only as correction rounds. The full suite runs once at the end, or is left to the repository's CI when it runs it. The plan is plain markdown plus git, no engine to install. Stack-agnostic. For multi-step work: a module, an integration, a migration, a refactor. Triggers: "/dev-plan", "build a plan for", "resume the plan", "monta un plan por fases", "retoma el plan", "en qué fase vamos", "cierra el plan". Not for scheduling, effort estimation or Gantt charts.
argument-hint: "status | research [--deep] | run [--all] | brief | close | [--level simple|media|alta] <objective>"
---

# dev-plan — phases, a shell gate, and the main conversation as reviewer

Talk and write the plan in the user's language; headings, keys and status values stay English.

## Router

Read the first word of `$ARGUMENTS` and load **only** that mode's file.

| First word | File |
|---|---|
| `status` | `modes/status.md` |
| `run` | `modes/run.md` — `--all` runs every phase |
| `research` | `modes/research.md` — reconnaissance; `--deep` only when the user asks for it |
| `close` | `modes/close.md` |
| `brief` | `modes/brief.md` — handoff for another session |
| anything else | `modes/new.md` — the whole `$ARGUMENTS` is the objective; `--level` sets the preset |

Nothing else is loaded on spec.

## The plan is markdown and git — there is no engine

```
plans/draft/<slug>/     written, not runnable      ─┐
plans/active/<slug>/    runs                        ├─ the directory is the lifecycle
plans/done/<slug>/      archived                   ─┘
  goal.md · plan.md · decisions.md · GOTCHAS.md · BRIEF.md · research.md (optional)
  phases/NN-slug/phase.md · result.md
  docs/    what the plan documents        logs/    verify output, never committed
```

**`plan.md` is the state.** Its phase table carries `Status` — `pending`, `done` or `blocked` —
and, once done, one line of what the phase delivered. A blocked phase also gets a `## Blocked`
section: what was tried and what the last verify said. Each closed phase is a commit. Nothing else
holds state, so nothing else can disagree with it.

## How a phase runs

```
main conversation ──► executor: inline | haiku | sonnet | opus (fresh context, paths only)
        ▲                     │ implements, runs ## Verify, drafts result.md
        └──── reviews ◄───────┘
   small fix → does it · big fix → back to the same subagent · wrong approach → relaunch one model up
```

There are no reviewer or specialist agents. The main conversation is the only reviewer and the
verify's exit code the only gate.

## Levels are presets, phases decide

Each `phase.md` declares its own `executor`, `rounds` and `red_first`. The plan's `level` only
supplies the defaults for the last two, so one plan mixes them.

| | simple (default) | media | alta |
|---|---|---|---|
| `rounds` (correction rounds before a person is needed) | 1 | 1 | 2 |
| `red_first` (see the verify fail before the work) | false | true | true |
| review by the main conversation | light | light; full on phases with a `risk` | full |

**The level never picks the executor.** `simple` says how much can go wrong, not how closed the
phase is. Each phase gets one by its own content (`modes/new.md`): `sonnet` by default, `haiku`
only with a closed brief, `opus` where there is design to do.

Flow: research (optional) → new (draft) → activate → run → close.

## The verify, run the same way by everyone

```bash
mkdir -p <plan>/logs; L=<plan>/logs/phase-NN-<attempt|red>.log
( <the ## Verify command> ) > "$L" 2>&1; echo "exit=$?"; tail -n 25 "$L"
```

The full output stays in the log; only the tail enters the context. **Exit 126 or 127, a
`command not found`, a timeout: the check could not run.** That is neither a pass nor a red — it is
a broken verify or a missing prerequisite, and it is reported as such.

## Tests: each phase its own, the full suite once

- **A phase's verify runs the tests of the code that phase touches** — its test file, package or
  selection — never the whole suite. The phase writes or extends those tests, and they must be
  green to close it.
- **The full suite runs once, at the end, and only where nothing else runs it.** If the
  repository's CI runs the whole suite on every PR/MR, the plan delegates it there. If not, `close`
  runs it before anything else and a red suite does not close the plan. `plan.md` records which,
  in its `Full suite:` line, decided in `new`.
- **dev-plan knows no stack.** How to build and test honestly in Odoo, Django, Go or anything else
  comes from the skills the plan names, never from this skill.

## Permanent rules

These apply in every mode. The mode files do not repeat them.

1. **The exit code is the gate.** A phase is `done` only when its `## Verify` exited 0 in this
   session, and the commit that closes it is made right after. The review may send a phase back;
   it never replaces a failed verify.
2. **Never claim what you did not see.** To say something works you must have run the command
   **in this same turn** and read its output. Banned: "it should work now", "I ran it earlier",
   "probably fine". Never fill a field you cannot know.
3. **Rounds are capped.** A correction round is one return of the phase to an executor. After
   `rounds` of them, if the verify is still red or the review still fails: mark it `blocked` in
   `plan.md`, write `## Blocked`, stop. Do not lower the bar or redefine the objective.
4. **The reviewer never writes a risky phase.** An `inline` phase is done by the main
   conversation, which then cannot review it with distance — so a phase with a non-empty `risk` is
   never inline.
5. **Only `active/` runs.** Moving a draft there is cheap and reversible, and you may do it once
   the activation checklist in `modes/new.md` holds. Unattended execution of the whole plan
   (`run --all`) is asked once, and never on behalf of someone who is not there.
6. **Constant context.** Read only `goal.md`, `plan.md`, `GOTCHAS.md` and the current
   `phases/NN/phase.md`. Never another phase's folder on spec: the `plan.md` table has one line per
   finished phase, and when that line concerns yours you may open that single `result.md`.
7. **Documentation travels with the change.** A phase updates the documents its change makes
   untrue in its own commit — the ones in its `## Docs` and any other it finds. There is no debt to
   settle later.

## GOTCHAS

1. **A verify that cannot fail is not a gate.** Some runners exit 0 having run nothing: a filter
   that matches no test, a framework that counts zero failures among zero tests as success. The
   tail must show how many tests ran; none is a red, whatever the exit code.
2. **A green from minute one measured nothing** — it cannot tell "done" from "never started". With
   `red_first`, the verify runs before any change and its red goes into `result.md`. A green with
   no red recorded is reported as **unproven**.
3. **`git diff` omits untracked files**, so a phase that creates a module reviews as an empty diff.
   `git add -N .` first. `[tests/unit/test_tooling.py::test_run_reviews_untracked_files]`
4. **A reservation left only in `result.md` is invisible forever** (rule 6). Each one goes to
   `GOTCHAS.md`, to a later phase that exists, or is discarded with its reason.

## Delegates to

| Skill | When |
|---|---|
| `developer-skills:git-manager` | branch and PR work; the per-phase commit is a plain `git commit` |
| the skills `plan.md` and each `phase.md` name | how to build, test and review in that stack |
| `common-skills:dynbrain` | publishing the plan's documents at close |

## REFERENCES

`modes/` one file per mode · `templates/` the plan files. No scripts: the plan is markdown and git.
