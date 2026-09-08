---
name: ba-requirement-tracing
description: Trace requirements to Jira tickets and maintain the RTM.
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

# Skill: Requirement Tracing

**Slug**: `requirement-tracing`
**Category**: governance
**One-liner**: Trace requirements to Jira tickets and maintain the RTM.

## When to activate
- User wants to verify that all FSD requirements are covered by Jira tickets
- User wants to update the Requirement Traceability Matrix after a Jira push
- User wants to link an FSD ID to a Jira ticket

## Tools to use
| Capability | Purpose |
|---|---|
| compare retrieved requirements with Jira evidence and the local RTM | Trace a requirement through stories and epics |
| Jira issue search by JQL (`discover`, then `executeRead`) | Find tickets covering a requirement |
| Jira issue read (`discover`, then `executeRead`) | Inspect ticket details for traceability |
| read the named CSV in .astound/ba/trackers using workspace file tools | Read the current RTM |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Append a new row to the RTM |
| Confluence search by CQL (`discover`, then `executeRead`) | Find the FSD or scope document for the requirement |
| Confluence page read (`discover`, then `executeRead`) | Read the full FSD page |

## RTM mandatory columns
`rtm_id` | `jira_id` | `fsd_id` | `description`

- `rtm_id`: Sequential ID, e.g. `RTM-001`, `RTM-002`
- `jira_id`: Exact Jira ticket key returned by the API (e.g. `BAP-42`)
- `fsd_id`: Source FSD feature ID from the Confluence scope document (e.g. `CT-005`, `CR-001`)
- `description`: Exact story title taken from Jira

## Guardrails
- NEVER invent FSD IDs — derive them from the Confluence scope document via Confluence page read (`discover`, then `executeRead`).
- ALWAYS update the RTM immediately after a successful Jira push.
- RTM entries must use the exact Jira ticket key returned by the API — not draft IDs or estimates.
- If an FSD ID cannot be found, flag the gap and ask the user rather than leaving the field blank.
- NEVER delete RTM rows — only append new rows.
