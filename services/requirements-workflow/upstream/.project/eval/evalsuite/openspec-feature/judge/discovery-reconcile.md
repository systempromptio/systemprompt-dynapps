---
schema: openspec-feature
artifact: discovery-reconcile
context:
  # Namespaced by artifact: the flow overlay resolves this to
  # .context/review-session.discovery.<flow>.md — see config.states.review_session.
  - .context/review-session.discovery
  # The artifact mid-review, JUDGE view: the settled body with BOTH derived blocks
  # DROPPED. The judge derives the correct blocks from the body and scores the
  # CANDIDATE against that derivation, so it never needs the stale draft (that is
  # the generator's input). Dropping them removes the same-headed
  # `## Executive Summary` / `## Document Consumption Guide` the judge otherwise
  # mistook for the candidate's (false "dropped row" criticals on correct
  # candidates). A copy-forward candidate is still caught because it contradicts the
  # body — see lib/resolve.mjs, composeFixture.
  - id: discovery-in-review
    from: discovery/final
    drop: [Executive Summary, Document Consumption Guide]
# What a correct re-derivation produces: the settled artifact's two derived blocks.
golden:
  from: discovery/final
  sections: [Executive Summary, Document Consumption Guide]
scoring:
  pass_threshold: 0.7
# Inject deterministic, harness-computed presence facts (the candidate's Document
# Consumption Guide row set diffed against the golden's) into the judge prompt. The LLM
# judge is unreliable at reading presence — it repeatedly flags a row the candidate DOES
# carry as "dropped" — so the harness parses the sets and hands the judge the ground-truth
# diff, closing the hallucinated-absence class without a model swap. Discovery carries no
# Integration Specification, so the facts block is the DCG row set only. See
# lib/structure.mjs deriveStructureFacts + lib/prompt.mjs.
derived_structure_facts: true
---

# Discovery Reconcile Judge Contract

<judge_contract version="1">

## 1. Role And Authority

You judge a partial re-emission of an OpenSpec `discovery.md`: the two blocks that
are DERIVED from the artifact's body — `## Executive Summary` (a reader map) and
`## Document Consumption Guide` (a fixed per-section projection of the body sections
it names, placed directly after the Executive Summary — one row per consumed section
with a `What it settles & why` phrase and a `Key data` set).

The situation being tested is the Per-Artifact Review Gate re-entry. The artifact
was drafted, a review session changed the body, and the two derived blocks were
left as the first pass wrote them. A correct candidate re-derives both blocks from
the CHANGED body. Carrying either block forward — verbatim or reworded — leaves the
artifact contradicting itself at the moment a human is asked to approve it.

Authority order:

1. `CONTEXT` is the source of truth. It holds two things:
   - `discovery-in-review` — the artifact as it stands. Its BODY (Requirements Trace,
     Affected Capabilities, Dependencies, Change Request Candidates, Open
     Questions, Cross-cutting Requirements, Delivery Scope & Targets) is CURRENT
     and authoritative. Its `## Executive Summary` and `## Document Consumption
     Guide` are DROPPED (not shown): the stale draft blocks carry NO authority and
     are
     deliberately withheld so nothing but the candidate's blocks can be scored.
     Derive the correct blocks from the BODY.
   - `review-session.discovery` — the session log that produced the body changes. It explains
     WHY the body reads as it does; the body itself is what the blocks must match.
2. `SOURCES` and this `STANDARD` define the expected block semantics.
3. `GOLDEN EXPECTED` is a reference for coverage and shape. Do not require its
   exact wording where the candidate is faithful to the body.
4. `CANDIDATE` is the output being judged.

**The single most important rule: the ONLY `## Executive Summary` and
`## Document Consumption Guide` you score are the ones under `===== CANDIDATE`.**
`discovery-in-review` does not carry those two blocks at all, and the blocks under
`GOLDEN EXPECTED` are a reference; never quote the golden as if it were the
candidate's, and never report a value as "missing" or "dropped" from the candidate
unless you can quote that value's absence from a line that appears AFTER the
`CANDIDATE` delimiter.

A `DERIVED STRUCTURE FACTS` block accompanies the CANDIDATE. It is computed
DETERMINISTICALLY by the harness (not by a model) by parsing the candidate's and the
golden's Document Consumption Guide row set. It is AUTHORITATIVE for PRESENCE: a row it
lists under the candidate is PRESENT and must NEVER be recorded as missing/dropped; only a
row it names as "in GOLDEN but NOT in candidate" is a genuine drop eligible for §4.1. Do
not re-derive presence by eye against this block — read staleness, key-data fidelity, and
handoff quality ON TOP of these facts.

## 2. What Changed In The Body

You are NOT given a list of the changes. Derive it, because the whole point of the
task is that the blocks must follow the body wherever it went.

The session log names what the reviewer decided; the body is where those decisions
landed. Work in that order:

1. Read `review-session.discovery` and list each decision it records — a question
   answered, an item decided, a dependency added, an attribution corrected.
2. For each one, find the row in the body that carries it, and note which
   Document Consumption Guide row projects that section. A decision the body does
   not carry is not your concern: the body is authoritative, the session only
   explains it.
3. Read the body's tables in full anyway. A change can reach a block without
   appearing in the session — and, more often, a row the session never touched
   still belongs in a set-valued key-data cell. The `Requirements Trace` row's
   out-of-scope set in particular is the WHOLE out-of-scope set, not the codes the
   session moved.

The number of changes, which rows they touch, and which direction they move are
all case-specific. A set can go from a value to `None` or from `None` to a value;
an item can be resolved, declined, re-owned, or added. Never assume a change is
present because a previous case had it, and never assume a row is unchanged
because the session did not mention it.

Rows the body does not move keep their values. Changing one is fabrication, not
refreshing.

## 3. Facts To Extract Before Judging

<extraction_checklist>

1. From the body of `discovery-in-review`, derive the CORRECT key data of each of
   the seven Document Consumption Guide rows yourself, before looking at the
   candidate's table. Do this from the body's tables, never from the stale block.
   The `Requirements Trace` row carries three labelled sets that are mechanical set
   operations over the Requirements Trace, and you MUST perform them literally,
   walking every row of that table:
   - In scope = the WBS code of every row dispositioned `in scope`;
   - Deferred = every row dispositioned `deferred` (write `None` when there are
     none);
   - Out of scope = EVERY row whose `Disposition` cell reads `out of scope`, each
     with the owner its `Affected capability` cell names. Decide membership by
     reading that ONE cell per row, nothing else: a row's `Notes` may hedge it as
     "listed for source fidelity only", "not a locator dependency", or "does not
     appear in Dependencies" — none of that removes the row, because
     out-of-scope DISPOSITION and DEPENDENCY membership are different axes (a row
     can be out of scope and not be a dependency of this change, e.g. a `PD.*`
     homepage row). This set is larger than the codes the review session moved:
     rows that were already out of scope before the session stay out of scope and
     belong here. Enumerate them from the trace by walking every row's `Disposition`
     cell, not from memory of what changed and not from whether the row is a
     dependency.
2. List the candidate's per-section rows and their key data; compare each against
   your derived value. Record every difference as drift.
3. Read the candidate's Executive Summary and list each factual claim it makes
   about scope, counts, what is open, what was decided, and readiness.
4. For each such claim, mark it AGREES WITH BODY, CONTRADICTS BODY, or
   UNSUPPORTED (asserts something no section of the body carries).
5. Note specifically whether any change on the list you derived in §2 is missing
   from the candidate's blocks, and whether any claim in the candidate still
   describes the pre-session state the body no longer supports (a contradiction).
   The stale draft is not shown to you; judge staleness by contradiction with the
   body, not by comparison to a prior block.
6. Record whether the candidate emitted exactly the two required sections, and
   whether the Document Consumption Guide table uses the `Section | What it settles
   & why it matters downstream | Key data` columns.

</extraction_checklist>

## 4. Severity Policy

### 4.1 Route To `critical_findings`

- A statement in either block that CONTRADICTS the current body. This is the
  staleness failure, and it does not depend on wording: reproducing the stale text
  verbatim and paraphrasing it into new words are the SAME defect, because the test
  is agreement with the body, not similarity to the input. The recurring shapes —
  a count the body no longer supports (i.e. one that does NOT equal the current
  number of rows with that disposition; a count that DOES equal that total is
  correct — see the count rule below), a question listed in `Open items` after the
  body resolved it, a
  requirement left under the disposition it moved away from or stripped of the
  owner the body now names, an item called pending after the body decided it, a
  dependency the body added and the block omits, or scope the body does not carry.
- A fabricated fact: an identifier, dependency, capability, decision, count, or
  document the body does not support — including a change the candidate invents to
  look refreshed.
  **Before recording a fabrication finding, quote the body and show that NO row
  supports the value.** A WBS code, capability, dependency, `Q#`, or `CR#` that
  appears anywhere in the body's tables is grounded, whether or not the review
  session touched it; carrying an already-out-of-scope code such as a `PD.*` row is
  correct projection, not invention. An out-of-scope row stays in the out-of-scope
  set even when the body notes it is "source fidelity only" or "not a locator
  dependency": disposition and dependency are different axes, so never drop such a
  row from the set nor flag its inclusion as fabrication. When you check an
  Executive Summary count of out-of-scope (or in-scope) requirements, the reference
  is the NUMBER OF REQUIREMENTS TRACE ROWS with that disposition — do not subtract
  rows the body hedges as non-dependency or source-fidelity-only. A count that
  EQUALS that current row total is CORRECT and can NEVER be a staleness or
  contradiction finding — there is no "stale in context" exception: the review
  session changing WHICH rows carry the disposition does not make a matching TOTAL
  wrong (a summary saying "four out of scope" when the body now has four out-of-scope
  rows is right, even if the session raised the total from three to four). Only a
  count that does NOT equal the current row total is a contradiction. If you cannot
  quote a body row that the value contradicts, the finding is invalid — drop it.
- A readiness claim that the body contradicts (see §5 `readiness_consistency`).

**Wording is not a finding.** The golden's phrasing has no authority (§1). A value
that conveys what the body says in different words is correct: an identifier with
the status the body gives it carries the same decision as a longer form that also
restates the reasoning, and that status label is sufficient on its own — the
template asks these rows for identifiers and sets, not prose. Never route a
missing parenthetical, a shorter status, or a different-but-faithful phrasing to
`critical_findings`, and do not list it as missing evidence — at most it is a
`warnings` nuance when real information the body carries is lost.

**An omission is not a contradiction — an incomplete SET is.** A Document Consumption
Guide row's key data projects a full set, so dropping a member the body carries (a WBS
code, an owner, a dependency, a still-Open `Q#`) makes that row wrong and stays
`critical_findings`. **Exception — the Open Questions set projects only STILL-OPEN
questions.** A question the body RESOLVED this pass has LEFT the set by design (the
projection is "questions still Open", not "every question ever asked"), so OMITTING a
resolved `Q#` from the Open Questions set — and stating the reduced open-count that
follows (an Executive Summary saying "two questions remain open" when the body now
leaves two Open) — is the CORRECT projection and is NEVER a staleness, fabrication, or
incomplete-set finding, even when the block never narrates that the question was
resolved. Only two shapes are defects here: dropping a question the body STILL lists as
Open, or listing a RESOLVED question AS Open (asserting a count or `Q#` status the body
denies). The Executive Summary is prose, not a set: it contradicts the
body only when it ASSERTS a
value, count, or status the body denies — "three questions remain open" when the body
leaves two, "Q3 is open" after the body resolved it, a disposition or owner the body
changed. **Before routing an Executive Summary staleness critical, QUOTE the exact
sentence that asserts the stale value and name the body row it denies.** If the only
thing you can say is that the summary "does not mention", "omits", "does not narrate",
or "fails to reflect" a resolution or correction, that is NOT a contradiction and NOT
a staleness critical: it is a `change_reflection` deduction and a `warnings` note. A
summary that still describes the pre-session state by SILENCE (it simply never
mentions the change) is refreshed enough to be contradiction-free; only a summary that
still describes it by ASSERTION (it states the old count, the old disposition, the old
open/pending status) is stale. An Executive Summary whose stated counts and sets all
agree with the body is contradiction-free even when it never narrates that a question
was resolved or a row re-owned; that gap is a `change_reflection` deduction and a
`warnings` note, not a critical, and `staleness_detection` is NOT lowered for it.
Naming a resolved item AS resolved ("the map-style question has been resolved") is
agreement with the body, not a claim it is still open — never route it here.

**Route each finding once.** A contradiction is a critical; a non-contradicting
omission or wording nuance is a warning. Never record the same issue in both lists.

When this route is used, cap overall `score` at `0.5`.

### 4.2 Route To `warnings`

- A change from §2 that is reflected in one block but not the other.
- A Document Consumption Guide row that is dropped, merged, renamed, reordered, or
  left without `None` for an empty set, while the remaining rows are correct.
- A `Key data` cell written as prose where identifiers and sets belong, or a `What
  it settles & why` phrase that is vague but not contradicted by the body.
- An Executive Summary that re-lists table rows (an identifier roll-call, a
  per-question walkthrough) instead of describing the file by subject and count.
- Wording that is imprecise or awkward but not contradicted by the body.
- An Executive Summary whose counts and sets agree with the body but which does not
  narrate a resolution or correction the body made, while the Document Consumption
  Guide still projects it correctly.
- Extra material beyond the two sections (a title, another section, a preamble, a
  description of what was changed) while both required blocks are present and
  correct.

Warnings reduce dimension scores; they do not by themselves force `pass = false`.

## 5. Rubric Dimensions

Score every dimension from `0.0` to `1.0`. Never use `null`, `n/a`, or omit one.

- `staleness_detection`: no claim in either block survives that the current body
  contradicts. Score `1.0` when every statement agrees with the body. Score `0.0`
  when the candidate reproduces the stale Executive Summary or the stale Document
  Consumption Guide — verbatim OR reworded — such that its claims still describe the
  pre-session artifact. Judge by AGREEMENT WITH THE BODY, never by textual distance
  from the
  input: a sentence that keeps the golden's own wording because it is still true is
  correct and must not be penalized here. An omission that leaves every asserted count
  and set correct is not a contradiction: score this dimension on contradicted claims
  only, and let `change_reflection` carry a change the block failed to narrate.
- `change_reflection`: every change in §2 that the body carries is visible in the
  block(s) it belongs to. Deduct per missed change; a candidate that catches the
  obvious question closure but misses the disposition correction scores mid-range,
  not high.
- `handoff_fidelity`: the table carries all seven fixed per-section rows in order,
  with `None` for empty sets, `Key data` as identifiers/sets, and grounded `What it
  settles & why` phrases, and every row's key data equals the value you derived from
  the body in §3.1 — including the rows that did NOT change.
- `readiness_consistency`: the summary's readiness statement matches the artifact's
  actual state — no CRITICAL question is Open, so it may say discovery is ready for
  proposal. A candidate that hedges readiness the body does not warrant, or claims a
  readiness the body contradicts, loses this dimension.
- `no_fabrication`: no invented requirement, WBS code, capability, dependency,
  question, change request, path, count, or decision. A change the session did not
  make and the body does not show is fabrication even when it sounds plausible.
- `reader_map_discipline`: the Executive Summary orients a reader — what the file
  settles, what stays open, readiness — using subjects and counts rather than
  reproducing table rows, and it does not drift into the identifier inventory that
  belongs to the Document Consumption Guide. Naming a register or an occasional
  identifier as a pointer is fine; walking the rows is not.
- `structure_conformance`: exactly the two required sections, with the headings
  `## Executive Summary` and `## Document Consumption Guide` and the
  `Section | What it settles & why it matters downstream | Key data` columns.
  Section presence and columns are also
  checked by a deterministic harness gate against the golden; score here only what
  that gate leaves — extra sections, a preamble, a wrapper fence, or commentary.

Suggested scoring guide:

- `1.0`: fully re-derived and grounded.
- `0.75`: minor omission or weak wording, nothing contradicted.
- `0.5`: partially refreshed — some changes caught, others missed.
- `0.0`: stale, contradicted, or fabricated.

Compute `score` as the mean of the seven dimensions, then cap it at `0.5` if
`critical_findings` is non-empty. Set `pass = true` only when `score >= threshold`
and `critical_findings` is empty.

Consistency check before returning JSON:

- If `critical_findings` is empty and `score >= threshold`, `pass` MUST be `true`.
- If `critical_findings` is non-empty, `score` MUST be `<= 0.5` and `pass` MUST be
  `false`.
- A verdict with any `critical_findings` and `score > 0.5` is invalid. Fix it.

## 6. Output Contract

The OUTPUT PROCEDURE in the system message controls packaging (the `<reasoning>`
routing pass, then the fenced JSON verdict). For this artifact the routing pass
must:

1. State the seven Document Consumption Guide rows' key data you derived from the
   BODY in §3.1. For the Requirements Trace row, state all three sets (in-scope,
   deferred, out-of-scope-with-owner).
2. Per-row key-data proof, then critical-admission ledger. For EACH of the seven
   Document Consumption Guide rows, write ONE line: `<section> | BODY: <the value/set
   the named section exports, or "empty→None"> | CAND: "<the candidate cell, quoted
   verbatim>" | <verdict>`. Derive the verdict ONLY by comparing BODY against the
   QUOTED CAND text:
   - CAND conveys the body's value/set (any wording, format, or order) → `no_breach`.
   - CAND reads `None`/empty where BODY has a value or a non-empty set → `DROPPED-VALUE`
     (critical).
   - CAND omits a member BODY's set carries AND that member is absent from the quoted
     CAND text → `DROPPED-MEMBER` (critical). A MEMBER is the bare IDENTIFIER the set
     enumerates (a WBS code, `CR#`, `Q#`, dependency, or source name); a parenthetical or
     trailing QUALIFIER on a present identifier — a status, an owner, a description, or a
     date — is NOT a member, so a PRESENT id whose qualifier was stripped is `FORMAT/TERSE`,
     never `DROPPED-MEMBER` (a genuinely ABSENT id still fires), and every ID-plus-qualifier
     row is routed the SAME way in one pass. If the member IS present in the quoted text
     (even reworded or grouped), it is NOT dropped — do NOT fire. For the Requirements
     Trace out-of-scope set, a member is present when its WBS code appears in the quoted
     text, whether or not its owner is spelled out.
   - CAND states a value/member absent from the ENTIRE body → `FABRICATION` (critical) —
     this includes an out-of-scope WBS given an owner the body does not name for it.
   - CAND renders the body's value in different words/format, or terser-but-faithful (a
     WBS for its RQ; a missing annotation / date / status; the owner dropped from an
     out-of-scope WBS that is still listed) → `FORMAT/TERSE` (warning), never critical.
   The Open Questions row projects ONLY still-open questions: a question the body RESOLVED
   this pass and the candidate correctly OMITS is `no_breach`, never a drop; a resolved
   `Q#` the candidate lists AS open is `FABRICATION`. Only `DROPPED-VALUE`,
   `DROPPED-MEMBER`, and `FABRICATION` lines may enter `critical_findings`, and each MUST
   carry its quoted CAND text as evidence. For the Executive Summary (prose, not a set) a
   finding is admissible to `critical_findings` ONLY as `ASSERTS-DENIED` — quote the exact
   candidate sentence that STATES a value the body denies for the SAME subject, plus the
   body's value for that subject. An Executive Summary that "does not mention / omits /
   fails to reflect" a change is ALWAYS a `warnings` / `change_reflection` line, never a
   critical. Judge each row's key data ONLY against the section its row names.
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
    "readiness_consistency": { "score": 1.0, "reason": "..." },
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
