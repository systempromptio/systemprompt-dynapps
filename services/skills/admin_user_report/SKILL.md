---
name: admin-user-report
description: Review real accounts, roles, status, and sessions; perform explicitly requested account
  changes.
---

# Review users and access

Start with `admin_report` using `{"report":"users"}`. The directory is paginated;
continue with `offset` in increments of 100 when a complete review is needed.
Deduplicate by user ID. Users can hold both `admin` and `user`; never add their role
counts. Separate anonymous fingerprint visitors from signed-in accounts. Do not
claim a role grants project access without checking the applicable entity policy.

For drill-down use `admin users search`, `admin users show <id>`,
`admin users role list <id>`, and `admin users session list <id>` after checking help.
Present account totals and role/status membership, distinguishing the directory page
from population statistics. For role changes, suspension, or session revocation,
inspect the exact subcommand's help and verify the identified account after execution.
Never export passkeys, tokens, or unrelated personal data as part of an access review.

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
