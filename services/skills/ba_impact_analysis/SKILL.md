---
name: ba-impact-analysis
description: Analyze scope and story impact of a proposed change.
title: Impact Analysis
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

# Skill: Impact Analysis

**Slug**: `impact-analysis`
**Category**: governance
**One-liner**: Analyze scope and story impact of a proposed change.

> **Standalone use:** This skill is self-contained. When used without `/ba-orchestrator`, the business rules below and Systemprompt execution contract above form your instruction set — `ba_persona` and `writing_standards` are not separately loaded.

---

## ⛔ Human Approval Required — Non-Negotiable

**Nothing gets written to trackers or Jira without human review.**
Before calling any write tool (append or update the named CSV in .astound/ba/trackers using workspace file tools, Jira issue update (`discover`, then `executeWrite`), Jira issue comment (`discover`, then `executeWrite`)):
1. Show the full impact analysis output in chat using Standard Markdown
2. Present proposed tracker rows (Risk_Register, Change_Log) as a preview table
3. Wait for explicit user confirmation of the concrete proposal (existing approval of that unchanged proposal remains valid)
4. Never auto-log risks or changes, never proceed on assumed approval

---

## When to activate
- User proposes a new feature, change, or architectural shift
- User wants to understand what existing stories or epics are affected by a change
- Product Owner asks "what changes if we do X?"

## Tools to use
| Capability | Purpose |
|---|---|
| analyze the retrieved scope and Jira evidence using the output structure below | Run structured impact analysis against current scope |
| Jira issue search by JQL (`discover`, then `executeRead`) | Find tickets that may be affected by the change |
| Jira issue read (`discover`, then `executeRead`) | Inspect specific tickets for impact depth |
| Confluence search by CQL (`discover`, then `executeRead`) | Find architecture or scope docs related to the change |
| Confluence page read (`discover`, then `executeRead`) | Read the full architecture or scope document |
| read the named CSV in .astound/ba/trackers using workspace file tools | Check Change_Log and Risk_Register for related entries |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Log newly identified risks to Risk_Register |

## Impact analysis output structure
1. **Scope Impact**: Is this change in-scope, out-of-scope, or a Change Request?
2. **Affected Jira Tickets**: List of tickets (ID + title + impact type)
3. **Risk Assessment**: New risks introduced — severity + likelihood
4. **Recommendation**: One of:
   - Proceed as-is (no scope change)
   - Raise a Change Request (triggers `change-request` skill)
   - Descope the conflicting item

## Guardrails
- NEVER proceed if the change clearly qualifies as a CR — activate the `change-request` skill instead.
- ALWAYS log identified risks to append or update the named CSV in .astound/ba/trackers using workspace file tools (Risk_Register) — do NOT only mention them in chat.
- If the change contradicts existing scope, surface the conflict explicitly and ask for a decision.
- Risk_Register entry must include: `risk_id`, `description`, `likelihood`, `impact`, `mitigation`, `status`.
