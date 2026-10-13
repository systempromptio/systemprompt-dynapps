---
title: "Agent Farm from Claude Code: Runs, Evals and Usage"
description: "The agent-farm MCP server: a run-centric tool set over core's agent_runs for developers and project managers, scoped to the caller's own runs and projects, with deterministic evals and per-person usage."
author: "Dynapps"
slug: "agent-farm"
keywords: "agent farm, agent runs, headless, claude code, cowork, mcp server, run_start, run_status, run_events, run_facts, evals, assertions, analytics, catalog, developer, project manager"
kind: "guide"
public: true
tags: ["enterprise", "engineering", "agents"]
published_at: "2026-10-13"
updated_at: "2026-10-13"
after_reading_this:
  - "Connect the agent-farm server from Claude Code or Cowork and know which tools it offers"
  - "Start a headless run, follow it to completion, and read its recorded facts"
  - "Say who may see which runs, and why a call was refused"
  - "Run an eval suite and read its deterministic verdicts"
related_docs:
  - title: "Access control: who reaches what, and why"
    url: "/documentation/access-control"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
---

The `agent-farm` MCP server gives developers and project managers the farm's
run lifecycle from inside Claude Code or Cowork: queue a headless run, poll it,
read its event stream and recorded facts, cancel it, run an eval suite, and read
usage for themselves or one of their projects. It sits beside the admin
`systemprompt` server on port 5011 and is scoped to the `user` OAuth scope.

## Who reaches it

Entitlement is declared in `services/access-control/rules.yaml` like every
other entity. The `dynapps-engineering` workspace, its plugin, the
`agent_farm_runs` skill and the `mcp_server/agent-farm` entity are closed by
default and allowed to the `developer`, `project_manager`, `admin`,
`platform_admin` and `super_admin` roles.

The role band only decides who sees the server. Inside it, every run tool
applies one rule:

- a person reaches a run they launched, or that ran as them;
- a person reaches every run of a project they belong to;
- an administrator reaches every run.

Anything else is refused with a `Forbidden:` error that names the run. The
`scope` word is the same on every listing tool: `mine`, `project`, or `all`,
and `all` is refused without an administrator role.

## Tools

| Tool | Read-only | What it does |
|------|-----------|--------------|
| `run_start` | no | Queue one headless run on a farm; returns the run id at once |
| `run_status` | yes | One run's lifecycle state, timings, tokens, cost, error class |
| `run_events` | yes | A page of the run's event stream, by `seq` |
| `run_cancel` | no | Ask a run to stop |
| `run_list` | yes | Runs the caller may see, newest first, with recorded facts |
| `run_facts` | yes | Everything recorded about one run, plus the origin's own record |
| `run_stats` | yes | Rollups over `24h`, `7d` or `30d`, by status, error class and origin kind |
| `farm_evals_run` | no | Start an eval suite: one run per case, deterministic assertions |
| `farm_eval_results` | yes | The latest verdict per case |
| `analytics_costs` | yes | Spend over a window for the caller or a project |
| `analytics_requests` | yes | Gateway requests, failures, denials, latency, by model |
| `analytics_skills` | yes | Skill adoption and the conversations it reached |
| `analytics_tools` | yes | MCP tool executions by server and tool |
| `catalog_list` | yes | What the caller's access grants |

Every input is flat: strings and numbers with defaults, so a client never has
to guess a flag. Evals are deterministic assertions only. No judge is attached
to an eval run and no judge field is ever returned.

## A run from start to facts

```text
run_start   {"farm": "odoo-delivery", "agent": "module-scaffold", "prompt": "..."}
run_status  {"run_id": "<id>"}          until terminal
run_events  {"run_id": "<id>", "after_seq": 0}
run_facts   {"run_id": "<id>"}
```

`run_start` goes through the farm's own launch service, so the agent's and
farm's limits, the prompt template, the run credentials and the farm's own run
row are resolved exactly as the dashboard resolves them. The run executes on
the farm runner; the tool never waits for it.

`run_facts` is the record the tables hold: outcome, tokens, reported and
metered cost, tool use counts by name, hook and governance decisions,
structured output and whether it matched the schema the request asked for,
and the workspace diff summary. For a farm run it adds the agent, prompt,
model and, when the run was an eval case, the assertion results.

## Operating it

The server is an internal MCP binary, `systemprompt-mcp-agent-farm`, declared
in `services/mcp/agent-farm.yaml` and built with the workspace. On this
instance it is compiled with the `farm` feature on; a build without it keeps
the read tools and refuses `run_start`, `run_cancel` and the eval tools with a
clear error. Access audit rows land in `user_activity` like every other
server's; tool usage lands in the same ledger the admin console reads.
