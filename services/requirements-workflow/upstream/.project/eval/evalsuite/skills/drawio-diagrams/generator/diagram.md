---
schema: skills/drawio-diagrams
artifact: diagram
skills:
  - drawio-diagrams
sources:
  - ../../../../../../skills/plan/drawio-diagrams/references/sequence.md
  - ../../../../../../skills/plan/drawio-diagrams/references/flow.md
---

# Scenario — Exercise the drawio-diagrams skill

You are the diagram author described by the drawio-diagrams SKILL. The CONTEXT has
two parts: a short **brief** (the ask — which diagram type to produce and the
target document section) and the **notes** (design / requirements / meeting
material for the change, most of it not diagram-worthy).

Do the skill's authoring work as a single shot:

1. Read the brief: it names the diagram type and the target section. Read that
   type's reference in SOURCES (its CONCEPT + CONTRACT).
2. **Select** from the notes only the facts that belong in the diagram, per the
   SKILL's authoring principles (high-level, concise, third-party at the periphery);
   leave the rest — it is noise. Author ONE ` ```drawio:<type>:<id> ` block
   grounded strictly in the notes, following the reference's vocabulary and
   Rules & limits, then add the `![](./assets/<id>.png)` image line.

The brief always asks for a diagram, so producing one is the task — do not answer in
prose instead.

The diagram generator, validator, renderer, and Cursor preview are NOT available
here (see the LIVE-REFERENCE SUBSTITUTIONS): do not claim to run them; author the
block so it is valid and renderable by construction.

Output only the finished document as Markdown — a `# <title>` diagram section with
one block + image — following the CONTRACT's Output Shape exactly, nothing before or
after it except the required `<plan>` block.
