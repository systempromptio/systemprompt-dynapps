---
name: ba-tracker-management
description: 'Maintain CSV trackers: RTM, Risk Register, Q&A log, to-do list.'
title: Tracker Management
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

# Skill: Tracker Management

**Slug**: `tracker-management`
**Category**: governance
**One-liner**: Maintain CSV trackers: RTM, Risk Register, Q&A log, to-do list.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets written to or deleted from any tracker without human review.**
Before calling any write tool (append or update the named CSV in .astound/ba/trackers using workspace file tools):
1. Show the exact rows to be added or updated in chat as a Standard Markdown table
2. For updates to existing rows: show old value → new value
3. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
4. Never auto-append or overwrite, never proceed on assumed approval

---

## When to activate
- User wants to read, update, or append to any BA tracker
- A workflow step requires logging (a CR, a risk, a clarification, a deferred to-do)
- User wants to view the current state of a tracker

## Available trackers and their purpose
| Tracker name | Purpose | Key columns |
|---|---|---|
| `Questions.csv` | Log all BA clarifications | timestamp, epic, feature/story, question, answer |
| `Requirement_Traceability_Matrix.csv` | Map FSD IDs to Jira ticket IDs | rtm_id, jira_id, fsd_id, description |
| `Risk_Register.csv` | Log project risks | risk_id, description, likelihood, impact, mitigation, status |
| `Change_Log.csv` | Record approved Change Requests | cr_id, date, feature, description, approved_by, status |
| `to_do_list.csv` | Track deferred and pending tasks | date, description, assignee, status, last_updated |
| `Descoped_items.csv` | Guard against scope creep | feature_id, feature_name, reason |
| `capacity.csv` | Dev/QA capacity per sprint | sprint, fe_points, be_points, total_points, notes |

## Tools to use
| Capability | Purpose |
|---|---|
| read the named CSV in .astound/ba/trackers using workspace file tools | Read any tracker by name |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Append or update rows in any tracker |

## Guardrails
- NEVER overwrite or delete live tracker rows without explicit user approval.
- ALWAYS append new rows at the END of the tracker — never insert between existing rows.
- Questions.csv timestamp format: `DD-MM-YYYY | HH:MM`
- to_do_list.csv: when a task is completed, update `status` to `Completed` and set `last_updated`.
- Use double quotes around `question` and `answer` fields in Questions.csv to handle commas correctly.
- Risk IDs must be sequential (RISK-001, RISK-002).
- CR IDs must be sequential (CR-001, CR-002).
- RTM IDs must be sequential (RTM-001, RTM-002).
