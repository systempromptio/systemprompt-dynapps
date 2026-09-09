---
name: sf-flow-deployment-strategy
description: "Use when implementing or reviewing Flow deployment strategy in Salesforce."
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

# Skill: Flow deployment strategy

## Purpose

Reduce deployment failures caused by Flow ordering, activation state, and environment-specific dependencies.

## Conventions

- Keep Flow API names stable and version intentionally (do not rename to "fix" minor changes).
- Avoid referencing profiles directly when possible; prefer permission sets and custom permissions for entry conditions and visibility.
- Naming: `Object_When_What` (e.g. `Account_AfterSave_Health_Sync`).

## Deployment approach

- Deploy **dependent metadata first** (objects / fields / record types / picklist values).
- Deploy Flows **inactive** when activation can fail due to missing runtime dependencies; activate as a controlled post-deploy step.
- For **screen flows** referenced by **quick actions**, deploy the flow **before** the quick action (the quick action references the flow API name).
- For **before-save flows** that reference a custom field, deploy the field first.
- Screen Flow gotcha: the first screen must have at least one of `allowBack=true` / `allowFinish=true`.

## Activation

- After deploy, confirm flows are **Active** in Setup -> Flows. Some pipelines deploy as inactive by default.
- Activation can be a separate post-deploy step in the runbook.

## Validation

- Validate with target business scenarios in a sandbox.
- Run regression smoke tests for any object whose record-trigger flows changed.
- For after-save flows that call invocable Apex (e.g. assignment-rule re-execution), confirm the Apex class is deployed and the Flow's action input mapping resolves to the right `@InvocableVariable`.
