---
schema: openspec-feature
artifact: grooming
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/grooming
skills:
  - project-context
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Grooming step of the real /opsx:propose workflow

Run the `grooming` step of the real `/opsx:propose` workflow (COMMAND block)
and produce ONE `grooming.md` working copy for this change. The OPENSPEC
INSTRUCTIONS block owns the grooming contract in full: its `<instruction>`
defines the migration law, the zero-self-approval discipline, categorization,
and the quality bar; its `<template>` defines the section order and headings.
Follow that block — do not restate or supplement its content policy here.

CHANGE IDENTITY — the harness compiles the OPENSPEC INSTRUCTIONS under a
placeholder change name (the fixture folder). The REAL change identity is the
one the CONTEXT discovery artifact carries in its title and anchors; use THAT
name for the grooming title and every `openspec/changes/<name>/…` anchor, and
ignore the placeholder wherever the two differ.

HARD RULE — the ENTRY GATE is already SATISFIED in this environment.
Discovery, design, and proposal — grooming's three upstream dependencies —
are PRESENT in the CONTEXT: the three "upstream phase artifact" blocks ARE
the completed `discovery.md`, `design.md`, and `proposal.md` for this change;
there are no other copies on disk, and no `openspec/changes/<name>/…` path
will resolve. Any "unmet dependencies / Missing: …" warning in the
CLI-compiled OPENSPEC INSTRUCTIONS is a harness artifact, never a stop
condition. Emitting a STOP / refusal / "cannot generate" message instead of
the artifact is a FAILED run: the only valid output is the finished
`grooming.md`, authored from the CONTEXT blocks.

THIS RUN PRODUCES THE PREPARED DRAFT, NOT THE SESSION RESULT. There is no
live grooming session in this environment and no human has answered anything:
every Decision and Approved by cell stays `—`, every Status stays OPEN (or
BLOCKED where a source records a blocker), and every Grooming Approval Gate
checkbox stays UNCHECKED. Do not simulate, transcribe, or anticipate a
session. The command's chat-facilitation step (ask one item at a time) is
live-session behavior — do NOT render it as chat text; emit only the
artifact.

There is no live project-context, Jira, openspec CLI, or interactive tool
here; obey every LIVE-REFERENCE SUBSTITUTIONS note, and use the generation
contract for how to weigh and ground CONTEXT (authority order, intra-CONTEXT
precedence, and disposition inheritance). Output only the finished
`grooming.md` working copy as Markdown.
