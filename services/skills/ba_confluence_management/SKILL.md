---
name: ba-confluence-management
description: Search, create, and update Confluence pages and spaces.
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

# Skill: Confluence Management

**Slug**: `confluence-management`
**Category**: confluence
**One-liner**: Search, create, and update Confluence pages and spaces.

> **Tool source:** All tools in this skill come from the `atlassian` MCP server. Read operations can be called directly without loading the orchestrator first.

## When to activate
- User wants to find, read, create, or update a Confluence page
- User needs to look up architecture, scope, FSD, or stakeholder documentation
- User wants to list available Confluence spaces

## Tools to use
| Tool | Purpose |
|---|---|
| `searchConfluenceUsingCql` | Search pages via CQL (space, title, content keywords) (Rovo MCP) |
| `getConfluencePage` | Fetch the full content of a page by ID (Rovo MCP) |
| `createConfluencePage` | Create a new page in a space (Rovo MCP — explicit approval required) |
| `updateConfluencePage` | Update an existing page (Rovo MCP — explicit approval required) |
| `getConfluenceSpaces` | List all accessible Confluence spaces (Rovo MCP) |

## Common CQL patterns
- Find scope docs: `space = "PROJ" AND type = page AND title ~ "FSD" ORDER BY created DESC`
- Find stakeholder page: `space = "PROJ" AND title = "Stakeholders"`
- Find architecture docs: `space = "PROJ" AND type = page AND text ~ "architecture"`

## Guardrails
- NEVER create or update a Confluence page without explicit user approval.
- Always confirm the target space key and parent page ID before creating.
- Use `searchConfluenceUsingCql` before creating to avoid duplicates.
- For the Field Validations table: NEVER insert rows between existing rows — always append at the end.
- Use the content format required by the advertised Confluence tool schema.
- Always confirm the page title and parent page with the user before creating.
