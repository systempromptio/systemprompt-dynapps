---
title: "Measure Which Skills Are Used"
description: "Step-by-step: sync the skill inventory, resolve name collisions, wait for the first snapshot, and read every column of the Analysis Skills table."
author: "Astound Digital"
slug: "analysis-measure-skills"
keywords: "skill usage, inventory sync, name collision, binding, attributed use, coverage, p95, marketplace, retained snapshot"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Sync the skill inventory and confirm a generation was minted"
  - "Resolve a name collision by binding an entry to the right managed resource"
  - "Read every KPI and every column on the Skills table without guessing"
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

**TL;DR:** Open `/admin/analysis/resources`, pick a marketplace, press **Sync inventory now**, clear any collisions, wait a minute for the first snapshot, then read the table. Figures are per marketplace and per time window.

## Prerequisites

- An administrative account on the admin console.
- At least one marketplace imported or pinned. If the Skills page shows an onboarding strip with **Import or pin a marketplace** unticked, start at [Code ↔ Instance](/documentation/services-sync).

## Step 1: Open Skills and pick a marketplace

Go to `/admin/analysis/resources`. The page opens on the marketplace with the most attributed use.

**Marketplace** is the first field of the toolbar filter above the table. Each option carries its skill count in brackets. Choose **All marketplaces** to group every marketplace at once and name the plugin beside each skill, then press **Apply**.

The line above the table summarises what you are looking at: the marketplace, the number of skills and how many plugins they span, how many people are entitled, and a **Sources** link to the generation timeline on Versions.

## Step 2: Sync the inventory

Press **Sync inventory now** in the page header. It captures every configured skill and publishes the latest revision immediately, instead of waiting for the scheduled pass.

**Expected result:** the status line reads `Inventory generation N · synced a few seconds ago · snapshot …`. If the generation number did not move, nothing on disk changed, which is the correct outcome for a repeat sync.

## Step 3: Resolve name collisions

If a red **Name collisions** notice appears, some skills exist both as files in `services/` and as an imported managed resource with the same id. Metrics cannot attach until you choose which one counts.

1. Expand **Resolve**.
2. For each row, pick the managed resource in the dropdown.
3. Press **Bind**.

A row with `no imported resource to bind to` has nothing to choose from yet; sync again after the resource is imported.

## Step 4: Wait for the first snapshot

Figures appear once a snapshot exists for the bound resource. The inventory refresh runs every minute; the feedback fact and snapshot passes run every five seconds. Give it a minute or two, then reload.

Until then the Invocations column shows one of three words:

| Word | Meaning |
|---|---|
| `unbound` | The entry is not bound to a managed resource, so nothing can attach to it. |
| `pending` | Bound, waiting for its first snapshot. |
| `suppressed` | The entry is excluded from measurement. |

## Step 5: Read the KPI tiles

| Tile | What it counts |
|---|---|
| **Skills** | Skills in this marketplace, with how many are measured. |
| **Attributed use** | Invocations carrying verified evidence. |
| **Failing** | Skills with a name collision or more than 10% failed requests. |
| **Requests** | Attributed requests, with the failed count beside it. |
| **Spend** | Related spend. It overlaps across skills and must never be summed. |

The tiles are organisation-wide for the window and include traffic not yet attributed to any skill, so they will not add up to the sum of the rows. That is intended.

## Step 6: Read the columns

Use the **Marketplace** field to switch marketplace, **Window** for 7, 30, 90 or 365 days, and **Skill** to search. **All columns** reveals two extra columns; **Fewer columns** hides them again. **JSON** returns the same data at `/admin/analysis/resources.json`.

| Column | Definition |
|---|---|
| **Skill** | The skill's resource key, and the link to its conversations. Hovering it reads "<name> — conversations that used this skill". |
| **Health** | Red for a name collision, an unavailable or withdrawn entry, or more than 10% failed requests. Amber for unbound or audience-less skills, and for measured skills with no attributed use. Grey while a bound resource waits for its first snapshot. |
| **Invocations** | Invocations attributed to this skill, with the share carrying verified evidence as a percentage beside it. |
| **Users** | Distinct consumers over audience. Audience is the number of signed-in people entitled to receive the skill through at least one marketplace. A red zero means nobody can receive it. |
| **Requests** | Attributed requests, with failures in red. |
| **Tokens** | Input and output tokens on attributed requests. Behind **All columns**. |
| **Cost** | Related spend on this skill's attributed requests. Overlaps across skills. |
| **p95** | 95th percentile latency. |
| **Assessed** | Scored conversations over assessed conversations. Behind **All columns**. |
| **Coverage** | Acknowledged devices over eligible devices. |
| **Published** | The generation the bridge serves. A dot means the latest capture is not the served generation yet. |
| **Actions** | **Optimize** starts an optimization campaign with the skill's current revision as the baseline, and appears only once the skill is bound to a managed resource that has one. **Catalog** opens the skill in the marketplace catalog, where its contents and entitlement live. The conversations link is the skill name itself, in the first column, not an action here. |

A skill carried by two plugins appears under each one. Group rollups count it once.

## Step 7: Open one skill's conversations

Click the skill name. Conversations opens at `/admin/analysis/skills` filtered to that skill, listing every session with an attributed invocation: session, consumer, first and last seen, invocations, requests, tokens, cost and the latest retained assessment.

**Verification:** the conversation count on that page should be consistent with the Users column you just read for the same window. If the skill link lands on an empty table, see [Create a Conversation and See It Land](/documentation/analysis-test-conversation).

## Troubleshooting

### The table is empty for this marketplace

**Symptom:** "No skills here".
**Cause:** No skill matches the current marketplace and search.
**Solution:** Press **Clear** to drop the search, change the **Marketplace** field, or sync the inventory.

### Coverage shows an em-dash

**Symptom:** The Coverage column has no numbers.
**Cause:** No installation record exists for the skill, so there is nothing to acknowledge.
**Solution:** Have someone install the distributed generation. Receipts are listed on [Publication](/admin/analysis/publications).

### Spend looks too high across the page

**Symptom:** Adding the Cost column gives a larger number than the Spend tile.
**Cause:** Related spend overlaps when several skills run in one conversation.
**Solution:** Read costs per skill and never sum the column.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Create a Conversation and See It Land](/documentation/analysis-test-conversation)
- [Improve a Skill: Campaign to Publication](/documentation/analysis-optimize)
