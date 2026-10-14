---
name: agent-farm-runs
description: Start, watch, cancel and score headless agent runs on a farm through the agent-farm
  MCP server, and read your own and your projects' run facts and usage.
---

# Agent farm runs

Use when someone wants to launch a headless agent run on a farm, follow a run
they started, see why a run failed, run or read an eval suite, or ask what their
own (or their project's) agent usage cost. Every tool lives on the `agent-farm`
MCP server; the instructions it sends on connect list them.

## What you can see

The server decides visibility, not you. A person reaches the runs they launched
(or that ran as them) and the runs of every project they belong to. An
administrator reaches every run. A refused call says so in its error; do not
retry it with a different id, and do not guess at another person's runs.

`scope` is the same word everywhere: `mine` (the caller's own), `project` (every
project they belong to; for analytics set `project` when there are several), and
`all` (administrators only).

## Starting and following a run

1. `catalog_list` tells you which workspaces the caller holds; a farm is usable
   only through its workspace.
2. `run_start {farm, agent, prompt?}` queues the run and returns `run_id` at
   once. It never waits. Set `max_turns`, `max_budget_usd` or `timeout_seconds`
   only to tighten the farm's defaults.
3. `run_status {run_id}` until `terminal` is true. Report `status` and
   `error_class` as returned.
4. `run_events {run_id, after_seq}` streams the transcript in pages: tool use
   and results, hook decisions, the final result. Pass `next_after_seq` back as
   `after_seq` to continue.
5. `run_facts {run_id}` is the full record: tokens, reported and metered cost,
   tools by name, hook and governance decisions, structured output and whether it
   matched its schema, the diff summary, and for a farm run its agent, prompt and
   assertion results.
6. `run_cancel {run_id}` asks a run to stop; `cancelled: false` means it had
   already finished.

## Lists and rollups

`run_list {scope, origin?, state?, limit?}` is the newest-first list with the
same facts per row. `run_stats {scope, window}` is the rollup: totals by status
and error class, cost, duration percentiles, tool use, hook denials, split by
origin kind. `scope: farm` with `farm` covers one farm.

## Evals

`farm_evals_run {farm, suite, case_filter?}` starts a suite: one run per case,
each scored by the suite's deterministic assertions. `farm_eval_results
{farm?, suite?}` is the latest verdict per case. Nothing here is judged by a
model; there is no score to interpret, only passed, failed or errored.

## Usage

`analytics_costs`, `analytics_requests`, `analytics_skills` and
`analytics_tools` take `{window: 24h|7d|30d, scope, project?}` and answer for
the caller or one of their projects. Skill rows overlap across conversations;
never sum them.
