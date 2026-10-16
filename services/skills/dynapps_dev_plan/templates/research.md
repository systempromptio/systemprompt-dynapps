---
research_for: "<the objective, one line>"
created: "YYYY-MM-DD"
stack: "<what was detected, with the evidence that proved it>"
questions_open: 0
---

<!-- Everything in an HTML comment is guidance for you and must not survive into
     the file you write. Keep the headings; replace the rest.

     questions_open must equal the OPEN rows in the table below: `new` reads it
     and will not plan over an open question. -->

# Research — <slug>

<!-- Reconnaissance (the default) fills only `Pieces involved`, `What already exists` and
     `Non-negotiable constraints`, under ~60 lines, questions_open: 0 unless something blocks.
     The Questions sections are for `research --deep`. -->

## Pieces involved

| Piece | Where | Role in the objective |
|---|---|---|
| `<module / model / file / test>` | `path:line` | ... |

<!-- How the tests of this area run: the future verify. The access the work needs: the future
     `## Requires`. -->

## Questions

| # | Question | Status |
|---|---|---|
| Q1 | ... | RESOLVED |
| Q2 | ... | OPEN |

<!-- RESOLVED or OPEN, nothing else. One row per question, at most five. -->

## Q1 — <short title>

**Decision:** what we do.

**Reason:** why, in one or two sentences.

**Rejected:** the alternative, and what makes it worse *here*.

<!-- No rejected alternative means this is not a decision. Replace the three
     lines above with a single `**Assumption:**` so the reader sees it is
     unproven, rather than dressing a guess as a decision. -->

**Sources:** un `path:line`, or a URL. One per claim.

**Impact on the plan:** which phases this adds, removes or reshapes.

<!-- That last line is what makes the research worth its cost. Research that
     changes nothing about the plan was not needed. -->

## Q2 — <short title>  [OPEN]

<!-- State the two readings that are incompatible, and what kind of answer would
     settle it: technical, business, or the client's. Never guess and mark it
     RESOLVED. -->

## What already exists

<!-- Findings about the codebase, each with path:line. Things already installed,
     patterns to copy, fields declared but unused, promises a docstring makes
     that the code does not keep. This is what stops the plan from rebuilding
     something that is already there. -->

## Non-negotiable constraints

<!-- Rate limits, versions, code already in production, an API that takes 200
     records at a time. Each one has to land somewhere in the plan: a gotcha, a
     `## Requires`, or a phase. One that lands nowhere was not a constraint. -->

## Not investigated

<!-- What was deliberately left out and why, including any front that could not
     be covered. Bounds the research and stops the next reader assuming it was
     covered. -->
