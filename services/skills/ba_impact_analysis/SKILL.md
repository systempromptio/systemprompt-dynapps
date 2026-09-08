---
name: ba-impact-analysis
description: Analyze scope and story impact of a proposed change.
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

# Skill: Impact Analysis

**Slug**: `impact-analysis`
**Category**: governance
**One-liner**: Analyze scope and story impact of a proposed change.

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
