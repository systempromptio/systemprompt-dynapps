---
schema: openspec-feature
artifact: design
command: ../../../../../commands/opsx-propose.md
schema_artifact: feature/design
skills:
  - project-context
  # Build-skill domain authorities the `design` instruction points at for
  # cross-cutting solution constraints. Mixed into the generation context so a
  # candidate can ground rendering/caching/security/performance/i18n/CMS
  # decisions the same way the real /opsx:propose design subagent would.
  - sfnext-routing
  - b2c-custom-caches
  - b2c-ecdn
  - sfnext-authentication
  - b2c-slas-auth-patterns
  - b2c-webservices
  - sfnext-performance
  - sfnext-i18n
  - sfnext-page-designer
  - b2c-page-designer
rules:
  - ../../../../../rules/workflow/openspec-workflow/openspec-workflow.mdc
---

# Scenario — Architecture / Solution Design step of the real /opsx:propose workflow

Run the `design` step of the real `/opsx:propose` workflow (COMMAND block) and
produce ONE architecture / solution `design.md` working copy for this change.
The OPENSPEC INSTRUCTIONS block owns the design contract in full: its
`<instruction>` defines when design is warranted, the DESIGN GATE, scope handling,
dispositions, what to cover, and the quality bar; its `<template>` defines the
section order and headings. Follow that block — do not restate or supplement its
content policy here.

Discovery — design's one upstream dependency — is represented in the CONTEXT (the
reconciled trace with its dispositions and open questions). That upstream file is
not on disk here, so the CLI-compiled OPENSPEC INSTRUCTIONS may show an "unmet
dependencies / Missing: discovery" warning — that is a harness artifact, not a
stop condition. Proceed and author `design.md`.

There is no live project-context, Jira, openspec CLI, or interactive tool here;
obey every LIVE-REFERENCE SUBSTITUTIONS note, and use the generation contract for
how to weigh and ground CONTEXT (authority order, intra-CONTEXT precedence, and
disposition inheritance). Output only the finished `design.md` working copy as
Markdown.
