---
name: ba-jira-management
description: Create, search, update, transition, and link Jira tickets.
---

## Systemprompt execution contract

Workspace access installs these skills. The `atlassian`, `salesforce`, and
`github` connectors are delivered separately after the signed-in user connects
and verifies each provider account. An installed skill does not prove a connector
is ready. Local file tasks can run without provider connections. For remote
operations, use only the currently advertised tools and input schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

## Workspace adaptation

Custom BA platform operations have been replaced with workspace file operations
or the `atlassian` connector (`discover`, `executeRead`, `executeWrite`). Paths are relative to the current repository.
Keep drafts, trackers, sessions, transcripts, and reports beneath `.astound/ba/`;
report the actual saved path as the local identifier. Read existing files first,
use proper CSV quoting, preserve existing rows, and avoid concurrent overwrites.
Missing required trackers or project standards are missing inputs, not empty
sources: stop and request the missing input. Never invent their contents.

Local session corrections persist only in this workspace and must be explicitly
read on later sessions. They do not modify global memory. Transcripts are local
Markdown files, not rows in the retired BA backend. For bulk Jira edits, show all
changes first, apply approved edits individually, and report partial failures.
For story creation, apply the writing checks below before Jira issue creation (`discover`, then `executeWrite`).
Use its advertised content schema, not an assumed legacy wiki or ADF format.
Notification and scheduling operations produce drafts/proposals only; explicitly
report that sending or scheduling was not executed. Never invent a job history.

# Skill: Jira Management

**Slug**: `jira-management`
**Category**: jira
**One-liner**: Create, search, update, transition, and link Jira tickets.

> Use the authenticated `atlassian` connector for remote operations and workspace tools for local artifacts.

## When to activate
- User wants to search, read, update, comment on, or transition Jira tickets
- User wants to link tickets, bulk-update fields, or inspect change history
- User wants to attach files or find user account IDs for assignment

## Tools to use

### Read tools — call directly via `atlassian` (no orchestrator needed)
| Capability | Purpose |
|---|---|
| Jira project listing (`discover`, then `executeRead`) | List all accessible Jira projects |
| Jira issue read (`discover`, then `executeRead`) | Fetch full details of a Jira issue |
| Jira issue search by JQL (`discover`, then `executeRead`) | Search using JQL |
| Jira transition listing (`discover`, then `executeRead`) | List available workflow transitions for an issue before transitioning |
| Jira link-type listing (`discover`, then `executeRead`) | List available link type options for the project |
| Jira account lookup (`discover`, then `executeRead`) | Find user account IDs for assignee or reporter fields |
| Atlassian current-user read (`discover`, then `executeRead`) | Get the current user's Atlassian account details |
| get jira changelog via `discover` on `atlassian` (stop and report if no capability matches) | View the field-change history for an issue (workspace adaptation) |
| Jira issue search by JQL (`discover`, then `executeRead`) | Backlog breakdown: open, in-progress, and done counts (workspace adaptation) |

### Write tools — require explicit user approval
| Capability | Purpose |
|---|---|
| Jira issue creation (`discover`, then `executeWrite`) | Create a BA-standard story after local writing checks using the advertised content schema (approval required) |
| Jira issue creation (`discover`, then `executeWrite`) | Simple issue creation without BA processing, for non-story types (approval required) |
| Jira issue update (`discover`, then `executeWrite`) | Update fields on an existing issue (approval required) |
| Jira issue transition (`discover`, then `executeWrite`) | Move an issue to a new workflow status — call Jira transition listing (`discover`, then `executeRead`) first (approval required) |
| Jira issue comment (`discover`, then `executeWrite`) | Add a comment to an issue (approval required) |
| Jira issue link creation (`discover`, then `executeWrite`) | Create issue links (blocks, is blocked by, relates to, duplicates) (approval required) |
| Jira issue update (`discover`, then `executeWrite`) | Update approved issues individually and report partial failures (approval required) |
| add jira attachment via `discover` on `atlassian` (stop and report if no capability matches) | Attach a file or document to an issue (approval required) |

## Common JQL patterns
- Backlog (ready): `project = XXX AND status = "To Do" ORDER BY priority DESC`
- In-progress: `project = XXX AND status = "In Progress"`
- Sprint issues: `sprint = "Sprint Name" AND project = XXX`
- Bugs in module: `project = XXX AND issuetype = Bug AND labels = "module-name"`

## Guardrails
- NEVER create or update Jira tickets without explicit user approval in the current chat turn.
- NEVER invent Jira IDs — always verify via Jira issue read (`discover`, then `executeRead`) or Jira issue search by JQL (`discover`, then `executeRead`) first.
- Before transitioning, call Jira issue read (`discover`, then `executeRead`) to confirm the current status and available transitions.
- For bulk updates, display the full list of affected tickets to the user and get approval before executing.
- Before transitioning to "Done", verify DoD compliance using the `dor-dod-validation` skill.
- Always pass labels as a separate `labels` array — NEVER embed them in description text.
