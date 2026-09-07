---
name: ba-epic-creation
description: Draft epics with persona, KPIs, scope, and story breakdown.
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

# Skill: Epic Creation

**Slug**: `epic-creation`
**Category**: authoring
**One-liner**: Draft epics with persona, KPIs, scope, and story breakdown.

## When to activate
- User asks to draft an epic or module-level feature group
- User asks to decompose an epic into individual stories
- User is starting a new module from scratch

## Pre-flight steps (mandatory before drafting)
1. `getVisibleJiraProjects` — confirm the active project key.
2. `searchConfluenceUsingCql` — research scope, architecture, and any industry context relevant to the feature.
3. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — cross-check that the module is not descoped.
4. `searchJiraIssuesUsingJql` (JQL: Epic type, matching module) — check for duplicate epics.

## Tools to use
| Tool | Purpose |
|---|---|
| `getVisibleJiraProjects` | Confirm project key and organisation context (Rovo MCP) |
| `searchConfluenceUsingCql` | Research scope, architecture, and industry docs (Rovo MCP) |
| `getConfluencePage` | Read architecture or industry context pages (Rovo MCP) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load Descoped_items |
| `searchJiraIssuesUsingJql` | Check for existing epics in the same module (Rovo MCP) |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save the epic draft for review |
| list, read, or edit .astound/ba/drafts using workspace file tools | View or edit epic drafts |
| `createJiraIssue` | Push the approved epic (explicit approval required) |

## Mandatory epic structure
**Description**
1. Persona-value narrative: "As a [persona], I want [action] so that [value]."

**Success Metrics (KPIs)**
1. At least 2 measurable, time-bound goals (e.g. "Increase checkout conversion by 10% within 2 sprints")

**In-Scope Features**
1. Numbered list of features included in this epic

**Descoped Items** (include only if relevant)
- Items explicitly excluded, cross-referenced from Descoped_items tracker

## After epic approval — story breakdown
- Propose 5–7 stories covering the epic's scope
- Separate FE and BE stories explicitly
- Map dependencies (BE before FE for blocking relationships)
- Show the full breakdown in chat before any Jira action

## Guardrails
- Present the epic for approval in chat BEFORE pushing to Jira.
- NEVER write labels inside the description text — always pass them as the `labels` parameter.
- ALWAYS ask for label values before creating in Jira.
- NEVER draft an epic for a descoped feature.
- NEVER assume a project key — verify via `getVisibleJiraProjects`.
