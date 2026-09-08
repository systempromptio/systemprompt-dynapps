---
schema: openspec-feature
artifact: isd
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/isd
skills:
  # Diagram authority the `isd` instruction points at: the drawio-diagrams skill
  # (SKILL.md workflow + block format) plus its two type references. The sequence
  # reference (participants/messages vocabulary) drives the mandatory Integration
  # Flow Overview; the flow reference (activity/plain vocabulary + the decision +
  # two-outcomes worked example) drives the per-section flow diagram the user
  # requested (see the `.context/isd-diagram-request` block). On a real project
  # the /opsx:propose
  # agent authors each block, runs the skill's generate script, and inserts the
  # .png; the eval has no shell, so the drawio scripts are neutralized by the
  # `drawio-diagrams` mappings stub (author the block + image line directly).
  - drawio-diagrams
  - ../../../../../skills/plan/drawio-diagrams/references/sequence.md
  - ../../../../../skills/plan/drawio-diagrams/references/flow.md
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Enrich the ISD scaffold (real /opsx:propose sourcing)

Run the ISD step of the real `/opsx:propose` workflow (COMMAND block) for the
change described in CONTEXT. In the shipped workflow the scaffold script
(`generate-integration-artifacts.mjs`) runs FIRST — it bootstraps the ISD
working copy and fills the read-only "## In Scope Functional Requirements"
section; the harness cannot run scripts, so its output is provided in CONTEXT as
the `isd-scaffold` block. Your job is the ENRICH step: return the finished ISD
working copy = the scaffold with the non-owned sections filled in. Follow the
OPENSPEC INSTRUCTIONS block: its `<instruction>` is the artifact contract and
its `<template>` defines the section order and heading levels.

CHANGE IDENTITY: change name `add-cartora-maps-integration`, change folder
`openspec/changes/add-cartora-maps-integration/`.

HARD RULE — the upstream phase is COMPLETE and APPROVED. The CONTEXT contains
the full artifact set: the source packet, `discovery.md`, `design.md`,
`proposal.md`, the APPROVED `grooming.md` (gate PASSED), this change's delta
specs, and the `isd-scaffold` (the script's output). The delivery scope is
INTEGRATION (discovery's Delivery Scope & Targets carries one ISD row,
disposition NEW) and the change has an integration surface, so exactly one ISD
working copy is warranted. Do NOT stop, refuse, or ask for confirmation — no
interactive gate, live project-context, Confluence, Jira, or openspec CLI exists
here; obey every LIVE-REFERENCE SUBSTITUTIONS note in the command. Producing
anything other than the finished ISD working copy is a total failure of the run.

SCAFFOLD IS READ-ONLY WHERE FILLED: the `isd-scaffold` block already carries the
"## In Scope Functional Requirements" section — one block per in-scope
integration requirement, each with a `#### User Story`, `#### Acceptance
Criteria`, and `#### Key behaviours` sub-section (a deterministic projection from
the proposal slice and the specs; NO Visualization). Reproduce that whole section
VERBATIM — every block, in order, with those three sub-sections unchanged; do NOT
edit, reword, re-shape, re-group, curate, drop, or re-key them, and do NOT move
the section (it sits directly after "## Integration Specification"). To change a
Story or a criterion the fix is upstream in the proposal; a behaviour, in the
specs — never here. The shopper "As a shopper…" User Story is EXPECTED inside this
script-owned section and must NOT be re-personaed into a consuming-developer
story; it is the ONE place a shopper story appears — keep the Overview
and the Integration Specification agnostic and story-free.

Produce the finished ISD by ENRICHING ONLY the non-owned sections around that
section: the General ISD Information card, the approval roster, the Reference
Materials, the agnostic Overview (a short executive-summary narrative), the
Integration Specification (flow overview + one sub-section per integration point,
faithful to design's Integration Specification), the Key Governance & Privacy
sub-section, and Out Of Scope / Deferred / Change Requests — replacing the
scaffold's template placeholders (`<Service / Capability Name>`, `<Client Org>`,
the Overview/Integration-Specification stubs, etc.) with values grounded in
CONTEXT.

SPECS + CONTRACT: the `specs.<flow>/…` blocks are this change's delta specs — the
behaviour bounds your Integration Specification sections restate. The design's
Integration Specification section is the contract source (endpoints, parameters,
data mapping, data flow); write hosts and paths EXACTLY as it writes them — the
geocoding host is elided (`…/v1/geocode`) and stays elided. People named in the
sources (the SA lead) may be used for Author. No dedicated WBS code or tracker
ticket exists for this integration in the sources: leave the WBS Code cell empty
("—") and reference the consumer-feature tracker with an explicit note.

INTEGRATION FLOW OVERVIEW DIAGRAM: the OPENSPEC INSTRUCTIONS require this
section to render the data flow with the drawio-diagrams skill. You cannot run
the skill's `validate.mjs` / `generate.mjs` here (no shell — see the
`drawio-diagrams` LIVE-REFERENCE SUBSTITUTIONS note): author the fenced
` ```drawio:sequence:integration-flow-overview ` block inline (participants +
messages per the sequence reference), then add
`![Integration Flow Overview](./assets/integration-flow-overview.png)` on the
next line yourself. The `.png` path is deterministic from the block id; do not
claim to have run the generator or rendered anything.

REQUESTED PER-SECTION FLOW DIAGRAM: the `.context/isd-diagram-request` block in
CONTEXT (a user-request source) is the user explicitly asking for one extra
diagram — an activity-style flow in
the "Geocoding & Proximity" integration sub-section showing the resolvable-vs-
unresolvable branch that the delta specs already bound. Run the schema's
diagram-opportunity analysis across the sub-sections; that section clears the
bar (a real decision with two distinct outcomes) AND is explicitly requested,
so author a ` ```drawio:flow:geocoding-resolution ` block there (a decision node
fanning to a "resolved → results" outcome and an "unresolvable → clear error"
outcome per the flow reference's worked example, boundary-neutral — arrows carry
logical steps, never Cartora endpoints / paths / query params / keys), then add
`![Geocoding resolution outcome](./assets/geocoding-resolution.png)` on the next
line, same no-shell rules as above. Do NOT scatter diagrams into sub-sections
that did not clear the bar or were not requested.

Output only the finished ISD working copy as Markdown — no plan, no commentary,
no status report, and (because this is a single-shot run where the user already
asked for the diagram) no separate "Diagram Recommendations" chat block.
