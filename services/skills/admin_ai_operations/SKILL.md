---
name: admin-ai-operations
description: Inspect AI providers, routing, rate limits, agents, tools, jobs, and evaluations;
  verify requested operational changes.
---

# Manage AI operations

Start with `admin_report` using `{"report":"operations"}`. Review providers, gateway
routes, rate limits, agents, MCP status, scheduled jobs, and existing evaluation runs.
An empty agent list may be valid; this instance normally has the Slack-facing
`admin_console`. Gateway inference need not have an agent task.

Use the CLI help tree for `admin config provider`, `admin config gateway route`,
`admin config rate-limits`, `admin agents`, `plugins mcp`, `infra jobs`, and
`admin evals`. Compare declared and runtime state before diagnosing a failure.
Provide a prioritized operational assessment with observable evidence.

For requested changes, record the prior nonsecret values and affected resource,
perform only the scoped operation, validate configuration, and read back effective
state. A request to inspect evaluations does not authorize `run` or `replay`: these
can invoke paid models. A health check must never restart services or run jobs.
Never show provider secrets. Do not weaken governance as an incidental repair.

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
