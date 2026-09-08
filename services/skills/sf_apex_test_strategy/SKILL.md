---
name: sf-apex-test-strategy
description: Use when implementing or reviewing Apex test strategy in Salesforce.
---

## Systemprompt execution contract

The marketplace provisions authenticated `atlassian`, `salesforce`, and `github`
MCP connectors for the signed-in user. Use their advertised tools and input
schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

# Skill: Apex test strategy

## Purpose

Provide a repeatable approach to choosing Apex test scope for PR vs release, with fast feedback while maintaining safety.

## PR checks (default)

- Run **impacted tests** when:
  - Change is small and localized.
  - No security / sharing model changes.
  - No schema changes affecting many classes.
- Escalate to **`RunLocalTests`** when:
  - Triggers, sharing, auth, or critical integrations are affected.
  - Large refactors or dependency changes occur.

## Release checks (required)

- Always run **`RunLocalTests`** for promotion to UAT / PreProd / Prod.
- Keep a curated **smoke test class list** for critical flows if runtime is a concern.

## Sandbox quirks

- When deploying non-Apex changes that fail under `RunSpecifiedTests` because of **unrelated** org-level validation rules or coverage requirements, use `--test-level NoTestRun` and run the affected test classes separately afterwards. Document this in the story's `deployment.md`.

## Test data & validation rules

- Tests must satisfy **all active validation rules** in the target org. Use a `TestDataFactory` (or equivalent) to centralise required-field defaults.
- When a new validation rule is added, sweep existing test classes for any factory call that no longer creates valid records and update the factory in the same PR.

## Evidence

- Store test output as pipeline artifacts.
- Capture failing test diagnostics and link to the story / Jira for traceability.
- Coverage report attached to the PR (per-class coverage on touched classes).
