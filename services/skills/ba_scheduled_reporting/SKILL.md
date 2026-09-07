---
name: ba-scheduled-reporting
description: Trigger or inspect standup, backlog grooming, and release-notes scheduled
  jobs.
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

# Skill: Scheduled Reporting

**Slug**: `scheduled-reporting`
**Category**: reporting
**One-liner**: Trigger or inspect standup, backlog grooming, and release-notes scheduled jobs.

## When to activate
- User wants to run a standup summary, grooming report, or release notes on demand
- User wants to check the status or execution history of a scheduled job
- User wants to see a summary of recent Jira activity

## Tools to use
| Tool | Purpose |
|---|---|
| prepare an on-demand report and a schedule proposal in .astound/ba/reports; scheduling is unsupported | Trigger a job or inspect its status/history |
| the advertised Atlassian capability for get recent jira events (fail explicitly if unavailable) | Fetch recent Jira webhook events for standup context |
| the advertised Atlassian capability for get jira event summary (fail explicitly if unavailable) | Summarise Jira activity over a configurable time window |

## Schedule proposals (not installed jobs)
| Job trigger name | Default cron schedule | Purpose |
|---|---|---|
| `standup` | Weekdays 9:00 AM UTC | Daily standup summary from Jira activity |
| `grooming` | Mondays 2:00 PM UTC | Stale backlog and issue analysis |
| `release_notes` | Fridays 5:00 PM UTC | Weekly release notes generation |

## prepare an on-demand report and a schedule proposal in .astound/ba/reports; scheduling is unsupported actions
| Action | Purpose |
|---|---|
| `trigger` | Run the job immediately (confirm project key first) |
| `status` | Check if a job is currently running |
| `history` | View recent execution history |
| `list` | List all available jobs and their schedules |

## Guardrails
- Always confirm the target project key with the user before triggering a job.
- Check `status` before triggering to avoid running a duplicate job.
- Save report drafts locally. Sending and unattended scheduling are unsupported in this marketplace.
- Do NOT trigger jobs in rapid succession — allow the previous run to complete first.
