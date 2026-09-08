---
schema: openspec-feature
artifact: proposal-reconcile
rules:
  - ../../../../../rules/workflow/openspec-workflow/references/derived-block-rederivation.md
context:
  # GENERATOR view of the artifact mid-review. The settled body carries the CHANGED
  # body (the review's boundary clarification already landed) but its `## Executive
  # Summary` is still the STALE first-pass reader map (spliced from the draft) and it
  # has NO `## Document Consumption Guide` — the candidate must re-derive the reader
  # map and ADD the guide, mapping the body's CURRENT `##` sections. Proposal carries
  # no Integration Specification, so there is no third derived block (that is design's
  # only). The judge sees a different projection (both derived blocks dropped) — see
  # judge/proposal-reconcile.md.
  - .context/review-session.proposal
  - id: proposal-in-review
    from: proposal/final
    override:
      from: proposal/initial
      sections: [Executive Summary]
    drop: [Document Consumption Guide]
---

# Scenario — Review-gate re-entry: re-derive proposal's derived blocks

`proposal.md` was drafted, the Per-Artifact Review Gate ran, and — the offer carrying no
new proposal-stage open questions — the reviewer was offered **Approve / Request
changes** and chose **Request changes**. The session is over: the reviewer asked for one
proposal-internal change to the body, the change was applied, and the reviewer has now
asked to see the artifact again before approving.

The CONTEXT block carries both halves of that situation:

- the session log — the reviewer's decision and the change asked for;
- the artifact as it stands right now. Its BODY already contains that change. Its
  `## Executive Summary`, however, is still the reader map from the first pass — never
  re-derived, describing a version of the file that no longer exists; and its
  `## Document Consumption Guide` has not been emitted yet.

Your job is step 4 of the review gate: **re-derive the Executive Summary reader map and
emit the Document Consumption Guide, both from the CHANGED body**, so the artifact is
internally consistent when it goes back to the reviewer.

Work from the body, not from the old block. Read the current Tracker, Proposal Source
Inventory, Requirements Lock, Change Request Dispositions, Re-usability Legend, Delivery
Breakdown, Out of Scope & Boundaries, External Dependencies & Access, Approach, Open
Questions, Review & Approval, and any section the review touched, and write what they say
NOW. The previous text of the Executive Summary has no authority: a statement it makes
that the body no longer supports is wrong, and rewording it instead of correcting it
leaves it just as wrong. Equally, do not invent a change the body does not carry — a
section, row, or set member the session never touched keeps its value.

Follow the re-derivation reference in RULES (`derived-block-rederivation.md`) for what
each block must be — including the per-section
`Key data` authority for the guide: the Executive Summary is a READER MAP (what the offer
settles, what it hands downstream, what stays open — by subject and count, no re-listing
of rows), and the Document Consumption Guide MAPS THE BODY'S CURRENT STRUCTURE — one row
per `##` section the body carries, in document order, EXCLUDING only the Executive
Summary, the guide itself, and the Proposal Gate. A `Key data` cell carries
identifiers/sets and `None` for an empty set; a row whose section the review changed
(e.g. Out of Scope & Boundaries gaining a boundary owner, or the Proposal Source
Inventory gaining the review-gate session) must carry the CURRENT set. The row set
follows the document — do not carry a fixed list from the stale block.

Keep the two out-of-scope registers DISTINCT as you re-derive: the Requirements Lock
row's `Key data` is the BARE WBS set from the lock table (never an RQ id, never
owner-decorated — the owners live in the Out of Scope & Boundaries row); and the
Executive Summary names the out-of-scope OWNERS by subject WITHOUT a total count (there
is no single out-of-scope set — it spans the lock, the declined change request, and the
boundary bullets, so a count would be arbitrary).

## Output scope — this case produces the derived blocks only, not a whole artifact

This is a partial re-emission, and it overrides the generation contract's "one finished
document" default: the TASK defines the output scope.

Output exactly the two derived blocks, in this order, with these headings verbatim, and
nothing else:

```text
## Executive Summary

<the re-derived reader map>

## Document Consumption Guide

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
<one row per current ## section, in document order, excluding Executive Summary, this guide, and Proposal Gate>
```

The Document Consumption Guide table MUST use those three columns with that exact header
verbatim — `Section | What it settles & why it matters downstream | Key data` — and no
others; the structure gate checks it against the golden.

Do NOT re-emit the rest of the artifact — no title, no Tracker, no Requirements Lock, no
Delivery Breakdown, no Proposal Gate, no Review & Approval, no commentary about what you
changed, and no diff. Just the two derived blocks as they should now read in the file.
