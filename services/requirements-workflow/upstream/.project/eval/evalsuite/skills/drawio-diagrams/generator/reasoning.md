# Generator Planning Pass (skills — drawio-diagrams)

This partial defines the planning pass the generator performs before it writes the
document. It is injected into the generation system message and forces the
diagram-authoring decisions to be COMMITTED in the plan rather than improvised
while drafting. It does NOT restate the SKILL's authoring rules — it applies them.

Perform the planning pass as a compact checklist, not prose:

- **Brief + target.** Restate the ask in one line: the diagram type named in the
  brief and the target document section.
- **Read the type reference.** Confirm you have the named type's CONCEPT (to keep
  the right shape) and CONTRACT (vocabulary + Rules & limits) in mind.
- **Select from the notes (element ledger).** List the participants/nodes and the
  messages/edges (or steps) you will draw, each traced to a specific fact in the
  CONTEXT notes. Then name what you are deliberately LEAVING OUT as noise or
  leaf-level detail. Apply the SKILL's principles (high-level, concise, third-party
  at the periphery) here — do not restate them, act on them.
- **Layout.** State the ordering per the reference: sequence columns left->right in
  call order; flow rows top->bottom (and which nodes share the column-0 spine).
- **Validity commitment (renderable by construction).** Walk the ledger against the
  chosen reference's Rules & limits so it renders on the first try:
  - sequence: a strict call stack — only the top participant acts; every `call` is
    closed by a `return` to its caller before that caller acts again; `call`s go
    left->right; keep `call`/`return` <= 20 chars and `title`/`subtitle` within
    budget (~15 / ~22).
  - flow: every edge `from`/`to` is a declared node; a `decision` has >= 2 outgoing
    edges each with a real guard label; `title` <= 16, `subtitle` <= 18, edge
    `text` <= 56; and the `row`/`col` assignment satisfies the reference's placement
    self-check, so no arrow is routed through a box it does not connect.
- **Output shape confirmation.** Confirm you will emit `# <title>` + one
  ` ```drawio:<type>:<id> ` block + one `![](./assets/<id>.png)` line, and nothing
  else after the plan.

Use the plan only to commit the selected element set (and the noise you drop), the
layout, the validity check, and the output shape before authoring.
