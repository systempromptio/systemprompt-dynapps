---
title: "Conversation History & Search"
description: "Every AI conversation through the gateway — from Claude Code, Claude Desktop, OpenCode or any SDK client — listed and searchable: your own at /admin/history, everyone's at /admin/conversations for console roles."
author: "Astound Digital"
slug: "enterprise-conversation-history"
keywords: "history, conversations, search, transcripts, audit, export, redaction, retention, gateway"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin"]
published_at: "2026-08-26"
updated_at: "2026-09-29"
after_reading_this:
  - "Find and search your own AI conversations at /admin/history, whichever client produced them"
  - "Search everyone's conversations at /admin/conversations as an admin"
  - "Open one conversation and read its prompts, responses and tool calls"
  - "Know who can see whose history, and what is not wired yet"
  - "Know how long conversation text is kept"
related_docs:
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Test a Conversation"
    url: "/documentation/analysis-test-conversation"
  - title: "Data Retention & Archives"
    url: "/documentation/data-retention"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
---

# Conversation History & Search

**TL;DR:** Every conversation that goes through the gateway is listed, whichever client sent it: Claude Code, Claude Desktop, OpenCode, or any Anthropic- or OpenAI-SDK client.

- **`/admin/history`** ("My conversations") shows every signed-in user their own conversations.
- **`/admin/conversations`** shows everyone's to admins, with a user column and a per-user filter.
- **Search** matches a conversation's title, its opening prompt, its id and its model.
- **Redaction:** snippets and bodies pass through the credential and PII redactor.
- **Retention:** conversation text is kept for 30 days, and the conversation itself for 90.

## What a conversation is

A conversation is the gateway's own request log, grouped by conversation (`context_id`). The same list and the same search work whatever the client, and the **Source** column says which client sent it. A conversation appears once, whichever client made it.

Each row shows:
- when it started and when it was last active;
- the number of turns;
- tokens in and out;
- cost;
- the model;
- the client session it ran in, when the client named one.

Cache probes and utility calls that clients send alongside real turns are **side calls**. They are hidden unless you add `?side=1`, and a conversation made only of side calls is listed only on request.

## Searching

Type into the search box on either page, or call `/admin/api/history/search`. The match is a case-insensitive substring over:
- the conversation's title (the judge's title, or the client's name for it);
- the opening prompt;
- the conversation id;
- the model.

It is **not** full-text search over every turn. To find a phrase deep in a conversation, export the conversations and search the export.

The window controls filter on a conversation's last activity. Both listings export their rows (**Export** on the page). Whole conversations export as JSON or Markdown one at a time, or as JSON Lines for a selected or filtered set (`/admin/export/transcripts`). The same scope applies: you can export your own conversations, and an admin can export anyone's.

## Reading one conversation

Selecting a conversation opens `/admin/history/conversations/{id}`. It shows every prompt, response and tool call in order, with the request telemetry beside each turn. The page is owner-facing, so you do not need an admin role to read your own conversation. Two things are done to the text on the way out:

- The gateway's internal `=== USER PROMPT ===` framing is removed, so you read what you wrote rather than the protocol wrapper around it.
- Every body passes through the credential redactor. The page says how many values were masked rather than altering the text silently.

Operational identifiers (trace ids, request ids, upstream request ids, route matches) are not on this page. An admin gets a link from it to the full context view, which carries them. For the judged analysis of the same conversation (outcome, score, skills used), see [Test a Conversation](/documentation/analysis-test-conversation).

Asking for a conversation you do not own returns **404**, the same answer as an id that does not exist. A 403 would confirm that the conversation exists and whose it is not, which would turn the URL into a way to enumerate other people's conversation ids.

## Who sees whose history

| You are | You can see |
|---|---|
| Any signed-in user | Your own conversations, always, at `/admin/history` |
| `admin` or `platform_admin` | Everyone's, at `/admin/conversations`, with a per-user filter |

Scope is resolved on the server for every request. A request naming a `user_id` outside your scope returns **403**, and the result set can never widen beyond the users your identity resolves to. Group and project scoping, which narrows the admin analytics pages, is deliberately not applied to your own history: it must never hide a person's own conversations from them.

Two gaps:

- **No manager view.** The scope resolver accepts a list of people a viewer manages, but nothing supplies that list yet. A project manager therefore sees only their own conversations, and a line-manager view of a team's history is not available.
- **No auditor role to grant.** The code gives full history view to an `auditor` role, but `auditor` is not among the roles the directory mapping or the role editor can assign. Today, full view means an admin role.

## Redaction carries through

Snippets in the search results use the same display-layer redaction as the rest of the audit surface. Values the safety scanners flagged (secrets, PII) stay masked, so reviewing history does not re-expose sensitive data that was caught on the way through. See [Content Safety & Guardrails](/documentation/enterprise-safety-guardrails).

## Retention

History reads the audit trail, so it keeps what the trail keeps:

| Kept | For |
|---|---|
| Stored prompt and completion text | 30 days |
| The conversation's requests, and so its row in the list | 90 days |
| The per-conversation facts (the judge's title, summary, outcome and score) | indefinitely |

Before raw rows expire, the weekly archive copies them off the database. See [Data Retention & Archives](/documentation/data-retention).

## Check it yourself

- `/admin/history`: your own conversations. Search for a word from a recent prompt.
- `/admin/conversations`: as an admin, filter to one person and compare with their `/admin/users/{id}?tab=usage`.
- `/admin/history/conversations/{id}` for someone else's conversation: expect a 404.
- From the CLI (admin):

  ```bash
  systemprompt analytics conversations list --since 7d --source gateway
  ```
