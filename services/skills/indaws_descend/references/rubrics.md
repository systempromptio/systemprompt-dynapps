# Starting rubrics — artifacts

Process rubrics live in `process-descent.md` §3; this file covers artifacts only.

Templates, not prescriptions. Adapt the dimensions and re-weight for the
specific artifact, then **anchor every dimension** (what a 3, 6, and 9 look
like *for this artifact*) before freezing. An unanchored dimension is not
measurable and will drift round to round.

Weights must sum to 1.00. Dimensions must be able to move independently — if
raising one always raises another, they are one dimension.

---

## Commercial proposal / quote

| Dim | W | Knobs |
|---|---|---|
| Value clarity — is the outcome bought, not the feature list | 0.30 | `outcome-framing`, `feature-to-benefit-ratio` |
| Proof | 0.25 | `proof-density`, `reference-specificity` |
| Objection pre-emption | 0.20 | `risk-section`, `alternative-comparison` |
| Price justification | 0.15 | `price-anchor`, `scope-boundary-clarity` |
| Scan-ability | 0.10 | `heading-quality`, `table-vs-prose` |

Anchor note: value clarity 3 = feature list; 6 = features with stated benefits;
9 = the client's own metric moving, quantified.

---

## Skill description (triggering accuracy)

The one case with a **genuinely measured** loss — run evals, don't estimate.
Use `skill-creator`'s eval harness for the score and treat variance as the
noise floor: if per-round ΔS is below observed variance, you have converged
regardless of what the mean says.

| Dim | W | Knobs |
|---|---|---|
| True-positive rate on target prompts | 0.40 | `trigger-phrase-coverage`, `verb-variety` |
| False-positive rate vs. sibling skills | 0.30 | `NOT-clauses`, `boundary-naming` |
| Multilingual trigger coverage | 0.15 | `language-variants` |
| First-line clarity | 0.15 | `opening-verb`, `scope-sentence` |

---

## Internal one-pager / plan

| Dim | W | Knobs |
|---|---|---|
| Decision-readiness — can a reader decide from this alone | 0.30 | `ask-explicitness`, `option-framing` |
| Evidence | 0.25 | `proof-density`, `assumption-labelling` |
| Concision | 0.20 | `compression`, `section-count` |
| Risk honesty | 0.15 | `downside-section`, `hedge-count` |
| Structure | 0.10 | `section-order`, `heading-quality` |

---

## Outbound / persuasive email

| Dim | W | Knobs |
|---|---|---|
| Relevance to *this* recipient | 0.35 | `personalisation-specificity` |
| Single clear ask | 0.25 | `cta-specificity`, `ask-count` |
| Brevity | 0.20 | `word-count`, `compression` |
| Credibility | 0.20 | `proof-density`, `hedge-count` |

Anchor note: personalisation 3 = company name merged in; 9 = a fact only
someone who researched them would know, tied to the ask.

---

## Designing a rubric from scratch

1. Write the artifact's **job** in one sentence: who reads it, what changes.
2. List the 4–6 ways it could fail at that job. Those are your dimensions.
3. Weight by how expensive each failure is — not by how easy it is to measure.
   Weighting by measurability is how you end up optimizing word count.
4. For each dimension, write the 3 / 6 / 9 anchors.
5. Name 1–3 knobs per dimension. If a dimension has no editable knob, you
   cannot descend on it — drop it or re-scope it.
6. Sanity check: could two dimensions move in opposite directions? If no
   dimension can ever trade off against another, the rubric is one-dimensional
   and every round will just say "make it better".
