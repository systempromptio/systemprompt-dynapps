---
name: ba-sprint-planning
description: Plan sprints with capacity, sequencing, and carry-over analysis.
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

# Skill: Sprint Planning

**Slug**: `sprint-planning`
**Category**: planning
**One-liner**: Plan sprints with capacity, sequencing, and carry-over analysis.

## When to activate
- User is preparing for a sprint planning meeting
- User wants to create a sprint, assign stories to a sprint, or review capacity
- User wants a defensible sprint plan with written justification

## Pre-flight steps (mandatory)
1. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `capacity`) — load dev/QA capacity per sprint.
2. get board info via `discover` on `atlassian` (stop and report if no capability matches) — confirm the active board and sprint configuration.
3. Jira issue search by JQL (`discover`, then `executeRead`) (JQL: `status = "Ready" ORDER BY priority DESC`) — fetch the plannable backlog.
4. Confirm sprint start date, end date, and sprint name with the user.

## Tools to use
| Capability | Purpose |
|---|---|
| get board info via `discover` on `atlassian` (stop and report if no capability matches) | Board configuration and active sprint info |
| get sprint info via `discover` on `atlassian` (stop and report if no capability matches) | Details about a specific sprint |
| Jira issue search by JQL (`discover`, then `executeRead`) | Issues already assigned to a sprint |
| Jira issue search by JQL (`discover`, then `executeRead`) | Backlog breakdown for capacity context |
| Jira issue search by JQL (`discover`, then `executeRead`) | Query backlog with JQL |
| create sprint via `discover` on `atlassian` (stop and report if no capability matches) | Create a new sprint (explicit approval required) |
| assign to sprint via `discover` on `atlassian` (stop and report if no capability matches) | Assign stories to the sprint (explicit approval required) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Read capacity.csv |
| get jira versions via `discover` on `atlassian` (stop and report if no capability matches) | Check fix version availability |

## Sprint plan output (show in chat before any Jira action)
| Jira ID | Title | Layer | Story Points | Sprint Half |
|---|---|---|---|---|

Include:
- **Carry-over list**: Stories that did not fit + reason
- **Written justification**: Sequencing rationale (BE blockers first, FE consumers second)
- **Capacity summary**: Total SP planned vs. available dev/QA capacity

## Guardrails
- NEVER create a sprint or assign tickets without explicit user approval.
- ALWAYS sequence BE blockers in the first half, FE consumers in the second half.
- A story is plannable ONLY IF both dev AND QA capacity exist for it.
- ALWAYS include a carry-over list with reasons.
- ALWAYS provide a written sequencing justification.
- Stories must be DoR-validated — run `dor-dod-validation` skill if not already done.
- Sprint name format: `<PROJECT> Sprint NN — DD MMM YYYY`
