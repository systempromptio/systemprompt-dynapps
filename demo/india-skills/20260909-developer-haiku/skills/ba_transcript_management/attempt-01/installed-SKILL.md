---
name: ba-transcript-management
description: "Store and retrieve discovery/interview transcripts from the BA platform."
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
For story creation, apply the writing checks below before Jira issue creation (`discover`, then `executeWrite`).
Use its advertised content schema, not an assumed legacy wiki or ADF format.
Notification and scheduling operations produce drafts/proposals only; explicitly
report that sending or scheduling was not executed. Never invent a job history.

# Skill: Transcript Management

**Slug**: `transcript-management`
**Category**: knowledge
**One-liner**: Store and retrieve discovery/interview transcripts from the BA platform.

## When to activate
- User wants to save notes from a discovery call, interview, or requirements session
- User wants to retrieve a previously saved transcript for analysis or story drafting

## Tools to use
| Capability | Purpose |
|---|---|
| read the transcript from .astound/ba/transcripts using workspace file tools | Fetch a transcript by numeric ID or exact title |
| create a new Markdown transcript in .astound/ba/transcripts using workspace file tools | Store a new local Markdown transcript |

## Recommended transcript title format
`[Session Type] — [Feature/Topic] — [DD-MMM-YYYY]`
Example: `Discovery Call — Checkout Feature — 24-Aug-2026`

## Transcript content guidance
Include:
- Stakeholder statements verbatim (or close paraphrase)
- BA questions and stakeholder responses
- Decisions made and their context
- Open questions flagged for follow-up

## Guardrails
- NEVER overwrite an existing transcript — always create a new entry via create a new Markdown transcript in .astound/ba/transcripts using workspace file tools.
- After storing, always confirm the saved transcript ID to the user.
- NEVER store sensitive information (PII, credentials, financial details) in transcripts.
- Transcripts are stored in this workspace; report the saved path and preserve prior entries.
- Use the exact title format for easy lookup in future sessions.
