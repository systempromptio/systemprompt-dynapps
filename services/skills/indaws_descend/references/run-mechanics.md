# Run mechanics — evaluator, log, step size, report, cost

Operational detail for the loop in SKILL.md. Read when running, not when deciding whether to run.

## Run dir

`references/shared/skill-path-resolution.md` defines the state root **already scoped per
skill** (`${DYNAPPS_HOME:-$HOME/.dynapps}/state/<skill-name>`). So a descend run is the slug alone:

```
<state-root>/state/descend/<slug>/     # rubric.md, log.tsv, per-round drafts
```

Two mistakes it is easy to make. Adding a second `descend/` segment, because the root already
carries the skill name. And hardcoding one agent's home (`~/.claude/descend/…`) — that path only
exists under Claude Code, so the run state vanishes the moment the skill runs anywhere else, and
nothing warns you: it just writes a fresh empty run.

## Evaluator contract

Spawn per round with `Agent(model: sonnet)` — `haiku` only when every dimension is a mechanical
count. The prompt contains exactly four things: the current target, `rubric.md`, the two lenses for
this round, and (process mode) the frozen case set. Nothing else — no history, no prior scores, no
conversation.

Required return shape:

```
dimension: <name>   score: <0-10>   because: <one line tied to an anchor>
...
finding:
  knob:      <knob name from rubric.md>
  direction: increase | decrease | restructure
  magnitude: 0.1–1.0        (expected ΔS if fully applied)
  location:  <line ref or quoted anchor>
  why:       <one sentence, tied to a rubric anchor>
```

Process mode adds, per case: `completed | stalled at <step>`, touches, handoffs, undefined points.

Reject any finding without a knob and log it under `rubric_gaps` — those feed Step 6, not this
round's patch.

## Log schema

`log.tsv`, one row per round, header carries the rubric sha256 and (process mode) the case-set hash:

```
# rubric_sha256=<hex>  cases_sha256=<hex>  target=<path>  weights=<d1:0.30,...>
round  d1  d2  d3  d4  d5  S  dS  eta  lenses  knobs_turned  rubric_gaps
0      6.0 5.5 7.0 6.5 6.0 6.2  —   —    —      —             —
1      7.5 6.0 7.0 7.0 6.5 7.4 +1.2 high  spine,evidence  section-order,proof-density  —
```

Print exactly one line per round to the user:
`round 3  S 7.4 → 8.1  (+0.7)  η=med  knobs: proof-density, opening-hook`

The log on disk is the memory. Never re-narrate it into the main context — that re-introduces the
quadratic growth the fresh-context design exists to avoid.

## Step size, momentum, line search

Decay schedule is in SKILL.md. Two refinements:

- **Momentum** — a knob that gained ≥1.0 last round gets pushed again this round even if it is not
  the top-ranked gradient. Directions that paid keep paying; momentum carries you over plateaus.
- **Line search** (`--linesearch`, ~3× per-round cost) — generate 3 candidate patches at small,
  medium and large step, score all three, keep the best, discard the rest. Worth it when one
  dimension dominates the weights and you cannot guess the right amplitude.

## Final report

```
Target:     <path>            Class: artifact | process
Rubric:     <4-6 dims, weights>        sha256 <first 8>
Cases:      <n typical / n exception / n pathological>     (process only)
Trajectory: 6.2 → 7.4 → 8.1 → 8.2 → 8.2   (converged, ΔS<0.3 ×2)
Exit:       converged | target | cap | diverged | constraint-boundary
Best round: 4
Knobs that moved:      proof-density +2.1, opening-hook +1.4
Knobs that did not:    section-order (0.0 — already optimal or wrong knob)
Rubric gaps logged:    <findings that could not name a knob>
Held-out judge:        agreed | FLAGGED: <what the rubric missed>
Cost:                  ~<N>k in / ~<N>k out across <R> rounds
```

Process runs append the change-plan table from `process-descent.md` §7.

Always report knobs that did not move. A knob that never moves was already optimal or was the wrong
knob to name — both inform the next rubric.

## Cost discipline (why this is ~2–3× a one-shot, not ~10×)

1. Evaluator in a **fresh subagent** each round → cost per round is constant, not O(round).
2. Evaluator is **sonnet or haiku**; only patch-writing uses opus.
3. Patches via **Edit**; no full rewrites after round 1 (output tokens ≈5× input).
4. Score log **on disk**, not in context.
5. Rubric **never restated** in the main context after freeze.

If you paste the target into the main context a second time, the design has been violated and the
run has become the expensive kind.
