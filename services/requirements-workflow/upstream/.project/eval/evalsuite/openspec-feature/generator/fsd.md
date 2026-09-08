---
schema: openspec-feature
artifact: fsd
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/fsd
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Enrich the client-facing FSD scaffold (real /opsx:propose sourcing)

Run the FSD step of the real `/opsx:propose` workflow (COMMAND block) for the
change described in CONTEXT. In the shipped workflow the scaffold script
(`generate-functional-artifacts.mjs`) runs FIRST and fills the three read-only sub-sections; the
harness cannot run scripts, so its output is provided in CONTEXT as the
`fsd-scaffold` block. Your job is the ENRICH step: return the finished FSD
working copy = the scaffold with the non-owned sections filled in. Follow the
OPENSPEC INSTRUCTIONS block: its `<instruction>` is the artifact contract and
its `<template>` defines the section order and heading levels.

CHANGE IDENTITY: change name `store-locator`, change folder
`openspec/changes/store-locator/`.

HARD RULE — the upstream phase is COMPLETE and APPROVED. The CONTEXT contains
the full artifact set: the source packet, `discovery.md`, `design.md`,
`proposal.md`, the APPROVED `grooming.md` (gate PASSED), this change's delta
specs, and the `fsd-scaffold` (the script's output). The delivery scope is
FUNCTIONAL (discovery's Delivery Scope & Targets carries one FSD row,
disposition NEW) and the change has a client-facing surface, so exactly one FSD
working copy is warranted. Do NOT stop, refuse, or ask for confirmation — no
interactive gate, live project-context, Confluence, Jira, or openspec CLI
exists here; obey every LIVE-REFERENCE SUBSTITUTIONS note in the command.
Producing anything other than the finished FSD working copy is a total failure
of the run.

SCAFFOLD IS READ-ONLY WHERE FILLED: the `fsd-scaffold` block already carries,
per requirement block, the `#### User Story`, `#### Acceptance Criteria`, and
`#### Key behaviours` sub-sections — a deterministic projection from the
proposal slice and the specs. Reproduce those three sub-sections VERBATIM in
every block; do NOT edit, reword, re-shape, re-group, curate, or drop them (to
change a Story or a criterion the fix is upstream in the proposal; a behaviour,
in the specs — never here). Produce the finished FSD by ENRICHING ONLY the
non-owned sections: the General FSD Information card, the approval roster, the
Reference Materials, each block's Visualization (grounded in the requirement's
behaviour), and any framing — replacing the scaffold's template placeholders
(`<WBS> - <Feature Name>`, `<Client — Programme>`, the Visualization `<state>`
cells, etc.) with values grounded in CONTEXT.

Output only the finished FSD working copy as Markdown — no plan, no
commentary, no status report.
