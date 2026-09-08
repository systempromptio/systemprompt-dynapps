---
schema: openspec-feature
artifact: discovery-reconcile
schema_artifact: feature/discovery
rules:
  - ../../../../../rules/workflow/openspec-workflow/references/derived-block-rederivation.md
context:
  # GENERATOR view of the artifact mid-review. Same two stored states as the judge,
  # but a different projection: the settled body carries the stale first-pass
  # `## Executive Summary` (spliced from the draft) and NO `## Document Consumption
  # Guide` — the candidate must re-derive the summary and ADD the guide. The judge
  # sees a different projection (both derived blocks dropped) — see
  # judge/discovery-reconcile.md.
  - .context/review-session.discovery
  - id: discovery-in-review
    from: discovery/final
    override:
      from: discovery/initial
      sections: [Executive Summary]
    drop: [Document Consumption Guide]
---

# Scenario — Review-gate re-entry: re-derive discovery's derived blocks

`discovery.md` was drafted, the Per-Artifact Review Gate ran, and the reviewer
chose **Walk the questions**. The session is over: answers were written back, a
client decision landed, a research finding was recorded, and one disposition was
corrected. The reviewer has now asked to see the artifact again before approving.

The CONTEXT block carries both halves of that situation:

- the session log — the reviewer's answers, the agent's research finding, and the
  correction the reviewer asked for;
- the artifact as it stands right now. Its BODY already contains every one of
  those changes. Its `## Executive Summary`, however, is still the text from the
  first pass — never re-derived, describing a version of the file that no longer
  exists — and its `## Document Consumption Guide` has not been emitted yet.

Your job is step 4 of the review gate: **re-derive the Executive Summary and emit
the Document Consumption Guide from the CHANGED body** so the artifact is internally
consistent when it goes back to the reviewer.

Work from the body, not from the old blocks. Read the current Requirements Trace,
Affected Capabilities, Dependencies, Change Request Candidates, Open Questions,
Cross-cutting Requirements, and Delivery Scope & Targets, and write what they say
NOW. The previous text of either block has no authority: a statement it makes that
the body no longer supports is wrong, and rewording that statement instead of
correcting it leaves it just as wrong. Equally, do not invent a change the body
does not carry — a row the session never touched keeps its value.

Follow the OPENSPEC INSTRUCTIONS for what each block must be: the Executive
Summary is a reader map (subjects and counts, no re-listing of table rows), and the
Document Consumption Guide is the fixed per-section projection of the body sections
it names — one row per consumed section with a short `What it settles & why` phrase
and a `Key data` set, identifiers and sets as key data and `None` for an empty set.

## Output scope — this case produces TWO sections, not a whole artifact

This is a partial re-emission, and it overrides the generation contract's
"one finished document" default: the TASK defines the output scope.

Output exactly these two sections, in this order, with these headings verbatim
from the discovery template, and nothing else:

```text
## Executive Summary

<the re-derived reader map>

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
<one row per consumed body section, in template order>
```

The Document Consumption Guide table MUST use those three columns with that exact
header verbatim — `Section | What it settles & why it matters downstream | Key
data` — and no others; the structure gate checks it against the golden.

Do NOT re-emit the rest of the artifact — no title, no Requirements Trace, no
Dependencies, no Discovery Gate, no Review & Approval, no commentary about what
you changed, and no diff. Just the two sections as they should now read in the
file.
