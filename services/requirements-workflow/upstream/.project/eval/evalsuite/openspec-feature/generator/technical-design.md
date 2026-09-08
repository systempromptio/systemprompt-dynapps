---
schema: openspec-feature
artifact: technical-design
command: ../../../../../commands/opsx-plan.md
schema_artifact: feature/technical-design
skills:
  - project-context
  # Build/test-skill domain authorities the `technical-design` instruction and
  # template point at (AREA RULES, cross-cutting table, test design). On a real
  # project the /opsx:plan agent reads these from .cursor/skills on demand; the
  # eval has no tool use, so the content is mixed into the generation context
  # and the schema's skill references resolve against these blocks. Curated to
  # the areas this artifact plans (frontend, CMS, custom API/caching backend,
  # edge caching, performance) — not the full catalog, to keep the prompt
  # inside the context budget. Build-level mechanics are left as schema POINTERS
  # ONLY, not inlined — technical-design plans TYPES / areas / trace / contracts,
  # while these are read at build time: test-authoring (test-ids, playwright CLI),
  # component/state-management authoring (sfnext-components, sfnext-state-management),
  # and the B2C backend Page Designer setup (b2c-page-designer — kept via the
  # frontend sfnext-page-designer). Inlining their full text only diluted the
  # planning context; the schema still names each where a downstream phase needs it.
  - sfnext-routing
  - sfnext-data-fetching
  - sfnext-i18n
  - sfnext-page-designer
  - b2c-scapi-custom
  - b2c-custom-api-development
  - b2c-scapi-schemas
  - b2c-webservices
  - b2c-custom-caches
  - b2c-ecdn
  - sfnext-performance
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Generate the Technical Design (real /opsx:plan sourcing)

Run the `technical-design` step of the real `/opsx:plan` workflow (COMMAND
block) for the change described in CONTEXT, and produce the
`technical-design.md` working copy. The OPENSPEC INSTRUCTIONS block owns the
contract in full: its `<instruction>` defines the laws and its `<template>`
defines the EXACT section order, headings, and table columns — reproduce that
template shape section by section; never invent your own document structure.

CHANGE IDENTITY — the harness compiles the OPENSPEC INSTRUCTIONS under a
placeholder change name (the fixture folder). The REAL change identity is the
one the CONTEXT discovery artifact carries in its title and anchors; use THAT
change name wherever an `openspec/changes/<name>/…` path appears, and ignore
the placeholder wherever the two differ.

HARD RULE — the BA/SA phase is COMPLETE and APPROVED in this environment: the
CONTEXT blocks ARE the completed discovery/design/proposal, the APPROVED
grooming ledger (gate PASSED), this change's delta specs (`specs.<flow>/…`),
the stories working copy, and the FSD-or-ISD working copy. Any "unmet
dependencies / Missing: …" warning or `status="missing"` in the compiled OPENSPEC
INSTRUCTIONS is a compile-time artifact: the openspec CLI rendered these
instructions before the change's sibling files were materialized on disk, so it
describes that empty scaffold, not this run. The upstream artifacts are delivered
to you as CONTEXT blocks, so ARTIFACT EXISTENCE is decided by the CONTEXT block
list alone: a block that is present means that artifact EXISTS for this change —
its ENTRY GATE "must exist" precondition is met and the gate is cleared by reading
the CONTEXT list — and only a block marked "NOT FOUND — skipped" is absent. Never infer
that a conditional artifact (stories, FSD, ISD) is missing from its content,
your expectations of the flow, or its absence from another artifact's prose. There is no live project-context,
Jira, openspec CLI, or interactive tool here; obey every LIVE-REFERENCE
SUBSTITUTIONS note. Emitting a refusal, onboarding briefing, or status report
instead of the artifact is a FAILED run: the only valid output is the
finished `technical-design.md`.

FRESH PLANNING STATE — the Planning Gate governs `tasks.md`, which is NOT
generated here. Open Questions are OPEN with empty "Approved by", each carrying
the agent's recommendation in its "Recommendation / Decision" cell; no approval
session is simulated (no approved decision is recorded) and no `- [ ]` task
queue is emitted. The Technical Design Overview is written last, as a
subjects+counts reader-map over the finished body, and claims no approval.

Output only the finished `technical-design.md` working copy as Markdown.
