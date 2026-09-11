---
name: ba-dor-dod-validation
description: Validate stories against Definition of Ready (DoR) or Definition of Done
  (DoD) criteria.
title: DoR / DoD Validation
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

# Skill: DoR / DoD Validation

**Slug**: `dor-dod-validation`
**Category**: operations
**One-liner**: Validate stories against Definition of Ready (DoR) or Definition of Done (DoD) criteria.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets updated in Jira without human review.**
Before calling any write tool (Jira issue update (`discover`, then `executeWrite`), Jira issue comment (`discover`, then `executeWrite`), Jira issue transition (`discover`, then `executeWrite`)):
1. Show the full validation result table in chat using Standard Markdown
2. Clearly list any proposed Jira updates (comments, field changes, status transitions)
3. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
4. Never auto-update, never proceed on assumed approval

---

## When to activate
- User asks to check if a story is ready for sprint (DoR)
- User asks to check if a story is ready to close (DoD)
- User asks "is story X ready to bring into the sprint?"
- User asks to validate a batch of stories before sprint planning or sprint review

---

## Clarifying Questions Protocol (Mandatory Before Validation)

Before validating, check if ALL of the following are clear. Stop and ask if missing:

| # | What to check | Example question |
|---|---|---|
| 1 | **Validation type** | "Should I check DoR (ready to enter sprint) or DoD (ready to close)?" |
| 2 | **Target story/stories** | "Which Jira issue key(s) should I validate?" |
| 3 | **Custom criteria** | "Does this project have custom DoR/DoD criteria I should use?" |
| 4 | **Outcome** | "Should I add a comment to the story with the results?" |

Ask a **maximum of 5 targeted questions** per turn. Resolve what you can from live Jira data first.

---

## Pre-flight Steps
1. Jira issue read (`discover`, then `executeRead`) on each target story — read current fields (description, AC, labels, story points, assignee, parent)
2. Confluence search by CQL (`discover`, then `executeRead`) — check for project-specific DoR/DoD definitions
3. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `RTM`) — verify the story is in the RTM

---

## Tools to use
| Capability | Purpose |
|---|---|
| Jira issue read (`discover`, then `executeRead`) | Read full story details |
| Confluence search by CQL (`discover`, then `executeRead`) | Find project-specific DoR/DoD definitions |
| Confluence page read (`discover`, then `executeRead`) | Read DoR/DoD definition page |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load RTM |
| Jira issue comment (`discover`, then `executeWrite`) | Add validation result as a comment (explicit approval required) |
| Jira issue update (`discover`, then `executeWrite`) | Update fields to remediate DoR gaps (explicit approval required) |
| Jira issue transition (`discover`, then `executeWrite`) | Transition story after DoD pass (explicit approval required) |

---

## Definition of Ready (DoR) Checklist
Present as a table for each story:

| Criterion | Status | Notes |
|---|---|---|
| Title follows naming convention | PASS / FAIL | |
| Persona-value description present | PASS / FAIL | |
| Requirements section complete (no vague terms) | PASS / FAIL | |
| ADA reference as last Requirements point | PASS / FAIL | |
| Acceptance Criteria present (non-Gherkin) | PASS / FAIL | |
| Every AC scenario has device and navigation path | PASS / FAIL | |
| Story points estimated | PASS / FAIL | |
| Assignee set | PASS / FAIL | |
| Labels applied | PASS / FAIL | |
| Parent Epic linked using the schema-supported relationship | PASS / FAIL | |
| Design links in description | PASS / FAIL | |
| Dependencies listed (or "None") | PASS / FAIL | |
| Appears in RTM | PASS / FAIL | |

**Verdict**: READY / NOT READY (count of fails)

---

## Definition of Done (DoD) Checklist
Present as a table for each story:

| Criterion | Status | Notes |
|---|---|---|
| All AC scenarios passed (as per QA sign-off) | PASS / FAIL | |
| No open comments on the Jira story | PASS / FAIL | |
| No linked unresolved bugs | PASS / FAIL | |
| Code merged to main branch | PASS / FAIL | |
| RTM row updated with story status | PASS / FAIL | |
| Design QA sign-off received | PASS / FAIL | |
| Stakeholder sign-off received | PASS / FAIL | |

**Verdict**: DONE / NOT DONE (count of fails)

---

## Guardrails
- NEVER update or close a story without explicit user approval even if it passes all DoD criteria
- NEVER skip the parent Epic check in DoR — orphaned stories are a DoR fail
- NEVER invent Jira IDs — always call Jira issue read (`discover`, then `executeRead`) to verify
- ALWAYS present the full validation table before proposing any Jira update
- ALWAYS flag missing the schema-supported parent relationship as a DoR fail

---

## Output Standards

### Two-mode rule
| Context | Format to use |
|---|---|
| Chat (showing validation results) | **Standard Markdown** — `| table |`, `**bold**`, `### Heading` |
| Jira comment or field value | **Advertised connector content schema**; wiki markup only when explicitly supported |

### Jira Wiki Markup for validation comments (wiki-capable schemas only)
- **Verdict line**: `*DoR Result: NOT READY — 3 criteria failing*`
- **Table**: Use `||Criterion||Status||Notes||` header row, `|row|values|here|` for data
- **Double newlines**: Empty line between table and verdict line for Jira hard return
