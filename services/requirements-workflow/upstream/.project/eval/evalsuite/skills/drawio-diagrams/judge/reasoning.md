# Judge Routing Pass (skills — drawio-diagrams)

This partial defines the routing pass the judge performs before it scores. The
concrete rules live in the artifact's judge contract (the STANDARD). The judge
emits this pass as a `<reasoning>` block just before the fenced JSON verdict.

Perform the routing pass as a focused analysis, not prose:

- **State the decisive tokens FIRST.** Whether the candidate emitted a
  ` ```drawio:<type>:<id> ` block at all; its chosen `<type>` vs the type named in the
  brief (and the golden); the participants/nodes declared; which of them the notes call
  third-party and what marks each as theirs.
- **Route each finding** to `critical_findings` or `warnings` per the STANDARD:
  - Critical: no ` ```drawio ` block at all (prose, a refusal, or a "no diagram
    needed" — the brief asked for a diagram and named its type); a type that does not
    match the brief's named type; a fabricated participant/system/message/step/branch
    the notes do not support; dropping a third-party interaction, or placing it in the
    centre as if we own it, so the diagram misrepresents scope.
  - Warnings / dimension deductions: over-detailed (leaf-level) labels; jargon-only
    or non-searchable labels; a third-party party present but carrying no marker of
    whose it is, or placed in the centre rather than at the periphery; drawing the
    notes' noise (config/ops/tangents) as diagram elements, or missing a
    diagram-worthy fact buried in noise; a `decision` drawn without a real choice in
    the notes, or a real either/or flattened into a linear chain; a missing `self` /
    `note` the notes warrant; a breach of the type's stated ordering convention; a
    title/image-line mismatch.
- **Do not derive renderability or geometry.** Undeclared ids, text budgets, the
  sequence call stack (R1/R2), and flow arrow/box collisions (F8) are checked by the
  skill's validator independently of this verdict; never raise them as findings and
  never speculate about the rendered picture.
- **Confirm the pass/score rule:** a non-empty `critical_findings` caps `score` at
  0.5 and forces `pass = false`; an empty one with `score >= threshold` is
  `pass = true`.

Apply the rules; do not restate CONTEXT or rubric text. This pass exists only to fix
severity routing before the verdict is written.
