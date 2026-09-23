---
name: admin-ai-usage
description: Explain AI usage, model/provider costs, latency, and adoption trends with evidence-backed
  recommendations.
---

# AI usage report

Call the `admin_report` **tool** with `{"report":"costs","days":7}` at the start of every
run; never reuse a payload from an earlier turn, even one minutes old. It is a typed MCP
tool on this server, not a CLI subcommand: the CLI has no equivalent, and passing the
tool name to it as a command or a domain fails. Compare the explicit period
with an equivalent previous period by re-running the same commands with `--since` and
`--until` set to the earlier window when asked for change. Read `analytics sessions stats`
and `analytics requests stats` for usage; counts of requests are not unique active people.
Use `analytics costs breakdown --by provider` and `--by model` for routing decisions.

## Who uses AI, and how

The dashboard is aggregate. A question about people — who does real work, who sends
one-line chats, whether someone's prompts improved — is answered with the typed tools
on the same server, in this order, and never by pulling whole transcripts:

1. `usage_by_user` `{"since":"30d","limit":100}`. Rank by `conversations` and
   `cost_usd`, not by requests: one Claude Code session is many requests. A user with
   many requests and one conversation is running a long task; many conversations of
   one request each is chat.
2. `users` to map each `user_id` to a name and role. Say who a user is only from that
   roster; the analytics projection carries no email.
3. For the users you are characterising, `request_log`
   `{"user":"<id>","since":"<start>","until":"<end>","limit":50}` for an early window
   and again for a recent one. Token shape is the first signal: `input+cachedc/output`
   with a large cached share and steady output is work inside a session; tiny input
   with tiny output is a greeting. Follow `next_cursor` only while the page is full and
   the question still needs older rows.
4. `conversation_audit` `{"request_id":"<id>","limit":10,"max_chars":400}` on two or
   three requests per window. The `user` turns are the prompts; judge specificity,
   context supplied, and whether the ask names an artifact. `message_count` tells you
   the session's depth without loading it. Never request `max_chars` 0 or `limit` 0 to
   "see everything" — a session transcript runs to hundreds of kilobytes.

Report per user: conversations, requests, spend, the model mix, and two or three quoted
prompts (trimmed) from each window with the judgement they support. "Improved over
time" needs both windows for the same user; say so when only one exists (a user who
joined last week has no early window). Do not rank people on volume alone, and do not
call a short prompt thoughtless when its session shows sustained cached-context work.

Money arrives in two forms: every `*_microdollars` value is the raw store, and its
`*_usd` sibling (a column, or the next heading/content row) is the converted figure.
Quote the `_usd` value in dollars and never convert the raw value yourself — there
are no millicents anywhere in this report. A model's request share is not its spend
share. Gateway inference is often unattributed to a named agent;
that is expected. Do not allocate platform spend to Jira projects or claim productivity
ROI from activity alone. Report scope, spend, usage, expensive patterns, and prioritized
recommendations with evidence and limits. Reporting does not authorize
provider or route changes; see the requested-changes section below.

## Execution contract

Use the admin `systemprompt` MCP server as the signed-in caller. Prefer its typed tools
(`usage_by_user`, `request_log`, `conversation_audit`, `users`); its CLI tool accepts
`{"command":"<command without systemprompt prefix>"}` and already forces structured
JSON output. Do not append output-format flags (`--json`, `--format`, `--export`), switch
profiles, or use database URLs. The flags each command takes are tabled in
`systemprompt_cli`, which also lists every verb each domain has; on an unknown-flag or
unknown-subcommand error read that table rather than guessing again, and run
`<command> --help` once if it is not covered there. A page is a page: follow `next_cursor` / `next_offset`, and set
`since`/`until` to reach an earlier window instead of paging back to it. A denied or
unavailable source is missing evidence, never a zero. Report the effective period, source, limitations,
and links or trace IDs behind conclusions. Never expose secrets or full prompt bodies
unless specifically needed for the user's investigation.

`admin_report` reads this platform's OWN audit tables through the CLI. There is no
third party in that path, so the figures are exact rather than sampled — use the
result as returned, and never recalculate attribution the data does not support.
The client renders the dashboard from that same result; do not describe it and do
not invent HTML for it.

It is the only report this server serves. Jira and Confluence are NOT read here: a
delivery question belongs to `admin_daily_brief` or `admin_critical_projects`, which
reach Atlassian directly as the caller's own linked account. Never allocate platform
AI spend to a Jira project — the two have different scopes and no shared key.

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
