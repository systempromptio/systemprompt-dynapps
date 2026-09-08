---
schema: openspec-feature
artifact: proposal
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/proposal
skills:
  - project-context
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Proposal step of the real /opsx:propose workflow

Run the `proposal` step of the real `/opsx:propose` workflow (COMMAND block) and
produce ONE `proposal.md` working copy for this change. The OPENSPEC
INSTRUCTIONS block owns the proposal contract in full: its `<instruction>`
defines the entry gate, the Requirements Lock, disposition and verification
rules, and the quality bar; its `<template>` defines the section order and
headings. Follow that block — do not restate or supplement its content policy
here.

CHANGE IDENTITY — the harness compiles the OPENSPEC INSTRUCTIONS under a
placeholder change name (the fixture folder). The REAL change identity is the
one the CONTEXT discovery artifact carries in its title and anchors; use THAT
name for the proposal title and every `openspec/changes/<name>/…` anchor, and
ignore the placeholder wherever the two differ.

HARD RULE — the ENTRY GATE is already SATISFIED in this environment. Discovery
and design — proposal's two upstream dependencies — are PRESENT in the CONTEXT:
the two "upstream phase artifact" blocks ARE the completed `discovery.md` and
`design.md` for this change; there are no other copies on disk, and no
`openspec/changes/<name>/…` path will resolve. Any "unmet dependencies /
Missing: discovery, design" warning in the CLI-compiled OPENSPEC INSTRUCTIONS
is a harness artifact, never a stop condition. Emitting a STOP / refusal /
"cannot generate" message instead of the artifact is a FAILED run: the only
valid output is the finished `proposal.md`, authored from the CONTEXT blocks.

There is no live project-context, Jira, openspec CLI, or interactive tool here;
obey every LIVE-REFERENCE SUBSTITUTIONS note, and use the generation contract
for how to weigh and ground CONTEXT (authority order, intra-CONTEXT precedence,
and disposition inheritance). Output only the finished `proposal.md` working
copy as Markdown.
