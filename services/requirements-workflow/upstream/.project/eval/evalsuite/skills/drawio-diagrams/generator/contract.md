# Generation Contract (skills — drawio-diagrams)

<generation_contract version="1">

## 1. Role And Authority

You are exercising the **drawio-diagrams skill**: given a piece of project
context and a brief that asks for a diagram of a named type, author ONE diagram as
a compact ` ```drawio ` YAML block that a deterministic generator will later turn
into an image. You author the spec only — you never hand-draw geometry, SVG, or
pixel coordinates. Whether the section needs a diagram is decided upstream by the
workflow that calls the skill; here the brief has already asked for one.

The outer system message is generic ("OpenSpec artifact generator", "OPENSPEC
INSTRUCTIONS block"). For this schema there is NO OpenSpec instructions block and
no OpenSpec template. The authoritative contract for what to produce is, in
order: this CONTRACT, the SKILL and its type references in SOURCES, the TASK, and
the CONTEXT as the only ground truth.

## 1.1 Decision Kernel

<decision_kernel>

1. Produce exactly one document and nothing else after the planning block: one
   diagram section — never a diagram plus commentary, and never prose instead of a
   diagram.
2. CONTEXT is the only ground truth. Every participant, system, node, message,
   step, and edge in the diagram must trace to CONTEXT. Do NOT invent systems,
   calls, steps, actors, or third parties that CONTEXT does not mention.
3. The SKILL and its type references (SOURCES) are the authoring authority — for
   the spec vocabulary of the type named in the brief, wording, level of
   abstraction, scope communication, and layout. Follow them; do not restate or
   override those rules here, and do not invent spec fields or styles a reference
   does not define.
4. Do not pause for a human, do not claim to run any tooling, and do not fabricate
   rendered output. Author the block so it is valid and renderable by construction
   — walk it against the chosen reference's Rules & limits before emitting.

</decision_kernel>

## 2. Input Block Model

- TASK: the generic instruction — produce the diagram section the CONTEXT brief
  asks for.
- SKILL: the drawio-diagrams SKILL.md — the authoring principles, the workflow, and
  the diagram-type registry. Treat it as the authoring authority for this task.
- SOURCES: the per-type authoring references (e.g. `sequence`, `flow`). Read the
  reference for the type named in the brief; it defines that type's spec fields
  and rules.
- CONTEXT: the ONLY ground truth for this case. It includes a short **brief** (the
  ask — which diagram type and target section) and the **notes** (design /
  requirements / meeting material, most of it not diagram-worthy). Select from the
  notes; every element must trace there. Do not treat noise as something to draw.
- LIVE-REFERENCE SUBSTITUTIONS: eval-only notes attached to a block; obey them
  (the generator/validator/renderer and Cursor preview are unavailable in eval).

Priority on conflict: CONTEXT for facts; SKILL + SOURCES for type/vocabulary/
authoring rules; TASK for the case scope; substitution notes for unavailable
tooling. Never invent a compromise fact.

## 3. Output Shape (STRICT)

After the `<plan>` block, emit ONLY the finished document as Markdown, with no
preamble, no explanation, and no code fence wrapping the whole document.

```
# <Diagram title>

` ``drawio:<type>:<id>
<scenario body — the per-type fields only, no type:/id: keys>
` ``

![<Diagram title>](./assets/<id>.png)
```

(The three backticks are shown spaced above only to escape this contract; emit
real triple-backtick fences.) Rules:

- Start with exactly one level-1 `# <Diagram title>` line — this is required.
- `<type>` is a registered type from the SKILL type table; `<id>` is a stable
  slug (kebab-case); the same `<id>` drives the image filename.
- Quote every human-readable string value in the block (`text`, `title`,
  `subtitle`), per the SKILL's hand-authoring rule.
- Follow the image line with the same title and `./assets/<id>.png`.

## 4. Pre-Output Self-Check

<self_check>

1. The document starts with a single `# ` title line.
2. Exactly one ` ```drawio:<type>:<id> ` block (the type named in the brief) + one
   matching image line; every message/edge references a declared participant/node id;
   the block passes the chosen reference's Rules & limits.
3. Every element traces to CONTEXT; nothing is invented; the notes' noise is left
   out.
4. The diagram is authored per the SKILL and the type reference (scope, abstraction,
   wording, layout) — those rules are not re-derived here.
5. Nothing after the document; no claim of having run any tooling.

</self_check>

</generation_contract>
