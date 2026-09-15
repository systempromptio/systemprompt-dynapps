---
name: ba-sprint-planning
description: "Run sprint planning sessions \u2014 prioritise backlog, size stories,\
  \ suggest sprint goals."
title: Sprint Planning
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

# Skill: Sprint Planning

**Slug**: `sprint-planning`
**Category**: operations
**One-liner**: Run sprint planning sessions — prioritise backlog, size stories, suggest sprint goals.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created, updated, or assigned in Jira without human review.**
Before calling any write tool (Jira issue update (`discover`, then `executeWrite`), Jira issue transition (`discover`, then `executeWrite`), Jira issue creation (`discover`, then `executeWrite`)):
1. Show the full proposed sprint plan in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Never auto-move stories into a sprint, never auto-assign without approval

---

## When to activate
- User asks to plan the next sprint
- User asks to prioritise or reorder the backlog
- User asks for a sprint capacity estimate
- User asks to recommend which stories should go into a sprint
- User asks for a sprint retrospective summary

---

## Clarifying Questions Protocol (Mandatory Before Planning)

Before running any sprint planning, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Sprint goal** | "What is the primary goal or theme for this sprint?" |
| 2 | **Team capacity** | "How many story points or days of capacity does the team have?" |
| 3 | **Priorities** | "Are there any must-have stories that must go in first?" |
| 4 | **Carryover** | "Are there any incomplete stories from the previous sprint to carry over?" |
| 5 | **Blocked items** | "Are there any blocked stories that should be excluded?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live Jira data first.

---

## Pre-flight Steps
1. Jira issue search by JQL (`discover`, then `executeRead`) (status = "Backlog", project = active project) — load the current backlog
2. Jira issue search by JQL (`discover`, then `executeRead`) (sprint = "active sprint", status != Done) — check for carryover stories
3. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `to_do_list`) — review any BA-flagged planning tasks
4. Jira issue search by JQL (`discover`, then `executeRead`) (status = "Blocked") — surface blocked stories

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Query backlog, current sprint, blocked items |
| Jira issue read (`discover`, then `executeRead`) | Read full story details for sizing |
| Jira project listing (`discover`, then `executeRead`) | Confirm active project |
| Jira issue update (`discover`, then `executeWrite`) | Update sprint, priority, or story points (explicit approval required) |
| Jira issue comment (`discover`, then `executeWrite`) | Add planning notes or blockers to stories (explicit approval required) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load to_do_list |
| Jira issue creation (`discover`, then `executeWrite`) | Create new planning stories (explicit approval of the concrete proposal required) |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save story drafts for review during planning |

---

## Sprint Planning Output Format
Present the proposed sprint plan as a table:

| # | Jira Key | Story Title | Points | Priority | Notes |
|---|---|---|---|---|---|
| 1 | EAICORE-45 | Checkout FE | 5 | High | Blocked on API — confirm before include |
| 2 | EAICORE-46 | Address BE | 3 | High | Ready for dev |

Followed by:
- **Sprint Goal**: One sentence summary of the sprint's primary objective
- **Total Capacity**: Points committed vs team capacity
- **Risks**: Blocked, missing dependencies, or under-defined stories
- **Excluded from Sprint**: Stories considered but not included, with reason

---

## Guardrails
- NEVER move stories into a sprint without explicit user approval
- NEVER size stories — only suggest sizes based on complexity; sizing is the team's decision
- NEVER invent Jira IDs — verify all keys via API
- ALWAYS surface blocked stories and dependency risks before finalising the plan
- ALWAYS show the full sprint plan in chat for approval before making any Jira updates
- ALWAYS include the schema-supported parent relationship when creating new child stories during planning

---

## Output Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing plan to user for review) | **Standard Markdown** — tables, `**bold**`, `### Heading` |
| Jira field values (descriptions, comments) | **Markdown-style formatting** — use `### **Heading**` and `**Bold Text**` for maximum compatibility |

### JIRA Description Formatting (Markdown Style)
- **Section headers**: Use `### **Section Name**` format (three hashes + bold text)
- **Subsection headers**: Use `**Subsection Name**` format (bold text only, no hashes)
- **Bold text**: Use `**text**` format instead of `*text*`
- **Double newlines**: Empty line between sections for hard returns in Jira

### Important formatting rules:
- **NEVER use wiki markup** (h3., h4., #, ##, *, etc. in Jira descriptions)
- **DO use markdown-style** (`### **Heading**`, `**Bold**`)
- **Space sections** with blank lines for readability
