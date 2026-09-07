---
name: admin-activity-report
description: Investigate recent AI activity, failed tool calls, governance findings, and request
  audit chains.
---

# Investigate governance and activity

Start with `admin_report` using `{"report":"activity","days":1}`. Summarize request
volume, failures, and the affected models. This is an activity sample, not a complete
incident count. For cost trends use `admin_cost_report`; for people use `admin_user_report`.

For a specific incident discover and run `infra logs trace --help`,
`infra logs tools --help`, `infra logs governance report --help`, and
`infra logs audit <request-id>` as appropriate. Distinguish governance findings,
policy denials, gateway inference failures, and downstream tool failures; a warning
report is not the complete denial ledger. Reconstruct identity → request → tool →
result using returned trace/request IDs. Quote only necessary redacted evidence.

Produce a concise incident brief: observation, affected scope, timeline, evidence,
likely cause (clearly labelled if inferred), and next action. If requested, create a
Jira remediation issue with sanitized evidence using Atlassian's exposed tool schema.

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
