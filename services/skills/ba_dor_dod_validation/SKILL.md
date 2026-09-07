---
name: ba-dor-dod-validation
description: Validate stories against Definition of Ready and Definition of Done.
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

# Skill: DoR / DoD Validation

**Slug**: `dor-dod-validation`
**Category**: governance
**One-liner**: Validate stories against Definition of Ready and Definition of Done.

## When to activate
- Before sprint planning, to determine which stories are truly Ready
- Before transitioning a story to "Done", to verify all completion criteria are met
- User asks to batch-validate multiple stories for sprint inclusion

## Tools to use
| Tool | Purpose |
|---|---|
| `searchJiraIssuesUsingJql` | Fetch stories for batch validation (Rovo MCP) |
| `getJiraIssue` | Inspect a specific story's fields and AC completeness (Rovo MCP) |
| read the named project document using workspace file tools | Read definition_of_ready.md / definition_of_done.md |
| `transitionJiraIssue` | Move to "Done" if DoD is satisfied (Rovo MCP — explicit approval required) |
| `addCommentToJiraIssue` | Add the validation result as a Jira comment for traceability (Rovo MCP) |

## DoR validation output (per story — show in chat as a table)
| Jira ID | Title | DoR Items Met | DoR Items Missing | Verdict |
|---|---|---|---|---|
| BAP-42 | FE \| Cart \| ... | 5/6 | AC scenarios missing device context | Not Ready |

## DoD validation output (per story)
| Jira ID | Title | DoD Items Met | DoD Items Missing | Verdict |
|---|---|---|---|---|

## Common DoR checks
- Title follows naming convention (`[Layer] | [Module] | [Action]`)
- Description contains a persona-value statement
- Acceptance Criteria are present, numbered, and free of Gherkin
- Dependencies are listed
- Parent Epic key is assigned
- Story has been reviewed in chat by the BA

## Guardrails
- NEVER transition a story to "Done" without explicit user approval.
- NEVER mark a story "Ready" if acceptance criteria are absent or contain vague terms.
- If DoR/DoD definitions are not found via read the named project document using workspace file tools, ask the user to provide them before proceeding.
- Add the validation result as a Jira comment so the team has a traceability record.
