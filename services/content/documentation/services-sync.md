---
title: "Code ↔ Instance: Sources, Planes and Sync"
description: "One sync system for every declared service, local and external: where declarations come from (sources), what they project into (planes), which hash proves what, the three directions, and how a kit published from another repository reaches the instance without a redeploy."
author: "Astound Digital"
slug: "services-sync"
keywords: "sync, services bundle, kit, marketplace, digest, content hash, composed hash, declared hash, provenance, refresh, pin, rules.yaml, owner, access control, drift"
kind: "guide"
public: true
tags: ["enterprise", "admin", "operations"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Read the Sync page and say, for every source, whether the instance is serving what the profile pins"
  - "Tell the four hashes apart and know which one a pin, an import and an apply each change"
  - "Ship a kit from its own GitHub repository to the instance: publish, pin, refresh, verify"
  - "Reconcile a plane in either direction and read back who applied what, when, from which declaration"
related_docs:
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
---

# Code ↔ Instance: Sources, Planes and Sync

**TL;DR:** Everything this instance serves is *declared* somewhere in code and *enforced* from what the process loaded and what the database holds. `/admin/sync` is the one page that compares the two. A **source** is where declarations come from — `base` is this repository's `services/` tree, `bundle:<name>` is each external kit the profile pins by digest — and every source has a content hash. A **plane** is what a source's declarations project into the database — access control today — and every plane records the declared hash it last applied, when, by whom and in which mode. Nothing writes on its own after the first seed: the page shows the difference and offers a direction.

## The model

| word | meaning | today |
|---|---|---|
| **source** | a tree of declarations with a content hash | `base` (this repository), `bundle:sfnext` (the Europe development kit), any further kit |
| **plane** | a projection of declarations into database tables, with drift and three directions | `access_control` (`rules.yaml` → `access_control_rules`, `access_control_entities`) |
| **ownership** | decided by id at composition | a bundle owns its marketplaces, plugins and skills; the base owns everything else, every entitlement included |
| **hash control** | every source reports what it declares; every plane records what it applied | the table below |

Planes are a contract, not a list: `groups` (groups.yaml → groups, projects, directory mappings) and `gateway_policies` are shaped for and not built. When they are, the page grows a card each; nothing else changes.

## The four hashes

| hash | comes from | changes when | proves |
|---|---|---|---|
| bundle **digest** `sha256:…` | the registry, per pushed artifact | a kit publishes a release | *which upload* — this is what the profile pins |
| bundle **content hash** | `bundle.json`: SHA-256 over sorted `path\0sha256` | the kit's files change | *which tree* — the same tree pushed twice has one content hash |
| base **tree hash** | computed on the page over `services/` with the same algorithm | this repository deploys | equals the base bundle's `content_hash` published at this release |
| **composed hash** | SHA-256 over ordered `name\0content_hash` of every active source | any source changes | *which composition* the process is serving; **reconcile pending** when it differs from `last_reconciled_hash` |
| plane **declared hash** | SHA-256 over the canonical declaration (for access control: every `(entity, band, subject, access, why)` plus defaults and awaiting entities) | the file's meaning changes (not its whitespace) | what the database was last made equal to; recorded per apply in `sync_state` |

A row on the Sources table is **in step** when the active digest is what the profile resolves to and the composition is reconciled. It is **pinned ≠ active** when the profile was re-pinned but the instance has not imported, **reconcile pending** when a composition was swapped in but not yet projected into the authz tables (Import finishes it), and **fallback: last_good / bundled** with the error when a fetch failed and the instance booted on a cached or baked tree instead. **channel** is information, not a warning: the profile follows a tag the kit's CI moves on every release, so a release is one Import away and the active digest on the row is what is actually served.

## Provenance

| kind | the process is serving |
|---|---|
| `bundled` | the baked `services/` tree; no sources are pinned |
| `fetched` | the composition of base + every pinned bundle, fetched and verified at boot |
| `last_good` | the previous composition, because a fetch failed (`on_fetch_failure: use_last_good`) — the error is shown |
| `bundled_fallback` | the baked tree, because a fetch failed (`on_fetch_failure: use_bundled`) — the error is shown |

## Import: in place, no restart

**Import sources** (`POST /api/v1/admin/services/refresh`) re-fetches every source, verifies it and recomposes. If the composition changed, core then — in the same request, in the running process — loads the new tree, projects its entitlements into the authz tables (`last_reconciled_hash` catches up to `composed_hash`) and refreshes the skill inventory. Nothing restarts: every route that hands marketplaces, plugins and skills to a client reads the services tree through the `current` link the recompose just swapped, so a marketplace-only kit is live when the call returns. The reply carries `changed`, `reconciled` and `restart_recommended`; the last is true only for a kit that ships governance hooks, which the process reads once at boot — the one case an in-place import cannot fully serve. `?restart=true` remains an explicit opt-in for that case.

## The three directions

Each plane card offers the same three actions, confirmed in words that name what will move:

| action | writes | leaves alone |
|---|---|---|
| **Insert only** | rows the declaration carries and the database lacks | everything that exists, even if it differs |
| **Overwrite from code** | inserts the missing, corrects what differs, **deletes** rows on declared entities the declaration does not carry | entities the declaration does not mention; for access control, every per-person override |
| **Export** | nothing — renders the database as the declaration file to commit | — |

Every apply runs in one transaction that re-reads the tables first, writes an activity row (who, plane, mode, counts), and updates `sync_state` with the declared hash it applied. The card then reads *declared `ab12…` · last applied `ab12…` by Ed 2 minutes ago (overwrite)*; if the file changes afterwards the card says *declaration changed since last apply*. The boot seed — the one write nobody presses — is recorded the same way with actor `boot` and mode `seed`.

## Kits: content from another repository

A **kit** is a GitHub repository in Anthropic marketplace format — `.claude-plugin/marketplace.json`, `plugins/<id>/` — plus two small systemprompt sidecars. Its CI publishes a signed **services bundle** on every release; the instance composes it with `services/` at boot. The Europe development workspace is a kit; the India workspace is being seeded as one from this repository with `just kit-export`.

**Kits own content; this repository owns access.** A kit ships no `access:` block. Who reaches `marketplace/astound-europe-dev` is declared in `services/access-control/rules.yaml` here with `owner: bundle:sfnext`, which lets the entity be declared before the digest is pinned: until the source is active the access-control card lists it as **awaiting its bundle** rather than failing boot. An access block a kit carries anyway is ignored as a declaration, flagged on its Sources row, and the rows it wrote at reconcile (`source = bundle:<name>`) appear as *bundle* orphans that an Overwrite deletes.

### End to end

1. **Kit CI** (`deploy/kit/.github/workflows/publish-bundle.yml`): import the authored tree → validate it composed under the **base** (the release's base-bundle asset, or `services/` packed from this repository's deploy branch when the release has none) → pack and sign with the kit's Ed25519 seed → publish to `ghcr.io/<org>/<kit>:<tag>` **and** move the kit's channel tag (`stable`) to the same digest → ask the instance to import.
2. **Pin**, once: `just services-pin <kit> stable` makes the profile follow the channel (image, public key and pull mode from `deploy/kit/known-kits.json`); `just services-pin <kit> sha256:<digest>` pins one upload instead. The instance never rewrites its own profile.
3. **Import**: the CI's call, or **Import sources** on `/admin/sync`. The reply says `changed`, `reconciled` and `restart_recommended`.
4. **Green**: the kit's row reads *in step* with the new active digest; Marketplaces lists the kit's marketplace; the access-control card shows the `owner:` entity as governed; Analysis → Versions minted a generation carrying the new hash. The process never restarted.

The first live kit is `astound-ba` (`systempromptio/astound-ba-skills`, public): two dummy plugins that also prove Claude Code's plugin Node install (`package.json` + lockfile) and `plugin.json` dependencies, including a cross-marketplace one on Salesforce's `b2c-developer-tooling`.

### Rollback

`just services-pin <kit> <previous digest>` and Import (or re-tag the channel in the kit). The cache keeps the last two content-addressed trees per source, so a rollback is a re-point, not a download.

### What fails boot

An id — marketplace, plugin or skill — present both in `services/` and in a bundle: composition names both sources and refuses. A bundle whose signature does not verify against the pinned public key, or whose digest does not match the pin. A second source carrying `config/`, `gateway/` or `access-control/`: only the first (base) source may. Everything else — a fetch that fails, a kit declaring access, an entity awaiting its bundle — is reported, not fatal.

## From a hash to a generation

A source hash answers *what is declared*. An **inventory generation** answers
*what the instance observed*, and the two are now joined.

Every reconcile pass records the composed hash, this repository's own tree hash
and each pinned bundle's digest, version and content hash beside the
observation. A pass only mints a new generation when the observed entry set
actually moved; an unchanged pass keeps its number and refreshes the timestamp.
That makes a generation a durable handle rather than a per-minute counter, so a
publication, a campaign and a skill revision can all be pinned to one.

**Versions** (Analysis → Versions) is where that is read. Its *Marketplace
sources* cards name each source with the hash the instance is composing today,
the generation that first carried it and when it last changed. Its *Generation
timeline* lists every generation with its composed hash, how many entries were
added, removed or changed against the previous one, and how many publications
that generation triggered. Any two neighbouring generations can be compared:
the skills that entered or left, the revision digests before and after, and the
source hashes that explain both.

Each bundle row on this page links straight through: **Versions** beside a
kit's content hash opens the timeline filtered to the generations carrying that
hash. A campaign records the publication generation and composed hash it was
created against, and is marked **stale** once the served generation moves past
it — the measurement is still evidence, but it no longer describes what users
receive.

## Where things are

| what | where |
|---|---|
| the page | `/admin/sync` (Platform → Sync); reads for the console tier, actions for administrators |
| the API | `/api/public/admin/sync/status`, `…/planes/{id}/drift`, `…/planes/{id}/apply`, `…/planes/{id}/export`, `…/sources/refresh` |
| the planes | `extensions/web/admin/src/repositories/sync/` — one `impl SyncPlane` and one registry line per plane |
| the state | table `sync_state`, one row per plane |
| the kit template and registry | `deploy/kit/` — runbook, publish workflow, sidecars, `known-kits.json` |
| the exporter | `just kit-export <marketplace> <dir>` — writes a kit and proves it re-imports identically |
| the pin | `just services-pin <kit> <digest \| channel> [profile]` |
