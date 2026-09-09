---
name: ba-notification-sending
description: Prepare unsent Slack and email drafts for BA events and updates.
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

# Skill: Notification Sending

**Slug**: `notification-sending`
**Category**: communication
**One-liner**: Prepare unsent Slack and email drafts for BA events and updates.

## When to activate
- User wants to notify the team about a sprint plan, release, or BA update via Slack
- User wants to send an email summary of a BA activity or report
- A scheduled job result needs to be shared with stakeholders

## Tools to use
| Capability | Purpose |
|---|---|
| prepare an unsent Slack draft in .astound/ba/notifications; sending is unsupported | Save an unsent Slack message or thread-reply draft |
| prepare an unsent email draft in .astound/ba/notifications; sending is unsupported | Save an unsent email draft with recipient, subject and body |

## Draft review checklist
For **Slack**:
- Confirm the target channel name or ID with the user
- Show the full message text for review; sending is unsupported

For **Email**:
- Confirm the recipient address(es) with the user
- Show subject line and full body for review; sending is unsupported

## Guardrails
- NEVER send a notification without explicit user approval — always show the draft first.
- NEVER include sensitive information (credentials, personal data, confidential decisions) in notifications.
- Always confirm the target channel or email address with the user before calling the tool.
- For Slack threads: confirm the thread timestamp before replying to avoid posting to the wrong thread.
- Notifications are one-way — do NOT use them to seek approvals or decisions from stakeholders.
