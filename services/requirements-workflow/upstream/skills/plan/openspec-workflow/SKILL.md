---
name: openspec-workflow
description: Route OpenSpec workflow requests to the correct opsx slash command. Use when the user asks to explore, propose, approve/publish, plan, apply, sync, or archive an OpenSpec change.
license: MIT
metadata:
  author: openspec
  version: "1.0"
---

# OpenSpec Workflow Router

This skill is only a routing layer. Detailed workflow steps live in `commands/opsx-*.md`; do not execute
OpenSpec phases from memory and do not duplicate phase logic here.

## Route By Intent

- Thinking, discovery, requirement clarification, or grooming facilitation -> `/opsx:explore`
- Create or continue BA/SA artifacts (`discovery`, `design`, `proposal`, `grooming`, `specs`, optional
  `fsd`/`isd`) -> `/opsx:propose` (the client doc's read-only sub-sections + `stories.md` are projected
  together by `scripts/generate-functional-artifacts.mjs` for the FSD or
  `scripts/generate-integration-artifacts.mjs` for the ISD; the agent enriches only the non-owned sections)
- Submit the completed proposal for review — commit the artifacts, open a review PR, and PREPARE the
  FSD/ISD Confluence publish (agent hands off publish commands; the user runs them) ->
  `/opsx:propose-submit`
- After the proposal is approved: GENERATE `stories.md` deterministically from the artifacts (story
  script), PREPARE Jira story creation with an Epic, backfill keys, and push to the working branch ->
  `/opsx:propose-approved`
- Developer planning (`technical-design.md`, `tasks.md`) -> `/opsx:plan`
- Implementation from `tasks.md`, including BE/FE/FULL scope selection and dependency gates ->
  `/opsx:apply`
- Final spec sync and archiving -> `/opsx:archive`
- Project-context + tooling refresh -> `/project-sync`. Only the entry-point, context-grounded commands
  (`explore`, `propose`, `plan`) run `/project-sync` as a MANDATORY preflight; the downstream commands
  (`propose-submit`, `propose-approved`, `apply`, `archive`) inherit that freshness and do NOT force it
  (they document a manual `/project-sync` escape hatch). `/project-sync` itself git-gates the tooling
  refresh: if the branch pull brings no new commits, it skips the npm install + schema re-materialize and
  only refreshes project-context.

## Invariants

- Only `explore`, `propose`, and `plan` start with a mandatory `/project-sync` preflight; the downstream
  commands (`propose-submit`, `propose-approved`, `apply`, `archive`) do not force it. The context-grounded
  commands (`explore`, `propose`, `plan`, `apply`) must actively re-consult project-context as they move
  between phases/artifacts — never coast on the change folder or memory of an earlier phase.
- Slash commands are the source of truth for OpenSpec workflow behavior.
- Skills must not carry phase-specific procedures, artifact templates, or approval gates.
- Consume any upstream artifact through its PUBLISHED INTERFACE rather than re-scanning raw prose:
  its reader-map / executive section (the `Executive Summary` for discovery, the `Solution Overview`
  for design) orients you, and its `Document Consumption Guide` maps every `##` section it carries
  and the key identifiers in each. An upstream artifact that reaches a downstream phase is ALREADY
  GATE-APPROVED — inherit its dispositions and never re-open them (design reads discovery this way;
  proposal and plan read discovery + design this way). That interface is a read aid only: each
  artifact still derives its OWN reader map + Document Consumption Guide last, from its own finished
  body — never by copying an upstream's forward.
- Never self-approve grooming or planning questions; named human approval is required where the command
  contract says so.
- `specs` are the behavior source of truth. Jira stories and FSD/ISD documents are handoff/publication
  context, not behavior authority.
- The client-facing flow's downstream artifacts are a DETERMINISTIC PROJECTION produced together (client
  doc + `stories.md` in one run, kept in sync), never hand-authored by the agent — the functional flow by
  `scripts/generate-functional-artifacts.mjs` (FSD), the integration flow by
  `scripts/generate-integration-artifacts.mjs` (ISD); both reuse the shared `scripts/lib/`:
  - the client doc's `#### User Story` + `#### Acceptance Criteria` (verbatim from the covering proposal
    slice) and `#### Key behaviours` (a table from this change's specs; several requirements sharing one
    WBS aggregate into one block, each contributing a bold sub-header row that carries its requirement
    name). For the FSD these are per in-scope block; for the ISD they live under a
    script-owned `## In Scope Functional Requirements` section (NO Visualization — an ISD is a contract).
    The agent runs the script first, then enriches ONLY the non-owned sections (FSD: General FSD
    Information, approval roster, Reference Materials, Visualization, framing; ISD: General ISD
    Information, approval roster, Reference Materials, the agnostic Requirements table, Integration
    Specification, governance);
  - `stories.md` (one story per slice: the slice Story, an acceptance-criteria reference to the client-doc
    block, and design-derived implementation notes) — NOT a schema/eval artifact; Jira tickets are
    created from it at `/opsx:propose-approved` (which re-syncs both, preserving backfilled Jira keys).
  Corrections go UPSTREAM (proposal / spec / design), then the script is re-run. Unlike `stories.md`, the
  FSD/ISD as a whole remains an LLM-authored schema artifact (only the script-owned sub-sections are
  projected).
- `tasks.md` is the parseable implementation queue. Respect role tags (`@BE`, `@FE`, `@SHARED`) and
  dependency metadata.
- QA handoff is not implementation scope unless the user explicitly asks to prepare QA assets.
