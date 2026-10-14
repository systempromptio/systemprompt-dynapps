# Examples that run

Every block below preceded by an `<!-- example {...} -->` marker is executed by
`scripts/run_examples.py` against the real API, in order, and its `expect` is checked. Change the
API and one of these fails, naming itself. They double as the shapes to copy when you need a raw
call.

Marker format (one line of JSON):

| Key | Meaning |
|---|---|
| `name` | unique name, shown on failure |
| `expect.status` | HTTP status (number or list) for an `http` block; default 200 |
| `expect.exit` | exit code (number or list) for a `bash` block; default 0 |
| `expect.has` / `absent` | dotted paths (`items[0].slug`) that must / must not exist in the JSON answer |
| `expect.eq` | `{path: value}` that must be equal |
| `expect.capture` | `{variable: path}` saved for later examples as `{variable}` |
| `expect.stdout_has` / `stderr_has` | texts a `bash` block's output must contain |
| `cleanup` | `true`: runs at the end, in reverse order, even if earlier examples failed |
| `invalid_body` | `true`: the body is wrong on purpose (the example shows the rejection), so its fields are not checked against the API description |
| `optional` | `"task"`: only runs with `--with-task` (launches one real, short task) |

An `http` block is `METHOD /farm/api/v1/path` and, after it, an optional JSON body. A `bash` block
is a `python3 scripts/farm.py …` command (`--json` makes its output checkable as JSON); the runner swaps
`python3` for the interpreter of the machine (`AGENT_FARM_PYTHON`, else `python3`, `python` or `py -3`).

## The key and the connection

<!-- example {"name": "check-key", "expect": {"stdout_has": ["Your key works"]}} -->
```bash
python3 scripts/farm.py check
```

## Who is calling

<!-- example {"name": "whoami", "expect": {"has": ["subject", "tenant", "scopes", "is_admin", "expires_at"]}} -->
```http
GET /farm/api/v1/whoami
```

<!-- example {"name": "whoami-cli", "expect": {"stdout_has": ["Key of"]}} -->
```bash
python3 scripts/farm.py whoami
```

## The API describes itself

<!-- example {"name": "api-list", "expect": {"stdout_has": ["create_task", "whoami"]}} -->
```bash
python3 scripts/farm.py api
```

<!-- example {"name": "api-detail", "expect": {"stdout_has": ["Request body (TaskCreate", "agent_id", "Error 404"]}} -->
```bash
python3 scripts/farm.py api create_task
```

## Reading: agents, plugins, tasks

The platform may have no agent of its own: what `GET /agents` returns depends on the person. These
examples create a personal agent first (and delete it at the end) so the list is never empty.

<!-- example {"name": "agent-first-create", "expect": {"status": 201, "eq": {"slug": "skill-example-first", "scope": "personal"}}} -->
```http
POST /farm/api/v1/agents
{"slug": "skill-example-first", "scope": "personal", "description": "Example agent of the skill",
 "instructions": "Answer in one sentence."}
```

<!-- example {"name": "agent-first-delete", "cleanup": true, "expect": {"status": [204, 404]}} -->
```http
DELETE /farm/api/v1/agents/skill-example-first
```

<!-- example {"name": "agents-list", "expect": {"has": ["items[0].slug", "items[0].description", "items[0].requires_workspace", "items[0].capabilities.credentials", "items[0].capabilities.egress", "items[0].default_limits"], "capture": {"agent": "items[0].slug"}}} -->
```http
GET /farm/api/v1/agents
```

<!-- example {"name": "agent-one", "expect": {"eq": {"slug": "{agent}"}}} -->
```http
GET /farm/api/v1/agents/{agent}
```

<!-- example {"name": "agents-cli", "expect": {"has": ["items[0].slug"]}} -->
```bash
python3 scripts/farm.py --json agents
```

<!-- example {"name": "plugins-list", "expect": {"has": ["items[0].name", "items[0].version"]}} -->
```http
GET /farm/api/v1/plugins
```

<!-- example {"name": "tasks-list", "expect": {"has": ["items", "next_cursor"]}} -->
```http
GET /farm/api/v1/tasks?limit=5
```

<!-- example {"name": "tasks-mine", "expect": {"has": ["items", "next_cursor"]}} -->
```http
GET /farm/api/v1/tasks?limit=5&owner=me
```

<!-- example {"name": "schedules-mine", "expect": {"has": ["items"]}} -->
```http
GET /farm/api/v1/schedules?owner=me
```

<!-- example {"name": "tasks-cli"} -->
```bash
python3 scripts/farm.py tasks --limit 3
```

<!-- example {"name": "credentials-list", "expect": {"has": ["items"]}} -->
```http
GET /farm/api/v1/credentials
```

## What a wrong request looks like

A made-up task id answers like a task that is not yours.

<!-- example {"name": "task-unknown", "expect": {"status": 404, "eq": {"error.code": "not_found"}}} -->
```http
GET /farm/api/v1/tasks/00000000-0000-7000-8000-000000000000
```

Never send the team: it comes from the key.

<!-- example {"name": "tenant-not-accepted", "invalid_body": true, "expect": {"status": 400, "eq": {"error.code": "tenant_not_accepted"}}} -->
```http
POST /farm/api/v1/tasks
{"prompt": "x", "tenant": "somebody-else"}
```

Exactly one of agent and prompt.

<!-- example {"name": "agent-and-prompt", "expect": {"status": 422, "eq": {"error.code": "agent_or_prompt_required"}}} -->
```http
POST /farm/api/v1/tasks
{"agent_id": "whatever", "prompt": "x"}
```

A misspelt field is an error, not silently ignored.

<!-- example {"name": "unknown-field", "invalid_body": true, "expect": {"status": 422, "eq": {"error.code": "payload_invalid"}}} -->
```http
POST /farm/api/v1/tasks
{"prompt": "x", "promt_text": "y"}
```

A repeat pattern needs five fields.

<!-- example {"name": "schedule-bad-cron", "expect": {"status": 422, "eq": {"error.code": "cron_invalid"}}} -->
```http
POST /farm/api/v1/schedules
{"task": {"prompt": "x"}, "recurrence": {"cron": "every day", "timezone": "Europe/Madrid"}}
```

The lifecycle below uses a free prompt, the way most people will ask. A team that may only use
catalogue agents gets `422 prompt_not_enabled_for_tenant` on those: the next example is the same
create-and-delete with an agent of the image's plugins (`<plugin>:<agent>`, listed by `farm.py plugins`), which works for every team.

## A schedule with an agent of the image's plugins

<!-- example {"name": "schedule-agent-create", "expect": {"status": 201, "eq": {"state": "active"}, "capture": {"sid3": "schedule_id"}}} -->
```http
POST /farm/api/v1/schedules
{"task": {"agent_id": "developer-skills:code-reviewer", "input": "Review my open merge requests",
          "workspace": {"repo": "https://gitlab.dynapps.be/infra-es/agent-farm.git"}},
 "recurrence": {"cron": "0 3 1 1 *", "timezone": "Europe/Madrid"}}
```

<!-- example {"name": "schedule-agent-delete", "cleanup": true, "expect": {"status": 204}} -->
```http
DELETE /farm/api/v1/schedules/{sid3}
```

## A schedule's whole life (created, read, paused, resumed, deleted)

Fires once a year (1 January, 03:00), so nothing runs while the example is alive; the cleanup at
the end deletes it.

<!-- example {"name": "schedule-create", "expect": {"status": 201, "has": ["schedule_id", "owner", "next_fire_at", "recurrence.cron", "overlap"], "eq": {"state": "active", "overlap": "skip", "runs_count": 0}, "capture": {"sid": "schedule_id"}}} -->
```http
POST /farm/api/v1/schedules
{"task": {"prompt": "Reply with the single word: ok"},
 "recurrence": {"cron": "0 3 1 1 *", "timezone": "Europe/Madrid"},
 "overlap": "skip"}
```

<!-- example {"name": "schedule-show", "expect": {"eq": {"schedule_id": "{sid}", "state": "active"}}} -->
```http
GET /farm/api/v1/schedules/{sid}
```

<!-- example {"name": "schedule-list", "expect": {"has": ["items[0].schedule_id"]}} -->
```http
GET /farm/api/v1/schedules
```

<!-- example {"name": "schedule-pause", "expect": {"eq": {"state": "paused"}}} -->
```http
POST /farm/api/v1/schedules/{sid}/pause
```

<!-- example {"name": "schedule-resume", "expect": {"eq": {"state": "active"}}} -->
```http
POST /farm/api/v1/schedules/{sid}/resume
```

<!-- example {"name": "schedule-runs", "expect": {"has": ["items"]}} -->
```http
GET /farm/api/v1/schedules/{sid}/tasks
```

<!-- example {"name": "schedule-delete", "cleanup": true, "expect": {"status": 204}} -->
```http
DELETE /farm/api/v1/schedules/{sid}
```

The same through the script people actually use.

<!-- example {"name": "schedule-cli-create", "expect": {"has": ["schedule_id"], "capture": {"sid2": "schedule_id"}}} -->
```bash
python3 scripts/farm.py --json schedule create --prompt "Reply with the single word: ok" \
  --cron "0 3 1 1 *" --timezone Europe/Madrid
```

<!-- example {"name": "schedule-cli-pause", "expect": {"stdout_has": ["paused"]}} -->
```bash
python3 scripts/farm.py schedule pause {sid2}
```

<!-- example {"name": "schedules-cli", "expect": {"stdout_has": ["0 3 1 1 *"]}} -->
```bash
python3 scripts/farm.py schedules
```

<!-- example {"name": "schedule-cli-delete", "cleanup": true, "expect": {"stdout_has": ["deleted"]}} -->
```bash
python3 scripts/farm.py schedule delete {sid2}
```

## Agents of your own, saved accesses and MCP servers

A personal agent is yours alone and any key can make one, so these run for everybody; what an
administrator does (shared accesses, team agents) is in `intents.md` and cannot be run here without
touching the team. The values below are not secrets: they only have the right shape. The cleanups
delete everything, the agent first.

<!-- example {"name": "credential-mcp-create", "expect": {"status": 201, "eq": {"scope": "personal", "type": "mcp_token", "description": "Example access of the skill, not a real one"}}} -->
```http
POST /farm/api/v1/credentials
{"name": "skill-example-mcp", "type": "mcp_token", "value": "example-value-not-a-secret", "scope": "personal", "description": "Example access of the skill, not a real one"}
```

<!-- example {"name": "credential-mcp-delete", "cleanup": true, "expect": {"status": [204, 404]}} -->
```http
DELETE /farm/api/v1/credentials/skill-example-mcp
```

A personal agent on a plugin agent of the image, with a variable and an MCP server of its own
(`url` + `mcp_token`). The values `url` and `header` are not secrets: the token is the saved access.

<!-- example {"name": "agent-personal-create", "expect": {"status": 201, "eq": {"slug": "skill-example-agent", "scope": "personal"}, "has": ["mcp_servers[0].credential", "env_credentials", "instructions"]}} -->
```http
POST /farm/api/v1/agents
{"slug": "skill-example-agent", "scope": "personal", "description": "Example agent of the skill",
 "instructions": "Answer in one sentence.",
 "plugin_agent": "developer-skills:code-reviewer",
 "env_credentials": {"EXAMPLE_TOKEN": "skill-example-mcp"},
 "mcp_servers": [{"name": "example", "url": "https://mcp.example.com/mcp", "credential": "skill-example-mcp"}]}
```

<!-- example {"name": "agent-personal-delete", "cleanup": true, "expect": {"status": [204, 404]}} -->
```http
DELETE /farm/api/v1/agents/skill-example-agent
```

Only the agent's own, and the shared accesses the team has (names and fingerprints, never values).

<!-- example {"name": "agents-mine", "expect": {"has": ["items[0].slug", "items[0].scope"]}} -->
```http
GET /farm/api/v1/agents?owner=me
```

<!-- example {"name": "credentials-shared", "expect": {"has": ["items"]}} -->
```http
GET /farm/api/v1/credentials?scope=shared
```

<!-- example {"name": "agent-one-cli", "expect": {"stdout_has": ["Built on the plugin agent: developer-skills:code-reviewer", "Variables it receives: EXAMPLE_TOKEN <- skill-example-mcp", "MCP servers it always has: example"]}} -->
```bash
python3 scripts/farm.py agents skill-example-agent
```

What a wrong one looks like: `shared:` belongs to the team's agents.

<!-- example {"name": "agent-shared-in-personal", "expect": {"status": 422, "eq": {"error.code": "shared_credential_not_allowed"}}} -->
```http
POST /farm/api/v1/agents
{"slug": "skill-example-bad", "scope": "personal", "instructions": "x", "env_credentials": {"EXAMPLE_TOKEN": "shared:whatever"}}
```

A secret never goes in the address of an MCP server.

<!-- example {"name": "mcp-secret-in-url", "expect": {"status": 422, "eq": {"error.code": "mcp_server_url_invalid"}}} -->
```http
POST /farm/api/v1/agents
{"slug": "skill-example-bad", "scope": "personal", "instructions": "x",
 "mcp_servers": [{"name": "example", "url": "https://user:secret@mcp.example.com/mcp", "credential": "skill-example-mcp"}]}
```

A plugin agent that the plugin does not define.

<!-- example {"name": "plugin-agent-unknown", "expect": {"status": 422, "eq": {"error.code": "agent_not_in_image_plugin"}}} -->
```http
POST /farm/api/v1/agents
{"slug": "skill-example-bad", "scope": "personal", "plugin_agent": "developer-skills:no-such-agent"}
```

## One real, short task (only with `--with-task`)

<!-- example {"name": "task-run", "optional": "task", "expect": {"has": ["task_id", "state"], "capture": {"tid": "task_id"}}} -->
```bash
python3 scripts/farm.py --json run --prompt "Reply with the single word: ok" --timeout-s 120 --wait 240
```

<!-- example {"name": "task-status", "optional": "task", "expect": {"stdout_has": ["Task {tid}"]}} -->
```bash
python3 scripts/farm.py status {tid}
```

<!-- example {"name": "task-logs", "optional": "task"} -->
```bash
python3 scripts/farm.py logs {tid}
```

<!-- example {"name": "task-result", "optional": "task", "expect": {"stdout_has": ["Task {tid}"]}} -->
```bash
python3 scripts/farm.py result {tid}
```

<!-- example {"name": "task-cancel", "optional": "task", "cleanup": true, "expect": {"exit": [0, 1]}} -->
```bash
python3 scripts/farm.py cancel {tid}
```
