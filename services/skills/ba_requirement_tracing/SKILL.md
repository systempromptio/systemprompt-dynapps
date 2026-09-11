---
name: ba-requirement-tracing
description: Trace requirements to Jira tickets and maintain the RTM.
title: Requirement Tracing
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

Chat and local drafts use Standard Markdown. Preserve the requested headings,
list hierarchy, bold scenario lines, spacing, and complete acceptance criteria
when converting to the advertised Jira or Confluence content schema. Wiki-markup
rules and examples below apply only when that schema accepts wiki markup;
otherwise translate their structure to the supported format, including ADF when
required. Never send wiki markup as literal text in an ADF document.

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

# Skill: Requirement Tracing

**Slug**: `requirement-tracing`
**Category**: governance
**One-liner**: Trace requirements to Jira tickets and maintain the RTM.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets written to the RTM or any tracker without human review.**
Before calling any write tool (append or update the named CSV in .astound/ba/trackers using workspace file tools, Jira issue update (`discover`, then `executeWrite`)):
1. Show the proposed RTM rows in chat using a Standard Markdown table
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Never auto-append, never proceed on assumed approval

---

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
