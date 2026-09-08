---
name: ba-bug-logging
description: Log defects with severity, steps to reproduce, and reproducibility.
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

# Skill: Bug Logging

**Slug**: `bug-logging`
**Category**: authoring
**One-liner**: Log defects with severity, steps to reproduce, and reproducibility.

## When to activate
- QA found a defect during testing
- User wants to log a regression or unexpected behaviour
- User wants to check for duplicate bugs before creating a new one

## Pre-flight steps
1. Jira issue search by JQL (`discover`, then `executeRead`) (JQL: issue type = Bug, matching keywords) — check for existing duplicates.
2. Confirm severity level with the user (Critical / Major / Minor / Trivial).

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Check for duplicate bugs before creating |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save the bug draft for review |
| list, read, or edit .astound/ba/drafts using workspace file tools | View or edit the bug draft |
| Jira issue creation (`discover`, then `executeWrite`) | Push the approved bug (issue type = "Bug", explicit approval required) |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Update RTM if the bug links to a source story |
| Jira issue read (`discover`, then `executeRead`) | Fetch the linked source story for RTM reference |

## Mandatory bug structure
**Title**: `[BUG] | [Module] | [Observed behaviour] vs [Expected behaviour]`

**Summary**: One concise paragraph describing the defect

**Environment**:
- Application: (e.g. Staging — staging.client.com)
- Build: (commit hash or version)
- Browser/Device: (e.g. Chrome 124, Windows 11)
- User type: (Guest / Logged-in)
- Locale: (e.g. en-IN)

**Steps to Reproduce**: Numbered steps starting from app entry point

**Expected Result**: What should happen per the AC scenario (reference Jira ID if known)

**Actual Result**: What actually happens — include exact UI text

**Reproducibility**: Always / N of N attempts

**Severity**: Critical / Major / Minor / Trivial (per bug_writing_standards)

**Labels**: `["bug", "<module>"]`

## Guardrails
- NEVER use Gherkin in the bug description.
- ALWAYS check for duplicate bugs via Jira issue search by JQL (`discover`, then `executeRead`) before creating.
- ALWAYS get explicit user approval before pushing to Jira.
- Update RTM via append or update the named CSV in .astound/ba/trackers using workspace file tools ONLY if this bug is directly linked to a source story.
- Severity must be justified — ask the user if it is not clear from the defect description.
- Do NOT use vague actual result descriptions — include the exact error message or UI state.
