---
name: ba-session-memory
description: "Manage conversation sessions and record agent corrections for learning."
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

# Skill: Session Memory

**Slug**: `session-memory`
**Category**: knowledge
**One-liner**: Manage conversation sessions and record agent corrections for learning.

## When to activate
- User wants to start, resume, or reference a named BA session
- User corrects the agent and the correction should be recorded for future sessions
- User wants to see a list of past sessions or retrieve context from a previous session

## Tools to use
| Capability | Purpose |
|---|---|
| manage named Markdown session files in .astound/ba/sessions using workspace file tools | Create, get, update, list, delete sessions, or retrieve context |
| append a correction to the session corrections.jsonl using workspace file tools | Record a user correction of the agent |

## Session actions
| Action | When to use |
|---|---|
| `create` | Start a new named session |
| `get` | Retrieve an existing session by ID |
| `update` | Update the session title or rolling summary |
| `list` | List all sessions for the current user |
| `delete` | Remove a session (explicit approval required) |
| `get_messages` | Retrieve message history from a session |
| `get_context` | Assemble context from a session for injection |

## Correction fields (all required for append a correction to the session corrections.jsonl using workspace file tools)
- `session_id`: The session where the correction occurred
- `agent_output`: What the agent said or did that was wrong
- `correction_text`: Exactly what the user said to correct it
- `correction_type`: One of `factual_error`, `preference`, `style`, `procedure`, `omission`
- `extracted_fact`: One-sentence learning from the correction
- `fact_key` / `fact_value`: Structured preference key-value if applicable

## Guardrails
- NEVER delete a session without explicit user approval.
- Record corrections faithfully — do not paraphrase or soften what was wrong.
- Corrections persist in this workspace; explicitly read them when resuming a session.
- After storing a correction, confirm the stored correction ID to the user.
