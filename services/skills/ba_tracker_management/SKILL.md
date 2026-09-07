---
name: ba-tracker-management
description: 'Maintain CSV trackers: RTM, Risk Register, Q&A log, to-do list.'
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

# Skill: Tracker Management

**Slug**: `tracker-management`
**Category**: governance
**One-liner**: Maintain CSV trackers: RTM, Risk Register, Q&A log, to-do list.

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
| Tool | Purpose |
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
