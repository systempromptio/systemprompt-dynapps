---
name: ba-change-request
description: Log, track, and draft approved Change Requests.
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

## Workspace adaptation

Custom BA platform operations have been replaced with workspace file operations
or advertised Atlassian tools. Paths are relative to the current repository.
Keep drafts, trackers, sessions, transcripts, and reports beneath `.astound/ba/`;
report the actual saved path as the local identifier. Read existing files first,
use proper CSV quoting, preserve existing rows, and avoid concurrent overwrites.
Missing required trackers or project standards are missing inputs, not empty
sources: stop and request the missing input. Never invent their contents.

Local session corrections persist only in this workspace and must be explicitly
read on later sessions. They do not modify global memory. Transcripts are local
Markdown files, not rows in the retired BA backend. For bulk Jira edits, show all
changes first, apply approved edits individually, and report partial failures.
For story creation, apply the writing checks below before `createJiraIssue`.
Use its advertised content schema, not an assumed legacy wiki or ADF format.
Notification and scheduling operations produce drafts/proposals only; explicitly
report that sending or scheduling was not executed. Never invent a job history.

# Skill: Change Request

**Slug**: `change-request`
**Category**: governance
**One-liner**: Log, track, and draft approved Change Requests.

## When to activate
- PO or stakeholder requests a feature NOT in the original scope
- User wants to formally log and track a Change Request (CR)
- User wants to draft an epic for a CR-approved feature

## Pre-flight steps (mandatory — do not skip)
1. `searchConfluenceUsingCql` — verify the requested feature is not already in the scope documents.
2. read the named CSV in .astound/ba/trackers using workspace file tools (Descoped_items) — confirm it is not explicitly descoped.
3. **Ask the user explicitly**: "This feature is not in the current scope. Should I categorise this as a Change Request (CR)?"
4. Proceed ONLY after the user confirms it is a CR.

## Tools to use
| Tool | Purpose |
|---|---|
| `searchConfluenceUsingCql` | Verify the feature is not already in scope (Rovo MCP) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Check Descoped_items and existing Change_Log |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Log CR to Change_Log.csv and Questions.csv |
| write the named document in .astound/ba/documents using workspace file tools | Update Temp_FSD with CR feature entry |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Draft the Epic for the CR feature |
| `searchJiraIssuesUsingJql` | Check for related existing Jira tickets (Rovo MCP) |
| `lookupJiraAccountId` | Confirm the approver's account details (Rovo MCP) |

## Change_Log.csv mandatory columns
`cr_id` | `date` | `feature` | `description` | `approved_by` | `status`

- `cr_id`: Format `CR-001`, `CR-002` (sequential)
- `approved_by`: Verify against the Stakeholders Confluence page

## Guardrails
- NEVER draft a CR epic without explicit user confirmation that it IS a Change Request.
- ALWAYS log the CR to Change_Log BEFORE drafting any stories or epics for it.
- ALWAYS assign a sequential CR ID in the format `CR-NNN`.
- ALWAYS verify the approver against the Stakeholders Confluence page — do not accept unverified names.
- NEVER add CR scope into the main FSD — keep it in a separate Temp_FSD entry.
- Log the CR in Questions.csv as well as Change_Log.csv.
