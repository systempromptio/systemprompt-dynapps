---
name: ba-epic-creation
description: Draft epics with persona, KPIs, scope, and story breakdown.
title: Epic Creation
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

# Skill: Epic Creation

**Slug**: `epic-creation`
**Category**: authoring
**One-liner**: Draft epics with persona, KPIs, scope, and story breakdown.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created or updated in Jira without human review.**
Before calling any write tool (Jira issue creation (`discover`, then `executeWrite`), save the draft as Markdown in .astound/ba/drafts using workspace file tools, Jira issue creation (`discover`, then `executeWrite`)):
1. Show the full epic draft in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Execute only the reviewed, approved proposal through the authenticated connector
4. Never auto-submit, never proceed on assumed approval

---

## When to activate
- User asks to draft an epic or module-level feature group
- User asks to decompose an epic into individual stories
- User is starting a new module from scratch

---

## Clarifying Questions Protocol (Mandatory Before Drafting)

Before writing, check if ALL of the following are clear. Stop and ask if any are missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Project key** | "Which Jira project key should this epic go into?" |
| 2 | **Module / feature name** | "What is the module or feature name for this epic?" |
| 3 | **Persona** | "Who is the primary user persona for this epic?" |
| 4 | **Scope** — is this in-scope / not descoped? | Run read the named CSV in .astound/ba/trackers using workspace file tools (Descoped_items) first |
| 5 | **Labels / components** | "What labels or components should be attached?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live data first.

---

## Pre-flight Steps (Mandatory Before Drafting)
1. Jira project listing (`discover`, then `executeRead`) — confirm the active project key
2. Confluence search by CQL (`discover`, then `executeRead`) — research scope, architecture, and industry context
3. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — **refuse** if the module is descoped
4. Jira issue search by JQL (`discover`, then `executeRead`) (issuetype = Epic, matching module) — check for duplicate epics

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira project listing (`discover`, then `executeRead`) | Confirm project key |
| Confluence search by CQL (`discover`, then `executeRead`) | Research scope, architecture, industry docs |
| Confluence page read (`discover`, then `executeRead`) | Read architecture or industry context pages |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load Descoped_items |
| Jira issue search by JQL (`discover`, then `executeRead`) | Check for existing epics in the same module |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save the epic draft for review |
| list, read, or edit .astound/ba/drafts using workspace file tools | View or edit epic drafts |
| Jira issue creation (`discover`, then `executeWrite`) | Push the approved epic (explicit approval of the concrete proposal required) |

---

## Mandatory Epic Structure

**Description**
1. Persona-value narrative: "As a [persona], I want [action] so that [value]."

**Success Metrics (KPIs)**
1. At least 2 measurable, time-bound goals (e.g. "Increase checkout conversion by 10% within 2 sprints")

**In-Scope Features**
1. Numbered list of features included in this epic

**Descoped Items** (include only if relevant)
- Items explicitly excluded, cross-referenced from Descoped_items tracker

---

## After Epic Approval — Story Breakdown

Propose 5–7 stories covering the epic's scope:
- Separate FE and BE stories explicitly
- Map dependencies (BE before FE for blocking relationships)
- Show the full breakdown in chat **before any Jira action**

### Parent Key Requirement (Mandatory for Child Stories)
When creating child stories after the epic is pushed:
1. The pushed epic returns a Jira key (e.g. `EAICORE-42`)
2. Every child story **must** pass this key as the schema-supported parent relationship in the local draft and map it to the advertised relationship fields for Jira issue creation (`discover`, then `executeWrite`)
3. This is what creates the Epic → Story hierarchy in Jira — without it, stories are orphaned
4. **Always confirm the epic key with the user** before pushing child stories

---

## Guardrails
- NEVER push the epic without showing the full draft and getting explicit user approval
- NEVER write labels inside the description text — always pass them as the `labels` parameter
- ALWAYS ask for label values before creating in Jira
- NEVER draft an epic for a descoped feature
- NEVER assume a project key — verify via Jira project listing (`discover`, then `executeRead`)
- NEVER invent Jira IDs — verify via API first
- ALWAYS pass the epic's Jira key as the schema-supported parent relationship when subsequently creating child stories

---

## Jira Formatting Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing draft to user for review) | **Standard Markdown** — `### **Heading**`, `**Bold Text**`, `1.` lists |
| Jira remote write | **Markdown-style formatting** (`### **Heading**`, `**Bold Text**`) translated to the advertised connector content schema (ADF when required); wiki markup only when explicitly supported |

### JIRA Description Formatting (Markdown Style)
Use this formatting for all JIRA epic descriptions to ensure proper rendering:

- **Section headers**: Use `### **Section Name**` format (three hashes + bold text)
- **Subsection headers**: Use `**Subsection Name**` format (bold text only, no hashes)
- **Numbered lists**: Use standard `1.`, `2.`, `3.` numbering for features and KPIs
- **Bullet lists**: Use standard `-` for bullet points in dependencies and notes
- **Line breaks**: Include blank lines between major sections for visual separation
- **Labels**: Pass as the `labels` array parameter — never embed in description text

### Example Jira description structure (Markdown style - PREFERRED FORMAT)
```
### **Description**

As a customer, I want to manage my notification preferences so that I only receive relevant updates.

### **Success Metrics (KPIs)**

1. Reduce unsubscribe rate by 15% within 2 sprints
2. Achieve 80% opt-in rate for in-app notifications within 1 quarter

### **In-Scope Features**

1. Email notification preferences
2. SMS notification preferences
3. In-app notification preferences
```

### Important formatting rules:
- **NEVER use wiki markup** (h3., h4., #, ##, *, etc. in Jira descriptions)
- **DO use markdown-style** (`### **Heading**`, `**Bold**`)
- **Space sections** with blank lines for readability
- **Keep numbering** consistent (1., 2., 3., etc. not bullet points for features)
- **Bold major sections** — they should stand out from content
