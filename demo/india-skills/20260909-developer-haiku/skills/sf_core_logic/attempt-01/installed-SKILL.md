---
name: sf-core-logic
description: "Use when implementing or reviewing Core Logic (Domain Knowledge) in Salesforce."
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

# Skill: Core Logic (Domain Knowledge)

Use this when implementing or reviewing backend and data logic.

## Expertise areas

- **Apex:** Triggers (handler pattern), bulkification, governor limits, order of execution.
- **SOQL / SOSL:** Selective queries, relationship queries, bulk-safe patterns, limits.
- **Salesforce order of execution:** Record-triggered flows vs triggers vs process builders, before / after save, commit order.
- **Async:** Batch, Queueable, Schedulable, Platform Events.

## Key patterns

- One trigger per object -> handler class; no logic in the trigger body beyond delegation.
- Collect IDs in loops; run SOQL / DML once outside loops.
- Use `Security.stripInaccessible()` (or `WITH SECURITY_ENFORCED` in SOQL) before DML; respect sharing and FLS unless explicitly elevated and documented.
- Static `Boolean` flags used for re-entrancy or bypass control must be **explicitly initialized to `false`** — unboxing a null `Boolean` in `if (flag)` causes runtime errors.
- For after-save work that needs to re-run platform features (e.g. assignment rules), use an `@InvocableMethod` callable from a Flow with `Database.DMLOptions.assignmentRuleHeader.useDefaultRule = true`.
- Avoid hardcoding profile names; prefer **Custom Permission** checks via `FeatureManagement.checkPermission('Permission_Api_Name')`.

## Order-of-execution reminders

| Phase | What runs |
|---|---|
| 1 | System validation (required fields, foreign keys, max length) |
| 2 | Before-save record-triggered Flows (fast field updates on `$Record`) |
| 3 | Before triggers (Apex) |
| 4 | Custom validation rules |
| 5 | Save (not committed) |
| 6 | After triggers (Apex) |
| 7 | Assignment rules, auto-response, workflow rules |
| 8 | Process Builder / Workflow field updates -> after-save flows -> after triggers re-fire |
| 9 | Roll-up summary, sharing rules, commit, post-commit (async, email) |

Use this to decide between **before-save flow** (cheap, same record) and **after-save flow / Apex** (cross-object, async, callouts).
