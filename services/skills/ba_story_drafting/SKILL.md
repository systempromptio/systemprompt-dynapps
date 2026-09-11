---
name: ba-story-drafting
description: Draft FE/BE user stories per org writing standards.
title: Story Drafting
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

# Skill: Story Drafting

**Slug**: `story-drafting`
**Category**: authoring
**One-liner**: Draft FE/BE user stories per org writing standards.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets created or updated in Jira without human review.**
Before calling any write tool (Jira issue creation (`discover`, then `executeWrite`), save the draft as Markdown in .astound/ba/drafts using workspace file tools, append or update the named CSV in .astound/ba/trackers using workspace file tools):
1. Show the full draft in chat using Standard Markdown
2. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
3. Execute only the reviewed, approved proposal through the authenticated connector
4. Never auto-submit, never proceed on assumed approval

---

## When to activate
- User asks to draft, write, or create a user story (FE, BE, or paired FE+BE)
- User asks to refine or improve an existing story draft
- User asks to review a story against writing standards

---

## Clarifying Questions Protocol (Mandatory Before Drafting)

Before writing a single line, check if ALL of the following are clear. If any are missing, **stop and ask** — never guess:

| # | What to check | Example question |
|---|---|---|
| 1 | **What** — Story type (FE, BE, paired)? | "Should this be a FE story, BE story, or paired?" |
| 2 | **Who** — User persona? | "Who is the primary user for this feature?" |
| 3 | **Scope** — Is this feature in-scope / not descoped? | Run read the named CSV in .astound/ba/trackers using workspace file tools (Descoped_items) first |
| 4 | **Constraints** — Parent Epic key? | "What is the parent Epic key for this story?" |
| 5 | **Context** — FSD or Confluence spec page? | "Is there an FSD or spec page I should read first?" |

Ask a **maximum of 5 targeted questions** per turn — never a long list. Resolve what you can from live Jira/Confluence data first.

---

## Pre-flight Steps (Mandatory Before Drafting)
1. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — **refuse** if the feature is descoped
2. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Questions`) — check if similar logic was already clarified
3. Confluence search by CQL (`discover`, then `executeRead`) — find the FSD or scope document for the feature
4. Jira issue search by JQL (`discover`, then `executeRead`) (matching feature keywords) — check for existing similar stories

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue search by JQL (`discover`, then `executeRead`) | Check for existing similar stories |
| Confluence search by CQL (`discover`, then `executeRead`) | Find FSD, architecture, or scope documents |
| Confluence page read (`discover`, then `executeRead`) | Read the full FSD or architecture page |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load Descoped_items, Questions, to_do_list |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save the story as a draft for review before pushing |
| list, read, or edit .astound/ba/drafts using workspace file tools | View, edit, or delete saved drafts |
| Jira issue creation (`discover`, then `executeWrite`) | Push the approved story to Jira (explicit approval of the concrete proposal required) |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Update RTM after a successful Jira push |
| Jira issue comment (`discover`, then `executeWrite`) | Add cross-reference comments after push |

---

## Mandatory Story Structure
1. **Title**: `[Layer] | [Module/Epic] | [Action Summary]` — 15–20 words max
2. **Description**: Persona-value statement — "As a [User Role], I want [X] so that [Business Value]."
3. **Design Links**: From Confluence jira_details page
4. **Requirements**: Grouped logically (UI/UX, Interactions, API, Fail-safes) — last point = ADA reference link
5. **Acceptance Criteria**: Bold, unnumbered scenario headings with numbered step-by-step flows — NO Gherkin
6. **Dependencies**: Cross-story or third-party dependencies (write "None" if absent)
7. **Notes** (optional): Developer notes for BE stories with content slots only
8. **Technical Notes**: **LEAVE COMPLETELY BLANK — this is for developers only**

---

## Requirements Section Rules
- **ADA Reference**: The very last numbered point inside Requirements must be a hyperlinked ADA reference: `"ADA requirements are listed here: [Link]"`. Never put it in Description.
- **Form Validation**: For forms with input fields, do NOT write validation rules inline. Link to the Field Validations Confluence page in tabular format (Element, Page, Scenario, Mandatory?, Validation Rule, Error Message).
- **Fail-safes**: Define API failure fallbacks. If the architecture doesn't specify, write: "API failures will be logged for system review." If error message text isn't given in the FSD, draft an appropriate message — never leave it vague.
- **Grouping**: Use multi-level numbering to group related requirements (UI/UX Rendering, Interactions, API Integration) — never inline comma-separated lists.

---

## Acceptance Criteria Rules
- **NO Gherkin** (Given/When/Then) — hard block, zero exceptions
- **No vague terms**: Never use "all fields", "appropriate data", "relevant details", "etc." — list every item explicitly
- **Every error state** and every edit flow must have its own explicit scenario — never assume
- **Every field** in a form must have its behavior explicitly described — never group as "all fields"
- **Device in every scenario**: Always state desktop or mobile and the navigation path
- **Scenario format**: `**Scenario - Assuming a customer visits [page] via [device] and [action].**` (Markdown review; convert for the provider) — bold, not numbered, blank line before and after
- **Steps**: Multi-level numbered list (nested Markdown lists in drafts; schema-compatible lists for Jira) — first step = "Customer does [X]", subsequent = "they [Y]"
- **ADA last**: Final point of every scenario's AC must be: `"ADA compliance to be maintained."`
- **Technical restraint**: No HTTP status codes, no CSS classes, no exact API response keys in AC

---

## Guardrails
- NEVER use Gherkin (Given/When/Then) — this is a hard block
- NEVER use vague terms in AC — list every item explicitly
- NEVER specify HTTP status codes or exact API response keys in BE stories
- NEVER invent Jira IDs — always verify via Jira issue read (`discover`, then `executeRead`) or Jira issue search by JQL (`discover`, then `executeRead`) first
- ALWAYS separate FE and BE stories when parallel development is possible; BE first, FE depends on BE
- ALWAYS state device (desktop / mobile) and navigation path in every AC scenario
- ALWAYS add the ADA reference link as the last numbered point in Requirements
- ALWAYS ask for the parent Epic key before pushing — pass it as the schema-supported parent relationship in the local draft and map it to the advertised relationship fields for Jira issue creation (`discover`, then `executeWrite`)
- ALWAYS update RTM via append or update the named CSV in .astound/ba/trackers using workspace file tools immediately after a successful Jira push
- NEVER draft a story for a descoped feature — run read the named CSV in .astound/ba/trackers using workspace file tools (Descoped_items) first
- NEVER write labels inside the description — always pass them as the `labels` parameter

---

## Parent Epic Linking (Mandatory)
Before pushing any story to Jira:
1. **Ask the user for the parent Epic key** (e.g. `EAICORE-5`)
2. Pass it as the schema-supported parent relationship in the local draft and map it to the advertised relationship fields for Jira issue creation (`discover`, then `executeWrite`)
3. This is what creates the Epic → Story hierarchy in Jira — without it, the story is orphaned
4. If the user doesn't know the Epic key, call Jira issue search by JQL (`discover`, then `executeRead`) (issuetype = Epic) to find it

---

## Jira Formatting Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing draft to user for review) | **Standard Markdown** — `**bold**`, `### Heading`, `1.` lists |
| Jira remote write | **Advertised connector content schema**; wiki markup only when explicitly supported |

### Jira Wiki Markup rules (wiki-capable schemas only)
- **Bold headings**: `*Requirements*`, `*Acceptance Criteria*`, `*Description*`, `*Technical Notes*` — use `*text*`, NEVER `###` or `h2.`
- **Numbered lists**: `#` for level 1, `##` for level 2, `###` for level 3
- **Bullet lists**: `*` for level 1, `**` for level 2
- **Double newlines**: Empty line between every major section and heading so Jira renders hard returns (not soft wraps)
- **Scenario line**: `*Scenario - Assuming a customer...*` — bold, NOT numbered, blank line before AND after
- **Steps under scenario**: Start immediately after the blank line, use `#` for numbered steps

### Example Jira description structure (wiki-capable schemas only)
```
*Description*

As a customer, I want to reset my password so that I can regain account access.

*Requirements*

# UI/UX
## The page displays a password reset form
## The form includes an email input field

*Acceptance Criteria*

*Scenario - Assuming a customer visits the login page via a desktop device and clicks "Forgot Password".*

# Customer clicks "Forgot Password" on the login page
# They are navigated to the password reset page
## The page title displays "Reset Your Password"
# ADA compliance to be maintained.

*Technical Notes*

```
