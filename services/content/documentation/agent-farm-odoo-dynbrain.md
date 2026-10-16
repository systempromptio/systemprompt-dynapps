---
title: "Run Odoo and DynBrain Skills Through the Agent Farm"
description: "For people with a Dynapps Odoo portal account: connect Dynapps Odoo and DynBrain once, install the portal plugins in Claude Code, and launch farm runs that call the portal tools as you, as a client's service account, or as a named service account."
author: "Dynapps"
slug: "agent-farm-odoo-dynbrain"
keywords: "agent farm, odoo, dynbrain, portal.dynapps.es, connectors, oauth, run_as, service account, client, dynapps-odoo, mcp"
kind: "guide"
public: true
tags: ["enterprise", "engineering", "agents"]
published_at: "2026-10-15"
updated_at: "2026-10-15"
after_reading_this:
  - "Connect your portal.dynapps.es account to Dynapps Odoo and DynBrain on the Connectors page"
  - "Install the marketplace and plugins that carry the portal servers and skills"
  - "Launch a farm run and say whose Odoo data it reads and writes"
  - "Explain a refused launch or tool call and fix it"
related_docs:
  - title: "Agent Farm from Claude Code"
    url: "/documentation/agent-farm"
  - title: "Access control: who reaches what, and why"
    url: "/documentation/access-control"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
---

Dynapps runs two MCP servers on its portal, `portal.dynapps.es`: **Dynapps
Odoo** (`dynapps-odoo`, live ERP records) and **DynBrain** (`dynbrain`, the
Dynapps RAG knowledge system). This instance proxies both, so every tool call
passes the governance chain before it leaves. This guide takes a person who
already holds a valid portal account from first connection to a governed farm
run that uses those tools.

## 1. Check you are entitled

Entitlement is declared in `services/access-control/rules.yaml`. All of these
are `default: closed`:

| Entity | Allowed to |
|--------|-----------|
| `marketplace/dynapps-portal`, `plugin/dynapps-portal` | roles `admin`, `platform_admin`, `super_admin`; group `staff` |
| `plugin/dynapps-showcase`, `plugin/dynapps-farm` | roles `admin`, `platform_admin`, `super_admin`; group `staff` |
| `mcp_server/dynapps-odoo`, `mcp_server/dynbrain` | roles `admin`, `platform_admin`, `super_admin`; groups `staff`, `service-accounts` |

To see and run a farm's agents you also need the marketplace the farm names
in `services/web/config/agent-farms.yaml` (`dynapps-agent-farm` for the
nightly audit farm, `dynapps-night-audit`). If any of these is missing, ask an admin to add you to
the `staff` group; the Access control page shows which band decided.

## 2. Connect Odoo and DynBrain

Open **/admin/connectors** and connect **Dynapps Odoo**, then **DynBrain**.
Each is a separate connector because the portal publishes a separate
protected resource for each, and a grant is bound to its resource.

Both use the generic OAuth connector: it discovers the server's
protected-resource metadata, registers a public client dynamically, and runs
authorization code with PKCE S256 for the `odoo` scope. You sign in on
`portal.dynapps.es` with your own Odoo account. From then on every call is
made as you, and Odoo's record rules decide what you see and change.

`dynapps-odoo` is declared `tool_policy: prompt`: it writes to Dynapps'
production ERP, so an interactive client asks before each call. `dynbrain` is
`tool_policy: allow`.

## 3. Install the plugins in Claude Code

After you [connect Claude Code](/documentation/connect-claude-code), install
from the instance's marketplaces:

| Marketplace | Plugin | What you get |
|-------------|--------|--------------|
| `dynapps-portal` | `dynapps-portal` | Both servers plus the `dynapps-portal-guide` skill, which routes how-to questions to DynBrain and live-record work to Dynapps Odoo |
| `dynapps-portal` | `dynapps-farm` | The `dynapps-agent-farm` skill and the `dynapps-accounting-auditor` it demos (launch farm tasks, follow logs, schedule them in plain language), with both servers |

Skill ids are snake_case in `services/` and dashed in Claude Code, so
`dynapps_portal_guide` appears as `/dynapps-portal:dynapps-portal-guide`.
Tools appear as `mcp__dynapps-odoo__<tool>` and `mcp__dynbrain__<tool>`.

## 4. How a farm agent uses the portal tools

A farm is a Git repository synced into the farm tables; each agent lists the
tools it may use. To give an agent the portal servers, list their tools by
full name, for example `mcp__dynbrain__<tool>` and
`mcp__dynapps-odoo__<tool>`.

At launch the runner derives the run's MCP servers from those names: each
`mcp__<server>__…` entry adds `<server>` at
`<instance>/api/v1/mcp/<server>/mcp`, with a bearer issued for the run. The
declared tools are passed to Claude Code as `--allowedTools` exactly (the
`Skill` tool is added automatically); any other `mcp__` tool is refused by the
run's governance hook under `farm_tool_allowlist`. There is no "all tools of
this server" form, so name each tool.

A farm can narrow this further with `tool_policy` in `agent-farms.yaml`.
`mcp_servers` is an allowlist: an agent naming any other server is refused at
launch. `deny` is appended to every run's `--disallowedTools`, so Claude Code
refuses those tools before a call leaves the runner. The nightly audit farm
allows only `dynapps-odoo` and `dynbrain` and denies every dynapps-odoo write
tool (`orm_create`, `orm_write`, `orm_unlink`, `orm_execute_kw`,
`odoo_request_upload`, `odoo_attachment_link`, `orm_post_message`, `orm_send_mail`): an unattended run can read Odoo but never change it.
In an attended session the same writes pass governance only for admin or a
holder of the `odoo_writer` role.

## 5. Choose whose identity the run uses (`run_as`)

`run_as` is set on the farm in `agent-farms.yaml`, and an agent can override
it under `agents:`. The agent's setting wins. The person who launched the run
is always recorded on it, whatever the mode.

| Mode | Credentials issued to | Whose Odoo / DynBrain data it touches | Refused when |
|------|-----------------------|----------------------------------------|--------------|
| `user` | The person who launched the run | Your own, through the grants you connected in step 2, as if you called the tools yourself | You have not connected the server (the tool call fails upstream) |
| `client` | The `service_account` of the run's client (the launch's client, else the farm's `client:`) | The client's service account's portal data | The run names no client, or that client declares no `service_account` |
| `service_account` | One named account (`service_account: <email or id>`) | That account's portal data | No active service account resolves |

```yaml
farms:
  - id: dynapps-night-audit
    client: dynapps
    run_as:
      mode: user
    agents:
      some_shared_agent:
        run_as:
          mode: service_account
          service_account: svc-farm@dynapps.es
```

A row with no mode predates modes: an account set means `service_account`,
otherwise `user`. The legacy plain form `run_as: svc@example.com` still loads
as `service_account`.

A service account reaches the portal servers only through an OAuth grant an
admin connects once, in the **Portal grants** section of its page under
**/admin/identity/service-accounts**. Runs in `client` or `service_account`
mode spend that grant; the credential broker issues it only to a
`farm:<run id>` caller.

## 6. Launch a run

- **Console:** open **/admin/farm**, pick the farm, then the agent
  (`/admin/farm/<farm>/agents/<agent>`), and launch from its dialog.
- **Claude Code:** ask the `dynapps-agent-farm` skill, or use the
  [agent-farm MCP server](/documentation/agent-farm) (`run_start`).
- **API:** `POST /api/public/agent-farm/runs` with the farm, agent and
  optional prompt, fixture, model, `max_turns`, `max_budget_usd` and
  `timeout_seconds`. Farm defaults apply otherwise (the nightly audit farm:
  `acceptEdits`, 2400 s, 40 turns, USD 8.00, `claude-opus-5-5`).
- **Scheduler:** the `farm_scheduled_run` job (`services/scheduler/config.yaml`)
  queues `accounting_night_audit` on `dynapps-night-audit` at 02:00 UTC,
  launched as the person its `launcher` parameter names. In `user` mode the
  run spends that person's own dynapps-odoo and dynbrain grants. It is
  recorded with trigger `scheduler`, which the run report and the farm page
  show. Run it by hand with `systemprompt infra jobs run farm_scheduled_run`.

## 7. Follow runs and cost

| Where | What |
|-------|------|
| `/admin/farm` | The "Nightly audit" strip: the last 30 days of scheduler-started runs, their success, cost, MCP reads and writes, governance warnings, DynBrain delivery and eval score |
| `/admin/farm/runs`, `/admin/farm/runs/<id>` | Run list and detail; cancel from the detail page. The Report tab shows the trigger, start and end, turns, tokens, cost, MCP calls per server (reads and writes), governance warnings, artifacts, DynBrain delivery and the eval score |
| `/admin/farm/clients`, `/admin/farm/clients/<id>` | Runs, evals and reports attributed to each client |
| `/admin/farm/people` | Usage per person |
| `/admin/farm/evals` | Eval suites and their runs |
| `GET /api/public/agent-farm/runs/<id>`, `/events`, `/report` | The same over the API |
| `systemprompt infra logs request list`, `analytics costs breakdown --by user` | Every inference call and its cost |
| `systemprompt infra logs trace list` | Every MCP tool call, including the portal servers |

## 8. Smoke check the servers

Before a long run, confirm the gateway reaches the portal as you. With a
personal access token (a CLI session token is not a gateway credential), send
an MCP `tools/list` to the proxied endpoint:

```bash
curl -s https://<instance>/api/v1/mcp/dynbrain/mcp \
  -H "Authorization: Bearer $PAT" \
  -H "x-session-id: smoke-$(date +%s)" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

Repeat with `dynapps-odoo`. A tool list means the grant works; an
authorization error means the connector needs connecting or reconnecting.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Portal servers or plugins not offered | Not in the `staff` group or an admin role | Ask an admin; check the Access control page |
| Tool call fails with an upstream 401 | No grant, or the grant expired or was revoked | Reconnect the server on `/admin/connectors` (or the service account's Portal grants) |
| Tool refused with `farm_tool_allowlist` | The agent does not declare that tool | Add the full `mcp__<server>__<tool>` name to the agent |
| Launch returns 422 `run_as mode client: the run names no client` | `client` mode with no client on the launch or the farm | Launch for a client, or set the farm's `client:` |
| Launch returns 422 `client "<slug>" names no service_account` | The client has no `service_account:` in `agent-farms.yaml` | Declare one on the client, or switch the agent to `user` mode |
| Launch returns 422 `run_as mode service_account: no active service account is resolved` | The named account is missing or inactive | Fix the email or id, or reactivate the account |
| Run sees the wrong person's records | The mode is not what you expected | Check the agent's `run_as` first, then the farm's |
| Each Odoo call asks for confirmation | `dynapps-odoo` is `tool_policy: prompt` by design | Expected for interactive use |
