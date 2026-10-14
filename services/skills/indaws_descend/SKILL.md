---
name: indaws-descend
version: 1.1.1
model: opus
effort: medium
description: >
  Iteratively improve an artifact OR a process by gradient descent on an explicit rubric — freeze a
  scored rubric, extract knob-attributed critique as the gradient, apply decaying-size patches, stop
  on measured convergence. Two targets: artifacts (document, proposal, quote, email, plan, skill
  description) and processes (workflow, SOP, handoff, pipeline, way of working) — scored by
  dry-running the spec against real historical cases and delivered as a change plan, never applied
  live. Triggers: "/descend", "descend on this", "iterate until it converges", "optimize this
  document", "optimize this process", "improve our workflow", "mejora este proceso", "optimiza este
  flujo", "itera hasta que converja". NOT decisions or discrete choices (use grill-me), NOT one-shot
  analysis (use deep-think), NOT live system changes.
---

# Descend — gradient descent on an artifact or a process

<role>Optimizer, not reviewer. Make a measured quantity move; stop when it stops moving.</role>

Target: `$ARGUMENTS`

## Method

Score S (0–10) = weighted rubric over 4–6 dimensions, frozen before the first edit. Each round a
**fresh-context** evaluator returns critique attributed to named **knobs** with direction and
magnitude — the gradient. Patch size decays; stop on measured convergence; a held-out judge that
never saw the rubric checks you optimized the right thing.

## Pick the target class (first decision)

| Target | It is | Artifact of record | Scored by |
|---|---|---|---|
| **Artifact** | a text that IS the deliverable | the text itself | reading it |
| **Process** | a way of working — who does what, in what order | its written spec | dry-running it against real past cases |

Process mode changes the gate, rubric, lenses and exit — **read `references/process-descent.md`
before Step 0.**

## Step 0 — Don't-bother gate

| Trip | Route to |
|---|---|
| Artifact < ~500 words | one `deep-think` pass |
| Cannot name 4 dimensions that move independently | `deep-think` |
| Target is a decision / choice between options / yes-no | `grill-me` |
| Nothing exists yet | draft or write the spec first (one shot), then descend |
| **Process:** spec and observed reality differ | stop — that gap is finding #1, no descent yet |
| **Process:** fewer than 5 real past cases available | stop — you would score prose, not the process |

Say which gate tripped and what you would do instead — do not proceed anyway.

## Step 1 — Freeze the rubric [OPUS]

1. Run dir = the slug alone under this skill's state root — resolve it per `run-mechanics.md`
   §Run dir, never a hardcoded home. Starting sets in `rubrics.md` or `process-descent.md` §3.
2. 4–6 dimensions, weights summing to 1.00, each **anchored** — what a 3, a 6 and a 9 look like
   *for this target*. Unanchored = not measurable.
3. Name the **knobs** — concrete editable levers. Knobs are what you turn, dimensions what you
   measure; different lists.
4. **Process only:** freeze the **case set** (≥5 real past cases: ≥3 typical, ≥2 exception, 1
   pathological), chosen *before* studying the spec.
5. Write `rubric.md`, get user sign-off, record its sha256 in `log.tsv`'s header.

**The rubric never changes after this.** If a later round shows it was wrong, stop and start a new
run — amending mid-run is label leakage: the target moves with the output and every score stops
meaning anything.

## Step 2 — Baseline: score as-is (round 0) with the Step 3 evaluator; every claim is relative to it

## Step 3 — The gradient [SONNET SUBAGENT, fresh context, every round]

`Agent(model: sonnet)` — haiku for mechanical-count rubrics. It receives **only** the current target,
`rubric.md`, the round's two lenses, and (process mode) the case set. **Never** previous drafts,
critiques, scores or this conversation: constant per-round cost *and* no anchoring to its own prior
scores. Rotate 2 lenses per round, no repeats until the catalog is exhausted — differently-biased
estimates stop you overfitting to one critic's taste. Prompt shape in `references/run-mechanics.md`.

Every finding carries `knob` · `direction` (increase|decrease|restructure) · `magnitude` (0.1–1.0
expected ΔS) · `location`. **A finding with no knob is rejected** — "the value prop is weak" is a
complaint; "knob=proof-density, increase, 0.6, §2 has one reference where the rubric anchors 9 at
three" is a gradient. Unattachable findings mean the *rubric* is incomplete: log them, carry to
Step 6, do not act this round.

## Step 4 — The step [OPUS]

Apply the highest-magnitude findings via **Edit**, never a full rewrite. η by round: **1** high
(structural — reframe, reorder, cut or add whole sections/steps) · **2–3** medium (paragraph or step
level) · **4+** low (sentence and word only).

**No structural change after round 3** — late structural edits invalidate every dimension at once and
the trajectory oscillates instead of settling. **Momentum:** a knob that gained ≥1.0 last round gets
pushed again even if it is not the top gradient. Line search, log schema, report template and cost
rules in `references/run-mechanics.md`.

## Step 5 — Convergence

Append to `log.tsv`: `round, <dim columns>, S, ΔS, η, lenses, knobs_turned`. Stop when **any**:
ΔS < 0.3 twice running (converged, the normal exit) · S ≥ target (default 8.5) · round cap (default
5, `--rounds N`) · S decreased twice (past the minimum — revert to the best round) · **process:**
the top gradient points twice at a knob the user cannot turn (constraint boundary — stop and
escalate it as a decision, not a score).

## Step 6 — Held-out judge [SONNET SUBAGENT]

Once, at the end. Fresh subagent that has **never seen `rubric.md`**: final target + its purpose,
two questions — *is this good?* and *what is wrong with it?* (process mode: role-play the person who
must run it on a bad day — where do they get stuck?). Judge happy + converged → report the
trajectory. **Judge flags something the rubric does not cover → you optimized a proxy**: report it
prominently, do not patch it silently, offer a fresh run with an extended rubric. Something
deliberately traded away → say so, let the user decide.

## GOTCHAS

1. **Rubric amended mid-run** — commonest way a run becomes meaningless. New rubric = new run.
2. **Evaluator given history** — one paste of the prior critique and it turns self-congratulatory.
3. **Convergence read as quality** — ΔS < 0.3 means the *rubric* stopped moving; only Step 6 judges
   whether it was the right one.
4. **Optimizing the non-bottleneck (process)** — points on paper, nothing in reality. ×0.3 discount.
5. **The documented process is not the real one** — a spec/reality gap is the finding, not the start.
6. **Full rewrites after round 1** — blow the cost model (output ≈5× input) and break comparability.

## NEVER / ALWAYS

- **NEVER** change rubric, weights or case set mid-run; let the evaluator see prior rounds; or
  accept a finding without a knob.
- **NEVER** touch live systems during a process run — no cron, automation, CRM stage, permission or
  assignment edits, no messages to affected people. Deliverable = change plan; enacting it is the
  user's call.
- **ALWAYS** score a baseline first, report knobs that did *not* move, and quote the held-out judge
  verbatim — especially when it contradicts the score.

## REFERENCES

- `references/process-descent.md` — process mode: cases, rubrics, knobs, lenses, plan, traps (§8)
- `references/rubrics.md` — artifact rubrics (proposal, skill description, email, plan)
- `references/lenses.md` — artifact lens catalog + rotation contract
- `references/run-mechanics.md` — evaluator prompt, log schema, line search, report, cost

## Delegates to

`deep-think` (optional lens set) · `skill-creator` (optional eval harness when the target is a skill
description). Absent, built-in catalogs are used.

## Consumed by

Any suite with a deliverable or a way of working worth iterating — invoke by name, no code dep.
Shared refs: none, so `common:references/shared/` does not apply.
