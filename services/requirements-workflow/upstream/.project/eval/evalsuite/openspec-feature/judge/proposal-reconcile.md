---
schema: openspec-feature
artifact: proposal-reconcile
context:
  # Namespaced by artifact: the flow overlay resolves this to
  # .context/review-session.proposal.<flow>.md — see config.states.review_session.
  - .context/review-session.proposal
  # The artifact mid-review, JUDGE view: the settled body with the two derived blocks
  # DROPPED. The judge derives the correct blocks from the body and scores the
  # CANDIDATE against that derivation, so it never needs the stale draft (that is the
  # generator's input). Dropping them removes the same-headed `## Executive Summary` /
  # `## Document Consumption Guide` the judge otherwise mistook for the candidate's
  # (false "dropped row" criticals on correct candidates). A copy-forward candidate is
  # still caught because it contradicts the body — see lib/resolve.mjs, composeFixture.
  - id: proposal-in-review
    from: proposal/final
    drop: [Executive Summary, Document Consumption Guide]
# What a correct re-derivation produces: the settled artifact's two derived blocks.
golden:
  from: proposal/final
  sections: [Executive Summary, Document Consumption Guide]
scoring:
  pass_threshold: 0.7
# Inject deterministic, harness-computed presence facts (the candidate's Document
# Consumption Guide row set diffed against the golden's) into the judge prompt. The LLM
# judge is unreliable at reading presence — it repeatedly flags a row the candidate DOES
# carry as "dropped" — so the harness parses the sets and hands the judge the ground-truth
# diff, closing the hallucinated-absence class without a model swap. Proposal carries no
# Integration Specification, so the facts block is the DCG row set only. See
# lib/structure.mjs deriveStructureFacts + lib/prompt.mjs.
derived_structure_facts: true
---

# Proposal Reconcile Judge Contract

<judge_contract version="1">

## 1. Role And Authority

You judge a partial re-emission of an OpenSpec `proposal.md`: the two blocks that are
DERIVED from the artifact's body — `## Executive Summary` (a reader map) and
`## Document Consumption Guide` (a MAP of the body's current `##` sections, placed
directly after the Executive Summary — one row per section EXCEPT the Executive Summary,
the guide itself, and the Proposal Gate, each with a `What it settles & why` phrase and a
`Key data` set). A proposal carries no Integration Specification, so there is no third
derived block (that is design's only); score exactly these two.

The situation being tested is the Per-Artifact Review Gate re-entry. The artifact was
drafted, a review session changed the body, and the two derived blocks were left as the
first pass wrote them. A correct candidate re-derives both blocks from the CHANGED body.
Carrying either block forward — verbatim or reworded — leaves the artifact contradicting
itself at the moment a human is asked to approve it.

The proposal-specific test is that the guide FOLLOWS THE DOCUMENT and each row's `Key
data` tracks the CURRENT body. The offer's section set is fixed, so a review typically
does NOT add a `##` section — it MOVES items INSIDE a section: a boundary the review makes
explicit so the Out of Scope & Boundaries owner set grows, or the review-gate session
recorded so the Proposal Source Inventory source set gains a row. The existing row's `Key
data` must move to match. Rendering the guide with the draft's stale key data — the
draft's Out-of-Scope owners, or the Proposal Source Inventory missing the review session —
is the staleness failure this stage exists to catch. (If a real review DID add a brand-new
`##` section, the guide must gain a NEW row for it; judge the row set against the body's
current sections either way.)

Authority order:

1. `CONTEXT` is the source of truth. It holds two things:
   - `proposal-in-review` — the artifact as it stands. Its BODY (Tracker, Proposal Source
     Inventory, Requirements Lock, Change Request Dispositions, Re-usability Legend,
     Delivery Breakdown, Out of Scope & Boundaries, External Dependencies & Access,
     Approach, Open Questions, Review & Approval) is CURRENT and authoritative. Its
     `## Executive Summary` and `## Document Consumption Guide` are DROPPED (not shown):
     the stale draft blocks carry NO authority and are deliberately withheld so nothing
     but the candidate's blocks can be scored. Derive the correct blocks from the BODY.
   - `review-session.proposal` — the session log that produced the body change. It
     explains WHY the body reads as it does; the body itself is what the blocks must
     match.
2. `SOURCES` and this `STANDARD` define the expected block semantics.
3. `GOLDEN EXPECTED` is a reference for coverage and shape. Do not require its exact
   wording where the candidate is faithful to the body.
4. `CANDIDATE` is the output being judged.

**The single most important rule: the ONLY `## Executive Summary` and `## Document
Consumption Guide` you score are the ones under `===== CANDIDATE`.** `proposal-in-review`
does not carry those blocks at all, and the blocks under `GOLDEN EXPECTED` are a
reference; never quote the golden as if it were the candidate's, and never report a value
as "missing" or "dropped" from the candidate unless you can quote that value's absence
from a line that appears AFTER the `CANDIDATE` delimiter.

A `DERIVED STRUCTURE FACTS` block accompanies the CANDIDATE. It is computed
DETERMINISTICALLY by the harness (not by a model) by parsing the candidate's and the
golden's Document Consumption Guide row set. It is AUTHORITATIVE for PRESENCE: a row it
lists under the candidate is PRESENT and must NEVER be recorded as missing/dropped; only a
row it names as "in GOLDEN but NOT in candidate" is a genuine drop eligible for §4.1. Do
not re-derive presence by eye against this block — read staleness, key-data fidelity, and
handoff quality ON TOP of these facts.

## 2. What Changed In The Body

You are NOT given a list of the changes. Derive it, because the whole point of the task is
that the blocks must follow the body wherever it went.

The session log names what the reviewer decided; the body is where those decisions landed.
Work in that order:

1. Read `review-session.proposal` and list each decision it records — a boundary made
   explicit, the review session recorded as a source, and so on.
2. For each one, find where it landed in the body, and note which Document Consumption
   Guide row projects that section — and whether it is an existing row whose `Key data`
   moves (the review changed items inside a section) or, in the rarer case, a row that
   must be ADDED (the review introduced a whole `##` section).
3. Read the body's structure and tables in full anyway. The DCG row SET is derived from
   the body's current `##` sections, so FIRST enumerate them in document order and drop
   the three that never map (Executive Summary, the guide itself, Proposal Gate) — that
   enumeration IS the correct row set, whatever the draft had. Then read each remaining
   section's exported identifiers/sets for its `Key data`.

The number of changes, which sections they touch, and whether they add a row or move a
row's key data are all case-specific. Never assume a change is present because a previous
case had it, and never assume a section is unchanged because the session did not mention
it.

Sections and rows the body does not move keep their values. Changing one is fabrication,
not refreshing.

## 3. Facts To Extract Before Judging

<extraction_checklist>

1. From the body of `proposal-in-review`, derive the CORRECT Document Consumption Guide
   yourself, before looking at the candidate's table, in two steps:
   - THE ROW SET: enumerate the body's `##` sections in document order; the correct rows
     are all of them EXCEPT the Executive Summary, the Document Consumption Guide, and the
     Proposal Gate. A section the body does not carry is NOT in the set.
   - EACH ROW's KEY DATA, read from the named section: Tracker → the tracker id; Proposal
     Source Inventory → the input source set (a review-gate session is a source row when
     one ran); Requirements Lock → the in-scope WBS set and the out-of-scope WBS set (by
     disposition, never the RQ id); Change Request Dispositions → the `CR#` set with
     status (`None` for a genuinely empty table); Re-usability Legend → the marker set;
     Delivery Breakdown → the slice set `S1…Sn` (one per in-scope WBS); Out of Scope &
     Boundaries → the set of items no slice delivers, each with its owner; External
     Dependencies & Access → the dependency set, each with the slice it blocks; Approach →
     the FSD/ISD working-copy target; Open Questions → the proposal-stage `Q#` set (`None`
     when the offer raised none — discovery/design `Q#` stay upstream); Review & Approval →
     the approval state. An empty set is `None`.
2. List the candidate's rows and their key data; compare its ROW SET against your derived
   set (a missing section-row or an extra one is drift) and each row's key data against
   your derived value. Record every difference as drift.
3. Read the candidate's Executive Summary and list each factual claim it makes about what
   the proposal settles, what it hands downstream, what stays open, counts, and readiness.
4. For each such claim, mark it AGREES WITH BODY, CONTRADICTS BODY, or UNSUPPORTED
   (asserts something no section of the body carries).
5. Note specifically whether any change on the list you derived in §2 is missing from the
   candidate's blocks — most importantly whether a review-changed set is stale (the Out of
   Scope & Boundaries owner set still the draft's, or the Proposal Source Inventory row
   missing the review session) — and whether any claim in the Executive Summary still
   describes the pre-session state the body no longer supports (a contradiction). The stale
   draft is not shown to you; judge staleness by contradiction with the body, not by
   comparison to a prior block.
6. Record whether the candidate emitted exactly the two required blocks — the Executive
   Summary and the Document Consumption Guide — and whether the guide table uses the
   `Section | What it settles & why it matters downstream | Key data` columns.

</extraction_checklist>

## 4. Severity Policy

### 4.1 Route To `critical_findings`

- A statement in either derived block that CONTRADICTS the current body. This is the
  staleness failure, and it does not depend on wording: reproducing the stale text
  verbatim and paraphrasing it into new words are the SAME defect, because the test is
  agreement with the body, not similarity to the input. The recurring shapes — a Document
  Consumption Guide whose ROW SET is the draft's rather than the body's (a review-added
  `##` section with no row, or a row for a section the body no longer carries), a row whose
  `Key data` set the body no longer supports (a stale Out-of-Scope owner set, a Proposal
  Source Inventory set missing the review session, a stale slice or WBS set), an Executive
  Summary count that does NOT equal the current body count (e.g. "eight slices" when the
  body carries nine), or a passed-forward item the body decided differently.
- A fabricated fact: an identifier (`S#`, `CR#`, `Q#`, WBS code), owner, dependency,
  section, count, or slice the body does not support — including a change the candidate
  invents to look refreshed.
  **Before recording a fabrication finding, quote the body and show that NO section
  supports the value.** An `S#`, `CR#`, `Q#`, WBS code, owner, or dependency that appears
  anywhere in the body is grounded, whether or not the review session touched it; carrying
  a row or member the review never moved is correct projection, not invention.
- A count in the Executive Summary is checked against the body: the reference is the
  CURRENT number in the body (of slices `S#`, or in-scope WBS). A count that EQUALS the
  current body number is CORRECT and can NEVER be a staleness finding. Only a count that
  does NOT equal the current body number is a contradiction. Compare a count ONLY against
  the body's number for the SAME subject (a slice count against `S#`, an in-scope count
  against in-scope WBS); a count that matches its own subject is correct even if it
  differs from some OTHER subject's number. **There is NO single canonical "out of scope"
  set** — it spans the Requirements Lock out-of-scope WBS, the declined Change Request,
  and the Out of Scope & Boundaries narrative bullets — so never manufacture an
  out-of-scope total and route a candidate's out-of-scope count against it; an
  out-of-scope count is a `warnings` nuance at most, never a staleness critical.
- A Document Consumption Guide row's `Key data` is judged ONLY against the section that row
  names. A value the body places in a DIFFERENT section is not "missing" from this row and
  is never a critical here; look for it in the row for the section that owns it, and judge
  THIS row against its own section's exported set alone.
- **The Requirements Lock row and the out-of-scope registers.** The Requirements Lock
  row's `Key data` is the BARE WBS set from the LOCK TABLE only — its in-scope WBS and its
  out-of-scope WBS by disposition. The declined Change Request (`CR1`) belongs to the
  Change Request Dispositions row, and an Out of Scope & Boundaries narrative boundary
  (e.g. store-record authoring) belongs to the Out of Scope & Boundaries row; NEITHER is
  "missing" from the Requirements Lock row, and demanding them there is a judge error, not
  a drop. A lock member written as its RQ id (`RQ-708`) or decorated with its owner
  (`SLS.INV → inventory workstream`) is the SAME grounded set member in a different FORMAT
  — the WBS↔RQ map is 1:1 and both appear in the body — so route it to `warnings` (format
  drift), NEVER to `no_fabrication` or `staleness_detection` as a critical.
- **Single-value cells (Approach, Review & Approval).** A row whose section exports ONE
  value — Approach → the FSD/ISD working-copy target; Review & Approval → the approval
  state — is judged like a set of size one. Reading `None` (or leaving the value out)
  where the body carries that value is a DROPPED value and is `critical_findings`: the
  handoff pointer is gone. But a value that RENDERS the body's state in different words is
  faithful, not stale — `Pending` and `Not approved` both carry the body's `Approved: no` —
  so route that wording difference to `warnings`, never here. The test is the same as every
  other cell: `None`/absent where the body has a value = dropped (critical);
  present-but-reworded = faithful (warning).

**Wording is not a finding.** The golden's phrasing has no authority (§1). A value that
conveys what the body says in different words is correct: an identifier with the status the
body gives it carries the same decision as a longer form. Never route a missing
parenthetical, a shorter status, or a different-but-faithful phrasing to
`critical_findings`; at most it is a `warnings` nuance when real information the body
carries is lost.

**An omission is not a contradiction — an incomplete SET or ROW SET is.** The Document
Consumption Guide's rows project the body's sections and each row's key data projects a
full set, so DROPPING a section-row the body carries or a set member the body carries (a
review-added Out-of-Scope owner, the Proposal Source Inventory's review-session row) makes
the guide wrong and stays `critical_findings`. The Executive Summary is prose, not a set:
it contradicts the body only when it ASSERTS a value, count, or status the body denies.
**Before routing an Executive Summary staleness critical, QUOTE the exact sentence that
asserts the stale value and name the body it denies.** If the only thing you can say is
that the summary "does not mention", "omits", or "fails to reflect" the change, that is
NOT a contradiction and NOT a staleness critical: it is a `change_reflection` deduction and
a `warnings` note. An Executive Summary that still describes the pre-session state by
SILENCE (it never mentions the change) is refreshed enough to be contradiction-free; only
one that describes it by ASSERTION (it states an old count, an old owner set) is stale.
Naming an added item as added ("store-record authoring is out of scope") is agreement with
the body, never route it here.

**Route each finding once.** A contradiction is a critical; a non-contradicting omission or
wording nuance is a warning. Never record the same issue in both lists.

When this route is used, cap overall `score` at `0.5`.

### 4.2 Route To `warnings`

- A change from §2 that is reflected in one block but not the other (e.g. the Document
  Consumption Guide moves the Out of Scope & Boundaries row's key data but the Executive
  Summary never narrates the new boundary, while its counts still agree).
- A Document Consumption Guide row that is merged, renamed, reordered, or left without
  `None` for an empty set, while the row SET and the remaining rows are correct.
- A `Key data` cell written as prose where identifiers and sets belong, or a `What it
  settles & why` phrase that is vague but not contradicted by the body.
- An Executive Summary that re-lists table rows (a slice roll-call, an owner-by-owner
  walkthrough) instead of describing the offer by subject and count.
- Wording that is imprecise or awkward but not contradicted by the body.
- Extra material beyond the two blocks (a title, another section, a preamble, a description
  of what was changed) while both required blocks are present and correct.

Warnings reduce dimension scores; they do not by themselves force `pass = false`.

## 5. Rubric Dimensions

Score every dimension from `0.0` to `1.0`. Never use `null`, `n/a`, or omit one.

- `staleness_detection`: no claim in either derived block survives that the current body
  contradicts. Score `1.0` when every statement agrees with the body — including the
  Document Consumption Guide row set matching the body's current sections and each row's
  key data matching the body. Score `0.0` when the candidate reproduces the stale Executive
  Summary or the stale Document Consumption Guide — verbatim OR reworded — such that its
  claims or its key-data sets still describe the pre-session artifact (the classic miss: the
  Out of Scope & Boundaries row missing the review-added owner, or the Proposal Source
  Inventory row missing the review session). Judge by AGREEMENT WITH THE BODY, never by
  textual distance from the input.
- `change_reflection`: every change in §2 that the body carries is visible in the block(s)
  it belongs to — the review-changed set has its current members in the guide row AND, when
  the reader map narrates that subject, is named by subject in the Executive Summary.
  Deduct per missed change; a candidate that moves the guide row but never reflects it in
  the summary scores mid-range, not high.
- `handoff_fidelity`: the Document Consumption Guide maps the body's CURRENT `##` sections —
  one row per section, in document order, excluding only the Executive Summary, the guide
  itself, and the Proposal Gate — with `None` for empty sets, `Key data` as
  identifiers/sets, and grounded `What it settles & why` phrases, and every row's key data
  equals the value you derived from the body in §3.1, INCLUDING the rows that did not
  change.
- `no_fabrication`: no invented `S#`, `CR#`, `Q#`, WBS code, owner, dependency, section,
  count, or slice. A change the session did not make and the body does not show is
  fabrication even when it sounds plausible.
- `reader_map_discipline`: the Executive Summary orients a reader — what the offer settles,
  what it hands downstream, what stays open — using subjects and counts rather than
  reproducing table rows, and it does not drift into the identifier inventory that belongs
  to the Document Consumption Guide. Naming a register or an occasional identifier as a
  pointer is fine; walking the rows is not.
- `structure_conformance`: exactly the two required blocks — `## Executive Summary` and
  `## Document Consumption Guide` (the latter with the `Section | What it settles & why it
  matters downstream | Key data` columns). Their presence and columns are also checked by a
  deterministic harness gate against the golden; score here what the gate leaves — extra
  sections, a preamble, a wrapper fence, or commentary.

Suggested scoring guide:

- `1.0`: fully re-derived and grounded.
- `0.75`: minor omission or weak wording, nothing contradicted.
- `0.5`: partially refreshed — some changes caught, others missed.
- `0.0`: stale, contradicted, or fabricated.

Compute `score` as the mean of the six dimensions, then cap it at `0.5` if
`critical_findings` is non-empty. Set `pass = true` only when `score >= threshold` and
`critical_findings` is empty.

Consistency check before returning JSON:

- If `critical_findings` is empty and `score >= threshold`, `pass` MUST be `true`.
- If `critical_findings` is non-empty, `score` MUST be `<= 0.5` and `pass` MUST be `false`.
- A verdict with any `critical_findings` and `score > 0.5` is invalid. Fix it.

## 6. Output Contract

The OUTPUT PROCEDURE in the system message controls packaging (the `<reasoning>` routing
pass, then the fenced JSON verdict). For this artifact the routing pass must:

1. State the Document Consumption Guide row SET you derived from the BODY's `##` sections
   in §3.1, and each row's key data.
2. Per-row key-data proof, then critical-admission ledger. For EACH Document Consumption
   Guide row, write ONE line: `<section> | BODY: <the value/set the named section exports,
   or "empty→None"> | CAND: "<the candidate cell, quoted verbatim>" | <verdict>`. Derive
   the verdict ONLY by comparing BODY against the QUOTED CAND text:
   - CAND conveys the body's value/set (any wording, format, or order) → `no_breach`.
   - CAND reads `None`/empty where BODY has a value or a non-empty set → `DROPPED-VALUE`
     (critical) — this covers the single-value cells too (Approach, Review & Approval):
     `None` there drops the working-copy target / approval state.
   - CAND omits a member BODY's set carries AND that member is absent from the quoted CAND
     text → `DROPPED-MEMBER` (critical). A MEMBER is the bare IDENTIFIER the set enumerates
     — an `S#`, `CR#`, `Q#`, WBS code, owner, dependency, slice, or source name. A
     parenthetical or trailing QUALIFIER on a present identifier — a status, a disposition,
     an owner, a slice-blocking tag, a description, or a date — is NOT a member: if every
     identifier BODY's set carries appears in the quoted CAND text, the row is `no_breach`
     (or at most `FORMAT/TERSE` for a stripped qualifier), NEVER `DROPPED-MEMBER`, however
     many qualifiers are missing. This holds even when the parenthetical is the row's ONLY
     human-readable content: for an ID-plus-description member — a change request
     `CR1 (declined)`, an open question `Q1 (<topic>) — <owner>`, a dependency
     `<dep> (unconfirmed, Q#)` — the MEMBER is the bare id and the status/description/owner
     is the qualifier, so a PRESENT id whose description was stripped is `FORMAT/TERSE`,
     never `DROPPED-MEMBER` (a genuinely ABSENT id still fires). Route EVERY
     ID-plus-description row the SAME way in one pass: grading one row's stripped description
     critical while a sibling row's is a warning is the inconsistency this rule forbids. If
     the member IS present in the quoted text (even reworded or grouped), it is NOT dropped —
     do NOT fire.
   - CAND states a value/member absent from the ENTIRE body → `FABRICATION` (critical).
   - CAND renders the body's value in different words/format, or terser-but-faithful
     (`Pending`/`Not approved` for `Approved: no`; a WBS for its RQ; a present identifier
     with its parenthetical qualifier stripped — a missing annotation, status, owner, slice
     tag, description, date, or count) → `FORMAT/TERSE` (warning), never critical.
   Only `DROPPED-VALUE`, `DROPPED-MEMBER`, and `FABRICATION` lines may enter
   `critical_findings`, and each MUST carry its quoted CAND text as evidence. For the
   Executive Summary (prose, not a set) a finding is admissible to `critical_findings` ONLY
   as `ASSERTS-DENIED` — quote the exact candidate sentence that STATES a value the body
   denies for the SAME subject, plus the body's value for that subject. An Executive
   Summary that "does not mention / omits / fails to reflect" a change is ALWAYS a
   `warnings` / `change_reflection` line, never a critical. Judge each row's key data ONLY
   against the section its row names.
3. Confirm the pass/score rule: `critical_findings` non-empty forces `score <= 0.5` and
   `pass = false`.

Verdict shape:

```json
{
  "pass": true,
  "score": 1.0,
  "rubric_scores": {
    "staleness_detection": { "score": 1.0, "reason": "..." },
    "change_reflection": { "score": 1.0, "reason": "..." },
    "handoff_fidelity": { "score": 1.0, "reason": "..." },
    "no_fabrication": { "score": 1.0, "reason": "..." },
    "reader_map_discipline": { "score": 1.0, "reason": "..." },
    "structure_conformance": { "score": 1.0, "reason": "..." }
  },
  "critical_findings": [
    { "issue": "...", "evidence": "..." }
  ],
  "warnings": [
    { "issue": "...", "evidence": "..." }
  ],
  "missing_evidence": [
    "..."
  ],
  "recommendation": "..."
}
```

Reasons and evidence must quote the candidate and the body row that contradicts or
supports it. Keep the JSON valid.

</judge_contract>
