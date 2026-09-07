---
name: ba-story-drafting
description: Draft FE/BE user stories per org writing standards.
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

# Skill: Story Drafting

**Slug**: `story-drafting`
**Category**: authoring
**One-liner**: Draft FE/BE user stories per org writing standards.

## When to activate
- User asks to draft, write, or create a user story (FE, BE, or paired FE+BE)
- User asks to refine or improve an existing story draft
- User asks to review a story against writing standards

## Pre-flight steps (mandatory before drafting)
1. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Descoped_items`) — refuse if the feature is descoped.
2. read the named CSV in .astound/ba/trackers using workspace file tools (tracker_name = `Questions`) — check if similar logic was already clarified.
3. `searchConfluenceUsingCql` — find the FSD or scope document for the feature.
4. `searchJiraIssuesUsingJql` (JQL: matching feature keywords) — check for existing similar stories to ensure consistency.

## Tools to use
| Tool | Purpose |
|---|---|
| `searchJiraIssuesUsingJql` | Check for existing similar stories (consistency check) (Rovo MCP) |
| `searchConfluenceUsingCql` | Find FSD, architecture, or scope documents (Rovo MCP) |
| `getConfluencePage` | Read the full FSD or architecture page (Rovo MCP) |
| read the named CSV in .astound/ba/trackers using workspace file tools | Load Descoped_items, Questions, to_do_list |
| save the draft as Markdown in .astound/ba/drafts using workspace file tools | Save the story as a draft for review before pushing |
| list, read, or edit .astound/ba/drafts using workspace file tools | View, edit, or delete saved drafts |
| `createJiraIssue` | Push the approved story to Jira (explicit approval required) |
| append or update the named CSV in .astound/ba/trackers using workspace file tools | Update RTM after a successful Jira push |
| `addCommentToJiraIssue` | Add cross-reference comments after push (Rovo MCP) |

## Mandatory story structure
1. **Title**: `[Layer] | [Module/Epic] | [Action Summary]` — 15–20 words max
2. **Description**: Persona-value statement — "As a [User Role], I want [X] so that [Business Value]."
3. **Design Links**: From Confluence jira_details page
4. **Requirements**: Grouped logically (UI/UX, Interactions, API, Fail-safes) — last point = ADA reference link
5. **Acceptance Criteria**: Numbered scenarios with step-by-step flows — NO Gherkin
6. **Dependencies**: Cross-story or third-party dependencies (write "None" if absent)
7. **Notes** (optional): Developer notes for BE stories with content slots only
8. **Technical Notes**: **LEAVE COMPLETELY BLANK**

## Guardrails
- NEVER use Gherkin (Given/When/Then) in AC — this is a hard block.
- NEVER use vague terms in AC ("all fields", "appropriate details", "etc.") — list every item explicitly.
- NEVER specify HTTP status codes or exact API response keys in BE stories — describe functional outcomes only.
- ALWAYS separate FE and BE stories when parallel development is possible; BE first, FE depends on BE.
- ALWAYS state the device (desktop / mobile) and navigation path in every AC scenario.
- ALWAYS add the ADA reference link as the last numbered point in Requirements.
- If the story involves form inputs, link to the Field Validations Confluence page — do NOT write validation rules inline.
- ALWAYS pause and ask for the parent Epic key before pushing to Jira.
- After a successful Jira push, immediately update RTM via append or update the named CSV in .astound/ba/trackers using workspace file tools.
- Cross-reference other stories by Jira ID, not by placeholder names or FSD IDs.

## Output formats
- **Chat (draft review)**: Standard Markdown (`1.`, `**bold**`, `#` headings)
- **Jira push**: Use the content format required by the advertised Jira tool schema.
