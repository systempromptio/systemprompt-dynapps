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

**TL;DR:** Analysis is the part of the admin console that answers one question in stages: is a skill worth its cost, and did the marketplace version you shipped help? It starts at `/admin/analysis/skills`. A marketplace version is its content hash — the sha256 of everything it delivers — and every figure is keyed on that hash. Figures are read from retained reporting data, never computed live from raw events, and a skill only carries figures once its inventory entry is bound and attribution evidence exists. Analysis observes and compares; it does not run experiments.

## What the section is for

Four questions, in the order you normally ask them:

1. Which skills are actually used, by how many people, at what cost and failure rate?
2. What did those people do with them? Which conversations, which requests?
3. What did a new version of a marketplace do to real traffic, compared with the one before it?
4. What is the instance serving right now, and which devices received it?

## The tabs

They sit in workflow order on every Analysis page.

| Tab | Route | What it answers |
|---|---|---|
| **Skills** | `/admin/analysis/skills` | Cost, use and health for every skill, grouped by marketplace, each group stamped with the version hash it serves. |
| **Conversations** | `/admin/analysis/skills` | Which skills carry attributed use, and the sessions behind the one you select. |
| **Versions** | `/admin/analysis/versions` | Every marketplace by content hash: its history, what changed between versions, how each performed, and what devices received. |

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
