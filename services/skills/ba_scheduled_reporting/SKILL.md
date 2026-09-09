---
name: ba-scheduled-reporting
description: Prepare on-demand BA reports and schedule proposals; jobs are not installed.
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

# Skill: Scheduled Reporting

**Slug**: `scheduled-reporting`
**Category**: reporting
**One-liner**: Prepare on-demand BA reports and schedule proposals; jobs are not installed.

## When to activate
- User wants to run a standup summary, grooming report, or release notes on demand
- User wants to check the status or execution history of a scheduled job
- User wants to see a summary of recent Jira activity

## Tools to use
| Capability | Purpose |
|---|---|
| prepare an on-demand report and a schedule proposal in .astound/ba/reports; scheduling is unsupported | Save an on-demand report or schedule proposal; no job execution or history |
| get recent jira events via `discover` on `atlassian` (stop and report if no capability matches) | Fetch recent Jira webhook events for standup context |
| get jira event summary via `discover` on `atlassian` (stop and report if no capability matches) | Summarise Jira activity over a configurable time window |

## Schedule proposals (not installed jobs)
| Job trigger name | Default cron schedule | Purpose |
|---|---|---|
| `standup` | Weekdays 9:00 AM UTC | Daily standup summary from Jira activity |
| `grooming` | Mondays 2:00 PM UTC | Stale backlog and issue analysis |
| `release_notes` | Fridays 5:00 PM UTC | Weekly release notes generation |

## Report and proposal actions

- Generate a report from authenticated Jira reads, then save it locally.
- Save a proposed schedule with its time zone; do not activate a job.
- For status/history requests, explain that this marketplace installs no jobs
  and has no execution history. Do not invent a running job or previous result.

## Guardrails
- Verify the target project through authenticated reads before preparing a report.
- Inspect advertised read capabilities; report unavailable activity feeds explicitly.
- Save report drafts locally. Sending and unattended scheduling are unsupported.
