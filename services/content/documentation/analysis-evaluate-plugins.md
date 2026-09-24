---
title: "Evaluate a Plugin: Fixed Suite, PAT Export, Deterministic Metrics"
description: "How to measure a plugin's skills run over run: send a fixed prompt suite through a real client, export every conversation with a personal access token, compute deterministic metrics, and compare versions over time in the console and in git."
author: "Astound Digital"
slug: "analysis-evaluate-plugins"
keywords: "evaluate, evaluation, eval, plugin, skills, metrics, PAT, personal access token, export, transcripts, versions, compare, baseline, regression"
kind: "guide"
public: true
tags: ["enterprise", "admin", "operations"]
published_at: "2026-09-24"
updated_at: "2026-09-24"
after_reading_this:
  - "Run a plugin's eval suite and export every conversation it produced with a PAT"
  - "Name every metric the eval records and the file it is computed from"
  - "Compare a run against a baseline and read the change in the console's Versions page"
  - "Repeat the loop after each skill change, so improvement is tracked over time"
related_docs:
  - title: "Versions: Marketplace Hashes, History and Compare"
    url: "/documentation/analysis-versions"
  - title: "Analysis: The Record of Every Conversation and Skill"
    url: "/documentation/analysis"
  - title: "Measure Which Skills Are Used"
    url: "/documentation/analysis-measure-skills"
---

# Evaluate a plugin: fixed suite, PAT export, deterministic metrics

**TL;DR:** An eval sends the same prompts to every skill of a plugin through a real, governed client, then exports each resulting conversation with a personal access token (PAT) and computes metrics from the export. Every metric is a count, a sum or a fixed rule over exported files. No model scores anything, so the same export always gives the same numbers. Change the skills, ship them, run the same suite again, and compare. The console's **Versions** page shows the same change by marketplace hash.

## Why this shape

- **Fixed inputs.** The prompts live in `scripts/eval/prompts/<plugin>.yaml`, and their hash is stamped on every run. Runs with different prompt hashes are flagged as not comparable, so a change in the numbers is a change in the skills.
- **A real client.** The suite runs in Claude Code through the Astound bridge, so every turn goes through the gateway, governance, cost accounting and the audit record exactly as a user's would. The eval measures what users get.
- **Deterministic scoring.** Tokens, cost, turns, tool calls and errors come from the gateway's own record of each conversation. Error classes are fixed patterns over tool results. Quality review is done separately by a person or by the engineer changing the skills, and it is never mixed into the metrics.
- **Two histories.** Each run's metrics are committed under `evals/<plugin>/<label>/`. Each shipped change also produces a new marketplace version in the console, so the change is visible in both places.

## The loop

```
edit skills → ship (new image → marketplace hash moves → bridge syncs)
  → scripts/eval/eval.sh run <label>
  → scripts/eval/eval.sh export <label>
  → scripts/eval/eval.sh metrics <label> --against <previous label>
  → read the deltas; open Versions → Compare in the console
```

The first run on a plugin is its **baseline**. Every later run is compared with the baseline and with the run before it.

## Run

```bash
scripts/eval/eval.sh run r2-2026-09-25 astound-salesforce
```

This runs every skill in the suite four at a time. Each run is a headless Claude Code session on the bridge host, with the model set in the suite file. The prompt is the skill's slash command, the fixed prompt, and the suite's constant suffix (scope to the test records, never write, never ask). The run manifest records:

| Field | Meaning |
|---|---|
| `plugin_version` | The `version` in the plugin config at run time. |
| `prompts_hash` | The hash of the prompt file. Runs are compared only when this matches. |
| `skills` | A content hash per skill directory, so a skill that did not change can be told apart from one that did. |
| `runs.<skill>.session_id` | The client session ID. It is the key the export uses. |
| `runs.<skill>.attempts` | 2 when the first attempt started before the connector tools were ready. That is a harness retry, not a skill failure. |

## Export with a personal access token

A PAT (`sp-live-…`) is accepted on **GET requests to `/admin/export/…` only**. It resolves to its owner and sees exactly what that person's console session would. Any other method or path returns 401. Issue one under **Account → Devices → Personal access tokens**, or use the bridge's own token. `eval.sh export` does the following:

| Request | Returns |
|---|---|
| `GET /admin/export/transcripts?session_id=<client session id>` | JSON Lines, one `ConversationBundle` per conversation in that session (schema version 1). |
| `GET /admin/export/transcripts/{context_id}?format=json\|markdown` | One conversation. |
| `GET /admin/export/transcripts?source=sessions&from=…&to=…` | Every conversation in a window, capped at 500. |
| `GET /admin/export/analysis-skills?from=…&to=…&format=json` | One row per skill: invocations, people, requests, tokens, cost, tool failures, p95. |
| `GET /admin/export/analysis-versions?marketplace=<id>&from=…&to=…&format=json` | One row per marketplace version: served window, invocations, requests, cost, p50/p95. |
| `GET /admin/export/analysis-marketplaces?from=…&to=…&format=json` | One row per marketplace. |

Every dataset also takes `format=csv|json|jsonl|markdown`, and `/preview` returns the row count without the body.

```bash
curl -H "Authorization: Bearer $PAT" \
  "https://<gateway>/admin/export/transcripts?session_id=<id>" -o run.jsonl
```

A bundle carries `facts` (the conversation's record: tokens, cost, turns, latency, tool calls, governance decisions), `requests` (per turn), `tool_calls` and `tool_ledger`, `skills` (each invoked skill with its `plugin_id`, `marketplace_id` and **`marketplace_hash`**), `decisions`, `messages` and `transcript`. The export also saves the client's local transcript of the same session, because only it holds full tool results, which are needed to classify errors.

## The metrics

`eval.sh metrics` writes `metrics.json`, `metrics.csv` and `summary.md` for the run. Per skill:

| Metric | Source | Rule |
|---|---|---|
| `invoked` | bundle `skills[]` | 1 when the skill ran under this plugin. |
| `marketplace_hash` | bundle `skills[]` | The version that served the run. |
| `turns`, `requests` | bundle `facts` | `turn_count`, `request_count`. |
| `input_tokens`, `output_tokens`, `cache_read_tokens`, `cache_creation_tokens` | bundle `facts` | As recorded by the gateway. |
| `cost_usd` | bundle `facts` | `cost_microdollars` / 1,000,000. |
| `duration_s`, `p95_latency_ms` | bundle `facts` | Wall time of the conversation and its p95 request latency. |
| `mcp_calls`, `sf_calls`, `google_calls`, `builtin_calls` | client transcript | Tool calls by server. |
| `failed_calls` | client transcript | Tool results that are errors or match any error class. |
| `sf_invalid_field`, `sf_invalid_type`, `sf_malformed`, `sf_access`, `sf_5xx` | client transcript | Salesforce error codes in tool results. |
| `google_iam`, `google_bad_args` | client transcript | Workspace permission denials, and malformed Workspace calls. |
| `timeout`, `tool_missing` | client transcript | Timeouts, and calls to tools that were not loaded. |
| `unscoped_queries` | client transcript | Record queries with neither the suite's scope filter nor a pinned record ID. |
| `writes` | client transcript | Any create, update, delete or send call. It must be 0. |
| `answer_chars`, `mock_mentions` | client result | Length of the final answer, and how often it had to invent labelled mock data. |
| `completed` | client result | Finished without error, with an answer of at least 200 characters. |
| `success` | all of the above | `invoked` and `completed`, at least one Salesforce call, no unscoped queries, no writes, and no field, object or malformed-query errors. |

The plugin row sums every count and adds `success_rate`, `completed_rate` and per-skill averages. With `--against <label>`, every metric also gets its delta, per skill and for the plugin.

## Read the change in the console

- **Analysis → Versions → the plugin's marketplace → History.** The run's version appears as a new row when the shipped hash moves, with invocations, requests, cost and latency for the conversations that ran under it.
- **Compare.** Pick the baseline's version as *before* and the new one as *after*. Every skill either version carried is listed with its change and its figures under each.
- **Analysis → Skills → a skill.** Per-skill figures over the window, including tool failures and cost.
- **Analysis → Conversations.** Open any run's conversation by its `context_id` from `metrics.csv` to read the full transcript and tool ledger.

The deterministic metrics and the console agree on tokens and cost because both read the same record. The eval adds what the console does not classify: error classes, scoping and success.

## Repeat

After each change, run the same suite with a new label, export it, and compute the metrics against the baseline and against the previous run. Commit `evals/<plugin>/<label>/` (raw exports are gitignored), so the plugin's history is readable in git beside the skill diffs that caused it.
