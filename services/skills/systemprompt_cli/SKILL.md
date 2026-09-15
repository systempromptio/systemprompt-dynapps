---
name: systemprompt-cli
description: Navigable reference for the systemprompt CLI and the admin systemprompt MCP server -
  its typed analytics tools, the paging loop, the flags that actually exist, and the 8 CLI domains.
---

# systemprompt CLI Reference

A navigable map of the `systemprompt` CLI and the admin MCP server that fronts it. Use it to pick the right tool, then the right command, without guessing flags.

## When to Use

Use this skill whenever you need to operate the Astound Digital platform: who is using AI and how, what a session said, spend by user or model, governance logs, skills, services, config, cloud deploys, or MCP plugins. It tells you which tool or domain owns a task and how to discover the exact command.

## How to Use

### Typed tools first

The `systemprompt` MCP server (admin-only) exposes four typed tools for the questions an administrator asks most. Each carries its flags in its JSON schema, pages, and never returns more than a model can read. Prefer them over composing a CLI command by hand.

| Question | Tool | Call |
|----------|------|------|
| Who is spending, how much, on how many conversations? | `usage_by_user` | `{"since":"14d","limit":50}` |
| What did one user send, and when? | `request_log` | `{"user":"<user_id>","since":"2026-09-08","until":"2026-09-12","limit":50}` |
| What was actually said in one request? | `conversation_audit` | `{"request_id":"<request_id>","limit":10,"max_chars":400}` |
| Who is registered, with which role? | `users` | `{"limit":50,"offset":0}` |
| The dashboard | `admin_report` | `{"report":"costs","days":7}` |

`usage_by_user` names each row `<user_id> (<display name>)`; the id is what `request_log` takes as `user`. Emails are not in the analytics projection — map an id to a person with `users`.

### The paging loop

Every list is a page. Read the page, then look at `next_cursor`:

- `request_log`: a non-empty `next_cursor` means older rows exist — call again with `cursor` set to it. The same filters apply. A page shorter than `limit` is the last one.
- `conversation_audit`: `has_more` true means more messages — call again with `offset` set to `next_offset`. `message_count` and `tool_call_count` are the totals whatever the page holds.
- `users`: `next_cursor` is the next `offset`.
- `usage_by_user`: `truncated` true means the page was full — raise `limit` (max 200) or narrow the window.

`limit` is clamped to 200 on every tool. Ask for a wider window, not a larger page, when 200 is not enough: `request_log` with `since`/`until` bounds walks any period one page at a time. To reach a specific earlier window, set `since` and `until` to dates; do not page from today back to it.

Sample, do not exhaust. To judge how someone works, read `usage_by_user`, pick users by conversations and spend, take one early and one late page of their `request_log`, and audit two or three requests from each with `max_chars` 300-500. A whole transcript is never needed to characterise a prompt.

### Running any other command

The `systemprompt` tool executes a CLI command string. Pass it **without** the `systemprompt` prefix:

```json
{"command": "core skills list"}
```

From a terminal the same call is `systemprompt plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'`, and `systemprompt plugins mcp tools systemprompt` lists the server's tools.

The wrapper already sets JSON output and forwards your credential. Never pass `--json`, `--yaml`, `--format` or `--export` — they are stripped (`--export` writes a CSV on the server's disk, where no client can read it). Never override the caller profile or the database connection. A result larger than 64 KB comes back truncated with a note naming the narrower query; narrow it rather than asking again for the same thing.

### Flags that exist

These are the flags on the commands behind the typed tools. Use the table; do not invent a flag. If a command rejects a flag, run `<command> --help` **once** and use what it prints.

| Command | Flags |
|---------|-------|
| `infra logs request list` | `--since`, `--until`, `--user <id>`, `--model <substr>`, `--provider <substr>`, `--limit/-n`, `--before <cursor>` |
| `infra logs audit <id>` | `--messages/-m`, `--tools/-t`, `--offset`, `--limit/-n` (0 = all), `--max-content <chars>` |
| `analytics costs breakdown` | `--by model\|provider\|agent\|user`, `--since`, `--until`, `--limit/-n` |
| `analytics costs summary` | `--since`, `--until` |
| `analytics costs trends` | `--since`, `--until`, `--group-by` |
| `analytics requests list` | `--since`, `--until`, `--user <id>`, `--model`, `--limit/-n`, `--offset` |
| `analytics requests stats` | `--since`, `--until`, `--model` |
| `analytics requests models` | `--since`, `--until`, `--limit/-n` |
| `analytics conversations list` | `--since`, `--until`, `--source agent\|gateway\|all`, `--user <id>`, `--limit/-n` |
| `analytics sessions stats` | `--since`, `--until` |
| `analytics sessions trends` | `--since`, `--until`, `--group-by` |
| `analytics sessions live` | `--limit/-n`, `--refresh`, `--no-refresh` |
| `admin users list` | `--limit`, `--offset`, `--role admin\|user\|anonymous`, `--status <s>`, `--include-anonymous` — with `--role` the CLI ignores paging; `--status` is applied after the page |
| `admin users show <id or email>` | `--sessions`, `--activity` |
| `admin users search <text>` | `--limit`, `--include-anonymous` |
| `infra logs governance report` | `--since`, `--group-by policy\|tool\|user`, `--limit`, `--format table\|csv` |
| `infra logs trace list` | `--since`, `--agent`, `--status`, `--tool`, `--decision`, `--has-mcp`, `--all`, `--limit/-n` |

Time values: a duration back from now (`30m`, `1h`, `24h`, `7d`) or an absolute `YYYY-MM-DD` / `YYYY-MM-DDTHH:MM:SS`. `--until` takes the same forms and is exclusive.

A cursor is `<created_at RFC3339>@<request_id>`; every `request list` row carries its own in the `cursor` field, and `--before` takes the last row's.

### The 8 domains

```
systemprompt <domain> <subcommand> [args] [options]
```

| Domain | Purpose |
|--------|---------|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs (view, stream, request, trace, audit, governance) |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain, profiles |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build the core workspace and MCP extensions |

Most `analytics` subcommands need a further verb: `analytics costs summary`, `analytics requests stats`, `analytics agents list`, `analytics tools list`. `analytics costs` and `analytics requests` default to their summary; `agents` and `tools` do not.

Walk the help tree when a task is outside the tables above:

```bash
systemprompt --help                       # the 8 domains
systemprompt analytics --help             # a domain's subcommands
systemprompt analytics costs --help       # a subcommand's own verbs
```

### Fast paths to the task skills

| You want to... | Skill or command |
|----------------|------------------|
| AI spend, models, adoption, who uses AI and how well | `admin_ai_usage` |
| This morning's portfolio read, with a traffic light per project | `admin_daily_brief` |
| Which projects are in trouble, and the evidence | `admin_critical_projects` |
| Who is on the platform, by role and status | `users` tool / `systemprompt admin users stats` |
| Deep-dive one request and its trace | `conversation_audit` tool / `systemprompt infra logs trace show <trace-id>` |
| Providers, routing and jobs | `systemprompt admin config` / `infra jobs list` |
| Change a user's roles | `systemprompt admin users role promote\|demote <user-id>` |
| Start/inspect services, database, and jobs | `systemprompt infra services status` / `infra db` / `infra jobs` |

### Examples

```bash
systemprompt core skills list
systemprompt infra services status
systemprompt analytics overview --since 7d
systemprompt analytics costs breakdown --by user --since 30d -n 50
systemprompt infra logs request list --since 2026-09-08 --until 2026-09-12 --user 8a1ece9f-ff46-436e-99d3-21b589ac57f3 -n 100
systemprompt infra logs audit 7b72fc2f-a2b7-4c3e-8166-1390c594f891 --messages --limit 10 --max-content 400
systemprompt admin session show
```
