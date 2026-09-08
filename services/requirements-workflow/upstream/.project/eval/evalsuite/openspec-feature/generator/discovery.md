---
schema: openspec-feature
artifact: discovery
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/discovery
skills:
  - project-context
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Discovery step of the real /opsx:propose workflow

Run the discovery step of the real `/opsx:propose` workflow (COMMAND block). The
delivery scope of this change is NOT given to you — it is whatever the OPENSPEC
INSTRUCTIONS' pre-artifact gate (the delivery-scope gate) resolves from the
CONTEXT. The command naming a change (`/opsx:propose <change>`) selects which
change to work on; it does not decide the delivery scope, and does not mean the
scope is functional. Let the gate decide.

If a pre-artifact gate reaches a stop condition — the delivery-scope gate or the
command's pre-discovery split gate — output that gate's stop-and-decompose message
instead of a discovery artifact. Only when the gate resolves the scope do you
author `discovery.md` accordingly. Follow the OPENSPEC INSTRUCTIONS block: its
`<instruction>` defines the discovery steps and quality bar, and its `<template>`
defines the exact section order and headings to fill.

This is the BA/SA discovery step only — either produce the gate stop-and-decompose
message, or produce `discovery.md`; do not produce downstream artifacts.
There is no live project-context, Jira, openspec CLI, or interactive tool here;
obey every LIVE-REFERENCE SUBSTITUTIONS note. Everything discovery would have
retrieved from project-context is in the CONTEXT block (the primary sources:
BRD/SRD requirements, meeting notes, tracker). Extract faithfully from CONTEXT,
use the real RQ/WBS codes, drop nothing, and invent nothing.

Output only the finished `discovery.md` as Markdown, unless a pre-artifact gate
requires stopping before discovery — in that case output only that gate's
stop-and-decompose message.
