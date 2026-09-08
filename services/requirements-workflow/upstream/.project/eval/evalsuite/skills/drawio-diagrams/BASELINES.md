# drawio-diagrams eval — baselines

Suite: `evalsuite/skills/drawio-diagrams` · schema `skills/drawio-diagrams` · artifact `diagram`.
Generator `openai.gpt-oss-120b-1:0` / judge `mistral.mistral-large-3-675b-instruct`, threshold 0.7.

Design of this suite:

- Each case's CONTEXT is split into a thin **brief** (`.context/brief.md` — names the diagram
  type + the target section) and a rich, noisy **notes** pile (`.context/notes.md` — design /
  requirements / meeting material, most of it NOT diagram-worthy). The model must SELECT the
  diagram-worthy facts and leave the noise out.
- **Notes convention (hybrid).** Inside the design/behaviour block, each diagram-worthy fact is
  its OWN bullet (one bullet ~ one participant, message, step, or branch); the noise stays prose
  in `Config / ops`, `Meeting notes`, and the glossary. So the case tests selecting signal from
  noise — not unpacking a dense paragraph — and `coverage` is comparable across cases. Facts that
  belong on the diagram only as a `note` (a trust boundary, a decline path) may stay in prose.
- **Every case asks for a diagram, and its type.** The skill only teaches HOW to draw; whether a
  section needs a diagram at all is an OpenSpec-artifact decision, so it is out of this suite's
  scope. A candidate that answers in prose with no ` ```drawio ` block has not done the task: the
  validity gate fails it and the judge raises a critical.
- **The judge reads the same doctrine the author does.** `judge/diagram.md` inlines the skill's
  `SKILL.md` plus both type references as SOURCES, so a rubric dimension can be scored against
  the stated convention instead of the model's priors. Renderability and geometry (ids, budgets,
  the sequence call stack, flow's F8 arrow-through-box check) are owned by the deterministic
  validity gate — the judge is told not to re-derive them or speculate about the rendered image.
- The brief NAMES the type, so `type_conformance` (used the named type) replaces the old
  `type_selection`. A `selection_from_noise` dimension rewards drawing signal and dropping noise.
- All cross-type authoring rules live in the skill's `SKILL.md` (Authoring philosophy); the
  harness contract/reasoning no longer re-teach them, and the per-type references follow one
  Concept -> Contract shape.

> **Stale.** The numbers below predate the F8 geometry rule, the flow layout doctrine, the hybrid
> notes convention, and the judge's SOURCES/rubric actualization. They are kept only as the
> before-picture until the next baseline replaces them.

Numbers below are **dev grade (5 runs × 1 judge sample)** — a diagnostic signal, not a pinned
release baseline. Read per-dimension means + recurring criticals, not the n=5 pass-rate. Re-run
at release grade (`--mode release --runs 30 --judge-samples 5 --concurrency 20 --save`) to pin
headline numbers. Golden ceilings are judged at 3 samples.

Run one case:

```bash
node run.mjs eval --schema skills/drawio-diagrams --artifact diagram --dir <case> --save
# then render every saved candidate to PNG for visual review:
node evalsuite/skills/drawio-diagrams/render-candidates.mjs .results/<saved-run-dir>
```

## Cases

| case | expected behaviour | golden ceiling | dev pass | mean | renders (gate) |
|---|---|---|---|---|---|
| `sequence-checkout-payment` | `sequence`; mark Bolt (PSP) out-of-scope from the SAQ-A fact | 100% | 3/5 | 0.69 | 5/5 |
| `flow-search-integration` | plain `flow`; single entry (Shopper), third-party at the periphery; the PIM feed publishes to SFCC + Bloomreach via async (dashed) edges | 100% | 5/5 | 0.86 | 5/5 |
| `activity-bopis-pickup` | `flow` with a decision; Fluent OMS marked 3rd-party | 100% | 4/5 | 0.75 | 4/5 (1 invalid) |

All three goldens hold a **100% ceiling** — the rubric and contract are sound; the gaps below are
generator behaviour under the noisy contexts, not contract defects.

## Judge-criteria means (dev grade, stale)

| dimension | seq | flow | bopis | what it measures |
|---|---|---|---|---|
| type_conformance | 1.00 | 1.00 | 1.00 | used the type named in the brief — never wrong across 15 runs |
| scope_clarity | 0.70 | 0.80 | 0.85 | third-party / out-of-scope work shown and marked |
| abstraction_level | 0.70 | 0.90 | 0.70 | high-level, no leaf detail |
| label_searchability | 0.75 | 0.73 | 0.65 | plain, business+tech searchable labels |
| conciseness | 0.80 | 0.83 | 0.60 | fewest elements that still carry the meaning |
| selection_from_noise | 0.85 | 0.90 | 0.65 | drew the signal, left the notes' noise out |
| layout_convention | 0.95 | 0.85 | 0.70 | left->right / top->bottom ordering |
| coverage | 0.60 | 1.00 | 0.85 | all diagram-worthy parties/steps present |
| no_fabrication | 0.80 | 1.00 | 0.80 | no invented elements; renderable by construction |

## Verdict

- **The rework holds.** `type_conformance` is 1.00 everywhere (the brief names the type) and
  **renderability is 14/15 runs valid** (was ~7/15 before the harness de-dup + the reasoning
  validity-commitment). `selection_from_noise` is a live, discriminating dimension (0.65-0.90) —
  the noisy fixtures exercise exactly the behaviour we wanted to measure.
- **The suite is now harder and more honest.** The noisy `sequence` and `bopis` contexts pull the
  mean down on `coverage` / `conciseness` / `abstraction` — the generator sometimes drops a
  diagram-worthy fact buried in noise, or pulls in a low-value one. That is the intended signal, not
  a regression.
- **Scope survives as position, not a subtitle.** `flow-search` carries its boundary by *placement*
  — third-party systems sit at the periphery (Bloomreach in the side column, the PIM feed on its own
  row publishing via dashed async edges) — and holds `scope_clarity` 0.80 with no outliers;
  `sequence` and `bopis` (boundary = a participant/node subtitle) swing to 0.00 on the runs that drop
  the marker. Same lesson: a boundary carried by position or a labelled edge survives; a lone
  subtitle does not.

## Known issues (contract vs 120b capacity)

- **Contract/rubric:** none blocking — all three ceilings are 100%. Severity of a missing
  third-party marker is now settled in the rubric: dropping the party or weaving it into our centre
  is critical; a present-but-unmarked party is a heavy `scope_clarity` deduction (<= 0.5), not a
  gate failure.
- **120b capacity (improve prompt/fixtures, never swap the model):**
  - occasional `no_fabrication` 0.00 outliers on `sequence`/`bopis` (a fabricated element or one
    unrenderable spec — 1/5 `bopis` runs invalid);
  - `label_searchability` ~0.65-0.75 (jargon-ish labels vs the golden's plain verb phrases);
  - `coverage`/`conciseness` tails under the noisy contexts (a buried fact dropped, or a low-value
    one drawn).
