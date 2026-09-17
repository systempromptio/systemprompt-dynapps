---
title: "Artifacts and the Tool-Call Ledger"
description: "Every tool result from every client is one linked artifact: how a call's intent, execution and result are joined by the client tool_use_id, what each vantage point contributes, how results are scanned before they are stored, and what each dashboard column means."
author: "Astound Digital"
slug: "artifacts-and-tool-calls"
keywords: "artifacts, tool calls, tool_use_id, ledger, mcp_artifacts, mcp_tool_executions, ingestion, secret scanning, Claude Code, Cowork, OpenCode, Codex, hooks, gateway"
kind: "guide"
public: true
tags: ["enterprise", "admin", "observability"]
published_at: "2026-09-17"
updated_at: "2026-09-17"
after_reading_this:
  - "Say for any client which vantage points record its tool calls, and read the Tool calls, Executed and Artifacts columns as the same numbers on every page"
  - "Open an artifact and read its provenance strip: where it was seen, whether its join is exact or inferred, and the request, decision and session of the same call"
  - "Know what is stored (a typed, scanned body, once by digest) and what is never stored (secret spans, image bytes)"
related_docs:
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Audit & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Analysis"
    url: "/documentation/analysis"
---

# Artifacts and the Tool-Call Ledger

A tool call is one thing that the platform sees from up to four places, and the
console counts it once. This page is the contract behind the **Artifacts** page
and the **Tool calls** column on every AI-activity surface.

## Three entities, one key

| Entity | Table | Meaning | Who writes it |
|---|---|---|---|
| **Intent** | `ai_request_tool_calls` | the model emitted a `tool_use` block | the gateway, for every client on every wire |
| **Execution** | `mcp_tool_executions` | the call ran, with `source` (where it was seen) and `correlation` (how surely it was joined) | an in-process server, the proxy tap, or a client hook |
| **Result** | `mcp_artifacts` | one typed artifact per execution; body stored once by digest in `artifact_payloads` | the ingest, from any of the above |

They join on the client's `tool_use_id`. The `tool_call_ledger` view joins the
three for every reader, and names each row's **state**: `intended` (asked,
never reported as run), `executed`, or `unattested` (ran with no gateway
intent — the call bypassed the gateway).

## What each client contributes

| Client | Intent (gateway) | Execution and result |
|---|---|---|
| Claude Code, Cowork | yes | every tool, via the `PostToolUse` hook |
| OpenCode | yes | every tool, via the `tool.execute.after` hook |
| Codex, Hermes, Pi, other API clients | yes | MCP tools proxied through the gateway (the proxy tap); builtin tools have no reporting hook in those hosts, so they stay `intended` |
| In-process MCP servers | yes | always, at the server |

The **Tool calls** column is the intent count and is the same for every
client. **Executed** and **Artifacts** need a second vantage point, which is
why a Codex session can show tool calls without executions for its builtin
tools — that is a fact about the host, and the page shows it rather than
inferring it away.

## Ingestion: the narrow waist

Every result, whichever vantage point delivers it, goes through one path:

1. **Normalise.** The wire `CallToolResult` (or a hook's `tool_response`, or a
   gateway history block) becomes a typed artifact. A body that declares a
   known `x-artifact-type` is stored as that type; everything else is a
   `tool_result` with its content blocks, structured content and error flag.
   Image and audio bytes are never stored — only their size and digest.
2. **Resolve the execution.** By the server execution id carried in `_meta`,
   by the execution the vantage point itself minted, or by the client
   `tool_use_id` — all exact. A client-reported result with none of those is
   matched by session, tool, digest and time and marked **inferred**, which the
   console shows as a warning badge rather than hiding.
3. **Scan before write.** The installation's secret scanner runs over every
   string in the body and redacts each match in place; the gateway's safety
   scanners then run over the redacted text. Findings are recorded against the
   artifact. A scanner failure refuses the artifact rather than storing it
   unscanned.
4. **Store once.** The body is content-addressed: a history replayed on every
   turn, or the same file read ten times, is one stored body with a reference
   count.
5. **Enrich, never duplicate.** A call already stored from one vantage point is
   enriched with the keys the next one knew (session, trace, `tool_use_id`,
   last seen from) and its body is never replaced by a client's copy.

The proxy tap stamps the execution id into the response `_meta` before it
leaves the gateway, so a hook that later reports the same result carries the
exact server key.

## Reading the pages

- **Artifacts** (`/admin/artifacts`) lists structured results by default —
  the ones a server declared as data — and *Show all results* widens it to
  every tool result, builtin tools included. Filters: user, session, tool,
  source, skill, failed only.
- **Artifact detail** shows the typed body, its digest and size, a provenance
  strip (source, join quality, client, `tool_use_id`, execution, intent
  request, governance decision, session, trace), every scanner finding, and a
  sandboxed preview for typed artifacts.
- **Request detail** lists each tool call's state and links its artifact.
- **Conversations** link each tool step to its stored result.
- **Traces** link hook tool events to the artifact they produced.
- **Analysis › Skills / Conversations** count artifacts per skill and per
  conversation from the `artifact` fact on the feedback plane.

## Where the numbers come from

A hook event keeps the `tool_use_id` it reported and the execution the ingest
resolved (`plugin_usage_events.tool_use_id`, `mcp_execution_id`). A governance
decision keeps the `tool_use_id` it governed. Skill attribution of an artifact
is the most recent skill invocation in the same session before the result was
produced. The feedback plane's `artifact` fact carries the execution, the
invocation and request keys where known, the vantage point, the join quality
and the finding count, and is what the Analysis pages aggregate.
