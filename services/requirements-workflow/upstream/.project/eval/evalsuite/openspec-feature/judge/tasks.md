---
schema: openspec-feature
artifact: tasks
context:
  - .context/initial-context
  - grooming/final
  - specs
  - stories
  - fsd
  - isd
  - technical-design
  - planning-decisions
sources:
  - ../../../../templates/openspec/schemas/feature/templates/tasks.md
scoring:
  pass_threshold: 0.7
---

# Tasks Standard — Contract and Rubric

This is the canonical definition of a good `tasks.md` working copy in the
OpenSpec `feature` workflow. The judge uses it to score a candidate. It is
intentionally explicit and self-contained: do not assume the judge has ambient
project knowledge beyond this standard, the inlined SOURCES (the tasks
template), and the CONTEXT (the completed technical-design, this change's delta
specs, the APPROVED grooming ledger, the stories working copy, the FSD/ISD
working copy when present, and the PLANNING DECISIONS block that records the
approval-session outcome).

## What tasks.md is (essence)

- tasks.md is the EXECUTION LAYER and the LAST feature artifact: `/opsx:plan`
  generates it from an APPROVED technical-design as the phase-oriented,
  role-tagged, machine-tracked work queue that `/opsx:apply` runs task-by-task.
- It AUTHORS the low-level implementation contract. For each unit of work it
  records the files, the public interface / signatures (each with a one-line
  description), the concrete steps, and a done-when — the signature-level HOW
  that lets `/opsx:apply` implement with no open questions. This is the DIVISION
  OF LABOUR: technical-design.md owns decomposition + high-level architecture
  (the phases, the Building Block NAMES and responsibilities, the runtime flow);
  tasks.md realises it in code-level detail. Authoring a file-internal signature
  is EXPECTED here and is NOT a fabrication.
- The BOUNDARY it must not cross: it adds NO shopper-facing behaviour beyond the
  specs, and NO scope or phase beyond the technical design. A task's SUBJECT
  (the surface / block / phase it delivers) is SOURCED from the technical design
  and named verbatim; its DETAIL (interface, steps) is AUTHORED here, grounded
  in the platform convention for that surface and within the specs behaviour
  bound. Adding behaviour or scope is the violation; authoring a signature is
  not.
- Each `### P#` is an INDEPENDENT, human-approved increment. tasks.md authors,
  per phase, a `Process` slot (a backlink to Phase Processing Rules + a `tier`),
  a `Verify` PLAN (frozen acceptance + a manual-style scenario + the evidence to
  capture + a verdict), and a `Status`. IMPORTANT: the `Verify` block is a PLAN
  the agent executes at APPLY time — a freshly generated tasks.md describes what
  WILL be checked (Status `pending`), it does NOT contain executed results,
  captured screenshots, or a `Status: approved`. Claiming executed evidence in a
  generated queue is an over-claim, not a merit.
- The `## Phase Processing Rules` block is INVARIANT house-standard boilerplate
  fixed by the template (the per-phase loop through commit and advance, the
  verdicts, the sensitive-tier consequence). It is the SAME in every change;
  score it for PRESENCE only, never for its content, and never flag its wording
  as a finding.
- It is the only artifact with mutable, machine-tracked state: `/opsx:apply`
  flips `- [ ]` -> `- [x]` and the OpenSpec CLI counts checkbox progress. Each
  task's Files / Interface / Steps / Done-when / Trace detail AND the per-phase
  `Process` / `Verify` / `Status` slots are INDENTED `-` sub-content (never
  `- [ ]`), so neither the progress parser nor the `##` structure gate sees them.
- The Planning Gate is PASSED in this environment. The PLANNING DECISIONS
  context block records the approval-session outcome: every CRITICAL Open
  Question in the technical-design is APPROVED by a named human. So the
  technical-design's `per Q#` deferrals are RESOLVED — a task that states the
  DECIDED value (the resolved contract shape, TTL, stubbing strategy) is
  CORRECT, not a fabrication. A surviving `per Q#` placeholder, an Open Questions
  section, or a re-opened decision is the VIOLATION.
- The frame is FIXED; the phases carry the content. The `##` section frame is
  unnumbered and identical for every flow — Solution Overview; Planning Source;
  Story Trace (when stories exist); Prerequisites; Test Setup; Phase Processing
  Rules; Delivery Phases; Verification; Review & Approval. There is NO Frontend /
  Backend `##` section (role is on the task tag), NO Dependency Map, and NO QA
  Handoff section (formal QA is out of pipeline). A functional change is FE-heavy
  (most task tags @FE, with `#### CMS / Page Designer` and `#### Translation
  Sync` sub-areas inside the phases that carry them); an integration change is
  BE-heavy. Execution order is carried ONLY by each phase's `Depends on:` slot
  (consumer -> producer); tasks inside a phase run top-to-bottom.

## Invariants the candidate MUST satisfy

1. **Trace coverage (projection closed law).** Every row of the
   technical-design's Implementation Building Blocks and Verification Strategy
   is delivered by at least one task or check: developer-owned verification ->
   Test Setup / Verification; implementation block -> a task block under the
   phase that delivers it. Every technical-design Delivery Phase is realised as a
   `### P#` block; every in-scope RQ/WBS and every story key is covered. A
   capability/account/data/CMS blocker is recorded in the affected phase's
   `Status` (there is no Blocked Tests section). A dropped block/evidence
   row/phase/RQ/story, or a task with no technical-design basis, is a violation.
2. **Projection & authorship fidelity.** Each task's SUBJECT names the
   technical-design surface / building block / phase it delivers (verbatim) plus
   its RQ/WBS trace. Its authored DETAIL — the Interface / signatures / Steps —
   is a file-internal contract grounded in the platform convention for that
   surface and coherent with the technical-design's responsibility for that
   block; it stays WITHIN the specs and approved-grooming behaviour bounds
   (nothing contradicted, weakened, or extended; a task may strengthen a
   privacy/security bound, never relax one). The violation is adding
   shopper-facing behaviour absent from specs, adding a scope/phase the
   technical design does not carry, or contradicting a bound — NOT authoring a
   file-internal function/type/prop/constant signature the technical design left
   to implementation.
3. **Planning-gate state.** The queue is built from the APPROVED technical
   design: every `per Q#` deferral is resolved to its recorded decision and the
   decided value appears in the affected task. There is NO Planning Open
   Questions section, NO surviving `per Q#` placeholder, NO re-opened or
   re-decided planning question, and NO invented approver/session.
4. **Parser & role conformance.** Every parseable task is a line starting at
   column 0 with `- [ ]` (never indented, never inside a table) and carries
   exactly ONE role tag: @FE, @BE, or @SHARED, with its detail and the per-phase
   `Process` / `Verify` / `Status` slots INDENTED `-` sub-content beneath the
   phase. There is NO `@QA` checkbox task. The role matches where the code lives
   (storefront/nav/CMS-rendering -> @FE; SFCC cartridge / custom-SCAPI / hooks /
   jobs / services / caches / metadata / Business-Manager -> @BE; prerequisite /
   shared verification / genuinely-both -> @SHARED).
5. **Implementation actionability.** Every implementation task carries enough
   for `/opsx:apply` to execute with no open question: the Files it creates or
   edits; a usable Interface (the exported symbols and their signatures the
   consuming code and unit test bind to, each described in one line) OR an
   explicit note that it is pure wiring/content/config exposing no callable
   symbol; the ordered Steps; and a Done-when that names an objective,
   developer-observable completion state. A hollow task — a title with no files,
   no interface and no justification, or no steps/done-when — is a violation.
   Prerequisites, Test Setup lines, and Verification checks are one-liners by
   design and are exempt.
6. **Phase sequencing.** Delivery work is `### P#` phase blocks, one per
   technical-design Delivery Phase, each carrying its slots — `Story` (when
   stories exist), `Delivers` (the observable/callable increment + RQ/WBS),
   `Areas`, `Depends on`, `Process`, `Jira`. Each `Depends on:` names the
   producer phase(s) the phase consumes an in-change artifact from, direction
   CONSUMER -> PRODUCER, following the technical design's Runtime Flows; a phase
   that consumes nothing another phase produces has `Depends on: —`. There is no
   Dependency Map and no inline `[depends:]`. Consuming an EXTERNAL vendor
   capability/SDK is a Prerequisite or a `Status: blocked` note, never a phase
   dependency.
7. **Phase processing (per-phase gate).** The `## Phase Processing Rules` block
   is present and reproduced VERBATIM from the template (invariant; score
   presence only). Every phase carries a `Process` slot (a backlink to that
   section + `tier: standard | sensitive`) and a `Verify` PLAN with: frozen
   Acceptance (derived from the phase's Delivers + specs bound); the Unit tests
   that prove it; a manual-style Scenario the agent will walk (an @FE phase via
   `playwright-cli` navigate/act/screenshot; an @BE phase via a call capturing
   request/response); the Evidence to capture (FE: screenshots of key states +
   clean console; BE: request/response + logs + cache); a Verdict. A
   `tier: sensitive` phase (surface touching a vendor key/credential, shopper
   coordinates/geolocation, payments, or PII) MUST force the Verdict to
   SHIP_WITH_HUMAN_REVIEW and add a security pass on that bound. `Status` is
   present (`pending` in a fresh queue; a known blocker recorded as
   `Status: blocked — … (ref 0.x / P#)`). The Verify is a PLAN — executed
   results / captured screenshots / a `Status: approved` in a freshly generated
   queue are an over-claim, not a merit.
8. **Solution Overview.** A leading `## Solution Overview` gives ONE concise
   prose paragraph — what the change delivers, what it consumes-not-builds, and
   the delivery shape (N phases) — projected from the upstream overviews
   (discovery/proposal Executive Summary, technical-design Overview). It is
   orientation, not a bullet in/out-of-scope list, not a phase re-listing, and it
   invents no scope beyond the upstream.
9. **QA / verification boundary.** The per-phase `Verify` (agent self-verification
   — a live manual-style scenario via `playwright-cli`/request-capture, per
   `verification-standards.mdc`) is IN this pipeline. Developer-owned automated
   checks (unit suites, ESLint, `tsc --noEmit`, a grounded bundle/perf check) are
   parseable @FE/@BE/@SHARED tasks. Formal functional testing — authored
   automated E2E suites, manual QA, accessibility audit, UAT — is QA-owned, OUT
   of this pipeline, and NOT queued as a task; there is no QA Handoff section. A
   blocker is a `Status: blocked` note, not a section.
10. **No fabrication.** RQ/WBS codes, story keys, people, tracker ids, vendor
    contract facts, magnitudes, and the resolved planning-decision VALUES (a
    decided endpoint path, response field, TTL, quota the technical-design or
    PLANNING DECISIONS carries) are closed-world: they must trace to CONTEXT and
    must not be re-invented with a different value. Task IDs, phase numbering and
    count, task granularity/grouping, wording, AND authored FILE-INTERNAL symbols
    (a function/type/prop/constant name and signature the technical design did not
    spell out, and file paths under a named building block) are the artifact's own
    synthesis — judge them for grounding and consistency, never for verbatim
    presence in CONTEXT. Executed Verify results in a fresh queue are a fabrication.
11. **Structure conformance.** The fixed, unnumbered `##` section frame in the
    template's order (Solution Overview first; Phase Processing Rules present; no
    QA Handoff); the `### P#` phase-block slots; the Planning Source / Story Trace
    table columns. A deterministic code gate enforces exact `##` presence/order
    against the golden; score this dimension for heading and table-shape fidelity
    only.

## Forbidden (each is a critical finding when present)

- A dropped Implementation Building Block row, Verification Strategy row,
  Delivery Phase, in-scope RQ/WBS, or story key (nothing delivering it); or a
  task, RQ, or story key with no basis in CONTEXT (phantom).
- Out-of-boundary authorship: a task that adds shopper-facing behaviour absent
  from specs, adds a scope/phase the technical design does not carry, or
  contradicts/weakens an approved bound (holiday hours in v1, a temporary
  redirect, an eagerly-loaded map, a browser-visible server key, coordinates
  persisted or sent to analytics). Authoring a file-internal interface/signature
  the technical design left to implementation is NOT this finding.
- A non-actionable implementation task: no Files; or no Interface AND no
  pure-wiring justification; or no Steps / no Done-when.
- A phase-processing breach: the `## Phase Processing Rules` block missing or
  materially altered from the invariant template text (not reproduced verbatim);
  a phase missing its `Process` slot, `Verify` block, or `Status`; a hollow
  `Verify` (no frozen acceptance, or no scenario, or no verdict); a
  `tier: sensitive` phase whose Verdict is not SHIP_WITH_HUMAN_REVIEW or that
  lacks a security pass; or executed results / captured screenshots / a
  `Status: approved` fabricated into a freshly generated queue.
- A missing `## Solution Overview`, or a Solution Overview that invents scope
  beyond the upstream or is a phase re-listing rather than a concise projection.
- A planning-gate breach: an Open Questions section, a surviving `per Q#`
  placeholder, or a re-opened/re-decided planning question.
- A parser/role breach: a parseable task with no role tag or more than one; a
  `@QA` checkbox task; a `- [ ]` checkbox inside a table or indented so the
  OpenSpec parser misses it; a per-phase slot promoted to a `- [ ]`.
- A phase-sequencing breach: a `Depends on:` that names an absent phase, is
  INVERTED (producer made to depend on its consumer), or is FABRICATED (onto a
  Prerequisite, a Test Setup line, a Verification check, or an external vendor
  seam).
- Queuing formal QA as dev work: an authored automated E2E suite, a manual-QA /
  accessibility-audit / UAT task as a `- [ ]`, or a resurrected `## QA Handoff` /
  Blocked Tests section.
- A fabricated closed-world fact: an invented RQ/WBS/story key, person, tracker
  id, URL, vendor fact, magnitude, or a resolved contract value (endpoint, field,
  TTL, quota) that differs from the technical-design / PLANNING DECISIONS. An
  authored file-internal signature is NOT this finding.
- Internal leakage: agent/process commentary, project-context wrappers, or
  eval-harness references.
- A missing mandatory element: no Solution Overview, no Phase Processing Rules
  block, no `## Delivery Phases` with any phase block (an empty queue), or — when
  stories exist — no Story Trace.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE place.

- Artifact presence (run FIRST): if the candidate is not a tasks.md working copy
  at all — a refusal or STOP message, a status report, an onboarding summary, a
  technical-design copy, or a document with no phase blocks and no tasks — record
  the critical finding `no_artifact`, score EVERY dimension 0.0, and stop. A
  missing-upstream complaint is never valid: the technical-design, specs,
  stories, and approved ledger are given in CONTEXT.
- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A difference
  from GOLDEN — different task IDs, phase numbering or count, task granularity, a
  different but coherent function signature or file path, a different Verify
  scenario, different wording — is NEVER by itself a finding. Every finding MUST
  quote the candidate text AND the STANDARD rule or CONTEXT fact it violates.
- Golden-anchor filter (apply to EVERY candidate finding): name the authority the
  finding rests on. If that authority is a GOLDEN task, phase, interface,
  scenario, or row the candidate lacks — and no CONTEXT fact or STANDARD law
  independently REQUIRES it — the finding is invalid: drop it or record it as
  `missing_evidence`, never as a critical. Only the projection closed law and a
  missing mandatory element may be critical omissions, and both cite CONTEXT.
- Closed-world vs open-world. RQ/WBS codes, story keys, behaviour bounds,
  approved decisions and their resolved contract VALUES, people, tracker ids,
  vendor facts, and magnitudes are CLOSED-world: they must trace to CONTEXT with
  the same value. Task IDs, phase numbering/count, granularity, the AUTHORED
  file-internal interface, file paths under a named building block, the Verify
  scenario/acceptance wording, step wording, and phase-dependency phrasing are
  OPEN-world synthesis: judge them for grounded, consistent, traceable authoring
  — never for verbatim presence anywhere.
- Coverage sweep (run BEFORE `trace_coverage`): list every Implementation
  Building Block row and every Verification Strategy row in the CONTEXT
  technical-design, plus every Delivery Phase, the in-scope RQ/WBS, and the story
  keys. Diff the candidate's phases / tasks / checks against them. Fire a critical
  only on: a delivered block/evidence row/phase/RQ/story with nothing delivering
  it, a phantom task/RQ/story, or a task for scope the approved lock defers. Task
  COUNT, IDs, granularity, and phase count are never findings when coverage is
  total. A blocker recorded in `Status` (not a section) is CORRECT. A finding
  claiming phases are BUNDLED, merged, or missing MUST first enumerate the
  candidate's `### P#` headings verbatim and map each to the technical-design
  Delivery Phase it realises; a `Delivers` line listing several RQ/WBS codes is
  not evidence of bundling, because an RQ may repeat across phases (pattern (i)).
- Solution-overview sweep (run BEFORE `solution_overview`): confirm a leading
  `## Solution Overview` exists and is a concise projection of the upstream
  overviews (what is delivered + consumed-not-built + N phases). Fire on: absence;
  invented scope not in the upstream; or a bullet in/out-of-scope list / phase
  re-listing instead of a concise paragraph. Different wording is not a finding.
- Phase-processing sweep (run BEFORE `phase_processing`): confirm the
  `## Phase Processing Rules` block is present and matches the invariant template
  text (presence — do NOT grade its wording). Then for each `### P#` confirm a
  `Process` slot (backlink + `bounds:` + tier) and a `Verify` PLAN carrying frozen
  Acceptance + a manual-style Scenario + the Evidence-to-capture + a Verdict, and
  a `Status`.
  Score the bound BEFORE the label, per phase, as a closed set. Read what the
  phase's own tasks DO — their Steps, Files, and Interface, not their titles — and
  record which of `vendor-credential`, `shopper-coordinates`, `payment-instrument`,
  `personal-record` any task touches, then compare with the emitted `bounds:`.
  Decide by behaviour: a task that defines a service credential or its
  Business-Manager metadata, loads a vendor key, or requests / receives / forwards
  a device-derived position IS a touched bound even when nothing in its title says
  so; consuming an already-published id or a business location's public details is
  NOT. `bounds:` and `tier:` must agree (any named bound -> `sensitive`, `none` ->
  `standard`). Wording of the bound name is not graded, and a missing task-id
  citation or missing "what the task does" clause is `missing_evidence`, not a
  critical.
  Fire a critical on: the block missing/altered; a phase missing Process/Verify/
  Status; a hollow Verify (no acceptance, or no scenario, or no verdict); an
  emitted `bounds:` contradicted by the phase's own tasks, or a `bounds:`/`tier:`
  pair that disagree; a `tier: sensitive` phase not forced to
  SHIP_WITH_HUMAN_REVIEW or lacking a security pass; or FABRICATED executed
  evidence in a freshly generated queue.
  Before firing that last one, establish the TENSE of the line you would quote:
  the template REQUIRES an evidence-to-capture slot, so NAMING the artifact to
  capture ("Screenshots of the default / empty / error states", "Server request/
  response logs showing the key stays server-side") is the mandatory PLAN shape
  and is never this finding. A fabricated-evidence critical MUST quote wording
  asserting the work already happened — past tense ("captured", "scenario
  passed", "console was clean"), an achieved state (`Status: approved`, a verdict
  recorded as reached), or an attached artifact. An imperative or future line
  ("Capture …", "Assert …", "Confirm …", "will be walked") never qualifies. The
  Verify is a PLAN — judge that it is usable and grounded in the phase behaviour,
  NOT whether it was run.
- Planning-gate sweep (run BEFORE `planning_gate_state`): the Planning Gate is
  PASSED here. A resolved value in a task (matching PLANNING DECISIONS) is CORRECT.
  Fire only on: an Open Questions section, a surviving `per Q#`, a re-decided
  question, or a decision value NOT in (or differing from) PLANNING DECISIONS.
- Parser/role sweep (run BEFORE `parser_role_conformance`): scan every `- [ ]`
  line for a single @FE/@BE/@SHARED tag; flag any `@QA` checkbox, any
  untagged/multi-tagged task, and any checkbox inside a table. Then walk the
  phases as a CLOSED SET rather than waiting to notice one: for EACH `### P#`,
  record which of its slots (`Story`, `Delivers`, `Areas`, `Depends on`,
  `Process`, `Verify`, `Status`, `Jira`) sit on a `- [ ]` line instead of as
  indented `-` sub-content — `P# -> promoted: <slot(s)> | none`. Every slot
  recorded as promoted is a parser/role critical, because the OpenSpec progress
  parser would count it as a task. Finally check each task's role against the
  area its Files touch.
- Actionability sweep (run BEFORE `implementation_actionability`): for each
  implementation task confirm Files + a usable Interface (or an explicit
  pure-wiring note) + Steps + Done-when, coherent with the named surface. Fire on
  a hollow task. A signature that differs from the golden's but is coherent is NOT
  a finding — judge USABILITY, not a specific library API you cannot see.
- Phase-sequencing sweep (run BEFORE `phase_sequencing`): confirm the slots
  (Story when stories exist / Delivers / Areas / Depends on / Process / Jira).
  Build the phase order from the technical-design Runtime Flows, then read each
  `Depends on:` and record what it literally NAMES before judging it. ABSENT,
  INVERTED and FABRICATED are claims about a named target, so such a finding MUST
  quote the phase (or Prerequisite / Test-Setup line / vendor seam) the slot names
  and state the producer relation it contradicts. `Depends on: —` names no target
  and therefore CANNOT be absent, inverted, or fabricated: it is correct for a
  phase that consumes nothing in-change, and where the Runtime Flows show a
  producer it omits, record `missing_evidence` — never a critical. Fire a critical
  only on a missing slot, or on a NAMED edge that is absent, inverted, or
  fabricated.
- Authorship / fabrication sweep: verify closed-world facts and behaviour against
  specs, approved grooming bounds, and PLANNING DECISIONS. A task that adds
  shopper behaviour, adds scope/phase, states a resolved contract value differing
  from CONTEXT, or contradicts a bound is a violation; an authored file-internal
  signature grounded in the surface, a thinner task list, or compression is NOT.
  Run the allowed-patterns list BELOW as a GATE before recording any fabrication
  finding: when the string you would quote is a file path, a module / cartridge
  name, a symbol, a type, a prop, or a signature, name the allowed pattern you
  tested it against and why it does not apply. A path or symbol under a building
  block the technical design names is pattern (b) — the expected authoring — and
  is DROPPED, unless CONTEXT states a different value for that same name, which
  makes it a closed-world value mismatch and a correct fire.
- One defect, one primary critical. Classify by the actual fault:
  - a dropped/phantom block/evidence row, phase, RQ, or story -> `trace_coverage`;
  - added shopper behaviour, added scope/phase, a contradicted bound, or a
    subject with no technical-design basis -> `projection_fidelity`;
  - an Open Questions section, a surviving `per Q#`, a re-decided
    question -> `planning_gate_state`;
  - a missing/duplicate role tag, a `@QA` checkbox, a mis-parsed/mis-placed
    checkbox, a wrong role for the area -> `parser_role_conformance`;
  - a hollow / non-executable implementation task -> `implementation_actionability`;
  - a missing phase slot, or a `Depends on:` inverted/dangling/fabricated ->
    `phase_sequencing`;
  - an altered/missing Phase Processing Rules block, a phase missing Process/
    Verify/Status, a hollow Verify, a wrong `tier:` value, a sensitive phase not
    forced to human review or lacking its security pass, or fabricated executed
    evidence -> `phase_processing`. A tier claim belongs to this dimension ALONE:
    never file it under `parser_role_conformance`, `no_fabrication`, or
    `phase_sequencing`, and never let a tier judgement reduce those scores;
  - a missing/invented/over-listed Solution Overview -> `solution_overview`;
  - an invented code, person, magnitude, vendor fact, URL, or a resolved contract
    value differing from CONTEXT -> `no_fabrication`.
  Do not raise the same underlying claim under several dimensions.
- A gate disposition binds the score. An item a sweep above dispositioned
  `no_breach` MUST NOT reappear in `critical_findings`, and no dimension may be
  reduced for a claim its own sweep cleared. Where a sweep and a finding
  disagree, the sweep wins and the finding is dropped.
- Allowed patterns are never findings — under ANY dimension: (a) ANY task-ID
  scheme, phase count, and task granularity — open-world synthesis; (b) an
  authored file-internal interface / signature / type / prop / file path the
  technical design left to implementation, coherent for the named surface and
  within a specs bound — the EXPECTED low-level authoring, never a fabrication;
  (c) a phase with `Depends on: —` that produces without consuming in-change;
  (d) a task stating a resolved planning-decision value from PLANNING DECISIONS;
  (e) the `## Phase Processing Rules` block reproduced verbatim (never grade its
  wording); (f) a `Verify` block that DESCRIBES what will be checked (acceptance /
  scenario / screenshots-to-capture) with `Status: pending` — this is the correct
  PLAN shape, not "unverified"; (g) a `Status: blocked — … (ref 0.x / P#)` note
  standing in for the removed Blocked Tests section; (h) referencing grooming/
  technical-design ids (P1, Q1, RQ-…) as trace; (i) repeating an RQ id across
  tasks; (j) skill / rule names in parentheses as pointers (e.g. `playwright-cli`,
  `verification-standards.mdc`); (k) a task that strengthens a privacy/security
  bound; (l) a "Cross-cutting" Story Trace row; (m) an omitted `#### CMS / Page
  Designer` or `#### Translation Sync` sub-area when it has no work; (n) a
  `- [ ]` one-liner sitting under the `## Prerequisites`, `## Test Setup`, or
  `## Verification` heading — the template's REQUIRED shape for those three
  sections, exempt from the Files / Interface / Steps / Done-when bar. This
  exemption is scoped to those headings ONLY: inside a `### P#`, a `- [ ]` whose
  text carries a per-phase slot name (`Process`, `Verify`, `Status`, `Delivers`,
  `Depends on`, `Areas`, `Story`, `Jira`) is a PROMOTED SLOT and stays a
  parser/role critical, because those slots must be indented `-` sub-content.
- Omission is not a critical — under ANY dimension — EXCEPT the projection closed
  law and a missing mandatory element (Solution Overview; Phase Processing Rules;
  any phase block / implementation task; Story Trace when stories exist).
  Everything else missing is `missing_evidence` or a warning.
- Authority check for every finding: if the only authority is "the GOLDEN
  contains this", the finding is invalid — DROP it. A valid finding cites a
  STANDARD rule or a CONTEXT fact the candidate violates.
- Completeness does not offset a violation, and fluency is not correctness.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by the
dimension id below. 1.0 = fully satisfied with evidence, ~0.5 = partial / minor
gaps, 0.0 = violated or absent.

- `trace_coverage` — every technical-design Implementation Building Block row and
  Verification Strategy row is delivered, every Delivery Phase realised as a
  `### P#` block, every in-scope RQ/WBS and story covered, deferred scope
  excluded, nothing phantom; blockers recorded in `Status`.
- `projection_fidelity` — each task's subject names a technical-design surface /
  block / phase (verbatim) with trace; authored detail stays within specs and
  approved bounds and adds no shopper behaviour and no scope/phase. Authoring a
  file-internal signature is not penalised here.
- `planning_gate_state` — built from resolved decisions; no Open Questions
  section, no surviving `per Q#`, no re-decided question, no invented approver.
- `parser_role_conformance` — every parseable task is a column-0 `- [ ]` with
  exactly one @FE/@BE/@SHARED tag and its detail + per-phase slots indented
  beneath; no `@QA` checkbox; no checkbox in a table; role matches the area.
- `implementation_actionability` — every implementation task carries Files + a
  usable Interface (or a justified pure-wiring omission) + Steps + Done-when.
- `phase_sequencing` — every `### P#` carries its slots and each `Depends on:`
  names a real producer in consumer -> producer direction (or `—`), with no
  inverted, dangling, or fabricated edge.
- `phase_processing` — the `## Phase Processing Rules` block is present (verbatim;
  wording not graded); every phase carries `Process` (+ `bounds:` + tier), a usable
  `Verify` PLAN (frozen acceptance + manual-style scenario + evidence-by-flow +
  verdict), and `Status`; each `bounds:` matches what that phase's own tasks touch
  and fixes the tier; sensitive phases force SHIP_WITH_HUMAN_REVIEW + a security
  pass; no fabricated executed evidence.
- `solution_overview` — a leading `## Solution Overview` concisely projects the
  upstream (delivered + consumed-not-built + N phases), invents no scope, and is
  a paragraph (not a bullet in/out list or phase re-listing).
- `no_fabrication` — every closed-world fact and resolved contract value traces to
  CONTEXT with the same value; task IDs, phase numbering/granularity, and authored
  file-internal signatures/paths are exempt as own synthesis.
- `structure_conformance` — the fixed, unnumbered `##` frame in the template's
  order (Solution Overview first; Phase Processing Rules present; no QA Handoff)
  and the `### P#` phase-block slots and table columns (deterministic gate owns
  exact `##` presence/order; score heading and table-shape fidelity).

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it: if any
  item in `critical_findings` is present, `score` MUST NOT exceed 0.5.
- `pass` is true only when `score >= threshold` (given in the user message) AND
  `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or of invariants 1-10
  (a projection-law breach, out-of-boundary authorship, a planning-gate breach, a
  parser/role breach, a non-actionable task, a phase-sequencing breach, a
  phase-processing breach, a missing/invented Solution Overview, a QA-boundary
  breach, fabrication, internal leakage, a missing mandatory element). These block
  acceptance.
- **warnings** — quality issues that do not make the document wrong (e.g. a task
  that lightly restates design rationale instead of turning it into steps, a
  heading-level slip, a thin `Delivers` line, a thin Verify acceptance).
- **missing_evidence** — expected content the CONTEXT implies but the candidate
  omits without breaking a law (e.g. a thinner verification list, a phase
  `Depends on:` that omits a secondary producer while the primary chain is
  correct). Record it ONLY here: it MUST NOT appear in `critical_findings` and
  MUST NOT cap the score.

Always ground findings in concrete evidence. Every critical finding MUST quote a
string that appears VERBATIM in the candidate. Before recording a critical,
locate that exact string (the task line, the role tag, the phase slot, the
heading) in the candidate; if it is not literally present, DROP the finding — do
not infer or paraphrase. For a missing-item finding, name the specific CONTEXT
fact (the technical-design block/phase, the RQ/WBS, the story key) the candidate
omits. A finding whose evidence you cannot copy verbatim is a hallucination, not a
violation.
