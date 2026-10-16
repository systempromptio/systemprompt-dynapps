# Mode: research

Position the plan before it is written. Writes `plans/draft/<slug>/research.md` and nothing else:
**never writes code, never writes a plan**, and never write into `plans/active/`.

Optional: `new` runs its own reconnaissance anyway. Use this mode when you want the map reviewed
before planning, or when the user asks for a deep investigation.

## Default: reconnaissance

For a defined task the only research is finding **every piece involved**, so the plan is written
against the real codebase. One `Explore` subagent with `model: haiku`, one pass, read-only:

- the modules, models, files and entry points the objective touches — `path:line`
- what already exists that does the job or part of it, and the pattern to copy
- the tests that cover that area and how they run (this is the future verify)
- versions and dependencies that constrain the work
- the external access the work will need (future `## Requires`)

Every claim carries its source — a `path:line` — or is written as `**Assumption:**`. No web search,
no alternatives analysis, no questions to the user unless one blocks the plan.

## Deep — only when the user explicitly asks

`research --deep`, or the user's words ask for an investigation of options. Then four fronts:

| Front | How |
|---|---|
| The repository | the reconnaissance above, first — half the questions dissolve |
| Outside | `/deep-research` when available, otherwise `WebSearch`; say which |
| The client | ask the user, batched, only what the other fronts cannot find out |
| The decision | each `RESOLVED` question names its rejected alternative; a decision with no rejected alternative is an assumption |

Limits: **at most 5 questions** (the rest under `Not investigated`), **one round per front** — a
question surviving its round is `OPEN`, not a reason to keep digging.

## An OPEN question blocks the plan

`questions_open` in the frontmatter is the count; `new` will not activate a plan while it is above
zero. Close it, or accept it as a risk in `goal.md` — never drop it silently.

## The file

From `templates/research.md`. It guides you in HTML comments: **strip every one of them**, and
their placeholder prose. Under ~60 lines for a reconnaissance. `new` keeps it in the plan directory.

`/dev-plan research` with no objective picks up the research already in `draft/`: show the open
questions, take the answers, recount `questions_open`.

Close by reporting the path, the pieces found, the open count, and the next step without running
it: `Next: /dev-plan <objective>   (will consume this research)`.

