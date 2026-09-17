---
title: "Analysis: Measure, Compare and Publish a Skill"
description: "What the Analysis section of the admin console answers, what each of its tabs is for, and the end-to-end path from an unmeasured skill to a marketplace version whose performance you can read and compare."
author: "Astound Digital"
slug: "analysis"
keywords: "analysis, skills, conversations, versions, marketplace hash, distribution, inventory sync, attribution, retained snapshot"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Name what each Analysis tab answers, and open the right one for a given question"
  - "Tell a configured entry, an imported managed resource and a binding apart"
  - "Explain why a skill shows figures, and why another one shows unbound or pending instead"
  - "Follow the whole path from syncing the inventory to comparing two versions of a marketplace"
related_docs:
  - title: "Measure Which Skills Are Used"
    url: "/documentation/analysis-measure-skills"
  - title: "Create a Conversation and See It Land"
    url: "/documentation/analysis-test-conversation"
  - title: "Versions: Marketplace Hashes, History and Compare"
    url: "/documentation/analysis-versions"
  - title: "Code ↔ Instance: Sources, Planes and Sync"
    url: "/documentation/services-sync"
---

# Analysis: measure, compare and publish a skill

**TL;DR:** Analysis is the part of the admin console that answers one question in stages: what did people do with the gateway, how well did it go, is a skill worth its cost, and did the marketplace version you shipped help? It starts at `/admin/analysis/conversations`; the zoomed-out answer is `/admin/analysis/evaluation`. A marketplace version is its content hash — the sha256 of everything it delivers — and every figure is keyed on that hash. Figures are read from retained reporting data, never computed live from raw events, and a skill only carries figures once its inventory entry is bound and attribution evidence exists. Analysis observes and compares; it does not run experiments.

## What the section is for

Four questions, in the order you normally ask them:

1. What did people do with the gateway, and how well did it go — as the classifier and the judge read each conversation?
2. Which skills are actually used, by how many people, at what cost, and how does the judge score the sessions that used them?
3. Across everything: what is going wrong, and what should the owners of these skills and marketplaces change?
4. What did a new version of a marketplace do, compared with the one before it, and which devices received it?

## The pages

Every Analysis page lives at `/admin/analysis/<noun>`, the same noun and identity the Platform section uses at `/admin/<noun>` — Platform says what is configured, Analysis says how it performs. The sidebar lists them in workflow order.

| Page | Route | What it answers |
|---|---|---|
| **Conversations** | `/admin/analysis/conversations` | Every gateway conversation with its purpose and outcome (classifier), the judge's four scores for the session behind it, the skills used and the cost. |
| **Skills** | `/admin/analysis/skills`, `/admin/analysis/skills/{plugin:skill}` | Every skill with the judge's scores inherited from the sessions that invoked it, cost, use and health, grouped by marketplace and stamped with the version hash it serves; a skill's own page has its trend and the conversations behind it. |
| **Evaluation** | `/admin/analysis/evaluation`, `/admin/analysis/evaluation/{marketplace_id}` | The zoomed-out report: an AI-written account of what the judge is seeing and what to do, every marketplace's current version against its history, the trend, and the judge's configuration. |
| **Versions** | `/admin/analysis/versions`, `/admin/analysis/versions/{marketplace_id}` | Every marketplace by content hash: its history with the judge's scores per version, what changed between versions, how each performed, and what devices received. |

Two AI judges feed these pages. The `conversation_classifier` job reads each quiet conversation and records its category, summary and outcome. The `session_evaluation` job scores each closed Claude Code session on four criteria — intent, quality, speed, cost — 0–100 with rationale and quoted evidence (`services/evaluation/`). A skill inherits the scores of every judged session that invoked it; a marketplace version inherits those of every session served while it was current. The `evaluation_report` job then writes the report on the Evaluation page from those figures.

Versions has three views per marketplace — **History**, **Compare** and **Distribution** — described in [Versions: Marketplace Hashes, History and Compare](/documentation/analysis-versions). The old Impact and Publication pages redirect there.

## Three concepts to get right first

**Configured entry against imported managed resource.** A configured entry is a skill declared in this repository's `services/` tree or in a kit the profile pins. An imported managed resource is a captured, versioned copy that the platform owns and can publish. They are different objects with the same name, which is why collisions happen.

**Binding.** Metrics attach to a managed resource, not to a file on disk. A row is *measured* once its inventory entry is bound to a managed resource and a snapshot exists for it. Until then the Skills table shows `unbound` or `pending` in place of figures.

**Retained snapshot.** Nothing on these pages is recomputed from raw events. A background pass drains facts into snapshots, and the Skills table reads a window of 7, 30, 90 or 365 days from them; Users and p95 are read from the retained facts for the same window, so p95 is a real percentile over request latency rather than a histogram bound. Attribution needs verified evidence: a device-authenticated consumer, a session binding and a verified installation receipt. An invocation without that chain still counts in the headline totals, but not in its own skill row.

**Marketplace version.** The sha256 of everything a marketplace delivers — its own config, every plugin it includes and every skill those plugins ship — hashed exactly as a kit's `content_hash` is. It is recorded at boot and on every inventory sync, a new version whenever the bytes move, and every invocation fact is stamped with the hash serving at the time. That hash is the only version identity Analysis shows; there are no generation numbers.

## The end-to-end path

Work through this once and the rest of the section reads itself.

- [ ] Import or pin a marketplace on [Code ↔ Instance](/documentation/services-sync), so there are skills to measure.
- [ ] On **Skills**, press **Sync inventory now** so every configured skill has an entry and a published revision.
- [ ] Resolve any **Name collisions** by binding each entry to the right managed resource.
- [ ] Wait for the first snapshot. Figures appear once it lands.
- [ ] Connect a client and invoke a skill for real, then find the conversation. See [Create a Conversation and See It Land](/documentation/analysis-test-conversation).
- [ ] Read the Skills table and open a skill's conversations. See [Measure Which Skills Are Used](/documentation/analysis-measure-skills).
- [ ] Change a skill and sync again; open **Versions** to see the new hash, what changed and how it performs against the version before.
- [ ] Check **Distribution** on that marketplace to confirm devices received it.

The three-step strip at the top of the Skills page tracks the first three of these and ticks each one off as it completes.

## Access

Analysis is an administrative surface. The manual review form and the withdrawal decisions on a marketplace's Distribution view require manage rights; without them the pages render read-only. See [User & Access Management](/documentation/enterprise-user-access).

## Troubleshooting

### Every skill says "unbound"

**Symptom:** The Skills table lists skills but the Invocations column reads `unbound`.
**Cause:** The inventory has entries but nothing is bound to a managed resource.
**Solution:** Press **Sync inventory now**, then resolve any collisions listed under **Name collisions**.

### Figures stay empty after a sync

**Symptom:** Rows are bound, the meta line says the inventory synced, and the figures are still blank.
**Cause:** The snapshot for that window has not landed yet.
**Solution:** Wait for the next pass. The inventory refresh runs every minute and the feedback facts and snapshot passes run every five seconds, so a first snapshot is normally a minute or two away, not an hour.

### A skill has invocations but no attributed use

**Symptom:** The organisation totals move, one skill row does not.
**Cause:** The invocations lack verified evidence, so they cannot be attributed to that skill.
**Solution:** See [Create a Conversation and See It Land](/documentation/analysis-test-conversation#when-it-does-not-appear).

## Related pages

- [Measure Which Skills Are Used](/documentation/analysis-measure-skills)
- [Create a Conversation and See It Land](/documentation/analysis-test-conversation)
- [Versions: Marketplace Hashes, History and Compare](/documentation/analysis-versions)
- [Code ↔ Instance: Sources, Planes and Sync](/documentation/services-sync)
