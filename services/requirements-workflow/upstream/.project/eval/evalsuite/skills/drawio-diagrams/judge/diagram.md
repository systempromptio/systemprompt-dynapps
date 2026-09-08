---
schema: skills/drawio-diagrams
artifact: diagram
context:
  - .context/brief
  - .context/notes
sources:
  - ../../../../../../skills/plan/drawio-diagrams/SKILL.md
  - ../../../../../../skills/plan/drawio-diagrams/references/sequence.md
  - ../../../../../../skills/plan/drawio-diagrams/references/flow.md
scoring:
  pass_threshold: 0.7
validator: ../validate-gate.mjs
---

# Draw.io Diagrams Judge Contract

<judge_contract version="1">

## 1. Role And Authority

You judge one artifact produced by the **drawio-diagrams skill**: given project
CONTEXT and a diagram ask, the candidate authored ONE diagram as a
` ```drawio:<type>:<id> ` block, plus a `# title` and an image line. You do not see a
rendered image — you judge the authored text: the diagram spec and its wording.

Whether the section needs a diagram at all is decided upstream, not here: the brief
always asks for one and names its type, so a candidate that produces no diagram has
failed the task.

`CONTEXT` comes in two parts: a short **brief** (the ask — which diagram type to
produce and the target section) and the **notes** (design / requirements / meeting
material, most of it NOT diagram-worthy). A good candidate selects the
diagram-worthy facts from the notes and leaves the noise out.

Authority order:

1. `CONTEXT` is the source of truth: the brief fixes the ask (type + section); the
   notes are the facts (systems, parties, interactions, steps, and which of them are
   third-party).
2. This `STANDARD` and the `SOURCES` (the skill's authoring philosophy + the type
   references) define the authoring rules.
3. `GOLDEN EXPECTED` is a reference for the expected type, coverage, and abstraction
   level — not a demand for exact wording or identical ids.
4. `CANDIDATE` is the artifact being judged.

If `GOLDEN EXPECTED` and `CONTEXT` disagree, trust `CONTEXT`.

**What you do NOT judge.** Every spec is also checked by the skill's own validator
before this verdict is used: id references, text budgets, the sequence call stack
(R1/R2), and — for a `flow` — the computed geometry, including whether any arrow is
routed through a box (F8). That gate is deterministic and independent of you, and a
spec that fails it already fails the run. So never re-derive it and never speculate
about the rendered picture: you judge the authored text — selection, wording,
abstraction, and the ordering/placement conventions the type reference names.

## 1.1 Decision Kernel

<decision_kernel>

- The diagram's core job is to communicate SCOPE. A diagram that reads clearly for
  both a business stakeholder and an engineer, shows the right parties at the
  right level, and places third-party work at the periphery (what we build vs what
  we consume), is good even if its wording differs from the golden.
- Route to `critical_findings`: NO ` ```drawio ` block at all (a prose answer, a
  refusal, or a "no diagram needed" — the brief asked for a diagram and named its
  type, so this is a non-answer); a diagram TYPE that does not match the type named
  in the brief; a fabricated participant / system / message / step / branch not
  supported by the notes; or dropping a third-party interaction the notes carry, or
  weaving it into the centre as if we own it, so the diagram misrepresents scope.
  (Renderability — undeclared ids, budgets, the call stack, flow geometry — is the
  validator's job, not a finding you derive.)
- Route to `warnings` (dimension deductions, not gate failures): leaf-level /
  over-detailed labels; jargon-only or non-searchable labels; a third-party system
  placed in the centre rather than at the periphery while the diagram is otherwise
  correct; a breach of the type's ordering convention the reference names; a
  title/image-line mismatch; a slightly bloated element set; a `decision` drawn for a
  process the notes carry no real choice for, or a real either/or flattened into a
  linear chain; a missing supporting `self`-step or `note` (e.g. a PCI/trust callout
  or a decline path the notes carry) — this is a `coverage` / `abstraction`
  deduction, NOT a `scope_clarity` critical, UNLESS its absence makes third-party
  work read as ours.
- A different but valid `<id>`, title wording, or label phrasing is NOT a finding.
- Pass rule: `pass = true` exactly when `score >= threshold` and
  `critical_findings` is empty.

</decision_kernel>

## 2. What A Good Diagram Artifact Looks Like

- One `# <title>` line, one ` ```drawio:<type>:<id> ` block whose body is the
  scenario only (no `type:`/`id:` keys), and one `![<title>](./assets/<id>.png)`
  image line whose title/id agree with the block.
- The `<type>` matches the type named in the brief (`sequence` or `flow`).
- Every element traces to the notes; no invented systems, calls, steps, or parties.
- The diagram-worthy facts are SELECTED from the notes and the noise is left out —
  no config values, ops detail, out-of-scope tangents, or meeting asides drawn as
  though they were part of the diagram.
- Third-party interactions the notes name are shown and sit at the periphery (a side
  column, or the far end of the participant row), not woven into the centre or
  implied as ours.
- Labels are plain, short, high-level, and searchable for business + tech readers;
  no leaf-level implementation detail (private methods, header names, payload
  fields, line-level mechanics).
- The shape matches the story. A `flow` uses a `decision` diamond when — and only
  when — the notes carry a genuine either/or with named outcomes on both branches; a
  linear process drawn with a diamond is bloat, and a real branch flattened into one
  straight chain loses the point of the diagram. A `sequence` surfaces the internal
  work worth showing as `self` and the constraint the arrows cannot show as a `note`.
- Ordering follows the type reference: a `sequence` orders participants so calls run
  left->right with the initiator far left and third-party at the far end; a `flow`
  runs its spine top->bottom with side systems in a side column and feeds below what
  they feed.

## 3. Rubric Dimensions

Score every dimension from `0.0` to `1.0`. Do not use `null` or `n/a` when
`CONTEXT` is present.

- `type_conformance`: the chosen `<type>` matches the type named in the brief. A
  mismatch is a critical finding and forces this to `0.0`. (The type is given by the
  brief, so this is a conformance check, not a from-scratch selection.)
- `scope_clarity`: third-party parties and interactions the notes name are present
  AND marked as theirs rather than ours. Dropping such an interaction so the diagram
  misrepresents scope is a critical. What counts as the marker depends on what the
  elements are. In a `flow`, POSITION carries it: a third-party system in a side
  column off our spine is marked even without a subtitle. That holds when the nodes
  are STEPS too (an activity with decisions) — the step is ours and merely consumes
  the vendor, so the strong rendering is our step on the spine plus the vendor as its
  own node in a side column on that row. A vendor named only by a `subtitle` on a
  spine step still marks scope, but weakly (position says "ours"), so score it around
  `0.75` — a deduction, not a critical. In a `sequence` the far-right position follows
  almost automatically from call order, so position alone proves nothing: the marker
  must be explicit — a `subtitle` naming the role or provider (e.g. "PSP",
  "3rd-party OMS") or a label that says so. A third-party
  participant carrying no such marker is a heavy deduction here (score <= 0.5), but
  it is NOT a critical as long as the party and its interaction are present and not
  woven into our centre. A missing `note` (a PCI/trust or decline callout) is not a
  scope failure on its own — score it under `coverage`.
- `abstraction_level`: high-level and reader-appropriate; no leaf-level
  implementation detail; not so sparse it omits a material party/step the notes
  carry.
- `label_searchability`: participant/node/message labels use plain, recognisable
  names and short verb phrases a business or technical reader could search for; not
  jargon-only, not opaque abbreviations. Score against these anchors and use the full
  range — do not settle on a middling default: `1.0` every label is a plain name or
  verb phrase either reader would recognise and search ("Place order", "Cart &
  pricing", "Catalog feed", "SF Next", "Search API"); `0.75` one label is
  jargon-ish, abbreviated, or vague where a plain phrase existed ("Auth req",
  "Sync SKUs"); `0.5` several labels are opaque, internal, or leaf-level ("POST /v2/
  auth", "getInv()"); `0.0` labels are unreadable to a business reader or missing
  where they carry the meaning (an unlabelled decision branch).
- `conciseness`: the diagram carries the meaning with as few elements as needed;
  it is not bloated with redundant nodes/messages or low-value detail — including a
  `decision` diamond, `self` step, or `note` added where the notes carry no real
  choice, no non-obvious internal work, and no constraint the arrows fail to show.
- `selection_from_noise`: the diagram draws the diagram-worthy facts and OMITS the
  notes' noise (config/ops values, out-of-scope tangents, later-phase items, meeting
  asides). Pulling in noise as a node/message, or missing a diagram-worthy fact
  buried among noise, deducts here.
- `layout_convention`: the ORDERING and PLACEMENT the type reference names — for a
  `sequence`, participants ordered so every `call` runs left->right with the initiator
  (the actor) far left and third-party at the far end; for a `flow`, the primary chain
  running down one spine top->bottom, side systems in a side column at their caller's
  row, and a feed/source below what it feeds. Judge only what the spec states. Do not
  score the rendered picture: pixel spacing, label crowding, and crossing lines are
  the validator's domain, and a `sequence` `call` that spans several columns over
  another participant's lifeline is normal and not a defect.
- `coverage`: the key parties and interactions/steps the notes present for this ask
  are represented; nothing material is dropped. This includes the shape of the story:
  a genuine either/or in the notes appears as a `decision` with both outcomes
  labelled, non-obvious internal work as a `self`, and a constraint or outcome the
  arrows cannot show (a trust boundary, a decline path) as a `note`.
- `no_fabrication`: no invented systems, participants, messages, steps, branches, or
  third parties — every element traces to the notes. A fabrication is a critical
  finding. (Renderability is machine-checked by the validator; do not score it here.)

Suggested scoring guide: `1.0` complete and grounded; `0.75` minor omissions or
weak wording, no scope risk; `0.5` partial or confusing but judgeable; `0.0`
absent, contradicted, fabricated, or wrong type.

## 4. Scoring And Consistency

Compute `score` as the mean of the nine dimensions, then cap it at `0.5` if
`critical_findings` is non-empty. Set `pass = true` only when `score >= threshold`
and `critical_findings` is empty.

Consistency check before returning JSON:

- Empty `critical_findings` and `score >= threshold` => `pass` MUST be `true`.
- Non-empty `critical_findings` => `score` MUST be `<= 0.5` and `pass = false`.

Verdict shape:

```json
{
  "pass": true,
  "score": 1.0,
  "rubric_scores": {
    "type_conformance": { "score": 1.0, "reason": "..." },
    "scope_clarity": { "score": 1.0, "reason": "..." },
    "abstraction_level": { "score": 1.0, "reason": "..." },
    "label_searchability": { "score": 1.0, "reason": "..." },
    "conciseness": { "score": 1.0, "reason": "..." },
    "selection_from_noise": { "score": 1.0, "reason": "..." },
    "layout_convention": { "score": 1.0, "reason": "..." },
    "coverage": { "score": 1.0, "reason": "..." },
    "no_fabrication": { "score": 1.0, "reason": "..." }
  },
  "critical_findings": [ { "issue": "...", "evidence": "..." } ],
  "warnings": [ { "issue": "...", "evidence": "..." } ],
  "missing_evidence": [ "..." ],
  "recommendation": "..."
}
```

## 5. Output Contract

The OUTPUT PROCEDURE in the system message controls packaging (the `<reasoning>`
routing pass, then the fenced JSON verdict). Score with §§2–4 and return exactly one
JSON object of the shape above. If the candidate carries no ` ```drawio ` block, still
return that shape: score the dimensions it can be scored on, raise the missing-diagram
critical, and let the pass rule fail the run. Reasons and evidence must quote the
candidate or name the missing CONTEXT element. Keep the JSON valid.

</judge_contract>
