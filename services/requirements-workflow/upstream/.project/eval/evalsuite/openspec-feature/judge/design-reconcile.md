---
schema: openspec-feature
artifact: design-reconcile
context:
  # Namespaced by artifact: the flow overlay resolves this to
  # .context/review-session.design.<flow>.md — see config.states.review_session.
  - .context/review-session.design
  # The artifact mid-review, JUDGE view: the settled body with the derived blocks
  # DROPPED. The judge derives the correct blocks from the body and scores the
  # CANDIDATE against that derivation, so it never needs the stale draft (that is
  # the generator's input). Dropping them removes the same-headed `## Solution
  # Overview` / `## Document Consumption Guide` (and, for an integration design, the
  # Integration Specification's `### Overview`) the judge otherwise mistook for the
  # candidate's (false "dropped row" criticals on correct candidates). The rest of
  # the Integration Specification stays, so the judge derives the correct `### Overview`
  # signpost from the subsections present. The `?` makes the nested drop a no-op for a
  # functional design, which has no Integration Specification `### Overview`. A
  # copy-forward candidate is still caught because it contradicts the body — see
  # lib/resolve.mjs, composeFixture.
  - id: design-in-review
    from: design/final
    drop: [Solution Overview, Document Consumption Guide, Integration Specification > Overview?]
# What a correct re-derivation produces: the settled artifact's derived blocks (the
# Integration Specification `### Overview` is included only for an integration design).
golden:
  from: design/final
  sections: [Solution Overview, Document Consumption Guide, Integration Specification > Overview?]
scoring:
  pass_threshold: 0.7
# Inject deterministic, harness-computed presence facts (the candidate's Document
# Consumption Guide row set and Integration Specification `### Overview` signpost,
# each diffed against the golden) into the judge prompt. The LLM judge is
# unreliable at reading presence — it repeatedly flags a row/subsection the
# candidate DOES carry as "dropped" — so the harness parses the sets and hands the
# judge the ground-truth diff, closing the hallucinated-absence class without a
# model swap. See lib/structure.mjs deriveStructureFacts + lib/prompt.mjs.
derived_structure_facts: true
---

# Design Reconcile Judge Contract

<judge_contract version="1">

## 1. Role And Authority

You judge a partial re-emission of an OpenSpec `design.md`: the blocks that are
DERIVED from the artifact's body — `## Solution Overview` (a reader map) and
`## Document Consumption Guide` (a MAP of the body's current `##` sections, placed
directly after the Solution Overview — one row per section EXCEPT Solution Overview,
the guide itself, and Design Gate, each with a `What it settles & why` phrase and a
`Key data` set).

For an integration design the candidate re-emits a THIRD derived block: the
Integration Specification's own `### Overview` — a LOCAL reader map for that one
section, one or two sentences leading with the contract shape (and, when the section
carries more than one service or contract, naming each and how it splits) followed by
a `Read in order:` signpost that names the section's `###` subsections in reading
order. A functional design has no Integration Specification, so it re-emits only the
first two blocks; score the third ONLY when the body carries a `## Integration
Specification` (and therefore the golden and candidate carry a `### Overview`).

The situation being tested is the Per-Artifact Review Gate re-entry. The artifact was
drafted, a review session changed the body, and the two derived blocks were left as
the first pass wrote them. A correct candidate re-derives both blocks from the CHANGED
body. Carrying either block forward — verbatim or reworded — leaves the artifact
contradicting itself at the moment a human is asked to approve it.

The design-specific test is that the guide FOLLOWS THE DOCUMENT and each row's `Key
data` tracks the CURRENT body: when the review changes items INSIDE a section — a new
Architecture Decision so the `D#` set grows, or newly-routed planning deferrals so the
Planning Deferrals `PD#` set moves from `None` to `PD1…` — the existing row's `Key data`
must move; and if a real review ADDS a brand-new `##` section, the guide must gain a NEW
row for it. Rendering the guide with the draft's row set or stale key data is the
staleness failure this stage exists to catch.

The same follows-the-document test applies to the Integration Specification `###
Overview` when the design has one: its `Read in order:` signpost is a projection of the
section's `###` subsections, so when the review ADDS a subsection inside the Integration
Specification, the signpost must gain it — and it must not name a subsection the section
no longer carries. A review-added `###` subsection does NOT add a Document Consumption
Guide row (the guide maps `##` sections, not `###`), so the ONLY derived block that must
move for it is the Integration Specification `### Overview`.

Authority order:

1. `CONTEXT` is the source of truth. It holds two things:
   - `design-in-review` — the artifact as it stands. Its BODY (Affected Capabilities,
     Design Source Inventory, Gap Analysis, Architecture Decisions, Base Configuration
     & Cross-Cutting, Integration Specification, Risks / Mitigations, Dependencies,
     Open Questions, Planning Deferrals) is
     CURRENT and authoritative. Its `## Solution Overview` and `## Document Consumption
     Guide` are DROPPED (not shown): the stale draft blocks carry NO authority and are
     deliberately withheld so nothing but the candidate's blocks can be scored. Derive
     the correct blocks from the BODY. For an integration design the Integration
     Specification's `### Overview` is dropped too, while the rest of that section
     (Integration Documentation, Endpoints, Data Mapping, Data Flow, and any subsection
     the review added) stays — derive the correct signpost from the subsections present.
   - `review-session.design` — the session log that produced the body changes. It
     explains WHY the body reads as it does; the body itself is what the blocks must
     match.
2. `SOURCES` and this `STANDARD` define the expected block semantics.
3. `GOLDEN EXPECTED` is a reference for coverage and shape. Do not require its exact
   wording where the candidate is faithful to the body.
4. `CANDIDATE` is the output being judged.

**The single most important rule: the ONLY `## Solution Overview`, `## Document
Consumption Guide`, and Integration Specification `### Overview` you score are the ones
under `===== CANDIDATE`.** `design-in-review` does not carry those blocks at all, and
the blocks under `GOLDEN EXPECTED` are a reference; never quote the golden as if it
were the candidate's, and never report a value as "missing" or "dropped" from the
candidate unless you can quote that value's absence from a line that appears AFTER the
`CANDIDATE` delimiter.

A `DERIVED STRUCTURE FACTS` block accompanies the CANDIDATE. It is computed
DETERMINISTICALLY by the harness (not by a model) by parsing the candidate's and
the golden's Document Consumption Guide row set and Integration Specification
`### Overview` signpost. It is AUTHORITATIVE for PRESENCE: a row or subsection it
lists under the candidate is PRESENT and must NEVER be recorded as missing/dropped;
only a row/subsection it names as "in GOLDEN but NOT in candidate" is a genuine drop
eligible for §4.1. Do not re-derive presence by eye against this block — read
staleness, key-data fidelity, and handoff quality ON TOP of these facts.

## 2. What Changed In The Body

You are NOT given a list of the changes. Derive it, because the whole point of the
task is that the blocks must follow the body wherever it went.

The session log names what the reviewer decided; the body is where those decisions
landed. Work in that order:

1. Read `review-session.design` and list each decision it records — a section added,
   an architecture decision recorded, an item routed to planning, a question walked.
2. For each one, find where it landed in the body, and note which Document Consumption
   Guide row projects that section — and whether it is a row that must be ADDED (the
   review introduced a whole `##` section) or an existing row whose `Key data` moves
   (the review changed items inside a section, e.g. the `D#` or `PD#` set).
3. Read the body's structure and tables in full anyway. The DCG row SET is derived
   from the body's current `##` sections, so FIRST enumerate them in document order and
   drop the three that never map (Solution Overview, the guide itself, Design Gate) —
   that enumeration IS the correct row set, whatever the draft had. Then read each
   remaining section's exported identifiers/sets for its `Key data`.

The number of changes, which sections they touch, and whether they add a row or move a
row's key data are all case-specific. Never assume a change is present because a
previous case had it, and never assume a section is unchanged because the session did
not mention it.

Sections and rows the body does not move keep their values. Changing one is
fabrication, not refreshing.

## 3. Facts To Extract Before Judging

<extraction_checklist>

1. From the body of `design-in-review`, derive the CORRECT Document Consumption Guide
   yourself, before looking at the candidate's table, in two steps:
   - THE ROW SET: enumerate the body's `##` sections in document order; the correct
     rows are all of them EXCEPT Solution Overview, Document Consumption Guide, and
     Design Gate. Planning Deferrals is always present and IS in this set (reading `None`
     when the design defers nothing); a section the body does not carry is NOT.
   - EACH ROW's KEY DATA, read from the named section: Affected Capabilities → owned
     capabilities with ADD/MODIFY/REMOVE; Design Source Inventory → the input source
     set; Gap Analysis → the `WBS → gap-class` set; Architecture Decisions → the `D#`
     set (a proposed `D#` notes its linked `Q#`); Base Configuration & Cross-Cutting →
     the concerns carried; Integration Specification → contract keys (endpoints +
     mapping fields for an API shape; the SDK interface for an SDK shape) or `N/A —
     functional change` (`None`); Risks / Mitigations → the risk labels; Dependencies →
     the consumed-not-owned set; Open Questions → the `Q#` set with status; Planning
     Deferrals → the `PD#` set; Review & Approval → the approval state. An empty set is
     `None`.
2. List the candidate's rows and their key data; compare its ROW SET against your
   derived set (a missing section-row or an extra one is drift) and each row's key data
   against your derived value. Record every difference as drift.
3. Read the candidate's Solution Overview and list each factual claim it makes about
   what the design settles, why it is shaped that way, what constrains downstream, what
   is passed forward, counts, and approval state.
4. For each such claim, mark it AGREES WITH BODY, CONTRADICTS BODY, or UNSUPPORTED
   (asserts something no section of the body carries).
5. Note specifically whether any change on the list you derived in §2 is missing from
   the candidate's blocks — most importantly whether a review-ADDED section has NO new
   Document Consumption Guide row, or a review-changed `D#`/`PD#` set is stale — and
   whether any claim in the Solution Overview still describes the pre-session state the
   body no longer supports (a contradiction). The stale draft is not shown to you;
   judge staleness by contradiction with the body, not by comparison to a prior block.
6. Record whether the candidate emitted exactly the required blocks — the two
   top-level sections, plus the Integration Specification `### Overview` when the body
   carries an Integration Specification — and whether the Document Consumption Guide
   table uses the `Section | What it settles & why it matters downstream | Key data`
   columns.
7. When the body carries a `## Integration Specification`: from its subsections derive
   the CORRECT `### Overview` signpost — enumerate the section's `###` subsections in
   document order EXCEPT `### Overview` itself; that enumeration is the set the
   candidate's `Read in order:` must name (grouping by service/cluster is fine). Read
   the section's contract shape for the lead sentence (one shape, or several when the
   section carries more than one service/contract).
8. Read the candidate's Integration Specification `### Overview` and compare the
   subsections its `Read in order:` names against your derived set — a subsection the
   section carries but the signpost omits (including a review-added one), or one the
   signpost names but the section no longer carries, is drift. Note whether the lead
   sentence still asserts a shape the section no longer has.

</extraction_checklist>

## 4. Severity Policy

### 4.1 Route To `critical_findings`

- A statement in any derived block that CONTRADICTS the current body. This is the
  staleness failure, and it does not depend on wording: reproducing the stale text
  verbatim and paraphrasing it into new words are the SAME defect, because the test is
  agreement with the body, not similarity to the input. The recurring shapes — a
  Document Consumption Guide whose ROW SET is the draft's rather than the body's (a
  review-added `##` section with no row, or a row for a section the body no longer
  carries), a row whose `Key data` set the body no longer supports (a stale `D#` /
  `PD#` / capability set), a Solution Overview count that does NOT equal the current
  body count (e.g. "six architecture decisions" when the body now carries seven `D#`),
  a passed-forward item the body decided differently, or an Integration Specification
  `### Overview` whose `Read in order:` signpost omits a `###` subsection the section
  now carries (including a review-added one) or names one the section no longer carries.
- A fabricated fact: an identifier (`D#`, `Q#`, `PD#`), capability, dependency,
  section, count, or contract key the body does not support — including a change the
  candidate invents to look refreshed.
  **Before recording a fabrication finding, quote the body and show that NO section
  supports the value.** A `D#`, `Q#`, `PD#`, WBS code, capability, or contract key that
  appears anywhere in the body is grounded, whether or not the review session touched
  it; carrying a row or member the review never moved is correct projection, not
  invention.
- A count in the Solution Overview is checked against the body: the reference is the
  CURRENT number in the body (of `D#` decisions, in-scope facets, `PD#` deferrals, open
  `Q#`). A count that EQUALS the current body number is CORRECT and can NEVER be a
  staleness finding — the review changing the number does not make a matching count
  wrong. Only a count that does NOT equal the current body number is a contradiction.
  Compare a count ONLY against the body's number for the SAME subject (a decisions count
  against `D#`, a facets count against facets); a count that matches its own subject is
  correct even if it differs from some OTHER subject's number — never manufacture a
  contradiction by comparing one subject's count to a different subject's number.
- A Document Consumption Guide row's `Key data` is judged ONLY against the section that
  row names. A value the body places in a DIFFERENT section — routed elsewhere by the
  design — is not "missing" from this row and is never a critical here; look for it in
  the row for the section that owns it, and judge THIS row against its own section's
  exported set alone.
- **Single-value / equivalent-`None` cells.** Review & Approval exports one value (the
  approval state); the Integration Specification row exports `N/A — functional change`
  (equivalently `None`) for a FUNCTIONAL design and a contract-key SET for an integration
  design. Reading `None` where the body carries a VALUE or a non-empty set is a DROPPED
  value and is `critical_findings`. But `None` where the body's section is genuinely
  contract-free (a functional Integration Specification) is CORRECT, and a value that
  RENDERS the body's state in different words — `Not approved` for `Approved: no`, `None`
  for `N/A — functional change` — is faithful, so route that wording difference to
  `warnings`, never here.

**Wording is not a finding.** The golden's phrasing has no authority (§1). A value that
conveys what the body says in different words is correct: an identifier with the status
the body gives it carries the same decision as a longer form. Never route a missing
parenthetical, a shorter status, or a different-but-faithful phrasing to
`critical_findings`; at most it is a `warnings` nuance when real information the body
carries is lost.

**An omission is not a contradiction — an incomplete SET or ROW SET is.** The Document
Consumption Guide's rows project the body's sections and each row's key data projects a
full set, so DROPPING a section-row the body carries (including a review-added one) or a
set member the body carries makes the guide wrong and stays `critical_findings`. The
Integration Specification `### Overview` `Read in order:` signpost is likewise a SET
projection of the section's `###` subsections, so omitting one the section carries
(including a review-added one) or naming an absent one is `critical_findings`; its LEAD
sentence, though, is prose — it contradicts the body only when it ASSERTS a shape the
section denies, and staying silent about an added subsection in the lead (while the
signpost names it) is not a contradiction. The Solution Overview is prose, not a set: it
contradicts the body only when it ASSERTS a value, count, or status the body denies. **Before routing a Solution Overview staleness
critical, QUOTE the exact sentence that asserts the stale value and name the body it
denies.** If the only thing you can say is that the overview "does not mention",
"omits", or "fails to reflect" the added section or the change, that is NOT a
contradiction and NOT a staleness critical: it is a `change_reflection` deduction and a
`warnings` note. A Solution Overview that still describes the pre-session state by
SILENCE (it never mentions the change) is refreshed enough to be contradiction-free;
only one that describes it by ASSERTION (it states the old count, the old shape) is
stale. Naming an added item as added ("two items are routed to planning") is agreement
with the body, never route it here.

**Route each finding once.** A contradiction is a critical; a non-contradicting
omission or wording nuance is a warning. Never record the same issue in both lists.

When this route is used, cap overall `score` at `0.5`.

### 4.2 Route To `warnings`

- A change from §2 that is reflected in one block but not the other (e.g. the Document
  Consumption Guide adds the new section's row but the Solution Overview never narrates
  it, while its counts still agree).
- A Document Consumption Guide row that is merged, renamed, reordered, or left without
  `None` for an empty set, while the row SET and the remaining rows are correct.
- A `Key data` cell written as prose where identifiers and sets belong, or a `What it
  settles & why` phrase that is vague but not contradicted by the body.
- A Solution Overview that re-lists table rows (a `D#` roll-call, a per-question
  walkthrough) instead of describing the file by subject and count.
- Wording that is imprecise or awkward but not contradicted by the body.
- Extra material beyond the two sections (a title, another section, a preamble, a
  description of what was changed) while both required blocks are present and correct.

Warnings reduce dimension scores; they do not by themselves force `pass = false`.

## 5. Rubric Dimensions

Score every dimension from `0.0` to `1.0`. Never use `null`, `n/a`, or omit one.

- `staleness_detection`: no claim in any derived block survives that the current body
  contradicts. Score `1.0` when every statement agrees with the body — including the
  Document Consumption Guide row set matching the body's current sections AND the
  Integration Specification `### Overview` signpost matching the section's current
  `###` subsections. Score `0.0` when the candidate reproduces the stale Solution
  Overview, the stale Document Consumption Guide, or the stale Integration Specification
  `### Overview` — verbatim OR reworded — such that its claims, its row set, or its
  signpost still describe the pre-session artifact (the classic miss: the draft's row
  set with no row for the review-added section, or the draft's signpost missing the
  review-added subsection). Judge by AGREEMENT WITH THE BODY, never by textual distance
  from the input.
- `change_reflection`: every change in §2 that the body carries is visible in the
  block(s) it belongs to — the review-added `##` section has its new guide row AND is
  named by subject in the Solution Overview, a moved `D#`/`PD#` set is current, and a
  review-added Integration Specification `###` subsection is named in that section's
  `### Overview` signpost. Deduct per missed change; a candidate that adds the new row
  but never reflects it in the overview, or refreshes the guide but leaves the
  Integration Specification `### Overview` signpost stale, scores mid-range, not high.
- `handoff_fidelity`: the Document Consumption Guide maps the body's CURRENT `##`
  sections — one row per section, in document order, excluding only Solution Overview,
  the guide itself, and Design Gate — with `None` for empty sets, `Key data` as
  identifiers/sets, and grounded `What it settles & why` phrases, and every row's key
  data equals the value you derived from the body in §3.1, INCLUDING the rows that did
  not change and the row for any section the review added. When the design carries an
  Integration Specification, its `### Overview` also holds: a shape-first lead (naming
  each service/contract when there is more than one) and a `Read in order:` signpost
  that names every `###` subsection the section carries except itself, in reading order,
  grouped by service/cluster when the section is large.
- `no_fabrication`: no invented `D#`, `Q#`, `PD#`, capability, dependency, section,
  count, contract key, or decision. A change the session did not make and the body does
  not show is fabrication even when it sounds plausible.
- `reader_map_discipline`: the Solution Overview orients a reader — what the design
  settles, why it is shaped that way, what constrains downstream, what is passed forward
  — using subjects and counts rather than reproducing table rows, and it does not drift
  into the identifier inventory that belongs to the Document Consumption Guide. Naming a
  register or an occasional identifier as a pointer is fine; walking the rows is not.
- `structure_conformance`: exactly the required blocks — `## Solution Overview` and
  `## Document Consumption Guide` (the latter with the `Section | What it settles & why
  it matters downstream | Key data` columns), plus, for an integration design, the
  Integration Specification `### Overview` at its `###` level (a lead paragraph then a
  `Read in order:` line, no `## Integration Specification` wrapper and no other
  subsection). The two top-level sections' presence and columns are also checked by a
  deterministic harness gate against the golden; that gate reads only `##` sections, so
  the `### Overview` presence and level are scored HERE. Score here what the gate leaves
  — extra sections, a preamble, a wrapper fence, a missing or mis-levelled `### Overview`
  for an integration design, or commentary.

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
- If `critical_findings` is non-empty, `score` MUST be `<= 0.5` and `pass` MUST be
  `false`.
- A verdict with any `critical_findings` and `score > 0.5` is invalid. Fix it.

## 6. Output Contract

The OUTPUT PROCEDURE in the system message controls packaging (the `<reasoning>`
routing pass, then the fenced JSON verdict). For this artifact the routing pass must:

1. State the Document Consumption Guide row SET you derived from the BODY's `##`
   sections in §3.1, and each row's key data. When the body carries a `## Integration
   Specification`, also state the `### Overview` signpost SET you derived from its
   `###` subsections (§3.7).
2. Per-row key-data proof, then critical-admission ledger. For EACH Document Consumption
   Guide row (and, when the body carries a `## Integration Specification`, the `###
   Overview` signpost as one more line), write ONE line: `<section> | BODY: <the value/set
   the named section exports, or "empty→None"> | CAND: "<the candidate cell, quoted
   verbatim>" | <verdict>`. Derive the verdict ONLY by comparing BODY against the QUOTED
   CAND text:
   - CAND conveys the body's value/set (any wording, format, or order) → `no_breach`.
   - CAND reads `None`/empty where BODY has a value or a non-empty set → `DROPPED-VALUE`
     (critical) — this covers the single-value cells too (Review & Approval; an
     integration Integration Specification's contract keys). `None` where the section is
     genuinely contract-free (a functional Integration Specification) is `no_breach`.
   - CAND omits a member BODY's set carries (a Key-data member, a Document Consumption
     Guide row, or a `### Overview` subsection) AND that member is absent from the quoted
     CAND text → `DROPPED-MEMBER` (critical). A MEMBER is the bare IDENTIFIER the set
     enumerates — a `D#`, `PD#`, `Q#`, WBS code, capability, endpoint/channel, mapping
     field, source name, or subsection title. A parenthetical or trailing QUALIFIER on a
     present identifier — a status, a state word (`proposed`, `unconfirmed`), an owner, a
     linked `Q#`, a one-line description, or a date — is NOT a member: if every identifier
     BODY's set carries appears in the quoted CAND text, the row is `no_breach` (or at most
     `FORMAT/TERSE` for a stripped qualifier), NEVER `DROPPED-MEMBER`, however many
     qualifiers are missing. This holds even when the parenthetical is the row's ONLY
     human-readable content: for an ID-plus-description member — a Planning Deferral
     `PD1 (<description>)`, a proposed decision `D6 (proposed, links Q#)`, an open question
     `Q1 (<topic>) — <owner>` — the MEMBER is the bare id (`PD1`, `D6`, `Q1`) and the
     description/state/owner is the qualifier, so a PRESENT id whose description was stripped
     is `FORMAT/TERSE`, never `DROPPED-MEMBER` (a genuinely ABSENT id still fires). Route
     EVERY ID-plus-description row the SAME way in one pass: if a stripped `D#`/`Q#`
     description is a warning, a stripped `PD#` description is the same warning — grading one
     row's stripped description critical while a sibling row's is a warning is the
     inconsistency this rule forbids. If the member IS present in the quoted text (even
     reworded or grouped), it is NOT dropped — do NOT fire.
   - CAND states a value/member absent from the ENTIRE body → `FABRICATION` (critical).
   - CAND renders the body's value in different words/format, or terser-but-faithful
     (`Not approved` for `Approved: no`; `D1, D2` for `D1–D2`; a present identifier with
     its parenthetical qualifier stripped — a missing annotation, status, owner,
     `proposed`/`unconfirmed` state, linked `Q#`, description, date, or count) →
     `FORMAT/TERSE` (warning), never critical.
   Only `DROPPED-VALUE`, `DROPPED-MEMBER`, and `FABRICATION` lines may enter
   `critical_findings`, and each MUST carry its quoted CAND text as evidence. For the
   Solution Overview (prose, not a set) a finding is admissible to `critical_findings` ONLY
   as `ASSERTS-DENIED` — quote the exact candidate sentence that STATES a value the body
   denies for the SAME subject, plus the body's value for that subject. A Solution Overview
   that "does not mention / omits / fails to reflect" a change is ALWAYS a `warnings` /
   `change_reflection` line, never a critical. Judge each row's key data ONLY against the
   section its row names.
3. Confirm the pass/score rule: `critical_findings` non-empty forces `score <= 0.5`
   and `pass = false`.

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
