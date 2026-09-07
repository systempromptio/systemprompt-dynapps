---
name: admin-project-report
description: Read deterministic Jira delivery metrics and Confluence evidence; manage requested
  project follow-up work.
---

# Review project delivery

Call `admin_project_catalog` to select an accessible Atlassian site, then with its
`cloud_id` to list projects and spaces. Reuse an unambiguous selection already supplied.
Call `admin_report` with `report: projects`, `cloud_id`, `project_key`, `days` (default 7),
and optional `space_id`. The returned MCP artifact has deterministic selectors,
metrics, issue tables, and document links. Dashboard rendering does not require this skill.

Explain overdue unresolved work, ownership gaps, stale unresolved issues (seven days),
status distribution, completions in the period, and linked documentation. A capped
report is incomplete; narrow the query before claiming project-wide totals. Stale is
not blocked. Do not infer sprint velocity, completion percentages, risk ratings, or
AI spend by project from unavailable data. Link every material assertion to returned
issues/pages and distinguish observations from recommendations.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names. For requested
issue creation, assignments, transitions, comments, or Confluence publishing, inspect
available fields/transitions and current versions first. Preserve unrelated fields,
check for an existing equivalent update, and verify the returned issue/page.

## Execution contract

Use the admin `systemprompt` MCP server as the signed-in caller. Its CLI tool accepts
`{"command":"<command without systemprompt prefix>"}` and already forces structured
JSON output. Do not append output-format flags, switch profiles, or use database URLs.
Discover uncertain command syntax with `--help`. A denied or unavailable source is
missing evidence, never a zero. Report the effective period, source, limitations,
and links or trace IDs behind conclusions. Never expose secrets or full prompt bodies
unless specifically needed for the user's investigation.

The `admin_report` tool produces deterministic data and an interactive MCP artifact.
Use that result; do not invent dashboard HTML or recalculate unsupported attribution.
In Cowork, use the installed reporting artifacts. Installation belongs exclusively
to `systemprompt_setup_admin`; other hosts can use the MCP artifact directly.

## Requested changes

Reporting does not authorize writes. When the user requests a specific change, inspect
the target and current state, discover its exact command/tool schema, perform the
requested operation, then read back the result. Reuse authorization already given;
ask only for missing material scope or authorization. Stop on an explicit denial,
failed validation, or ambiguous target. Report partial completion before attempting
anything else. Never blindly retry a write after a timeout: first check its outcome.
Configuration changes may require restart and source YAML reconciliation; distinguish
saved state from effective runtime state. Publishing to Jira/Confluence requires a
requested destination and content; report the resulting link and version.
