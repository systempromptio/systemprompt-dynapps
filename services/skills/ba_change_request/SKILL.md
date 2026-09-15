---
name: ba-change-request
description: Analyse scope changes, assess impact, and log structured change request
  tickets.
title: Change Request
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

# Skill: Change Request

**Slug**: `change-request`
**Category**: authoring
**One-liner**: Analyse scope changes, assess impact, and log structured change request tickets.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created or updated in Jira or Confluence without human review.**
Before calling any write tool (Jira issue creation (`discover`, then `executeWrite`), save the draft as Markdown in .astound/ba/drafts using workspace file tools, Confluence page creation (`discover`, then `executeWrite`), append or update the named CSV in .astound/ba/trackers using workspace file tools):
1. Show the full change request draft in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Execute only the reviewed, approved proposal through the authenticated connector
4. Never auto-submit, never proceed on assumed approval

---

## When to activate
- A stakeholder proposes a scope change
- User asks to assess the impact of changing or adding a requirement
- User asks to log a CR ticket for a confirmed scope change
- User asks "what would be affected if we changed [X]?"

---

## Clarifying Questions Protocol (Mandatory Before Drafting)

Before drafting the CR, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Change description** | "Can you describe exactly what is changing?" |
| 2 | **Original requirement** | "What was the original requirement or feature?" |
| 3 | **Reason for change** | "What is the business reason for this change?" |
| 4 | **Affected stories / epics** | "Which existing Jira stories or epics does this affect?" |
| 5 | **Priority** | "Is this blocking the current sprint or a future sprint?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live Jira/Confluence data first.

---

## Pre-flight Steps (Mandatory Before Drafting)
1. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — flag if the changed feature was previously descoped
2. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `RTM`) — identify which existing stories are impacted
3. Jira issue search by JQL (`discover`, then `executeRead`) (matching the changed area) — find related open stories and epics
4. Confluence search by CQL (`discover`, then `executeRead`) — find the FSD or scope document for the original requirement

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Find related stories and epics |
| Jira issue read (`discover`, then `executeRead`) | Read full details of affected tickets |
| Confluence search by CQL (`discover`, then `executeRead`) | Find original FSD or spec |
| Confluence page read (`discover`, then `executeRead`) | Read original requirement details |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load RTM, Descoped_items |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save CR draft for review |
| list, read, or edit .astound/ba/drafts using workspace file tools | View or edit saved drafts |
| Jira issue creation (`discover`, then `executeWrite`) | Push approved CR ticket (explicit approval of the concrete proposal required) |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Update RTM after successful push |
| Jira issue update (`discover`, then `executeWrite`) | Update affected stories (explicit approval required) |
| Jira issue comment (`discover`, then `executeWrite`) | Add CR cross-reference comment to affected stories |

---

## Mandatory CR Structure
1. **Title**: `[CR] [Feature/Module] - [Brief change description]` — max 20 words
2. **Change Description**: What is changing, phrased as original → new behaviour
3. **Business Justification**: Why the change is needed
4. **Impact Analysis**:
   - List of affected Jira stories (by key + summary)
   - Stories that need to be created (net new work)
   - Stories that need to be updated
   - Stories that may be descoped as a result
5. **Effort Estimate**: Rough sizing in story points or days, flagged as "TBD" if unknown
6. **Dependencies**: Any other teams, systems, or third-party dependencies
7. **Risk**: What is the risk of NOT making this change? What is the risk of making it?

---

## Descope Pre-flight Check
Before recommending any descoping as part of the CR impact:
1. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — ensure the item is not already descoped
2. Add any newly descoped items to the Descoped_items tracker via append or update the named CSV in .astound/ba/trackers using workspace file tools
3. Never silently descope — always surface to the user and get confirmation

---

## RTM Update (Mandatory After Push)
After a successful Jira push:
1. Call append or update the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `RTM`) to update affected story rows with the CR ticket key
2. Add the CR ticket key as a reference in the `Change_Requests` column of the RTM
3. Confirm to the user: "RTM has been updated to reflect CR [KEY]."
4. Never skip this step — it is required for traceability

---

## Guardrails
- NEVER push a CR without showing the full draft and getting explicit user approval
- NEVER invent Jira IDs — verify all affected keys via Jira issue read (`discover`, then `executeRead`) or Jira issue search by JQL (`discover`, then `executeRead`) first
- NEVER modify or descope existing stories without explicit user approval per story
- ALWAYS surface conflicting descoped items before proceeding
- ALWAYS update the RTM after a successful push
- ALWAYS link the CR ticket to all affected stories via Jira issue comment (`discover`, then `executeWrite`)

---

## Jira Formatting Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing draft to user for review) | **Standard Markdown** — `### **Heading**`, `**Bold Text**`, `1.` lists |
| Jira remote write | **Markdown-style formatting** (`### **Heading**`, `**Bold Text**`) translated to the advertised connector content schema (ADF when required); wiki markup only when explicitly supported |

### JIRA Description Formatting (Markdown Style)
Use this formatting for all JIRA change request descriptions to ensure proper rendering:

- **Section headers**: Use `### **Section Name**` format (three hashes + bold text)
- **Subsection headers**: Use `**Subsection Name**` format (bold text only, no hashes)
- **Numbered lists**: Use standard `1.`, `2.`, `3.` numbering for analysis
- **Bullet lists**: Use standard `-` for bullet points in stories and dependencies
- **Line breaks**: Include blank lines between major sections for visual separation

### Example Jira description structure (Markdown style - PREFERRED FORMAT)
```
### **Change Description**

Original: The checkout page displays a single shipping address field.
New: The checkout page allows the customer to save and select from multiple saved addresses.

### **Business Justification**

Repeat customers currently re-enter shipping details on every order. Saved addresses are expected to reduce checkout abandonment by 12%.

### **Impact Analysis**

**Affected stories:**
- EAICORE-101 - Checkout page FE — needs update to support address selection UI
- EAICORE-102 - Order submission BE — needs update to pass selected address ID

**Net new stories required:**
- Address management FE story (new)
- Address management BE story (new)

### **Risk**

Without this change: High checkout abandonment for repeat customers.
With this change: 2 additional stories required, potentially delaying sprint goal by 3 days.
```

### Important formatting rules:
- **NEVER use wiki markup** (h3., h4., #, ##, *, etc. in Jira descriptions)
- **DO use markdown-style** (`### **Heading**`, `**Bold**`)
- **Space sections** with blank lines for readability
- **Keep numbering** consistent (1., 2., 3., etc.)
- **Bold major sections** — they should stand out from content
