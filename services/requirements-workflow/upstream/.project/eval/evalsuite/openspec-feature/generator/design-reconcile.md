---
schema: openspec-feature
artifact: design-reconcile
rules:
  - ../../../../../rules/workflow/openspec-workflow/references/derived-block-rederivation.md
context:
  # GENERATOR view of the artifact mid-review. Same two stored states as the judge,
  # but a different projection: the settled body carries the stale first-pass
  # `## Solution Overview` (spliced from the draft) and NO `## Document Consumption
  # Guide` — the candidate must re-derive the reader map and ADD the guide, mapping
  # the body's CURRENT `##` sections (including any the review added). For an
  # integration-flow design it ALSO carries the stale first-pass Integration
  # Specification `### Overview` (spliced from the draft), so its `Read in order:`
  # signpost still names the draft's subsections and must be re-derived to cover any
  # the review added. That block does not exist in a functional-flow design, so the
  # `?` selector makes the splice a no-op there. The judge sees a different projection
  # (all derived blocks dropped) — see judge/design-reconcile.md.
  - .context/review-session.design
  - id: design-in-review
    from: design/final
    override:
      from: design/initial
      sections: [Solution Overview, Integration Specification > Overview?]
    drop: [Document Consumption Guide]
---

# Scenario — Review-gate re-entry: re-derive design's derived blocks

`design.md` was drafted, the Per-Artifact Review Gate ran, and the reviewer chose
**Walk the questions**. The session is over: the open questions were walked, and the
reviewer asked for one or more design-internal changes to the body — recorded in the
session log. The reviewer has now asked to see the artifact again before approving.

The CONTEXT block carries both halves of that situation:

- the session log — the reviewer's decisions and the changes asked for;
- the artifact as it stands right now. Its BODY already contains those changes. Its
  `## Solution Overview`, however, is still the text from the first pass — never
  re-derived, describing a version of the file that no longer exists; its
  `## Document Consumption Guide` has not been emitted yet; and, for an integration
  design, the Integration Specification's own `### Overview` is likewise the stale
  first-pass reader map, its `Read in order:` signpost still naming the draft's
  subsections.

Your job is step 4 of the review gate: **re-derive the Solution Overview, emit the
Document Consumption Guide, and — when the design carries an Integration Specification
— re-derive that section's `### Overview` reader map, all from the CHANGED body** so
the artifact is internally consistent when it goes back to the reviewer.

Work from the body, not from the old block. Read the current Affected Capabilities,
Design Source Inventory, Gap Analysis, Architecture Decisions, Base Configuration &
Cross-Cutting, Integration Specification, Risks / Mitigations, Dependencies, Open
Questions, and any section the review ADDED (e.g. a `## Planning Deferrals` block),
and write what they say NOW. The previous text of either block has no authority: a
statement it makes that the body no longer supports is wrong, and rewording it
instead of correcting it leaves it just as wrong. Equally, do not invent a change the
body does not carry — a section or row the session never touched keeps its value.

Follow the re-derivation reference in RULES (`derived-block-rederivation.md`) for
what each block must be — including the
per-section `Key data` authority for the guide: the Solution Overview is
a READER MAP (what the design settles, why it is shaped that way, what constrains
downstream, what is passed forward — by subject and count, no re-listing of rows), and
the Document Consumption Guide MAPS THE BODY'S CURRENT STRUCTURE — one row per `##`
section the body carries, in document order, EXCLUDING only Solution Overview, the
guide itself, and Design Gate. A section the review added earns a NEW row in place; a
`Key data` cell carries identifiers/sets and `None` for an empty set. The row set
follows the document — do not carry a fixed list.

The Integration Specification `### Overview`, when present, is a LOCAL reader map for
that one section: one or two sentences leading with the contract shape (and, when the
section carries more than one service or contract, naming each and how the section
splits by them), then a `Read in order:` signpost that lists every `###` subsection
the section now carries, in reading order, grouped by service/cluster when the section
is large. Its subsection list follows the section — if the review added a subsection,
the signpost must name it; it must not name one the body no longer carries.

## Output scope — this case produces the derived blocks only, not a whole artifact

This is a partial re-emission, and it overrides the generation contract's
"one finished document" default: the TASK defines the output scope.

Output exactly the derived blocks, in this order, with these headings verbatim, and
nothing else. A functional-flow design has no Integration
Specification, so it produces the first two blocks only; an integration-flow design
adds the third:

```text
## Solution Overview

<the re-derived reader map>

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
<one row per current ## section, in document order, excluding Solution Overview, this guide, and Design Gate>

### Overview

<the re-derived Integration Specification reader map — ONLY when the design carries a `## Integration Specification`>
```

The Document Consumption Guide table MUST use those three columns with that exact
header verbatim — `Section | What it settles & why it matters downstream | Key
data` — and no others; the structure gate checks it against the golden.

Emit the `### Overview` block (at its `###` level) only for an integration design, as
the Integration Specification's local reader map; omit it entirely for a functional
design. Do NOT wrap it in a `## Integration Specification` heading and do NOT re-emit
any other Integration Specification subsection — just the `### Overview` itself.

Do NOT re-emit the rest of the artifact — no title, no Architecture Decisions, no
Dependencies, no Planning Deferrals table, no Design Gate, no Review & Approval, no
commentary about what you changed, and no diff. Just the derived blocks as they should
now read in the file.
