# Artifact Review & Approval Gate

The procedure the `openspec-workflow` rule delegates to: how to stop after an artifact, walk its open
questions, record a human approval, and — for the artifacts that carry them — re-derive its derived
blocks from the changed body. Read this when you reach a review gate — the rule carries only the pointer,
not the steps.

Applies after the agent creates or updates ANY authored artifact (`discovery`, `design`, `proposal`,
`grooming`, `specs`, `fsd`, `isd`, `technical-design`, `tasks`). (`stories.md` is not in this list: it
is not an authored artifact but a deterministic script projection produced alongside the FSD at
`/opsx:propose`, re-synced at `/opsx:propose-approved`.)

## Per-Artifact Review Gate (stop after every artifact)

Every artifact is human-reviewed before the next one is generated. After creating or updating one, STOP
and run this gate — never chain straight into the next artifact.

1. **Stop and summarize.** Post a short summary of what was produced (a few bullets, not a rewrite) and
   list the artifact's open questions / decisions that need a human. Do not dump the whole file.
   For **technical-design** only, precede the summary with a brief chat-only architecture & decomposition
   walkthrough — a review aid, NEVER written into the artifact and OUTSIDE the eval contour: explain the
   shape first plainly, then by an analogy to building a car on an assembly line. Convey the PRINCIPLE,
   not a part-by-part map — each delivery phase rolls off the line as a drivable car (a vertical,
   acceptance-closing slice, not a horizontal layer), cross-cutting concerns are built into every stage
   rather than bolted on as their own phase, and any part the sources leave deferred is fitted "per spec /
   pending decision", never improvised. Keep it short; it never gates approval.
2. **Offer the choice.** Use the **AskUserQuestion tool** for THIS routing choice only. Offer
   **Walk the questions** ONLY when the artifact actually has open questions / undecided items in
   this pass:
   - **Approve** — record approval and move to the next phase.
   - **Walk the questions (recommended)** — go through the open questions together before approving.
     *Offer this option only when open questions exist.*
   - **Request changes** — the human wants edits (or to pause); apply them, then re-run this gate.

   When the artifact has NO open questions, DROP "Walk the questions" and offer only **Approve /
   Request changes** — never manufacture a walkthrough by re-litigating already-settled decisions.
   If a walkthrough nonetheless feels needed on an "empty" design, the real miss is an unraised
   design-stage (b) architecture question — go raise it (schema `design` PIPELINE 7a), don't re-walk
   resolved discovery ones.
3. **Walk the questions.** If chosen, handle open questions **one at a time, each in its own chat
   message** (NOT via AskUserQuestion — that tool is only for the step-2 routing choice). For each: state
   what it is and why it matters, give the concrete options with a one-line implication / trade-off each,
   and add the agent's recommendation clearly labelled as a recommendation, not a decision. Wait for the
   human answer, write it back into the artifact, then take the next question. When done, **re-derive the
   artifact's derived blocks** if it has them (see Routing below), then re-offer step 2.
4. **Request changes.** Apply the requested edits, **re-derive the artifact's derived blocks** if it has
   them (see Routing below), then re-run this gate from step 1.
5. **Record approval.** On **Approve**, ask who is approving (a real name/role) and fill the artifact's
   approval marker with it. The agent MUST NOT check the box itself or use `Agent`/`LLM`/`Cursor` as
   approver. Only then generate the next artifact.

## Routing — which procedure to follow at the gate

Some artifacts carry derived blocks (a reader map, a Document Consumption Guide, an Integration
Specification overview) or an upstream-actualization pass; others do not. Before you re-present an
artifact, follow its artifact-specific procedure — never from memory:

| Artifact | Follow before re-presenting |
|---|---|
| discovery, design, proposal | [derived-block-rederivation.md](mdc:.cursor/rules/workflow/openspec-workflow/references/derived-block-rederivation.md) — rebuild the reader map + Document Consumption Guide (plus the Integration Specification `### Overview` for an integration design) from the CHANGED body |
| grooming | its own grooming procedure (separate reference); interim — the generic gate above only (grooming's Executive Summary is written last per its template) |
| fsd, isd | [fsd-isd-upstream-actualization.md](mdc:.cursor/rules/workflow/openspec-workflow/references/fsd-isd-upstream-actualization.md) — once the doc is settled, actualize the upstream (design / proposal / discovery / specs ← FSD/ISD); see the note below |
| specs, technical-design, tasks | no derived blocks — the generic gate above only |

## Approval marker per artifact

The deterministic thing the command reads before proceeding:

| Artifact | Approval marker |
|---|---|
| discovery, design, proposal, technical-design | `## Review & Approval` — `Approved` box checked + real `Approved by` |
| grooming | the existing `## Grooming Approval Gate` (all boxes checked, every approver a named human) |
| tasks | `## Review & Approval` — `Approved for implementation: yes` + real `Approved by` (no checkbox, by parser design) |
| specs, fsd, isd | no in-file box — these leave the repo verbatim (specs merge into the timeless baseline; fsd/isd publish to Confluence, where an internal box would leak), so approval is recorded at this gate's stop, not in the file |

At the **FSD/ISD** review gate — and again at `/opsx:propose-submit` before the artifacts are committed and
published — additionally check whether the settled FSD/ISD tailored or hardened anything the upstream
asserts; if so, run the FSD/ISD upstream-actualization pass (design / proposal / discovery / specs ←
FSD/ISD) before the document leaves the gate — see
[fsd-isd-upstream-actualization](mdc:.cursor/rules/workflow/openspec-workflow/references/fsd-isd-upstream-actualization.md).
It actualizes each upstream artifact within its ownership and is outside the eval contour.

Never self-approve and never infer approval from source documents, meeting notes, or the agent's own
recommendation. If the human is unavailable, stop and state exactly what is unapproved.
