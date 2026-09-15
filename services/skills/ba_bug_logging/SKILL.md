---
name: ba-bug-logging
description: Triage bug reports, check for duplicates, and log structured defect tickets.
title: Bug Logging
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

# Skill: Bug Logging

**Slug**: `bug-logging`
**Category**: authoring
**One-liner**: Triage bug reports, check for duplicates, and log structured defect tickets.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created or updated in Jira without human review.**
Before calling any write tool (Jira issue creation (`discover`, then `executeWrite`), save the draft as Markdown in .astound/ba/drafts using workspace file tools, Jira issue creation (`discover`, then `executeWrite`)):
1. Show the full bug report draft in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Execute only the reviewed, approved proposal through the authenticated connector
4. Never auto-submit, never proceed on assumed approval

---

## When to activate
- User reports an unexpected behaviour, regression, or defect
- QA or a stakeholder asks to log a bug ticket
- User shares an error log, screenshot description, or test step sequence

---

## Clarifying Questions Protocol (Mandatory Before Drafting)

Before writing the bug report, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Reproduce steps** | "Can you share the exact steps to reproduce this?" |
| 2 | **Expected vs actual** | "What was the expected result, and what actually happened?" |
| 3 | **Environment** | "Which environment did this occur in (dev/staging/prod)? Browser/device?" |
| 4 | **Parent Epic key** | "What is the parent Epic or Story key this bug is linked to?" |
| 5 | **Priority / severity** | "What is the severity — critical, major, minor, or cosmetic?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live Jira data first.

---

## Pre-flight Steps (Mandatory Before Drafting)
1. Jira issue search by JQL (`discover`, then `executeRead`) (similar description/component, issuetype = Bug) — check for duplicate bugs
2. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — **refuse** if the feature is descoped
3. Jira issue read (`discover`, then `executeRead`) on the linked story or epic — confirm the parent key is valid

If a duplicate is found:
- Show the duplicate ticket key and summary in chat
- Ask the user: "A similar bug already exists: [KEY]. Should I add a comment to it instead of creating a new ticket?"
- Do NOT create a new ticket without explicit confirmation

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Find duplicate bugs |
| Jira issue read (`discover`, then `executeRead`) | Verify parent story or epic key |
| read the named CSV in .astound/ba/trackers using workspace file tools | Check Descoped_items |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save bug draft for review |
| list, read, or edit .astound/ba/drafts using workspace file tools | View or edit saved drafts |
| Jira issue creation (`discover`, then `executeWrite`) | Push approved bug to Jira (explicit approval of the concrete proposal required) |
| Jira issue comment (`discover`, then `executeWrite`) | Add to existing ticket if duplicate found |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Update RTM after successful Jira push |

---

## Mandatory Bug Report Structure
1. **Title**: `[Bug] [Component] - [Brief description of failure]` — max 20 words
2. **Description**: One sentence summarising what is broken and in which environment
3. **Steps to Reproduce**: Numbered list, beginning with the exact entry point
4. **Expected Result**: What should have happened
5. **Actual Result**: What actually happened (include error messages verbatim if available)
6. **Environment**: OS, browser/device, app version, environment (dev/staging/prod)
7. **Severity / Priority**: Critical / Major / Minor / Cosmetic
8. **Attachments**: Note screenshot or log file names if provided
9. **Linked Story / Epic**: The parent story or epic this defect relates to

---

## Parent Epic / Story Linking (Mandatory)
Before pushing any bug to Jira:
1. **Ask for the parent Epic or Story key** if not already known
2. Pass it as the schema-supported parent relationship in the local draft and map it to the advertised relationship fields for Jira issue creation (`discover`, then `executeWrite`)
3. If the user doesn't know the key, call Jira issue search by JQL (`discover`, then `executeRead`) (by component or feature name) to find it

---

## RTM Update (Mandatory After Push)
After a successful Jira push:
1. Call append or update the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `RTM`) to add a defect row linking the bug key to the affected story
2. Confirm to the user: "RTM has been updated with bug [KEY]."
3. Never skip this step — it is required for traceability

---

## Guardrails
- NEVER push a bug without showing the full draft and getting explicit user approval
- NEVER create a duplicate — check via Jira issue search by JQL (`discover`, then `executeRead`) first
- NEVER invent Jira IDs — verify parent keys via Jira issue read (`discover`, then `executeRead`) before use
- NEVER log a bug for a descoped feature — run read the named CSV in .astound/ba/trackers using workspace file tools (Descoped_items) first
- ALWAYS pass the parent key as the schema-supported parent relationship in the local draft and map it to the advertised relationship fields for Jira issue creation (`discover`, then `executeWrite`)
- ALWAYS update the RTM after a successful push

---

## Jira Formatting Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing draft to user for review) | **Standard Markdown** — `### **Heading**`, `**Bold Text**`, `1.` lists |
| Jira remote write | **Markdown-style formatting** (`### **Heading**`, `**Bold Text**`) translated to the advertised connector content schema (ADF when required); wiki markup only when explicitly supported |

### JIRA Description Formatting (Markdown Style)
Use this formatting for all JIRA bug descriptions to ensure proper rendering:

- **Section headers**: Use `### **Section Name**` format (three hashes + bold text)
- **Subsection headers**: Use `**Subsection Name**` format (bold text only, no hashes)
- **Numbered lists**: Use standard `1.`, `2.`, `3.` numbering for steps
- **Bullet lists**: Use standard `-` for bullet points in environment details
- **Line breaks**: Include blank lines between major sections for visual separation
- **Never embed priority or severity labels inside description text** — pass them as Jira field parameters

### Example Jira description structure (Markdown style - PREFERRED FORMAT)
```
### **Description**

The checkout button fails to respond on the payment page when accessed via mobile Safari on staging.

### **Steps to Reproduce**

1. Navigate to the product listing page on a mobile Safari browser
2. Add an item to the cart
3. Proceed to checkout and fill in payment details
4. Click the "Complete Order" button

### **Expected Result**

The order is submitted and the user is redirected to the confirmation page.

### **Actual Result**

The button shows a loading spinner indefinitely. No order is created. Console shows: "TypeError: Cannot read property 'submit' of undefined".

### **Environment**

- OS: iOS 17 / Safari 17
- Environment: Staging
- App version: 3.2.1
```

### Important formatting rules:
- **NEVER use wiki markup** (h3., h4., #, ##, *, etc. in Jira descriptions)
- **DO use markdown-style** (`### **Heading**`, `**Bold**`)
- **Space sections** with blank lines for readability
- **Keep numbering** consistent (1., 2., 3., etc.)
- **Bold section headers** — they should stand out from content
