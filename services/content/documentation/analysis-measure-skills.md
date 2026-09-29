---
title: "Measure Which Skills Are Used"
description: "Step-by-step: pick a window and marketplace, sync the skill inventory, and read every column of the Analysis Skills table."
author: "Astound Digital"
slug: "analysis-measure-skills"
keywords: "skill usage, inventory sync, invocations, adoption, installs, attributed use, p95, marketplace, conversation rollup, ai judge"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-29"
after_reading_this:
  - "Sync the skill inventory and confirm it landed"
  - "Read every column on the Skills table without guessing"
  - "Open the conversations behind one skill"
related_docs:
  - title: "Analysis: Measure, Improve and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Create a Conversation and See It Land"
    url: "/documentation/analysis-test-conversation"
  - title: "Versions: Sources, Generations and Compare"
    url: "/documentation/analysis-versions"
---

# Measure which skills are used

**TL;DR:** Open `/admin/analysis/skills`, pick a window and a marketplace in the scope strip, then read the three views: **Overview** (one row per marketplace: entitled → installed → active), **Activity** (invocations over the window) and **Skills** (the table). Press **Sync inventory** on the Skills tab when the inventory is behind. Figures are per marketplace and per time window.

## Prerequisites

- An administrator account. Marketplace participants reach Conversations and Versions but not Skills; see [Roles](/documentation/admin-user-management#roles).
- At least one marketplace imported or pinned. If none is, start at [Code ↔ Instance](/documentation/services-sync).

## Step 1: Open Skills and pick a marketplace

Go to `/admin/analysis/skills`. The page opens on the **Overview** tab with **All marketplaces** in scope.

The scope strip above the tabs holds two choices — the **window** (7, 30 (default), 90 days or a year) and the **marketplace**. Both are links, so a choice is bookmarkable, and both survive a change of tab: pick a marketplace on Overview and the Activity chart and the Skills table show only its skills. The tabs are:

- **Overview** — one row per marketplace: people entitled, consumers holding a verified receipt (with the install rate over the entitled), people active in the window, skills used of those declared, invocations, conversations, cost and the judge's mean. The marketplace name opens its Skills table; the invocation count opens its Activity chart.
- **Activity** — invocations over the window as one chart (per day, per week for a year), with the skills it draws beneath it.
- **Skills** — the table this guide reads, grouped by marketplace and plugin, with the **Client**, **Sort** and search filters. A skill invoked on the instance is credited to the plugin its key names, whichever plugin carries the hooks.

## Step 2: Sync the inventory

Press **Sync inventory** in the toolbar of the Skills tab (administrators
only). It publishes every configured skill into the managed inventory now,
instead of waiting for `managed_inventory_refresh`, which runs every minute.
Versions are also recorded at boot, so a marketplace imported while the
server was down appears after a restart.

**Expected result:** the marketplace's version on
[Versions](/documentation/analysis-versions) is current. A repeat sync with
nothing changed on disk records nothing new, which is correct.

## Step 3: Give it a minute

Nothing on the Analysis pages is computed live from raw events. The
`conversation_rollup` job re-derives, every minute, each conversation touched
on any plane (gateway, hooks, governance, tools, artifacts) since its last
run, into `conversation_facts` and the per-skill `conversation_skill_facts`.
A skill invoked a moment ago appears on the next pass. If the page still
shows nothing after two minutes, follow
[Create a Conversation and See It Land](/documentation/analysis-test-conversation).

Skills that are configured but were not invoked in the window are not rows.
They are counted on the **unused** button in the toolbar, which lists each one
with its marketplace and how many people are entitled to it.

## Step 4: Read the table

The table is grouped by marketplace, then by plugin. Each marketplace header
shows its skills, its invocations and the most people on any one skill; the
marketplace name opens its history on Versions. The filter ribbon narrows by
**Client**, **Sort** and search, and **Export** downloads the rows in CSV,
JSON, JSON Lines or Markdown with the columns you pick. Every table page in
the console carries the same button.

| Column | Definition |
|---|---|
| **Skill** | The skill's name, opening its page. Beneath it: the share of invocations attributed with evidence, and the date first used. |
| **Invoked** | Invocations reported by the harness hooks — slash command and Skill tool. |
| **14 days** | Invocations per day over the last 14 days. |
| **People** | Distinct people who invoked it, over the people entitled to its marketplace. |
| **Installs** | Consumers holding a verified installation receipt. |
| **Conv.** | Gateway conversations whose session invoked the skill; hover for turns. |
| **Tokens** / **Cost** | Tokens and spend in those conversations. Spend overlaps when several skills run in one conversation, so never sum the column. |
| **/inv** | Cost per invocation. |
| Wrench | Tool calls in those conversations, failed calls in brackets. Opens them. |
| File | Artifacts those conversations produced. Opens them. See [Artifacts and the Tool-Call Ledger](/documentation/artifacts-and-tool-calls). |
| **Err** | Failed requests, and denied tool calls behind ⛔. |
| **p95** | 95th percentile turn latency. |
| **Models** / **Agents** | Models and clients seen. |
| **Last** | Last invocation. |
| Sparkle **AI** | The AI judge's mean completion score over judged conversations, with judged / total beside it. |

A skill carried by two plugins appears under each one; the marketplace
header counts it once.

## Step 5: Open one skill

Click the skill name. Its page (`/admin/analysis/skills/<plugin:skill>`)
breaks the same figures down (by client, model or kit version), lists its
workflow runs where the skill keeps a workflow log — succeeded, median and
p90 time, agent time against human wait, cost per run — and lists every
conversation that invoked it, with person, client and model, turns, tool
calls, artifacts, errors, cost and the judge's score.

**Check:** the conversation count on the skill's page matches **Conv.** in
the table for the same window.

## Troubleshooting

### A skill I just used is not in the table

**Cause:** the rollup has not passed yet, the window or marketplace excludes
it, or the plugin carrying the governance hooks is not installed on that
client — without the hooks, no invocation is reported.
**Solution:** wait a minute and reload, widen the window, clear the search,
then check the client with `astound-bridge status`.

### Invoked is high but Conv. is zero

**Cause:** the client's model traffic does not go through the gateway, so
there is no gateway conversation to join. Hook reports still arrive.
**Solution:** route the client through the gateway; see
[Connect Claude Code](/documentation/connect-claude-code).

### Spend looks too high across the page

**Cause:** spend overlaps when several skills run in one conversation.
**Solution:** read cost per skill and never sum the column.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Create a Conversation and See It Land](/documentation/analysis-test-conversation)
