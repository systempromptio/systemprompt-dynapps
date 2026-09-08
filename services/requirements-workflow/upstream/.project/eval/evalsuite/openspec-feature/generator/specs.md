---
schema: openspec-feature
artifact: specs
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/specs
skills:
  - project-context
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Specs step of the real /opsx:propose workflow

Run the `specs` step of the real `/opsx:propose` workflow (COMMAND block) and
produce the delta spec files for this change. The OPENSPEC INSTRUCTIONS block
owns the specs contract in full: its `<instruction>` defines the entry gate,
file inventory, coverage law, delta discipline, and the quality bar; its
`<template>` defines the ADDED / MODIFIED / REMOVED block shapes. Follow that
block — do not restate or supplement its content policy here.

MULTI-FILE OUTPUT — this artifact is a bundle of files emitted as ONE Markdown
response. Precede each file's content with a marker line, then the file body:

`--- specs/<capability>/spec.md ---`

Emit the files in alphabetical order by capability name. Output nothing except
the marker lines and the file bodies — no prose between files, no wrapper
fences, nothing after the last file.

CHANGE IDENTITY — the harness compiles the OPENSPEC INSTRUCTIONS under a
placeholder change name (the fixture folder). The REAL change identity is the
one the CONTEXT discovery artifact carries in its title and anchors; use THAT
name wherever an `openspec/changes/<name>/…` anchor is needed, and ignore the
placeholder wherever the two differ.

HARD RULE — the ENTRY GATE is already SATISFIED in this environment. All four
upstream dependencies are PRESENT in the CONTEXT: the "upstream phase artifact"
blocks ARE the completed `discovery.md`, `design.md`, and `proposal.md`, and
the `grooming/final` block IS the completed `grooming.md` for this change —
a grooming ledger whose Grooming Approval Gate has PASSED, with every item
APPROVED or DEFERRED-APPROVED by a named human and every gate box checked.
There are no other copies on disk, and no `openspec/changes/<name>/…` path will
resolve. Any "unmet dependencies / Missing: …" warning in the CLI-compiled
OPENSPEC INSTRUCTIONS is a harness artifact, never a stop condition. Emitting a
STOP / refusal / "cannot generate" message instead of the artifact is a FAILED
run: the only valid output is the finished spec files, authored from the
CONTEXT blocks.

BASELINES — the CONTEXT blocks named `specs/<capability>/spec` are the
current archived capability specs (`openspec/specs/<capability>/spec.md`),
scoped to the capabilities this change's discovery marks affected; a baseline
block absent from CONTEXT means that capability has no archived spec yet (a
new ADD) or is not in this change's scope — its absence is not a gap to fill,
and no baseline is ever an instruction to author a file for another change.

There is no live project-context, Jira, openspec CLI, or interactive tool here;
obey every LIVE-REFERENCE SUBSTITUTIONS note, and use the generation contract
for how to weigh and ground CONTEXT (authority order, intra-CONTEXT precedence,
and disposition inheritance). Output only the finished spec files as Markdown
with their marker lines.
