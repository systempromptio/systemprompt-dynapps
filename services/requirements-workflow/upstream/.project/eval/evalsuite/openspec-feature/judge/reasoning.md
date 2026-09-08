# Judge Routing Pass (shared, injectable)

This partial defines the *routing pass* the judge performs before it scores. It is
injected into the judge system message for every artifact in this schema and is
artifact-agnostic on purpose: the concrete routing and gate rules live in each
artifact's judge contract (the `STANDARD`). The surrounding OUTPUT PROCEDURE has
the judge emit this pass as a `<reasoning>` block just before the fenced JSON
verdict, in one response.

Every closed-rule gate and sweep the STANDARD defines is part of this pass. Run
each one as the numbered procedure it is written as, and record its working in the
verdict's `gates` array: what the enumeration step produced, what an exclusion
step removed and which step removed it, and what reached the firing step. The
enumeration is the point of the gate — a gate you close with a count, a global
closure sentence, or a conclusion you reached without listing its items has not
been run. Every critical you raise must trace back to an item that survived one of
these gates, and every item that survives must be dispositioned: an item the
exclusions did not remove has reached the firing step, so say what it is —
`critical`, `warning`, `missing_evidence`, or `no_breach` with the reason it
reaches the firing step without breaching. A long enumeration with nothing
dispositioned is the same failure as no enumeration at all.

Perform the routing pass as a focused analysis, not prose:

- State the decisive tokens you observed in the candidate — for example the gate
  checkbox token, severity tokens, and dispositions — exactly as the STANDARD
  defines them.
- For each candidate finding, write one line that names or quotes it and then
  routes it to `critical_findings` or `warnings` by applying the STANDARD's
  routing and gate rules. Route by actual source-truth and handoff risk, never by
  how strong the wording sounds.
- SEVERITY DEMOTION GATE — before you route ANY finding to `critical_findings`,
  run these checks IN ORDER and stop at the first that hits:
  (1) Is the finding an ABSENCE — expected or grounded content the candidate
  simply does not contain (a missing row, a dropped operational prerequisite, a
  `Depends on` tie the metadata omits, a cross-cutting / merchant-configuration
  obligation absent from an Acceptance Criteria list, a thin-but-well-formed AC)?
  Then it is `missing_evidence`: it MUST NOT enter `critical_findings` and MUST
  NOT cap the score. EXCEPTION — a DROPPED SIGNED SOURCE ROW that the STANDARD
  explicitly routes to `critical_findings` as LOST source truth (a signed
  requirement / WBS id present in `CONTEXT` or `GOLDEN EXPECTED` but absent from
  the artifact's required trace/coverage) is a POSITIVE loss of source truth, not
  a demotable absence: it does NOT stop here, it goes to (3). A signed row that is
  PRESENT but merely thin, or present and dispositioned out of scope, is still an
  ordinary absence and stops here. (A thin AC is an absence. Parroting is a critical ONLY on an
  INTEGRATION-consumed slice whose Details carry bullets AND whose EVERY criterion is
  a SEED-ECHO of them, per the whole-block SEED-ELABORATION count — that is a
  present-in-text duplication: it does not stop here, it goes to (3). A FUNCTIONAL
  slice, a slice with empty (`—`) Details, or a block with even ONE elaborated
  criterion is NOT parroting — its shortfall is an absence, so it stays here.)
  (2) Does the STANDARD name this exact pattern as a WARNING or an ALLOWED
  pattern (e.g. a re-listed upstream Q# in Open Questions, a system-persona
  story, an over-detailed non-Gherkin AC, a heading / WBS-grouping slip, a
  working-copy anchor citation, an inherited user-facing URL)? Then it is a
  `warning` (or no finding at all) — never a critical.
  (3) Only a finding that survives (1)-(2) — a Forbidden item or an invariant
  1-8 violation whose evidence you can quote VERBATIM from the candidate — may
  enter `critical_findings`. A critical is a POSITIVE, present-in-text violation
  (a fabricated identifier, a settled pending item, a Gherkin block, a
  duplicate-surface group, an ownership breach, an INTEGRATION-consumed slice's AC
  block whose Details carry bullets and whose EVERY criterion is a SEED-ECHO of
  them), NEVER an absence, a "too thin" judgement, a functional-slice or empty-Details
  block, or a block that carries at least one elaborated criterion.
- SET-RECONCILIATION — when the STANDARD defines an explicit set reconciliation
  (e.g. a signed-requirement-set diff between the source and the candidate trace),
  compute it MECHANICALLY and STATE both sets and their difference in this block
  before you route. Do not skip it and do not rationalize a non-empty difference
  away as "correctly scoped"; route every lost-source-truth element of that
  difference to `critical_findings` exactly as the STANDARD directs.
- End with one line confirming the STANDARD's pass/score consistency rule: a
  non-empty `critical_findings` forces the capped score and `pass = false`; an
  empty one with `score >= threshold` is `pass = true`.

Apply the rules; do not restate the CONTEXT or rubric text. This pass exists only
to make severity routing deterministic before the verdict is written.
