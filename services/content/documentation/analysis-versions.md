---
title: "Versions: Sources, Generations and Compare"
description: "Trace a skill from the source that declared it, through the inventory generation that observed it, to the immutable revision it published, and compare two generations line by line."
author: "Astound Digital"
slug: "analysis-versions"
keywords: "versions, sources, bundle, base, content hash, composed hash, inventory generation, compare, managed resources, revision, pin, kit"
kind: "guide"
public: true
tags: ["enterprise", "admin", "operations"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Read the Marketplace sources table and say whether the instance serves what the profile pins"
  - "Explain why a generation exists at all, and what its composed hash covers"
  - "Compare two generations and tell a changed skill from a moved kit"
  - "Filter the timeline to one hash, and find where a pinned kit first appeared"
related_docs:
  - title: "Analysis: Measure, Improve and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Code ↔ Instance: Sources, Planes and Sync"
    url: "/documentation/services-sync"
  - title: "Improve a Skill: Campaign to Publication"
    url: "/documentation/analysis-optimize"
---

# Versions: sources, generations and compare

**TL;DR:** `/admin/analysis/versions` is the provenance tab. Sources are where declarations came from, generations are the moments the observed skill set actually moved, and managed resources are the captured revisions behind each skill. Importing a baseline does not change active skills and does not start an experiment.

## Prerequisites

- An administrative account. The **Capture Super Admin baseline** action needs manage rights.
- At least one inventory sync, so a generation exists.

## Step 1: Read the sources

The **Marketplace sources** table lists every source with the generation that first carried each hash.

| Column | Meaning |
|---|---|
| **Source** | The source name with its owner key. `base` is this repository's own services tree. `bundle:<name>` is a kit the profile pins. |
| **Version** | The version the source declares, when it declares one. |
| **Content hash** | What the source currently declares. Hover for the full value. |
| **Pinned** | The digest the profile pins. |
| **Active** | The digest actually in use. |
| **First seen** | The generation that first carried this source. |
| **Last changed** | When the source last moved. |
| **Manage** | Opens Code ↔ Instance for that source. |

**Pinned and Active differing is the thing to look for.** It means the instance is not serving what the profile pins. Reconcile it on [Code ↔ Instance](/documentation/services-sync).

## Step 2: Read the generation timeline

An inventory generation is minted only when the observed entry set moves. Every row is therefore a real change, not a heartbeat.

| Column | Meaning |
|---|---|
| **Generation** | The generation number. |
| **Observed** | When the pass that minted it ran. |
| **Composed hash** | The hash over everything the generation observed, across all sources. |
| **Entries · change** | Total entries, then how many were added, removed and changed. |
| **Publications** | How many publications this generation triggered. |
| **Compare** | Opens the diff against the previous generation. The oldest row reads `first`. |

## Step 3: Compare two generations

Press **Compare** on any row. The compare page answers one question: did the skill change, or did the kit that declares it move?

It opens with five tiles: the baseline generation, the candidate generation, skills added, skills removed and revisions changed. Then three tables.

1. **Sources.** The content hash of each source at each generation, marked `moved` or `unchanged`.
2. **Skills entering and leaving.** Every skill added or removed. An entry counts as removed once a pass records it withdrawn.
3. **Revisions changed.** The captured revision digest before and after, for every skill present in both generations.

So a skill under **Revisions changed** while its source is `unchanged` means the bytes of that skill changed. A whole block of changes alongside a `moved` source means the kit moved underneath them.

The direct URL is `/admin/analysis/versions/generations/<older>/compare/<newer>`.

## Step 4: Filter the timeline to one hash

**Filter by hash** sits directly under the **Generation timeline** heading. Paste a composed hash, a base tree hash or a bundle content hash into the field and press **Filter**.

The timeline is reduced to generations carrying that hash, and matching source and generation rows are highlighted. A banner at the top of the page names the active filter, and **Clear** beside the field, or **Clear filter** in the banner, drops it. Paging through the timeline keeps the filter applied.

If no generation carries the hash, the table says "No generation carries this hash" and offers to clear the filter, rather than looking like an empty timeline. You can also set the filter directly in the URL as `?hash=<value>`.

## Step 5: Read the managed resources

**Managed resources** lists every captured skill with its source, its revision count and a link to the latest revision. The resource link opens its full revision chain at `/admin/analysis/versions/<id>`; the evidence link opens one revision at `/admin/analysis/revisions/<id>`. Use **Previous** and **Next** at the foot of the page to page through them.

**Capture Super Admin baseline** in the page header captures the current state as a baseline in your authoring scope. It is how you get a captured revision for a skill that has never had one, which is what Campaigns requires.

## How pinning or importing a kit shows up here

The chain is visible end to end, and it is worth walking once so the numbers stop being mysterious.

1. You pin the kit by digest on [Code ↔ Instance](/documentation/services-sync), or change the digest it pins.
2. The source appears in the **Marketplace sources** table with its content hash, and **Pinned** takes the new digest.
3. The next inventory pass observes a different entry set, so a generation is minted. Its **Entries · change** cell shows the kit's skills arriving as additions.
4. **Active** catches up with **Pinned** on the source row, and **First seen** records the generation that first carried the hash.
5. **Compare** against the previous generation shows that source as `moved` and lists each arriving skill.
6. Each new skill becomes a managed resource with a captured revision, and shows up as a normal row on [Skills](/documentation/analysis-measure-skills).

A kit owns its content. This repository owns access, so entitlement for a kit's marketplace is still declared in `rules.yaml` and will report as awaiting its bundle until the source is active.

## Troubleshooting

### No generations observed

**Symptom:** The timeline is empty.
**Cause:** The inventory has not reconciled yet.
**Solution:** Press **Sync inventory now** on Campaigns or Skills, or wait for the scheduled pass, which runs every minute.

### No sources resolved

**Symptom:** The sources table is empty.
**Cause:** The profile pins no bundles and the base tree has not been hashed yet.
**Solution:** Run a sync, then check the profile's `services.sources[]` list.

### A source has no content hash

**Symptom:** The Content hash cell is blank.
**Cause:** The source predates hash recording.
**Solution:** Re-sync. Generations minted before hash recording cannot be back-filled, and the compare page says so when neither side carries provenance.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Code ↔ Instance: Sources, Planes and Sync](/documentation/services-sync)
- [Improve a Skill: Campaign to Publication](/documentation/analysis-optimize)
