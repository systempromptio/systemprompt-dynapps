---
name: ba-release-management
description: Validate releases, manage versions, and generate release notes.
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

# Skill: Release Management

**Slug**: `release-management`
**Category**: planning
**One-liner**: Validate releases, manage versions, and generate release notes.

## When to activate
- Sprint is closing and release notes are needed
- User wants to validate whether a release is ready to ship
- User wants to create or manage Jira fix versions

## Tools to use
| Tool | Purpose |
|---|---|
| evaluate retrieved Jira issues against the project release checklist | Run all automated release readiness checks |
| summarize evidenced release checks and unresolved blockers | High-level readiness assessment summary |
| the advertised Atlassian capability for get jira versions (fail explicitly if unavailable) | List existing fix versions in the project |
| the advertised Atlassian capability for create jira version (fail explicitly if unavailable) | Create a new fix version (explicit approval required) |
| `searchJiraIssuesUsingJql` | Fetch all issues in a sprint for release scope |
| `searchJiraIssuesUsingJql` | Query done/remaining/deferred issues (Rovo MCP) |
| prepare an on-demand report and a schedule proposal in .astound/ba/reports; scheduling is unsupported | Trigger the `release_notes` scheduled job |
| write the named document in .astound/ba/documents using workspace file tools | Save the generated release notes document |
| `createConfluencePage` | Publish release notes to Confluence (Rovo MCP — explicit approval required) |

## Release notes output format
Group tickets in this order:
1. **New Features** — Stories with status "Done"
2. **Enhancements** — Stories with label "enhancement" and status "Done"
3. **Bug Fixes** — Bugs with status "Done"
4. **Known Issues** — Tickets with status "On Hold" or "Deferred"

Always show the draft to the user before saving or publishing.

## Guardrails
- NEVER create a Jira version without explicit user approval.
- ALWAYS show the release notes draft before saving or pushing to Confluence.
- If evaluate retrieved Jira issues against the project release checklist returns failures, list them clearly — do NOT declare the release ready.
- Known Issues section MUST include every ticket with status "On Hold" or "Deferred".
- Confirm the fix version name with the user before creating in Jira.
