---
schema: openspec-feature
artifact: tasks
command: ../../../../../commands/opsx-plan.md
schema_artifact: feature/tasks
skills:
  - project-context
  # The platform CONVENTIONS the low-level tasks author their interfaces /
  # signatures / steps against, plus the Test Setup / Verification authorities the
  # `tasks` instruction points at, and the per-phase Verify DRIVER. On a real
  # project the /opsx:plan agent reads these from .cursor/skills on demand; the
  # eval has no tool use, so the content is mixed into the generation context.
  # Curated to the storefront (sfnext-*) and backend (b2c-*) surfaces this artifact
  # authors against — not the full catalog — to stay in the context budget.
  # Storefront (FE) surfaces:
  - sfnext-routing
  - sfnext-data-fetching
  - sfnext-components
  - sfnext-state-management
  - sfnext-test-ids
  - sfnext-i18n
  - sfnext-performance
  # Backend (BE) surfaces:
  - b2c-webservices
  - b2c-custom-api-development
  - b2c-custom-caches
  - b2c-metadata
  # Per-phase Verify DRIVER — the browser walk + screenshots the agent self-check
  # uses at apply time per verification-standards.mdc. NOT for authoring automated
  # e2e suites (those stay QA-owned, out of this pipeline).
  - playwright-cli
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
  - ../../../../../rules/workflow/verification-standards.mdc
---

# Scenario — Generate the Tasks queue (real /opsx:plan sourcing)

Run the `tasks` step of the real `/opsx:plan` workflow (COMMAND block) for the
change described in CONTEXT, and produce the `tasks.md` working copy. The
OPENSPEC INSTRUCTIONS block owns the contract in full: its `<instruction>`
defines the laws (the authorship boundary — a name the technical design carries
is SOURCED and honoured verbatim, a file-internal symbol the implementation and
its unit test bind to is AUTHORED-HERE, a shopper behaviour or scope the design
does not carry is OUT; the parser law; the coverage / role / phase-sequencing
rules; the task-block slots; the per-phase Process / Verify / Status contract)
and its `<template>` defines the EXACT frame — the fixed, unnumbered `##` section
set (`## Solution Overview` first; the VERBATIM `## Phase Processing Rules` block;
no QA Handoff), the `### P#` phase-block shape, and the table columns. Reproduce
that frame — including the `## Phase Processing Rules` block copied VERBATIM;
never invent your own document structure, never number or reorder the `##`
sections.

CHANGE IDENTITY — the harness compiles the OPENSPEC INSTRUCTIONS under a
placeholder change name (the fixture folder). The REAL change identity is the
one the CONTEXT technical-design carries in its title and anchors; use THAT
change name wherever an `openspec/changes/<name>/…` path appears, and ignore
the placeholder wherever the two differ.

HARD RULE — the BA/SA phase and the PLANNING phase are COMPLETE and APPROVED in
this environment. The CONTEXT blocks ARE the completed technical-design, this
change's delta specs (`specs.<flow>/…`), the APPROVED grooming ledger, the
stories working copy, and the FSD-or-ISD working copy. Any "unmet dependencies
/ Missing: …" warning in the compiled OPENSPEC INSTRUCTIONS is a harness
artifact, never a stop condition. There is no live project-context, Jira,
openspec CLI, or interactive tool here; obey every LIVE-REFERENCE SUBSTITUTIONS
note. Emitting a refusal, onboarding briefing, or status report instead of the
artifact is a FAILED run: the only valid output is the finished `tasks.md`.

PLANNING GATE PASSED — the PLANNING DECISIONS context block records the
approval-session outcome: every CRITICAL Open Question in the
technical-design is APPROVED by a named human (its answer recorded in that
question's Recommendation / Decision cell), so tasks.md may be generated.
Treat every `per Q#` deferral in the technical-design as RESOLVED to the
matching recorded decision: carry the DECIDED value into the affected task
(e.g. the resolved contract shape, TTL, or stubbing strategy). Do NOT emit an
Open Questions section, a `per Q#` placeholder, or a re-opened
decision — those belong to technical-design.md, not here.

APPLY-TIME vs AUTHORING — the per-phase `Verify` block is a PLAN the agent
executes at APPLY time. Author it (what WILL be checked) with `Status: pending`;
do NOT fabricate executed results, captured screenshots, or a `Status: approved`
into this freshly generated queue.

MANDATORY PLAN STEPS — in your plan, before writing the artifact, produce these
enumerations the OPENSPEC INSTRUCTIONS define, with verbatim quotes:
1. The Solution Overview source (SOLUTION OVERVIEW RULES): quote the upstream
   overview lines (discovery / proposal Executive Summary, technical-design
   Overview) you will condense into the one-paragraph `## Solution Overview`
   (what is delivered + consumed-not-built + N phases). Orientation only — not a
   bullet in/out-of-scope list, not a phase re-listing.
2. The coverage inventory (COVERAGE & TRACE RULES): list EVERY row of the
   technical-design's Implementation Building Blocks and EVERY Verification
   Strategy row, plus every Delivery Phase, every in-scope RQ/WBS, and every
   story key. Mark where each lands: a developer-owned verification target ->
   Test Setup or Verification; an implementation block -> a task block under the
   `### P#` phase that delivers it; every Delivery Phase -> its own `### P#`
   block; a capability/account/data-blocked verification -> a per-phase
   `Status: blocked` note (there is NO Blocked Tests section). A CROSS-CUTTING
   obligation (accessibility, localization) is NOT a phase: place its Building
   Block / story / verification as WOVEN tasks in the phases it touches (a11y via
   markup / test-ids; localization as a `#### Translation Sync` sub-area). Formal
   E2E / manual QA / a11y-audit / UAT is QA-owned and NOT queued (no QA Handoff
   section). A dropped block/evidence/phase/RQ/story, or a task with no
   technical-design basis, is a defect — this list is the coverage proof.
3. The resolved-decision ledger (PLANNING GATE): for each `per Q#` deferral,
   quote it and record the decision it resolves to from PLANNING DECISIONS — that
   decided value is what the affected task states. A surviving `per Q#` is a defect.
4. The phase plan (PHASE & ROLE RULES): list the `### P#` blocks (one per Delivery
   Phase, same order) with each phase's slots — Story (when stories exist),
   Delivers (+ RQ/WBS), Areas (the @FE/@BE rollup), Depends on (CONSUMER ->
   PRODUCER from the Runtime Flows; `—` when it consumes nothing in-change),
   Process (`per [Phase Processing Rules](#phase-processing-rules)` +
   `tier: standard | sensitive` — sensitive when the surface touches a vendor
   key/credential, shopper coordinates/geolocation, payments, or PII), Jira (`—`).
   Do NOT depend a phase on a Prerequisite, a Test Setup line, a Verification
   check, or an external vendor seam.
5. The low-level authoring plan (TASK-BLOCK RULES): for each implementation task,
   name its Files (paths under the Building Block the technical design named); the
   public Interface (exported symbols + signatures grounded in the cited
   `sfnext-*` / `b2c-*` convention, within the specs behaviour bound); the ordered
   Steps; the Done-when. Omit Interface only for a pure wiring / content / config
   task (say so in Steps).
6. The per-phase Verify plan (VERIFY & STATUS RULES): for each phase, the frozen
   Acceptance (from its Delivers + specs bound), the Unit tests that prove it, the
   manual-style Scenario the agent will WALK (an @FE phase via `playwright-cli`
   navigate / act / screenshot the key states; an @BE / integration phase via a
   call capturing request/response + logs + cache), the Evidence to capture, a
   Security pass for a `tier: sensitive` phase, and the Verdict (forced
   SHIP_WITH_HUMAN_REVIEW when sensitive). `Status: pending`.

Reproduce the `## Phase Processing Rules` block VERBATIM from the template — it is
invariant house-standard content; do not re-author, trim, or vary it.

Output only the finished `tasks.md` working copy as Markdown.
