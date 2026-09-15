---
name: ba-notification-sending
description: Prepare unsent Slack and email drafts for BA events and updates.
title: Notification Sending
tags:
- india
- business-analysis
category: business
display_category: Business Analysis
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
For story creation, apply the writing checks below before the Jira issue write.
Use its advertised content schema, not an assumed legacy wiki or ADF format.
Notification and scheduling operations produce drafts/proposals only; explicitly
report that sending or scheduling was not executed. Never invent a job history.

### Review, relationships, and formatting

Approval covers the concrete draft, target, and proposed changes shown to the
user. Wait for explicit confirmation before those writes; approval of an earlier
unchanged proposal remains valid. A material change requires renewed review.
Use the connector's normal authorization; this workspace has no BA confirmation
token service. Never request or manufacture such a token.

Verify the parent issue and the relationship allowed for the project's issue
types through authenticated reads. Store that relationship in the local draft,
then use the advertised parent or issue-link fields when creating the issue.
Do not assume a field named parentKey exists, or that a Bug can be a child of a
Story. If the required relationship cannot be represented, stop the affected
write and explain the missing capability. Verify the saved relationship.

Chat, local drafts, and Jira text follow the Markdown style specified below
(`### **Heading**`, `**Bold Text**`, `1.` lists). Preserve the requested
headings, list hierarchy, bold scenario lines, spacing, and complete acceptance
criteria when translating to the advertised Jira or Confluence content schema,
including ADF when required. Wiki-markup rules below apply only when that
schema accepts wiki markup. Never send wiki markup or literal Markdown syntax as
plain text in an ADF document.

### Tracker review and partial failures

Show the proposed tracker rows and old-to-new values before changing them,
including RTM follow-ups to a Jira write. Approval may cover the displayed Jira
draft and the corresponding tracker rows together, with the returned Jira key
filled in after success. If rows were not reviewed, request their approval first.
An instruction below to update RTM immediately is subject to that review.
Update RTM only after confirmed Jira success, using the actual returned key.
If Jira fails or its outcome is unknown, do not log success or blindly retry.
If Jira succeeds but the tracker write fails, report the issue key and pending
rows; retry only the tracker update after checking for an existing row. Report
RTM completion only after reading back the saved rows. Never recreate the issue
to recover a tracker failure.

# Skill: Notification Sending

**Slug**: `notification-sending`
**Category**: communication
**One-liner**: Prepare unsent Slack and email drafts for BA events and updates.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Review the notification draft before saving it. Sending is unsupported, even after approval.**
Before calling any write tool (prepare an unsent Slack draft in .astound/ba/notifications; sending is unsupported, prepare an unsent email draft in .astound/ba/notifications; sending is unsupported):
1. Show the full message draft (channel/recipient + subject + body) in chat
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Never auto-send, never proceed on assumed approval
4. Confirm target channel name/ID or email address(es) with the user before saving the unsent draft

---

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
- NEVER send a notification from this workspace — show the draft for explicit review before saving.
- NEVER include sensitive information (credentials, personal data, confidential decisions) in notifications.
- Always confirm the target channel or email address with the user before calling the tool.
- For Slack threads: confirm the thread timestamp before saving a reply draft so it identifies the intended thread.
- Notifications are one-way — do NOT use them to seek approvals or decisions from stakeholders.
