---
name: admin-cost-report
description: Explain AI usage, model/provider costs, latency, and adoption trends with evidence-backed
  recommendations.
---

# Review AI cost and adoption

Use `admin_report` with `{"report":"costs","days":7}`. Compare the explicit period
with an equivalent previous period through CLI help-discovered `--since`/`--until`
commands when asked for change. Read `analytics sessions stats` and
`analytics requests stats` for usage; counts of requests are not unique active people.
Use `analytics costs breakdown --by provider` and `--by model` for routing decisions.

Money is stored in microdollars: divide by 1,000,000 exactly once. A model's request
share is not its spend share. Gateway inference is often unattributed to a named agent;
that is expected. Do not allocate platform spend to Jira projects or claim productivity
ROI from activity alone. Report scope, spend, usage, expensive patterns, and prioritized
recommendations with evidence and limits. User-requested provider/route adjustments go
through `admin_ai_operations`.

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
