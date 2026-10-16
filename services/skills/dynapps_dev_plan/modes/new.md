# Mode: new

Write a plan into `plans/draft/<slug>/`. **Never execute it.**

Never ask what you can propose with a written reason: a bad proposal is fixed while reviewing the
draft, a question interrupts. You may ask three things: the termination criteria, a real ambiguity,
and the skill selection.

## Consume the research, if there is one

Look for `plans/draft/*/research.md` matching this objective; if several, ask which.

- A `RESOLVED` question is settled: do not re-open it, reference it from `decisions.md`.
- A constraint becomes a gotcha, a `## Requires` entry or a phase — it must land somewhere.
- Keep `research.md` in the plan directory: it is part of the record.
- **An `OPEN` question stops you.** Send the user back to `/dev-plan research`, or write it into
  `goal.md` as accepted as a risk. Never plan over a silent guess.

## Reconnaissance — always

Whether or not research ran. Position yourself before phasing: for each deliverable, find the
pieces involved and whether something already does that job (`ls`, `grep`, the project's
manifests, its tests). If something already covers the objective, say so and stop: the most
expensive plan is the one for work that did not need doing. Write what you found in `decisions.md`, including "nothing". A deliverable
that duplicates an existing one passes every check that only looks at the deliverable.

Keep it to one pass. Use an `Explore` subagent with `model: haiku` when the sweep spans more than
a handful of files, so the reads do not stay in this window. No web search here.

## Skills — from this session only

**The source is the skills loaded in this session — nothing else.** Not a directory and not a
manifest. **Never name a skill this session does not have**, and copy the full name exactly as
listed, namespace included, never from memory: a real run wrote
`developer-skills:odoo-module-documenter` [wrong-on-purpose] for a skill living in
`common-skills`, and the invocation found nothing. When a name looks relevant but the listing gave
no description for it, propose it and say so.

| Kind | Found by | Goes in | Cap |
|---|---|---|---|
| Phase skills | what the phase touches | `phases/NN/phase.md`, `## Skills` | **2 per phase** |
| Crosscutting | **the five axes**: version control, code style, testing, documentation, review | `plan.md` | **4 total** |

The stack lives here. dev-plan knows no framework: how to design, build and test honestly in
Odoo, Django or anything else is what the phase skills and the testing axis bring. A stack with
skills of its own in this session gets them.

Crosscutting skills are never found from the objective — an objective about a catalogue never
mentions commits — so go axis by axis. An axis with no good fit is reported empty, not filled.
They go in `plan.md` because rule 6 fixes what gets read. If a phase needs a third skill, propose
it and ask; with nobody to ask, write two and record the third in `decisions.md` — a cap that
bends unobserved is not a cap.

Write them as instructions, not as suggestions, each with a one-line fallback:
`Use developer-skills:git-manager for this phase's commit. If unavailable: commit by hand.`

Confirm the selection with the user before writing the plan: a selection nobody confirmed is a
guess with a checkmark next to it.

## Ask at most two things, and propose the level with them

One message: restate the objective, draft the `Done when` list for confirmation, raise an
ambiguity only if two readings give materially different plans — and propose the plan's level
with its evidence (`--level` given by the user wins):

| Level | Signals |
|---|---|
| simple | defined task, a pattern to copy, `git revert` undoes it |
| media | several modules or files, new logic with precedent, a central model touched |
| alta | a version migration, a module other projects reuse, irreversible data migration, unattended `--all` over 8 phases |

## Phases

One unit of work per phase; over 150 lines of detail means split it. For each, from
`templates/phase.md`:

| Field | How to decide |
|---|---|
| `executor` | per phase, never from the level. `sonnet` — the default: new logic with precedent, a known API, a choice left to the executor. `haiku` — only with a **closed brief** (below). `opus`: no precedent, a central model, data migration, security, version change, an architecture decision. `inline`: see below |
| `risk` | the signals present: `core-model`, `customer-data`, `security`, `migration`, `irreversible`. Non-empty forces full review and forbids `inline` |
| `rounds` | from the level; 2 when `risk` is non-empty |
| `red_first` | from the level; `true` whenever `risk` is non-empty |
| `## Docs` | the documents this phase writes or must update — rule 7 |
| `## Requires` | access only this phase needs |

**A closed brief — required for `haiku`.** Haiku costs a fraction of the others and executes well
what it is told exactly; what it does not do well is decide. A Haiku phase that comes back wrong
costs more than a Sonnet one that comes back right, because the correction is paid here, in the
main conversation. So `phase.md` must leave it nothing to decide:

- `## Files touched` — every place to change as `file:line`, from the reconnaissance
- `## Design` — the decisions, numbered, none left open
- `## Not in this phase` — what stays out, including the options rejected and why
- `## What gets built` — names the tests to add and the existing ones to update
- `## Verify` — targeted, with the minimum number of tests it must report

If writing any of them needs exploration you have not done, the brief is not closed: do the
reconnaissance first or choose `sonnet`. A plan being `simple` is never the reason for `haiku`.

**`inline`** — the main conversation does the phase itself. Propose it when delegating costs more
than doing: a small, fully specified change (1–2 files, under ~30 lines: a version bump, config,
a rename), a phase that needs what only this conversation knows, one that needs the user mid-way
(a login, a choice), or a closing phase (docs coherence, changelog, the PR). Never when the phase
needs heavy exploration — those reads would stay in this window for the rest of the plan — and
never with a non-empty `risk` (permanent rule 4).

**The verify** — a shell command with an exit code that runs **the tests of the code this phase
touches**, never the whole suite: `pytest -q tests/test_invoice.py`, `go test ./internal/invoice/...`,
`npm test -- invoice.spec.ts`. The phase writes or extends those tests (`## What gets built`). The
command comes from the repository and from the testing skill, if the plan has one; nothing
recognisable → propose one and say it is a guess. A runner that exits 0 having run nothing gets a
check on its count (GOTCHA 1). A check that asks a service how it went must be pinned to
this commit (`select(.headSha=="$(git rev-parse HEAD)")`).

**The verify must be satisfiable inside its own phase.** It must pass with only that phase's
deliverable in place; a verify that asserts on a later phase's work forces the phase out of its
own scope. Symptoms: the same suite gating two phases, or a deliverable nothing asserts. Fix the
split now — once running there is no legitimate way out.

**The full suite** — decided once, for `plan.md`'s `Full suite:` line. Look at the repository's CI
(`.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile`, `.circleci/`…) for a job that runs the
whole suite on PRs/MRs:

- it does → `Full suite: CI — <file>, job <name>`. The plan never runs the suite; the PR does.
- it does not, or it runs only part → ``Full suite: `<command>` — run by close``.
- the last phase's verify already is the whole suite (a migration's final replay) →
  `Full suite: phase NN`, with why.

Quote the file and job you found, or say there is none; the user corrects it while reviewing.

**Requires** — one entry per external access (cluster, API, database, VPN host, a binary): a
command that proves access, not presence; read-only always; never the secret's value; a real
command rather than a shell built-in. Say how to get access both when it was never set up and when
it expired.

**Out of scope** — when the objective is broad, propose the two or three things left out and why.

## Files to write

Into `plans/draft/<slug>/` (`YYYY-MM-short-description`), from `templates/`:

- `BRIEF.md` — mandatory, under 40 lines, written now while the context is alive
- `GOTCHAS.md` — empty, with its header
- `goal.md` — objective, `level`, `Done when`, `Out of scope`, stop criterion
- `plan.md` — the `Full suite:` line, the phase table with every row `pending`, and the
  crosscutting skills
- `decisions.md` — every decision taken while drafting, with its reason
- `phases/NN-slug/phase.md` — one per phase
- `docs/` — empty; `logs/.gitignore` containing `*`

Never write into `plans/active/` from this mode. Never name anything `project-plan`: another skill
owns that name.

## Activating

Check the draft against this list — it is what makes a plan runnable, and there is no script to do
it for you:

- every `haiku` phase has a closed brief — all five items above; otherwise complete it or make it
  `sonnet`
- every phase has an `executor` among `inline`, `haiku`, `sonnet`, `opus`, `rounds` 1 or 2, and a
  `## Verify` that is a real command running the tests of what that phase touches, not the suite
- `plan.md` has its `Full suite:` line, naming the CI job, the command or the last phase
- every phase with a `risk` is not `inline`, has `rounds: 2` and `red_first: true`
- every `Done when` line is asserted by some phase's verify
- `research.md`, if there is one, has `questions_open: 0` or its open questions are accepted as a
  risk in `goal.md`
- every `## Requires` command, run now, succeeds

A failing item: say which and **do not offer** to activate; offer to fix the plan instead.

All hold: **show the phase index** (number, name, executor, verify; one line each) and ask whether
to activate. Whether the breakdown is right is the one judgement no list makes. On yes,
`git mv plans/draft/<slug> plans/active/<slug>` (or `mv`) and commit the plan. Activating makes the
plan runnable; it does not certify it, and moving it back is free. Never activate without asking,
and never read a silence as a yes.

Recommend running it from a fresh session: the main conversation's window is what every turn of
the run pays for.
