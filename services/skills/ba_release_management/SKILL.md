---
name: ba-release-management
description: Draft and publish release notes and release summary documents.
title: Release Management
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

# Skill: Release Management

**Slug**: `release-management`
**Category**: authoring
**One-liner**: Draft and publish release notes and release summary documents.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created or updated in Jira or Confluence without human review.**
Before calling any write tool (Confluence page creation (`discover`, then `executeWrite`), Confluence page update (`discover`, then `executeWrite`), Jira issue creation (`discover`, then `executeWrite`), save the draft as Markdown in .astound/ba/drafts using workspace file tools):
1. Show the full release notes draft in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Never auto-publish, never proceed on assumed approval

---

## When to activate
- User asks to draft release notes for a sprint or version
- User asks to create a release summary document on Confluence
- User asks to transition Jira issues to "Done" or update fix versions for a release
- User asks to check what was completed in the current sprint before a release

---

## Clarifying Questions Protocol (Mandatory Before Drafting)

Before drafting, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Release / sprint ID** | "What is the sprint name or fix version for this release?" |
| 2 | **Scope** — what is included? | "Should I pull all Done stories from the sprint, or specific epics only?" |
| 3 | **Audience** | "Is this for an internal team, stakeholders, or end-users?" |
| 4 | **Confluence space / parent page** | "Which Confluence space and parent page should I publish to?" |
| 5 | **Known issues** | "Are there any known issues or caveats to include?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live Jira/Confluence data first.

---

## Pre-flight Steps (Mandatory Before Drafting)
1. Jira issue search by JQL (`discover`, then `executeRead`) (sprint = active sprint, status = Done) — get the full list of completed stories
2. Confluence search by CQL (`discover`, then `executeRead`) — check if a release page already exists for this version
3. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `to_do_list`) — check for pending BA tasks related to this release
4. Jira issue search by JQL (`discover`, then `executeRead`) (sprint = active sprint, status != Done) — flag any incomplete stories

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Get completed and in-progress stories for the sprint |
| Jira issue read (`discover`, then `executeRead`) | Fetch full story details for release notes |
| Confluence search by CQL (`discover`, then `executeRead`) | Check for existing release page |
| Confluence page read (`discover`, then `executeRead`) | Read existing release document |
| Confluence page creation (`discover`, then `executeWrite`) | Publish new release notes page (explicit approval required) |
| Confluence page update (`discover`, then `executeWrite`) | Update existing release document (explicit approval required) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load to_do_list |
| Jira issue transition (`discover`, then `executeWrite`) | Transition stories to Done (explicit approval per-story required) |
| Jira issue update (`discover`, then `executeWrite`) | Update fix version on stories (explicit approval required) |

---

## Mandatory Release Notes Structure
1. **Release Version / Sprint**: Name and date
2. **Summary**: 2–3 sentence overview of what was delivered
3. **Features Delivered**: Grouped by epic — each story as a bullet point with Jira key + title
4. **Bug Fixes**: Defects resolved in this release (key + brief description)
5. **Known Issues**: Outstanding bugs or partial deliveries flagged explicitly
6. **Pending / Carried Over**: Stories not completed — with reason if known
7. **Dependencies Resolved**: Any cross-team or third-party dependencies closed in this release

---

## Guardrails
- NEVER publish to Confluence without showing the full draft and getting explicit user approval
- NEVER transition Jira issues to Done in bulk — present the full list and get per-batch or per-story approval
- NEVER invent Jira IDs — always query via Jira issue search by JQL (`discover`, then `executeRead`) or Jira issue read (`discover`, then `executeRead`)
- ALWAYS check if a release page already exists before creating a new one (avoid duplicates)
- ALWAYS include known issues and carried-over stories — never present an incomplete release as complete
- NEVER write personal opinions or sentiment in release notes — factual, professional language only

---

## Jira Formatting Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing draft to user for review) | **Standard Markdown** — `**bold**`, `### Heading`, `1.` lists |
| Jira remote write | **Markdown-style formatting** (`### **Heading**`, `**Bold Text**`) translated to the advertised connector content schema (ADF when required); wiki markup only when explicitly supported |

### Confluence page formatting rules (wiki-capable schemas only)
- **Bold headings**: Use `### **Release Version**` or markdown-style formatting for Confluence
- **Story references**: Use `[KEY|Jira URL]` format to create clickable links in Confluence
- **Double newlines**: Empty line between every major section for clean rendering
- **Tables**: Use `||Heading||` for headers and `|Cell|` for rows in Confluence

### Example Confluence table format (wiki-capable schemas only)
```
||Jira Key||Story Title||Epic||Status||
|EAICORE-101|Checkout page address selection FE|Address Management|Done|
|EAICORE-102|Order submission address ID BE|Address Management|Done|
```

### Example Jira description structure (Markdown style - PREFERRED FORMAT)
```
### **Release Summary**

Sprint 14 delivered the Address Management module (FE + BE) and resolved 3 critical checkout bugs.

### **Features Delivered**

**Address Management**
- EAICORE-101 - Checkout page address selection FE
- EAICORE-102 - Order submission address ID BE

### **Bug Fixes**

- EAICORE-145 - Fixed checkout button unresponsive on mobile Safari
```

### Important formatting rules:
- **NEVER use wiki markup** (h3., h4., #, ##, *, etc. in Jira descriptions)
- **DO use markdown-style** (`### **Heading**`, `**Bold**`)
- **Space sections** with blank lines for readability
- **Bold major sections** — they should stand out from content
