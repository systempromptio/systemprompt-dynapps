---
name: ba-jira-management
description: Transition issues, update fields, bulk-manage Jira operations.
title: Jira Management
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

# Skill: Jira Management

**Slug**: `jira-management`
**Category**: operations
**One-liner**: Transition issues, update fields, bulk-manage Jira operations.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created, updated, transitioned, or deleted in Jira without human review.**
Before calling any write tool (Jira issue update (`discover`, then `executeWrite`), Jira issue transition (`discover`, then `executeWrite`), Jira issue comment (`discover`, then `executeWrite`), Jira issue creation (`discover`, then `executeWrite`), Jira issue creation (`discover`, then `executeWrite`)):
1. Show a clear summary of every change in chat using Standard Markdown
2. For bulk actions: list every affected ticket key + proposed change
3. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
4. Never auto-submit, never proceed on assumed approval

---

## When to activate
- User asks to transition Jira issues (e.g. "move these stories to Done")
- User asks to update fields on existing Jira issues (description, labels, assignee, fix version)
- User asks to bulk-create or bulk-link Jira issues
- User asks to add comments or cross-reference links to Jira stories
- User asks for a summary of Jira project or sprint state

---

## Clarifying Questions Protocol (Mandatory Before Acting)

Before any Jira operation, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Target issue(s)** | "Which Jira issue keys are we updating?" |
| 2 | **Transition / field change** | "What status should these move to?" |
| 3 | **Confirmation** | "Can you confirm you want me to update all [N] issues at once?" |
| 4 | **Scope limit** | "Should this apply to the entire sprint or only specific stories?" |

---

## Pre-flight Steps (Mandatory Before Any Write)
1. Jira issue read (`discover`, then `executeRead`) on every target issue — verify the key exists and confirm current state
2. Jira transition listing (`discover`, then `executeRead`) — get the valid transition IDs (never guess transition names)
3. Jira issue search by JQL (`discover`, then `executeRead`) for bulk operations — list all matching issues and confirm scope with user

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue read (`discover`, then `executeRead`) | Read full issue details |
| Jira issue search by JQL (`discover`, then `executeRead`) | Query issues by sprint, epic, label, or component |
| Jira transition listing (`discover`, then `executeRead`) | Get valid transition IDs |
| Jira project listing (`discover`, then `executeRead`) | Confirm project key |
| Jira issue update (`discover`, then `executeWrite`) | Update fields on an issue (explicit approval required) |
| Jira issue transition (`discover`, then `executeWrite`) | Move issue to a new status (explicit approval required) |
| Jira issue comment (`discover`, then `executeWrite`) | Add a comment or cross-reference (explicit approval required) |
| Jira issue creation (`discover`, then `executeWrite`) | Create a new issue directly (explicit approval required) |
| Jira issue creation (`discover`, then `executeWrite`) | Push a draft issue to Jira (explicit approval of the concrete proposal required) |
| `getJiraIssueRemoteIssueLinks` | View external links on an issue |

---

## Parent Key for Child Issues (Mandatory)
When creating child stories or sub-tasks linked to an epic:
1. **Verify the epic key** via Jira issue read (`discover`, then `executeRead`) — never assume it
2. Pass the epic key as the schema-supported parent relationship in Jira issue creation (`discover`, then `executeWrite`) or Jira issue creation (`discover`, then `executeWrite`)
3. Without the schema-supported parent relationship, child issues are created as orphans — they will NOT appear under the epic in Jira

---

## Never Invent Jira IDs — Hard Rule
- **Never fabricate or guess** Jira issue keys, transition IDs, field IDs, or project keys
- Always verify via Jira issue read (`discover`, then `executeRead`), Jira issue search by JQL (`discover`, then `executeRead`), Jira transition listing (`discover`, then `executeRead`), or Jira project listing (`discover`, then `executeRead`) first
- If unsure, call the relevant API and show the result to the user before proceeding

---

## Output Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (previewing changes to user) | **Standard Markdown** — `**bold**`, `### Heading`, `| table |` |
| Jira field values (descriptions, comments) | **Advertised connector content schema**; wiki markup only when explicitly supported |

### Jira Wiki Markup for field values (wiki-capable schemas only)
- **Bold headings**: `*text*` — NEVER `###` or `h2.`
- **Numbered lists**: `#` for level 1, `##` for level 2
- **Double newlines**: Empty line between sections for hard returns in Jira

---

## Guardrails
- NEVER update or transition more than 10 issues in a single operation without user confirmation per batch
- NEVER guess transition IDs — always call Jira transition listing (`discover`, then `executeRead`) first
- NEVER invent Jira IDs — verify via API before every write
- NEVER add comments that contain invented or unverified information
- ALWAYS show a full preview of changes in chat before executing any write
- ALWAYS include the schema-supported parent relationship when creating child issues
