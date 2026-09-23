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
| `admin users role assign <user-id>` | `--roles <role>[,<role>]` — the only roles are `admin`, `user`, `anonymous`. There is no `developer`, `member` or `viewer`; "platform admin" is the `admin` role plus platform-organization membership, which the deployment grants and no CLI command does |
| `admin users api-key issue` | `--user <id>`, `--name <label>`, `--expires <RFC3339>` — the secret prints once. `api-key list --user <id>`, `api-key revoke --user <id> --id <key-id>` |

`infra logs request list` has **no** `--offset` and no `--export`: it pages with
`--before <cursor>` (each row carries its own `cursor`) and never writes a file. To reach
an earlier window set `--since` and `--until` rather than paging back to it. Only
`analytics requests list` takes `--offset`.

Time values: a duration back from now (`30m`, `1h`, `24h`, `7d`) or an absolute `YYYY-MM-DD` / `YYYY-MM-DDTHH:MM:SS`. `--until` takes the same forms and is exclusive.

A cursor is `<created_at RFC3339>@<request_id>`; every `request list` row carries its own in the `cursor` field, and `--before` takes the last row's.

### The 8 domains, and every verb each one has

```
systemprompt <domain> <subcommand> [verb] [args] [options]
```

This is the whole tree as the binary defines it. A verb that is not listed here does not
exist — guessing one costs a failed tool call and an error the caller sees. Where a
subcommand has verbs, one of them is required unless noted.

**`core`** — `artifacts` (list, show) · `content` (list, show, search, edit, delete,
delete-source, popular, verify, status, link, analytics, files) · `files` (list, show,
upload, delete, validate, config, search, stats, ai) · `contexts` (list, show, create,
edit, delete, use, new) · `skills` (list, show) · `plugins` (list, show, validate,
generate) · `marketplace` (explain, import) · `hooks` (list, validate) · `services`
(validate, bundle, keygen, publish, refresh, inspect)

**`infra`** — `services` · `db` (query, execute, tables, describe, info, migrate,
migrate-down, migrations, migrate-plan, migrate-status, migrate-repair,
migrate-mark-applied, assign-admin, status, count, indexes, size, doctor) · `jobs`
(list, show, run, history, enable, disable, cleanup-sessions, log-cleanup,
session-cleanup) · `logs` (view, search, stream, export, cleanup, delete, summary, show,
trace, governance, request, tools, audit)

`infra logs` sub-trees: `request` (list, show, stats) · `trace` (list, show) ·
`governance` (report) · `tools` (list). `audit <request-id>` takes the id directly.

**`admin`** — `users` · `agents` · `config` · `setup` · `bootstrap` · `session` (show,
list, login, logout) · `bridge` (enroll-cert, issue-code, list, rotate-signing-key) ·
`access-control` (export-yaml, lint) · `keys` (generate, issue-plugin-token) · `identity`
(generate). There is no `admin report` and no top-level `admin_report` command — the
dashboard is the `admin_report` **MCP tool**, not a CLI subcommand.

`admin users`: list, show, search, create, update, delete, count, export, stats, merge,
bulk (delete, update), role (assign, promote, demote), session (list, end, cleanup), ban
(list, add, remove, check, cleanup), webauthn (generate-setup-token), api-key (issue,
list, revoke). Note `role` has no `list` — read a user's roles with
`admin users show <id>` — and `api-key` issues with `issue`, not `create`.

`admin config`: show, list, validate, rate-limits, server, runtime, security, services,
paths, provider, gateway, governance, catalog, secret. The sub-trees that have verbs:
`gateway` (enable, disable, route {add, remove, list}, default-provider {set, clear} —
there is **no** `config gateway show`; `admin config show` prints the whole config),
`provider` (list, set, enable, disable), `catalog` (provider {list, add}, model {add},
discovery), `governance` (show, set), `security` (show, set, trusted-issuer add),
`server` (show, set, cors {list, add, remove}), `runtime` (show, set), `services` (show,
set), `paths` (show, validate), `secret` (set, check), `rate-limits` (show, set, enable,
disable, validate, reset, preset {list, show, apply}, export, import, diff).

**`analytics`** — `projection` (status, rebuild) · `overview` · `conversations` (stats,
trends, list) · `agents` (stats, list, trends, show) · `tools` (stats, list, trends,
show) · `requests` (stats, list, trends, models) · `sessions` (stats, trends, live) ·
`content` (stats, top, popular, trends) · `traffic` (sources, pages, navigation, geo,
devices, bots) · `costs` (summary, trends, breakdown).

`costs` and `requests` default to their summary/stats aggregate when no verb is given;
`sessions`, `tools`, `agents`, `conversations`, `content`, `traffic` and `projection`
each require one. There is no `costs users` — spend per user is
`costs breakdown --by user`. There is no `tools usage` — use `tools stats` or
`tools list`. There is no bare `sessions --since`: pick `sessions stats`,
`sessions trends` or `sessions live`.

**`cloud`** — `auth` (logout, whoami) · `init` · `tenant` (create, list, delete,
rotate-credentials) · `profile` (create, list, delete, edit) · `deploy` · `backup` ·
`doctor` · `status` · `dockerfile`

**`web`** — `content-types` (list, show, create, edit, delete) · `templates` (list, show,
create, edit, delete) · `assets` (list, show) · `sitemap` (show, generate) · `validate`

**`plugins`** — `list` · `show` · `run` · `validate` · `config` · `capabilities` (jobs,
templates, schemas, tools, roles) · `mcp` (list, status, validate, logs, list-packages,
tools, call)

**`build`** — `core` · `mcp`

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
| Change a user's roles | `systemprompt admin users role promote\|demote <user-id>`, or `role assign <user-id> --roles admin\|user\|anonymous` |
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
