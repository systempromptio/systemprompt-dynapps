---
schema: openspec-feature
artifact: isd
context:
  - .context/initial-context
  - discovery/final
  - design/final
  - proposal/final
  - grooming/final
  - specs
  - isd-scaffold
  - .context/isd-diagram-request
sources:
  - ../../../../templates/openspec/schemas/feature/templates/isd.md
scoring:
  pass_threshold: 0.7
---

# ISD Standard — Contract and Rubric

This is the canonical definition of a good Integration Specification Document
(ISD) working copy in the OpenSpec `feature` workflow. The judge uses it to
score a candidate ISD. It is intentionally explicit and self-contained: do not
assume the judge has any ambient project knowledge beyond this standard, the
inlined SOURCES, and the CONTEXT (the source packet, the upstream discovery /
design / proposal artifacts, the APPROVED grooming ledger, this change's
delta specs — the `specs.<flow>/…` blocks — the `isd-scaffold` block, and the
`isd-diagram-request` block).

## Diagram requests (read with the diagram rules below)

The `.context/isd-diagram-request` block in CONTEXT (a user-request source,
labelled "initial requirements source") is the user's explicit ask for a
specific diagram in a NAMED sub-section (here: an activity-style `flow` diagram
under "Geocoding & Proximity" showing the resolvable-vs-unresolvable branch),
IN ADDITION to the always-present Integration Flow Overview sequence. When such
a request is in CONTEXT, the candidate is EXPECTED to carry that
` ```drawio:flow:<id> ` block (plus its `![…](./assets/<id>.png)` image) in the
requested section. Its ABSENCE is `missing_evidence` (a soft miss), never a
critical — diagrams stay soft here. If this block is absent from CONTEXT, no
per-section flow diagram is expected at all. As with every diagram, do NOT score the
diagram's INTERNAL spec quality (node/participant choice, layout, message
nesting, text budgets, activity-vs-plain choice) — a separate drawio eval owns
diagram correctness; score ONLY that a block of the requested type + its image
is present in the right section, and its YAML body is never a
`contract_fidelity` / `no_fabrication` finding.

## The scaffold-then-enrich model (read this FIRST)

- The ISD's "## In Scope Functional Requirements" section — one `### <WBS> — <name>`
  block per in-scope integration requirement, each with a `#### User Story`,
  `#### Acceptance Criteria`, and `#### Key behaviours` sub-section (NO
  Visualization) — is a DETERMINISTIC, script-owned projection produced by
  `generate-integration-artifacts.mjs`: the User Story and Acceptance Criteria are
  copied VERBATIM from the covering proposal slice, and Key behaviours is a TABLE
  (`# | Scenario | GIVEN | WHEN | THEN`) built from the covering spec
  requirement(s)' full scenario set — when several requirements share the WBS the
  script opens each one's rows with a bold sub-header row carrying that
  requirement's name. The EXACT projection is given in CONTEXT as
  the `isd-scaffold` block (the script's output, with the non-owned sections left
  as template stubs). This section sits directly AFTER "## Integration
  Specification" and OPENS with the same script's projection of the proposal's
  re-usability legend — the bold `**Re-usability Legend**` block ahead of the first
  requirement block, script-owned like the three sub-sections, and a bold label
  rather than a heading by design.
- The candidate is the ENRICH step: it must reproduce that whole section VERBATIM
  and fill only the NON-OWNED sections — the General ISD Information card, the
  approval roster, the Reference Materials, the agnostic Overview, the
  Integration Specification, the governance sub-section, and Out Of Scope /
  Deferred / Change Requests.
- Therefore the candidate is judged on: (1) did it PRESERVE the read-only In-Scope
  projection exactly (`read_only_integrity`), and (2) is the ENRICHMENT correct,
  grounded, and contract-faithful (the remaining dimensions). The LLM no longer
  authors the In-Scope User Story, Acceptance Criteria, or Key behaviours — do NOT
  score their WORDING, shape, grouping, or scenario selection as if it did; score
  ONLY whether they match the `isd-scaffold`.
- The shopper "As a shopper…" User Story is EXPECTED inside this script-owned
  section (it states why the integration exists) and is NEVER a finding there.
  The consumer-user-story ban applies ONLY to the agnostic Overview and
  the Integration Specification — those stay story-free.

## What an ISD is (essence)

- The ISD is the **engineering contract of an integration change** — the
  document a developer implements from, QA tests from, and business signs
  off. It lives as a working copy at
  `openspec/changes/<name>/isd/<service-integration>.md`;
  `/opsx:propose-submit` later publishes it to Confluence for approval.
- The document type follows the change's **delivery scope**: an
  `integration` change publishes an ISD; a `functional` change publishes an
  FSD — never both. An ISD therefore describes a service capability, not a
  shopper-facing feature.
- The ISD is **capability-agnostic**: it is named and scoped for the service
  capability itself (e.g. a maps/geocoding capability), not for the first
  feature that consumes it. Consumer features appear only as context ("the
  first consumer") and in the out-of-scope boundary — never as owned scope,
  and never as user stories or consumer RQ/WBS codes.
- The ISD is a **projection, not a decision layer**: its integration points
  come from the approved offer (the lock's integration obligations and the
  Delivery Breakdown), its integration contract comes from the design's
  Integration Specification (endpoints, parameters, data mapping, data
  flow), its behaviour bounds restate the delta specs, and the approved
  grooming decisions are its law (call placement, key policy, release
  prerequisites). It adds NO scope, NO contract surface, and re-decides
  nothing. It is generated only after the Grooming Approval Gate passed.
- Unlike an FSD, an ISD is a TECHNICAL document: vendor endpoints, HTTP
  methods, parameter tables, field names, environment URLs, and
  vendor-SDK mechanics are its proper content — when the sources carry
  them. What stays out is the project's INTERNAL machinery: repository
  paths, artifact names, agent commentary, and our own implementation
  wiring (component/file/hook names — those belong to technical-design).
- The ISD shares the FSD chrome skeleton: a `## General ISD Information`
  card, then H3 approval-roster groups (delivering org — ALWAYS titled
  `Astound Approval`, STATIC, never from CONTEXT — then the client group titled
  with the client org name from CONTEXT), then
  a `## Reference Materials` table, then the ISD body. On first generation
  the working copy is fresh: ISD Status `draft`, the delivering-org roster
  seeded FA/FE/BE/QA (`TBC` names) plus a client group, every approval row
  `not started`, no Confluence URL or Page ID, and an EMPTY Open Questions
  section (everything pre-build was decided at grooming). The Document
  Change Log footer is appended by the publish chrome — the candidate
  never authors a Change History section.

## Invariants the candidate MUST satisfy

1. **Capability scope (closed law).** The `## Overview` is a short
   executive-summary narrative (no SHALL, no table) that orients the reader,
   and every in-scope integration obligation of the Requirements Lock is
   single-homed in exactly one authoritative place — an In-Scope Acceptance
   Criterion, an Integration Specification section, a Key Governance &
   Privacy bullet, or an Out Of Scope boundary — with the Overview restating
   none of them. Nothing more, nothing less: no consumer/functional scope
   (UI, page behaviour, navigation, accessibility, copy) and no other
   service's scope appears as an owned spec section or governance bullet.
   Consumer features and sibling services appear ONLY under Out Of Scope /
   Limitations with their owning change named, or as brief consumer context.
2. **Contract fidelity.** Every endpoint, channel, parameter, field, and
   mapping in the Integration Specification traces to the design's
   Integration Specification (or another cited source): method and path as
   sourced, request/response fields with their sourced meanings, data-flow
   order as designed. Nothing contract-shaped is invented — no extra
   endpoints, fields, headers, environments, example payloads, or values
   the sources do not state.
3. **Decision fidelity.** Every approved grooming decision is encoded
   exactly, with its bound intact: the approved call placement (e.g.
   server-side geocoding — the vendor key never appears in a
   browser-visible request), the approved key policy (domain-restricted to
   the storefront origins; quota and billing guardrails on the client's
   account), release-gating prerequisites (the restricted key issued by the
   client's platform owner before first release), and privacy bounds
   (shopper coordinates transient — never persisted, never sent to
   analytics). No rejected or undecided alternative is asserted; nothing
   weakens an approved bound.
4. **Behaviour fidelity.** The behaviour statements in the Integration
   Specification and the In-Scope Acceptance Criteria restate what this
   change's delta specs state (lazy-loadable entrypoint semantics, error
   result on unresolvable input, no consumer-held key) — compressed but
   not contradicted, weakened, or extended.
5. **Dev-document hygiene.** No internal repository paths or artifact
   references (`openspec/…` paths, discovery/design/proposal/grooming/
   specs/technical-design/tasks as documents), no agent or process
   commentary, no project-context wrappers around references (vendor doc
   URLs appear as plain links), and no implementation wiring of our own
   codebase. Vendor/technical vocabulary is welcome; the subject vendor's
   name is expected throughout. Source documents in Reference Materials are
   resolved to the canonical client-facing link CONTEXT holds (link registry /
   project-context origin / wiki/Drive); a bare title is acceptable ONLY when
   CONTEXT carries no link for it and the row flags that — a bare title while
   CONTEXT holds a resolvable link is a finding, and a fabricated URL is a
   finding.
6. **Fresh working-copy state.** ISD Status `draft`; the delivering-org
   roster carries FA/FE/BE/QA (seeded `TBC`) and a client group, every
   approval row `not started`; no Confluence URL, Page ID, or publication
   claim; Open Questions carries no open item (the template's review sentence
   plus its bare table header, no rows, is the fresh state); no authored
   Change History section. ORG
   FIREWALL: the delivering-org "Astound Approval" roster holds delivering-org
   (Astound) people only — a CONTEXT person whose org is the CLIENT (even with a
   delivery-sounding title such as "FE lead"/"platform dev"/"architect") never
   fills an Astound FA/FE/BE/QA row — and the client group holds ONLY the client
   approver(s) (e.g. the PO). A client/non-Astound person seated in the Astound
   roster, or the client group padded with non-approver client stakeholders, is
   beyond the fresh state.
7. **No fabrication.** No invented WBS/codes (an empty WBS Code cell is
   correct when the sources carry none — inventing one is the violation),
   no invented tracker keys, URLs, endpoints, parameters, magnitudes
   (quotas, TTLs, limits), test accounts, sandbox hosts, people, or
   approvals. Everything named traces to CONTEXT.
8. **Template & structure conformance.** The H1 page title ends with " ISD"
   (e.g. "Cartora Maps Integration ISD") and the header card H2 is
   `## General ISD Information` — a missing suffix is at most a warning (the
   publisher can append the token). Template section order and heading
   levels; the General ISD Information card and H3 approval-roster groups
   (role | status | name) with badge values, then a Reference Materials
   table; the Overview as a short prose narrative (no table); Integration
   Specification with a flow overview and one sub-section per integration
   point in data-flow order; optional sections (How To Test) omitted when
   nothing grounded fills them; `None.` where a section is empty. A
   deterministic code gate already enforces exact section presence and order
   against the golden; score this dimension for heading and field fidelity
   only.
9. **Read-only integrity (closed law).** The "## In Scope Functional
   Requirements" section reproduces the `isd-scaffold` block VERBATIM: every
   `### <WBS>` block present, in the same set and order, each with its
   `#### User Story` (unchanged sentence), `#### Acceptance Criteria` (list and
   any `#####` group sub-headings unchanged — no criterion added, dropped,
   reworded, reordered, re-grouped), and `#### Key behaviours` TABLE unchanged
   (same rows, same `# / Scenario / GIVEN / WHEN / THEN` cells — including its bold
   sub-header rows `| **<name>** |  |  |  |  |`, each of which is a covering spec
   requirement's NAME, so an edited, added, dropped, or re-ordered sub-header row is
   as much a violation as an edited scenario row, and a table the scaffold projects
   flat stays flat), and NO Visualization sub-section. Any MEANINGFUL edit to these is a
   violation, even a "nicer" rewrite — corrections belong upstream (proposal /
   spec). Compare SEMANTICALLY: IGNORE pure typographic re-encoding — hyphen /
   dash variants (`-` vs `‑` vs `–`), straight vs curly quotes, other Unicode
   punctuation, and whitespace / line-wrapping are NOT violations. Fire ONLY on a
   content change: added / removed / reworded words, an added parenthetical or
   qualifier, a dropped / added / re-numbered criterion or scenario row, a changed
   group label, or a moved section.

## Forbidden (each is a critical finding when present)

- An owned specification section or governance bullet for consumer/functional
  scope (page UX, store detail display, filtering, navigation,
  accessibility, shopper copy) or for another service's capability (e.g.
  address autocomplete owned by a different supplier) — the flooding an
  agnostic ISD must not do.
- A dropped integration obligation: an in-scope lock requirement single-homed
  nowhere — absent from the In-Scope Acceptance Criteria, the Integration
  Specification, Key Governance & Privacy, and Out Of Scope.
- A contradiction or weakening of an approved decision: client-side calls
  where server-side was approved, the vendor key reaching the browser for
  the server-side path, an unrestricted or agency-billed key, a dropped
  release prerequisite, a privacy bound relaxed.
- A fabricated contract element: an endpoint, method, parameter, field,
  header, environment URL, example payload value, quota, TTL, or test
  fixture the sources do not state.
- An invented WBS/code, tracker ticket, person, approver, date, Confluence
  URL, or Page ID.
- Internal leakage: repository/artifact paths or names, agent commentary,
  project-context wrappers, or our own implementation wiring
  (component/file/hook names).
- Working-copy state beyond fresh: a status other than `draft`, a
  started/approved approval row, a claimed publication, an OPEN open
  question, an authored Change History section, or an approval roster padded
  beyond the fresh state — a client/non-Astound person seated in the Astound
  roster, or non-approver client stakeholders added to the client group.
- A mis-attributed author: a spreadsheet-/tracker-provenance name (a CSV
  `Created By` / `Updated By` / `Approved By`) or a client-org person promoted
  into Author/Owner (the delivering-org SA lead is the author). A `TBC` or blank
  Author/Owner is NOT this finding.
- Consumer user stories ("As a shopper…" / persona-goal-benefit sentences)
  in the agnostic Overview or the Integration Specification PROSE — those
  stay story-free. This bans a persona-goal-benefit SENTENCE in prose; a
  diagram participant/actor label named for the consumer/shopper is participant
  choice (see allowance d), never a user story and never a finding. (The
  script-owned "## In Scope Functional Requirements" section legitimately
  carries the shopper Story; a story THERE is required, not a finding.)
- ANY change to the "## In Scope Functional Requirements" section versus the
  `isd-scaffold` block: a reworded/moved story, an added/dropped/reworded/
  reordered acceptance criterion or `#####` group, a Key-behaviours scenario row or
  bold `| **<requirement name>** |` sub-header row added/dropped/reworded/renamed/
  re-numbered/re-ordered, a dropped table, an added Visualization, a
  dropped/renamed/re-keyed block, or a dropped/re-worded/re-ordered/extended/
  trimmed/moved `**Re-usability Legend**` entry. (`read_only_integrity`)
- A missing mandatory element: no Overview narrative, or a missing
  Integration Specification section.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE
place.

- Artifact presence (run FIRST): if the candidate is not an ISD working
  copy at all — a refusal or STOP message, a status report, planning notes,
  or an empty document — record the critical finding `no_artifact`, score
  EVERY dimension 0.0, and stop. A missing-upstream complaint is never a
  valid candidate: the upstream artifacts are given in CONTEXT.
- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A
  difference from GOLDEN, or something GOLDEN omits, is NEVER by itself a
  finding. Every finding MUST quote the candidate text AND the STANDARD
  rule or CONTEXT fact it violates — especially for prose phrasing, table
  cell wording, section grouping, and the ordering of governance notes,
  which are open-world.
- Closed-world vs open-world. Endpoints, methods, parameters, fields,
  mappings, environment URLs, codes, tracker keys, people, approved
  decisions and their bounds, magnitudes, and URLs are CLOSED-world: they
  must trace to CONTEXT. Overview wording, prose compression,
  governance-note placement (inline per integration point vs a
  consolidated governance section), diagram inclusion, and Integration
  Specification note wording
  are OPEN-world synthesis: judge them for grounded, faithful compression,
  not for verbatim presence.
- Read-only integrity sweep (closed rule — run BEFORE scoring
  `read_only_integrity`): find the "## In Scope Functional Requirements" section
  and compare it to the `isd-scaffold` block. For each `### <WBS>` block, match
  the same WBS in the scaffold and compare the three sub-sections SEMANTICALLY.
  First normalize away pure typographic re-encoding — hyphen/dash variants (`-`
  vs `‑` vs `–`), straight vs curly quotes, other Unicode punctuation, and
  whitespace/line-wrapping are NOT differences (gpt-oss commonly re-encodes a
  hyphen; that is not a violation). Then fire a critical ONLY on a CONTENT
  difference: a reworded/moved story; an added/removed/reworded word or an added
  parenthetical/qualifier in a criterion; an added/dropped/reworded/reordered
  criterion or `#####` group; a changed/added/dropped/re-numbered Key-behaviours
  table row — scenario rows AND the bold `| **<requirement name>** |` sub-header
  rows the scaffold projects (a group row the candidate invents, renames, drops, or
  re-orders is the same class of violation; the label is owned upstream by the spec
  requirement's name); an added Visualization; or a dropped/renamed/re-keyed block.
  Compare the `**Re-usability Legend**` block that opens the
  section the same way — same entries, same wording, same order as the scaffold's.
  Quote the candidate text and the scaffold text it should have
  matched, and confirm the difference is a content change, not typographic
  re-encoding, before firing. Reproducing the scaffold (modulo typography) —
  INCLUDING its shopper "As a shopper…" User Story — is REQUIRED and is never
  itself a finding.
- Scope sweep (closed rule — run BEFORE scoring `capability_scope`): list
  the in-scope integration obligations from THIS flow's Requirements Lock
  and Delivery Breakdown. Map each to its single home — an In-Scope
  Acceptance Criterion, an Integration Specification section, a Key
  Governance & Privacy bullet, or an Out Of Scope boundary; then scan the
  Overview, the Integration Specification, and the governance sub-section
  for scope belonging to consumer features or other services. Fire
  a critical only on: an obligation single-homed nowhere, or an owned
  section for foreign scope. Section COUNT, naming, and grouping are never
  findings when the mapping is total. The scope law judges the Overview,
  the Integration Specification, and the governance sub-section ONLY: an
  item listed under
  `## Out Of Scope / Limitations` is by definition NOT owned scope —
  listing a grounded boundary item there (a consumer behaviour, a sibling
  service, an operational concern from the packet) is exactly what that
  section is FOR and is never a scope finding. An out-of-scope bullet is
  a finding only when it asserts a fact absent from CONTEXT (then it is
  `no_fabrication`).
- Contract sweep (closed rule — run BEFORE scoring `contract_fidelity`):
  test every endpoint, parameter, field, mapping, and environment value
  against the design's Integration Specification and other cited sources.
  Fire on invented or altered contract elements — not on omitted optional
  detail (an omitted vendor doc row or a thinner parameter description is
  at most `missing_evidence`). Fabrication lives in the VALUE, not the
  layout: judge each cell/statement individually. A table that merely
  RECASTS facts the sources state (the SDK bundle channel, the key
  supplied at init) into rows is at most a warning for format drift; a
  critical requires at least one cell or claim whose content the sources
  do not state (an invented host, header, field, magnitude, or behaviour
  such as a made-up fallback message). Cite that specific cell as the
  evidence, and record the finding ONCE (`contract_fidelity` for
  contract-shaped inventions) — not again under `no_fabrication`.
- Decision sweep (closed rule — run BEFORE scoring `decision_fidelity`):
  test the candidate's placement/key/billing/privacy/release statements
  against the APPROVED grooming decisions. Fire on contradiction,
  weakening, or a resurrected alternative — not on restating a decision in
  different grounded words.
- Hygiene sweep (closed rule — run BEFORE scoring `dev_document_hygiene`):
  scan the whole document, including every table cell, for
  repository/artifact paths and names, agent commentary, project-context
  wrappers, and our-code implementation tokens. Vendor names, vendor
  endpoints/fields, HTTP verbs, and protocol vocabulary are NOT hygiene
  hits in an ISD.
- One defect, one primary critical. Classify by the actual fault:
  - any drift in the "## In Scope Functional Requirements" section vs the
    `isd-scaffold` (reworded story/criterion/scenario, dropped/added/re-keyed
    block, added Visualization) -> `read_only_integrity`;
  - foreign scope owned, or an uncovered obligation -> `capability_scope`;
  - an invented/altered endpoint, field, parameter, or environment ->
    `contract_fidelity`;
  - a contradicted/weakened approved decision or resurrected alternative ->
    `decision_fidelity`;
  - spec behaviour contradicted or extended -> `behaviour_fidelity`;
  - internal paths, artifact names, commentary, our-code wiring ->
    `dev_document_hygiene`;
  - a non-fresh status/approval/publication/open-question state ->
    `working_copy_state`;
  - an invented code, ticket, person, magnitude, URL, or test fixture ->
    `no_fabrication`.
  Do not raise the same underlying claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension: (a) an empty
  WBS Code cell ("—") when the sources carry no code for the capability;
  (b) the consumer feature's tracker in Jira Reference with a note that no
  dedicated ticket exists — it is the only tracker the sources carry; (c)
  naming the consumer feature as "the first consumer" (context, not
  scope); (d) the Integration Flow Overview rendered as a
  ` ```drawio:sequence:<id> ` block immediately followed by its
  `![<title>](./assets/<id>.png)` image line, restating the design's data
  flow — this is the REQUIRED form of the diagram, never a finding. Do NOT
  judge the diagram's INTERNAL spec quality (participant choice, message
  nesting, arrow direction, layout, text budgets): a separate drawio eval
  owns diagram correctness. Score ONLY whether the block + image of that
  shape is present; its YAML body (participant ids/titles/`kind`, note text,
  and arrow/message text) is NOT judged content and is NEVER a finding under
  ANY dimension — not `capability_scope`, `decision_fidelity`,
  `contract_fidelity`, or `no_fabrication`. A participant named for the
  consumer/shopper (e.g. `{ id: shopper, kind: actor }`) is diagram
  participant choice, not owned scope or a consumer user story; an arrow whose
  text echoes an endpoint/param the design's own data-flow diagram carries is
  diagram body, not an invented or leaked contract. Only PROSE outside the
  fenced diagram block is scored. The SAME allowance covers a
  ` ```drawio:flow:<id> ` block + image inside an Integration Specification
  sub-section when the `.context/isd-diagram-request` block asks for one there
  (or when the candidate adds one to reinforce genuine branching/structure): expected and
  never a finding, and its internal quality is likewise not judged; (e)
  parameter/mapping TABLES instead of raw example payloads when the
  sources carry tables — a fabricated JSON example would be the violation,
  not the table; (f) a consolidated "Key Governance & Privacy" section
  instead of per-endpoint auth notes (or vice versa); (g) `None.` sections
  (Deferred Requirements, Change Requests), an Open Questions section
  carrying the template's review sentence and its bare table header with no
  rows, and NO authored Change History section
  (the chrome appends the Document Change Log footer);
  (i) people named in CONTEXT used as Author (e.g. the SA lead); (j) a
  grounded deferral note that a value is settled downstream (e.g. "cache
  lifetime is settled in the technical design") — deferring an undecided
  magnitude is correct, asserting one would be fabrication; (k) an Overview
  that orients WITHOUT restating each obligation — omitting bounds that live
  in the Acceptance Criteria, Integration Specification, or Key Governance &
  Privacy is the REQUIRED shape, not a gap; (l) the retired `…/v1/geocode` ellipsis exactly as the
  design writes it — the sources elide the host, so must the ISD; (m) a
  delivering-org approval roster seeded FA/FE/BE/QA with `TBC` names plus a
  client group, every row `not started` — the REQUIRED fresh state, never
  "incomplete" or "invented approvers". The Astound rows carry delivering-org
  people or `TBC`, and the client group carries ONLY the client approver(s)
  (e.g. the PO) or `TBC`: a correctly org-seated name (an Astound person in an
  Astound row, the client PO in the client group) is never a finding, while a
  client/non-Astound person in an Astound row or a non-approver client
  stakeholder padded into the client group IS one (org-firewall); (n) the "## In Scope Functional
  Requirements" section reproduced VERBATIM from the `isd-scaffold` — including
  its shopper "As a shopper…" User Story and its NO-Visualization shape — is the
  REQUIRED outcome, never a finding under ANY dimension.
- Omission is not a critical — under ANY dimension — EXCEPT the scope law
  (an uncovered in-scope integration obligation IS critical), a missing
  mandatory element (no Overview narrative or a
  missing Integration Specification), and any drift in the script-owned
  "## In Scope Functional Requirements" section (critical under
  `read_only_integrity`). Everything else missing (an omitted
  optional How To Test, a thinner Integration Specification note, an absent
  vendor-doc reference row, an omitted overview diagram — the `drawio:sequence` block, or
  an omitted per-section `drawio:flow` block the `.context/isd-diagram-request`
  block asked for) is `missing_evidence` or a warning.
- Authority check for every finding: if the only authority you can cite is
  "the GOLDEN contains this", the finding is invalid — DROP it. A valid
  finding cites a STANDARD rule or a CONTEXT fact the candidate violates.
- Completeness does not offset a violation, and fluency is not
  correctness. Inspect each row, section, and status cell on its own
  merits regardless of how polished the document is.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by
the dimension id below. For each dimension, 1.0 = fully satisfied with
evidence, ~0.5 = partially satisfied or minor gaps, 0.0 = violated or absent.

- `read_only_integrity` — the "## In Scope Functional Requirements" section
  reproduces the `isd-scaffold` VERBATIM: the opening `**Re-usability Legend**`
  block unchanged, every block present in the same set and
  order, each block's User Story, Acceptance Criteria, and Key behaviours
  unchanged (no edit/reword/re-shape/re-group, no added/dropped/re-numbered
  criterion, group, scenario table row, or bold `| **<requirement name>** |`
  sub-header row), NO Visualization added.
- `capability_scope` — the closed scope law holds: agnostic capability
  framing, a short Overview narrative (no SHALL, no table), every in-scope
  obligation single-homed across the In-Scope Acceptance Criteria,
  Integration Specification, Key Governance & Privacy, or Out Of Scope, no
  foreign scope owned anywhere.
- `contract_fidelity` — endpoints, parameters, fields, mappings, and flow
  order trace to the design's Integration Specification; nothing
  contract-shaped invented or altered.
- `decision_fidelity` — approved call placement, key policy, quota/billing
  ownership, release prerequisite, and privacy bounds encoded exactly; no
  rejected or undecided alternative asserted.
- `behaviour_fidelity` — the Integration Specification behaviour and the
  In-Scope Acceptance Criteria restate the delta specs faithfully (lazy-load
  semantics, error results, no consumer key); nothing contradicted or extended.
- `dev_document_hygiene` — no internal repo/artifact references, agent
  commentary, project-context wrappers, or our-code wiring; vendor and
  protocol vocabulary used appropriately.
- `working_copy_state` — `draft` status, approval roster `not started`
  (delivering-org FA/FE/BE/QA + client group), no publication metadata,
  empty Open Questions, no authored Change History section.
- `no_fabrication` — every code, ticket, endpoint, field, magnitude, URL,
  person, and approval traces to CONTEXT; nothing invented.
- `structure_conformance` — template section order and heading levels,
  header/Approvals shapes and the Overview narrative (prose, no table),
  flow-ordered spec
  sub-sections, optional sections omitted when ungrounded, `None.` rows. The
  Integration Flow Overview, when present, is a ` ```drawio:sequence:<id> `
  block followed by its `![…](./assets/<id>.png)` image; a requested
  per-section diagram is a ` ```drawio:flow:<id> ` block + image inside its
  sub-section — score only that shape (presence + fenced block + image line),
  never the diagram's internal spec correctness.

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it:
  if any item in `critical_findings` is present, `score` MUST NOT exceed
  0.5.
- `pass` is true only when `score >= threshold` (given in the user message)
  AND `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or of invariants
  1-9 (read-only drift in the In-Scope section, a scope-law breach, an invented
  or contradicted contract element, a broken decision bound, internal leakage, a
  non-fresh working copy, fabrication). These block acceptance.
- **warnings** — quality issues that should be fixed but do not make the
  document wrong (e.g. an editorialised Overview sentence, a consumer user-story
  flavour creeping into the Overview or an Integration Specification note while
  the obligations stay correct, a heading-level slip, a section-ordering
  deviation inside the Integration Specification).
- **missing_evidence** — expected content the CONTEXT implies but the
  candidate omits without breaking a law (e.g. an omitted vendor-doc
  reference row, an absent flow diagram, a spec-grounded edge case missing
  from the Integration Specification). Record it ONLY here: it MUST NOT appear in
  `critical_findings` and MUST NOT cap the score.

Always ground findings in concrete evidence. Every critical finding MUST
quote a string that appears VERBATIM in the candidate. Before recording a
critical, locate that exact string (the table row, the cell, the heading,
the status value) in the candidate text; if it is not literally present,
DROP the finding — do not infer, paraphrase, or assume a row or section
exists. For a missing-item finding, name the specific CONTEXT fact (the
lock obligation, the design contract row, the approved decision) the
candidate omits. A finding whose evidence you cannot copy verbatim from the
candidate is a hallucination, not a violation.
