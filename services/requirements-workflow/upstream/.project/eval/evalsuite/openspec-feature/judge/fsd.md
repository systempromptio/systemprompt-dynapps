---
schema: openspec-feature
artifact: fsd
context:
  - .context/initial-context
  - discovery/final
  - design/final
  - proposal/final
  - grooming/final
  - specs
  - fsd-scaffold
sources:
  - ../../../../templates/openspec/schemas/feature/templates/fsd.md
scoring:
  pass_threshold: 0.7
---

# FSD Standard — Contract and Rubric (scaffold-then-enrich)

This is the canonical definition of a good client-facing Functional
Specification Document (FSD) working copy in the OpenSpec `feature` workflow,
under the Phase-C scaffold-then-enrich model. The judge uses it to score a
candidate FSD. It is intentionally explicit and self-contained: do not assume
the judge has any ambient project knowledge beyond this standard, the inlined
SOURCES, and the CONTEXT (the source packet, the upstream discovery / design /
proposal artifacts, the APPROVED grooming ledger, this change's delta specs —
the `specs.<flow>/…` blocks — and the `fsd-scaffold` block).

## The scaffold-then-enrich model (read this FIRST)

- The FSD's three per-requirement sub-sections — `#### User Story`,
  `#### Acceptance Criteria`, and `#### Key behaviours` — are a DETERMINISTIC,
  script-owned projection produced by `generate-functional-artifacts.mjs`: the User Story and
  Acceptance Criteria are copied VERBATIM from the covering proposal slice, and
  Key behaviours is a TABLE (`# | Scenario | GIVEN | WHEN | THEN`) built from
  the covering spec requirement(s)' full scenario set — when several requirements
  share the WBS the script opens each one's rows with a bold sub-header row
  carrying that requirement's name. The EXACT projection is
  given in CONTEXT as the `fsd-scaffold` block (the script's output, with the
  non-owned sections left as template stubs). The same script also projects the
  proposal's re-usability legend as the bold `**Re-usability Legend**` block that
  OPENS the "## In Scope Functional Requirements" section, ahead of the first
  requirement block — script-owned like the three sub-sections, and a bold label
  rather than a heading by design.
- The candidate is the ENRICH step: it must reproduce those three script-owned
  sub-sections and the projected legend VERBATIM and fill only the NON-OWNED sections — the General FSD
  Information card, the approval roster, the Reference Materials, each
  requirement block's Visualization, and framing.
- Therefore the candidate is judged on: (1) did it PRESERVE the read-only
  projection exactly (`read_only_integrity`), and (2) is the ENRICHMENT
  correct, grounded, and client-safe (the remaining dimensions). The LLM no
  longer authors the user story, the acceptance criteria, or the key
  behaviours — do NOT score their WORDING, shape, grouping, or scenario
  selection as if it did; score ONLY whether they match the `fsd-scaffold`.
- The FSD adds no behaviour and decides nothing; behaviour authority stays with
  specs, AC/story shape with the proposal. It is generated only after the
  Grooming Approval Gate passed. The header card H2 is type-specific
  (`## General FSD Information`), its status field is "FSD Status", and the H1
  page title ends with " FSD" (e.g. "SLS - Store Locator FSD"). On a fresh copy
  FSD Status is `draft`, the delivering-org "Astound Approval" roster carries
  SA/FE/BE/QA seeded `TBC` (real CONTEXT names substituted into matching roles),
  every approval-group row `not started`, the `## Open Questions` section
  carries only its review sentence and an empty table header (the section
  belongs to the client review cycle — reviewers add rows on the published
  page), and no Confluence URL / Page ID is claimed.

## Invariants the candidate MUST satisfy

1. **Read-only integrity (closed law).** In EVERY requirement block, the
   `#### User Story`, `#### Acceptance Criteria`, and `#### Key behaviours`
   sub-sections reproduce the `fsd-scaffold` block with the same WBS VERBATIM:
   the user-story sentence unchanged; the acceptance-criteria list and its
   `#####` group sub-headings unchanged (no criterion added, dropped,
   reworded, reordered, re-grouped, or re-shaped); the Key-behaviours TABLE
   unchanged (same rows, same `# / Scenario / GIVEN / WHEN / THEN` cells — no
   scenario row added, dropped, reworded, or
   re-numbered), INCLUDING its bold sub-header rows (`| **<name>** |  |  |  |  |`):
   each one is a covering spec requirement's NAME, so an edited, added, dropped, or
   re-ordered sub-header row is as much a violation as an edited scenario row, and
   a table the scaffold projects flat stays flat. Any MEANINGFUL edit to these three sub-sections is a violation,
   even a "nicer" rewrite — corrections belong upstream (proposal / spec), never
   in the FSD. Compare SEMANTICALLY: IGNORE pure typographic re-encoding — hyphen
   / dash variants (`-` vs `‑` vs `–`), straight vs curly quotes, other Unicode
   punctuation, and whitespace / line-wrapping are NOT violations. Fire ONLY on a
   content change: added / removed / reworded words, an added parenthetical or
   qualifier, a dropped / added / re-numbered criterion or scenario row, or a
   changed group label.
2. **Requirement coverage (closed law).** Every in-scope requirement block that
   the `fsd-scaffold` carries is present in the candidate exactly once, keyed
   by the same clean WBS code heading (e.g. `SLS.SRCH`), in the same set — the
   enrichment step must not drop, merge, split, rename, or add a requirement
   block. Deferred RQs appear ONLY under `## Deferred Requirements`; declined or
   pending CR asks ONLY under `## Change Requests`.
3. **Enrichment grounding.** The sections the LLM fills are grounded: the
   General FSD Information card values (Project Name = the full project/client
   identity the sources name; Author/Owner from CONTEXT), the approval roster,
   the Reference Materials rows, and each block's Visualization (meaningful
   states + Desktop/Mobile surfaces the requirement's behaviour implies) all
   trace to CONTEXT — nothing invented. ORG FIREWALL: the delivering-org
   "Astound Approval" roster is seeded by DELIVERING-ORG (Astound) people only —
   a CONTEXT person whose org is the CLIENT (even with a delivery-sounding title
   such as "FE lead", "platform dev", or "architect") belongs in the client
   group, never an Astound SA/FE/BE/QA row. Author/Owner names the
   delivering-org document author(s) (the BA, and the SA lead when named as
   co-owner), not the approval roster and not a spreadsheet-/tracker-provenance
   name (a CSV `Created By` / `Updated By` / `Approved By` value).
4. **Client-facing hygiene.** No internal repository paths, no references to
   internal artifacts (discovery/design/proposal/grooming/specs/
   technical-design/tasks or `openspec/…` paths), no authored Open-Questions
   row and no known-constraints section, no agent commentary, no code tokens
   (component/file/endpoint/hook names, HTTP status codes). Consumed
   dependencies stay invisible: the vendor/service names of capabilities this
   change merely consumes never appear in text the LLM authors. RQ/WBS/CR codes
   ARE allowed everywhere (client-shared trace vocabulary). Source documents in
   Reference Materials are resolved to the canonical client-facing link CONTEXT
   holds (link registry / project-context origin / wiki/Drive); a bare title is
   acceptable ONLY when CONTEXT carries no link for it and the row flags that —
   a bare title while CONTEXT holds a resolvable link is a finding, and a
   fabricated URL is a finding.
5. **Fresh working-copy state.** FSD Status is `draft`; every approval-group
   row uses an allowed badge value and reads `not started`; the Open Questions
   section carries no question row (the review sentence plus its bare table
   header is the fresh state); no Confluence URL, Page ID, or publication
   claim; no approval implied anywhere.
6. **Scope separation.** Out-of-scope content is split correctly: same signed
   scope delivered later -> `## Deferred Requirements`; any change to the signed
   BRD -> `## Change Requests`, each CR's Status reflecting the grooming
   DECISION (`Agreed` / `Declined` / `Pending client agreement`). Explicit
   `None.` when a section is empty.
7. **No fabrication.** In the enriched content: no invented RQ/WBS/CR codes,
   requirements, magnitudes, URLs or routes no cited source states, Visualization
   states or screens the requirement's behaviour does not imply, invented image
   files / design-tool links, CMS/attribute IDs, Confluence URLs, Page IDs,
   tracker keys, approvals, or people absent from CONTEXT. A Package/Set/Batch
   number no source states is fabrication (a BLANK cell is the correct grounded
   outcome).
8. **Template & structure conformance.** The H1 page title ends with " FSD" and
   the header card H2 is `## General FSD Information`. Template section order and
   heading levels: `## In Scope Functional Requirements` H2, each requirement H3
   keyed by its WBS code, sub-sections H4 (User Story, Visualization, Acceptance
   Criteria, Key behaviours), AC groups + Key-behaviours table under them.
   Key behaviours is a TABLE (`# | Scenario | GIVEN | WHEN | THEN`). Visualization
   is REQUIRED in every requirement block. A deterministic code gate already
   enforces exact section presence, order, and table columns against the golden;
   score this dimension for heading and field fidelity only.

## Forbidden (each is a critical finding when present)

- ANY change to a script-owned sub-section (`#### User Story` /
  `#### Acceptance Criteria` / `#### Key behaviours`) versus the `fsd-scaffold`
  block with the same WBS: a reworded story, an added/dropped/reworded/reordered
  acceptance criterion or `#####` group, a Key-behaviours scenario row or bold
  `| **<requirement name>** |` sub-header row added/dropped/reworded/renamed/
  re-numbered/re-ordered, or a dropped table. (`read_only_integrity`)
- ANY change to the projected `**Re-usability Legend**` block versus the
  `fsd-scaffold`: a dropped, re-worded, re-ordered, extended, or trimmed entry, a
  legend moved out of the section opening, or one promoted to a heading.
  (`read_only_integrity`)
- A dropped, merged, split, renamed, or added in-scope requirement block versus
  the scaffold's set; a functional block for a deferred RQ or a declined/pending
  CR ask. (`requirement_coverage`)
- Internal leakage in a client document: repo/artifact paths or names, an
  authored Open-Questions row, a known-constraints section, agent commentary,
  code tokens, or a consumed dependency's vendor/service name in authored
  text. (`client_facing_hygiene`)
- A non-fresh working copy: FSD Status other than `draft`, an approval-group row
  not `not started`, a question row written into Open Questions, or a claimed
  publication. (`working_copy_state`)
- A mis-split or mis-statused Deferred/CR item (a declined CR shown as pending or
  absorbed into functional scope). (`scope_separation`)
- A fabricated code, requirement, magnitude, route, Visualization state/screen the
  behaviour does not imply, image/screenshot reference, CMS/attribute ID,
  Confluence URL, Page ID, tracker key, person, or approval. (`no_fabrication`)
- A mis-attributed person: a CLIENT (or any non-Astound) person seated in the
  Astound delivering-org roster, or a spreadsheet-/tracker-provenance name (a CSV
  `Created By` / `Updated By` / `Approved By`) promoted into Author/Owner. A
  BLANK Author/Owner cell is NOT this finding — it is an omission
  (`missing_evidence`), never fabrication. (`no_fabrication`)
- A missing mandatory element: a requirement block without its Visualization.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE place.

- Artifact presence (run FIRST): if the candidate is not an FSD working copy at
  all — a refusal, a status report, planning notes, or an empty document —
  record the critical finding `no_artifact`, score EVERY dimension 0.0, and
  stop.
- The `fsd-scaffold` block is the EXACT canon for the three script-owned
  sub-sections; the GOLDEN is a non-exhaustive positive EXAMPLE of the finished
  (enriched) document, NOT an answer key for the enriched wording. A difference
  from GOLDEN in an enriched section (a different Visualization phrasing, a
  differently-worded Reference row) is NEVER by itself a finding; judge
  enrichment for grounded, faithful, client-safe content, not verbatim presence.
- Read-only integrity sweep (closed rule — run BEFORE scoring
  `read_only_integrity`): for each requirement block, find the `fsd-scaffold`
  block with the same WBS and compare the three sub-sections SEMANTICALLY. First
  normalize away pure typographic re-encoding — hyphen/dash variants (`-` vs `‑`
  vs `–`), straight vs curly quotes, other Unicode punctuation, and
  whitespace/line-wrapping are NOT differences (gpt-oss commonly re-encodes a
  hyphen; that is not a violation). Then fire a critical ONLY on a CONTENT
  difference: a reworded story; an added/removed/reworded word or an added
  parenthetical/qualifier in a criterion; an added/dropped/reworded/reordered
  criterion or `#####` group; a changed/added/dropped/re-numbered Key-behaviours
  table row — scenario rows AND the bold `| **<requirement name>** |` sub-header
  rows the scaffold projects (a group row the candidate invents, renames, drops, or
  re-orders is the same class of violation; the label is owned upstream by the spec
  requirement's name). Compare the `**Re-usability Legend**` block that opens the section
  the same way — same entries, same wording, same order as the
  scaffold's. Quote the candidate text and the scaffold text
  it should have matched, and confirm the difference is a content change, not a
  typographic re-encoding, before firing. Reproducing the scaffold (modulo
  typography) is REQUIRED and is never itself a finding.
- Coverage sweep (closed rule — run BEFORE scoring `requirement_coverage`): list
  the `fsd-scaffold` in-scope blocks (by WBS); confirm the candidate has exactly
  the same set, one block each, keyed by the WBS code. Fire on a dropped, merged,
  split, renamed, or added in-scope block, or a functional block for
  deferred/declined/pending scope.
- Enrichment sweeps (run BEFORE scoring the enrichment dimensions): test the
  authored sections (General FSD Information card, approval roster, Reference
  Materials, each Visualization, framing, Deferred/CR) against CONTEXT and the
  approved decisions. Fire on internal leakage / code tokens / consumed-vendor
  names (`client_facing_hygiene`), an invented value / state / screen / ID / URL
  / person / approval (`no_fabrication`), a non-fresh status/approval/publication
  claim (`working_copy_state`), or a mis-split/mis-statused Deferred/CR item
  (`scope_separation`).
- One defect, one primary critical — classify by the actual fault and do not
  raise the same claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension: (a) RQ/WBS/CR codes
  anywhere; (b) a Visualization whose cells are TEXT descriptions of the screens
  to capture rather than images (Visualization is REQUIRED and its text form is
  correct; only an INVENTED state/screen or a fabricated image reference is a
  finding); (c) explicit `None.` rows; (d) approved-decision bounds restated in
  the read-only sections ("permanently redirected", "regular weekly opening
  hours … not displayed in v1") — they come from the scaffold and are required,
  not leakage; (e) the sourced retired route (e.g. `/stores`); (f)
  `Tracker Ticket: TBD` in a CR block; (g) a CONTEXT-grounded person seated in the
  CORRECT org's roster row — a delivering-org (Astound) person in an Astound
  SA/FE/BE/QA row, or a client-org person in the client group — or named in
  Author/Owner as a delivering-org document author. A `TBC` placeholder is the
  correct fresh state and a BLANK Author/Owner is at most `missing_evidence` (an
  omission, never fabrication); an invented approver, a CSV provenance name
  (`Created By`/`Approved By`) promoted to author, or a CLIENT/non-Astound person
  seated in the Astound roster IS a finding (see the org-firewall rule); (l) the
  `## Open Questions` section carrying only the template's review sentence and
  an empty table header — that is the REQUIRED fresh state, never leakage; only
  an authored question ROW is a finding. (h) a Reference Materials row naming
  the BRD/SRD by title without a URL ONLY when CONTEXT provides no link for it
  and the row flags that (no canonical link on record) — but a bare title while
  CONTEXT holds a resolvable link IS a finding; (i) a
  grounded-but-abbreviated header value — at most a warning; (j) the
  delivering-org group titled `Astound Approval` (STATIC) and the client group
  titled with the CONTEXT client org name; (k) the three script-owned
  sub-sections reproduced VERBATIM from the scaffold — that is the REQUIRED
  outcome, never a finding.
- Omission is not a critical — under ANY dimension — EXCEPT: the coverage law (a
  dropped/merged/split/renamed in-scope block IS critical), a missing
  Visualization in a block (critical), and any drift in a script-owned
  sub-section (critical under `read_only_integrity`). Everything else missing
  (a thinner-than-ideal Visualization, an omitted optional Reference row) is
  `missing_evidence` or a warning.
- Authority check for every finding: if the only authority you can cite is "the
  GOLDEN contains this", the finding is invalid — DROP it. A valid finding cites
  a STANDARD rule, a CONTEXT fact, or the `fsd-scaffold` canon that the candidate
  violates.
- Completeness does not offset a violation, and fluency is not correctness.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by the
dimension id below. 1.0 = fully satisfied with evidence, ~0.5 = partially
satisfied or minor gaps, 0.0 = violated or absent.

- `read_only_integrity` — every block's User Story, Acceptance Criteria, and Key
  behaviours reproduce the `fsd-scaffold` block with the same WBS VERBATIM; no
  edit/reword/re-shape/re-group, no added/dropped/re-numbered criterion, group,
  scenario table row, or bold `| **<requirement name>** |` sub-header row; the
  `**Re-usability Legend**` block opening the section reproduces the scaffold's
  legend VERBATIM.
- `requirement_coverage` — the candidate's in-scope requirement blocks are
  exactly the scaffold's set, one block each, keyed by clean WBS code; deferred
  and CR items sit only in their bottom sections; no block dropped, merged,
  split, renamed, or added.
- `client_facing_hygiene` — authored content has no internal paths/artifact
  names, no authored open question, no agent commentary, no code tokens, no
  consumed-dependency vendor names; RQ/WBS codes as the only technical vocabulary.
- `working_copy_state` — FSD Status `draft`, every approval-group row
  `not started` (grounded approver names allowed), Open Questions carrying no
  question row, no publication metadata.
- `scope_separation` — Deferred vs Change Requests split per the law; CR statuses
  mirror the grooming decisions; `None.` where empty.
- `no_fabrication` — every enriched value, Visualization state/screen, asset
  reference, ID, URL, person, and approval traces to CONTEXT.
- `structure_conformance` — H1 title ends with " FSD", header card H2 is
  `## General FSD Information`, template section order and heading levels
  (H2/H3/H4/H5), blocks keyed by WBS code, Key behaviours as a TABLE with the
  `# | Scenario | GIVEN | WHEN | THEN` columns, Visualization present in every
  block.

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it: if
  any item in `critical_findings` is present, `score` MUST NOT exceed 0.5.
- `pass` is true only when `score >= threshold` (given in the user message) AND
  `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or of invariants 1-8
  (read-only drift, a coverage-law breach, internal leakage, a mis-statused CR, a
  non-fresh working copy, fabrication, a missing Visualization). These block
  acceptance.
- **warnings** — quality issues that do not make the document wrong (an
  editorialised card/Reference cell, a heading-level slip, a thin but grounded
  Visualization).
- **missing_evidence** — expected enrichment the CONTEXT implies but the
  candidate omits without breaking a law. Record it ONLY here: it MUST NOT appear
  in `critical_findings` and MUST NOT cap the score.

Always ground findings in concrete evidence. Every critical finding MUST quote a
string that appears VERBATIM in the candidate; for a read-only drift, ALSO quote
the `fsd-scaffold` text it should have matched. Before recording a critical,
locate that exact string in the candidate; if it is not literally present, DROP
the finding. A finding whose evidence you cannot copy verbatim is a
hallucination, not a violation.
