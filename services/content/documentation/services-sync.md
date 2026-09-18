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
updated_at: "2026-09-17"
after_reading_this:
  - "Read the Configuration page and say, for every kind of configuration, where it comes from and whether the database agrees with the code"
  - "Read Code sync and say, for every source, whether the instance is serving what the profile pins"
  - "Tell the four hashes apart and know which one a pin, an import and an apply each change"
  - "Ship a kit from its own GitHub repository to the instance: publish, pin, refresh, verify"
  - "Reconcile a plane in either direction from its own page's Sync tab and read back who applied what, when, from which declaration"
  - "Move the whole projected configuration as one zip: export it for a commit, or stage an import and preview every plane before applying"
related_docs:
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
---

# Code ↔ Instance: Sources, Planes and Sync

**TL;DR:** Everything this instance serves is *declared* somewhere in code and *enforced* from what the process loaded and what the database holds. Three places in the console show that: **Configuration** (`/admin/configuration`, the Platform home) lists every kind of configuration the instance loads with its source, hash and — where the database holds a projection — its state; **Code sync** (`/admin/sync`) is where declarations come from and the archive that moves them; and each projected plane has a **Sync tab on the page that owns it** (Access control, Groups, Gateway → Policies) with its drift and the three directions. A **source** is where declarations come from — `base` is this repository's `services/` tree, `bundle:<name>` is each external kit the profile pins by digest — and every source has a content hash. A **plane** is what a source's declarations project into the database — access control, groups and projects, gateway policies — and every plane records the declared hash it last applied, when, by whom and in which mode. Nothing writes on its own after the first seed: the page shows the difference and offers a direction.

## The model

| word | meaning | today |
|---|---|---|
| **source** | a tree of declarations with a content hash | `base` (this repository), `bundle:sfnext` (the Europe development kit), any further kit |
| **plane** | a projection of declarations into database tables, with drift and three directions | `access_control` (`rules.yaml` → `access_control_rules`, `access_control_entities`); `groups` (`web/config/groups.yaml` → `groups`, `projects`, `group_ad_mappings`, `project_ad_mappings`); `gateway_policies` (`gateway/policies.yaml` → `ai_gateway_policies`) |
| **ownership** | decided by id at composition | a bundle owns its marketplaces, plugins and skills; the base owns everything else, every entitlement included |
| **hash control** | every source reports what it declares; every plane records what it applied | the table below |

Planes are a contract, not a list: one `impl SyncPlane` and one registry line each, and the page grows a card. The three differ in what their loaders were built to do, and the drift table says so on every row:

| plane | insert only | overwrite from code | export | the other writer |
|---|---|---|---|---|
| `access_control` | missing rules and entity defaults | corrects what differs; deletes undeclared rules on declared entities | `rules.yaml` | the access-control console (rows stamped `dashboard`) |
| `groups` | missing groups, projects and AD mappings | upserts every set, reconciles `yaml`-sourced mappings. **Never deletes a group or project** — those own members, usage and rules, and are deleted from their own page. A console-written mapping is kept. | `groups.yaml`, every dashboard-created set and mapping included — never a member | the Groups and Projects pages |
| `gateway_policies` | missing policies | upserts every declared policy, deletes undeclared rows | `policies.yaml`, with calendar-month windows declared as `2678400` | `/admin/gateway/policies` — live within a minute, because core reads the table per request |
| `gateway_routes` | missing routes, appended after the rows already there | makes the table equal the file's `routes:` sequence, order included; deletes undeclared rows | the whole `ai/gateway.yaml` — the settings as the file has them, the `routes:` sequence from the table | `/admin/gateway` (rows stamped `dashboard`). **Core boots the dispatcher from the file and never reads the table**; every console write and every apply by a person regenerates the file's `routes:` sequence from the rows, so a change is dispatched at the next restart. The boot seed does not rewrite the file (it came from the file). |
| `governance` | missing policies | makes `governance_chain` and its settings row equal the file: switch, inherited mode, every entry, order | `governance/config.yaml`, rendered from the staged chain | none yet — the plane validates through core's own parser and stages. **Core builds the chain from the file at boot and never reads the table**; a staged change is enforced after export, commit and restart. |

**Two planes take effect after a restart.** `gateway_routes` and `governance` project a file core reads once at boot. Their cards carry a notice saying so, and an image built from a tree whose file moved on shows the move as drift on the next boot — the database is the console's ledger, the file is what the process runs, and the page is where the two are reconciled. A route the console wrote on the previous image is *In database, not in code* until it is applied or exported.

### One boot contract

Every plane meets the same rule at boot, in the order groups → access control → gateway policies (the rules name group and project ids, so those must exist first). If the plane's projection is **empty**, boot seeds it from code — *overwrite from code* run once, recorded in `sync_state` with actor `boot` and mode `seed`. Otherwise boot **compares and writes nothing**: the drift is logged and the card on this page shows it. A restart therefore never undoes a console edit, on any plane; reconciling code and database is an administrator's act here, in one of the three directions. A declaration that cannot be read — the file does not parse, or the services tree it validates against does not compose — fails the boot rather than starting a server on half a declaration.

**People are never declared in code.** Group and project membership, manually granted roles and per-person access overrides live only in the database and the directory; no plane declares them, no drift row names them and no export carries them. The role set itself is closed in code, and the directory → role map is sign-in configuration (`web/config/adfs.yaml`), not a projection.

`gateway_policies` is the one plane the console edits in place: a save at `/admin/gateway/policies` is enforced on the next request, and shows here as drift until it is exported and committed or overwritten from code. `/admin/governance/quotas` reads the same table's windows against `ai_quota_buckets` to show usage per subject against each ceiling.

## The four hashes

| hash | comes from | changes when | proves |
|---|---|---|---|
| bundle **digest** `sha256:…` | the registry, per pushed artifact | a kit publishes a release | *which upload* — this is what the profile pins |
| bundle **content hash** | `bundle.json`: SHA-256 over sorted `path\0sha256` | the kit's files change | *which tree* — the same tree pushed twice has one content hash |
| base **tree hash** | computed on the page over `services/` with the same algorithm | this repository deploys | equals the base bundle's `content_hash` published at this release |
| **composed hash** | SHA-256 over ordered `name\0content_hash` of every active source | any source changes | *which composition* the process is serving; **reconcile pending** when it differs from `last_reconciled_hash` |
| plane **declared hash** | SHA-256 over the canonical declaration (access control: every `(entity, band, subject, access, why)` plus defaults and awaiting entities; groups: every set and mapping; gateway policies: every policy's name, switch, priority and normalised spec; gateway routes: every route's fields in file order, the fallback pair included; governance: the chain switch and mode, then every policy's id, switch, mode and canonical-JSON entry in order) | the file's meaning changes (not its whitespace) | what the database was last made equal to; recorded per apply in `sync_state` |

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

## The configuration page

`/admin/configuration` is one row per kind of configuration under `services/` — the root aggregator, marketplaces, plugins, skills, MCP servers, gateway routes, providers, governance, evaluation, scheduler, the web and content files, and the three projected declarations. Each row says where the kind comes from (`base`, `bundle:<name>`, or both for a directory that holds the base's skills and a kit's), the content hash it declares, and its **mode**:

- **projected** — a sync plane owns it; the database is what is enforced. The row shows *in step*, *drift · n*, *never applied* or *unreadable*, who last applied which hash, and links to the owner page's Sync tab.
- **served from code** — the composed tree is read at boot or per request and the database never holds a copy. Its state is the hash; the row links to the catalog page that shows it. Marketplaces, plugins and skills are always this: a marketplace's content hash is its version, and version control lives in the repository.

The distinction is the honest one. Only the projected kinds have a database to disagree with; for everything else "what is running" is the tree, and the sources on Code sync say which tree.

## The three directions

Each projected plane's Sync tab — `/admin/access-control?tab=sync`, `/admin/groups?tab=sync`, `/admin/gateway/policies?tab=sync` — offers the same three actions, confirmed in words that name what will move:

| action | writes | leaves alone |
|---|---|---|
| **Insert only** | rows the declaration carries and the database lacks | everything that exists, even if it differs |
| **Overwrite from code** | inserts the missing, corrects what differs, **deletes** rows on declared entities the declaration does not carry | entities the declaration does not mention; for access control, every per-person override |
| **Export** | nothing — renders the database as the declaration file to commit | — |

Every apply runs in one transaction that re-reads the tables first, writes an activity row (who, plane, mode, counts), and updates `sync_state` with the declared hash it applied. The card then reads *declared `ab12…` · last applied `ab12…` by Ed 2 minutes ago (overwrite)*; if the file changes afterwards the card says *declaration changed since last apply*. The boot seed — the one write nobody presses — is recorded the same way with actor `boot` and mode `seed`.

## The archive: export and import as one zip

Code sync's **Export & import** tab moves every projected plane at once.

**Export** (`GET /api/public/admin/sync/export.zip`) renders each plane's database as its file, laid out as `services/<file>`, plus a `MANIFEST.yaml` naming the release, the tree and composition hashes, and per plane the declared and applied hashes at the moment of export. Extract at the repository root, review the diff, commit: the code is back in step with what the console decided.

**Import** (`POST /api/public/admin/sync/import`, the zip as the request body) **writes nothing on upload.** The archive is unpacked under strict rules — no absolute or parent paths, no symlinks, nothing outside `services/<a directory the instance loads>/` but the manifest, every size checked as bytes are read — and staged in memory for thirty minutes. The preview at `/admin/sync/import/{stage}` shows each plane the archive carries as the same component the owner page renders, with its drift computed from the uploaded file rather than the one on disk; entries the instance serves from code are listed as things to commit or to publish as a bundle; projected planes the archive did not carry are named. An apply from the preview is per plane and per direction, or every plane at once, and is recorded in `sync_state` with the uploaded declaration's hash — so the next page load, which reads the file on disk, honestly says *declaration changed since last apply* until that file is committed too.

## Kits: content from another repository

A **kit** is a GitHub repository in Anthropic marketplace format — `.claude-plugin/marketplace.json`, `plugins/<id>/` — plus two small systemprompt sidecars. Its CI publishes a signed **services bundle** on every release; the instance composes it with `services/` at boot. The Europe development workspace is a kit; the India workspace is being seeded as one from this repository with `just kit-export`.

**Kits own content; this repository owns access.** A kit ships no `access:` block. Who reaches `marketplace/astound-europe-dev` is declared in `services/access-control/rules.yaml` here with `owner: bundle:sfnext`, which lets the entity be declared before the digest is pinned: until the source is active the access-control card lists it as **awaiting its bundle** rather than failing boot. An access block a kit carries anyway is ignored as a declaration, flagged on its Sources row, and the rows it wrote at reconcile (`source = bundle:<name>`) appear as *bundle* orphans that an Overwrite deletes.

### End to end

1. **Kit CI** (`deploy/kit/.github/workflows/publish-bundle.yml`): import the authored tree → validate it composed under the **base** (the release's base-bundle asset, or `services/` packed from this repository's deploy branch when the release has none) → pack and sign with the kit's Ed25519 seed → publish to `ghcr.io/<org>/<kit>:<tag>` **and** move the kit's channel tag (`stable`) to the same digest → ask the instance to import.
2. **Pin**, once: `just services-pin <kit> stable` makes the profile follow the channel (image, public key and pull mode from `deploy/kit/known-kits.json`); `just services-pin <kit> sha256:<digest>` pins one upload instead. The instance never rewrites its own profile.
3. **Import**: the CI's call, or **Import sources** on `/admin/sync`. The reply says `changed`, `reconciled` and `restart_recommended`.
4. **Green**: the kit's row reads *in step* with the new active digest; Marketplaces lists the kit's marketplace; the access-control card shows the `owner:` entity as governed; Analysis → Versions minted a generation carrying the new hash. The process never restarted.

The live kits are `sfnext` (`Astound-Digital/sfnext-claude-dev-kit`, the Europe development marketplace) and `india` (`Astound-Digital/sfscore-claude-india-kit`, the India development marketplace), both private packages the instance pulls with `ghcr_pull_token`. A plugin may also carry `package.json` + lockfile (Claude Code runs `npm ci` on the client) and `plugin.json` dependencies, including cross-marketplace ones.

### Rollback

`just services-pin <kit> <previous digest>` and Import (or re-tag the channel in the kit). The cache keeps the last two content-addressed trees per source, so a rollback is a re-point, not a download.

### What fails boot

An id — marketplace, plugin or skill — present both in `services/` and in a bundle: composition names both sources and refuses. A bundle whose signature does not verify against the pinned public key, or whose digest does not match the pin. A second source carrying `config/`, `gateway/` or `access-control/`: only the first (base) source may. A declaration a plane cannot read: `rules.yaml`, `groups.yaml` or `policies.yaml` that does not parse, or a services tree that does not compose (a plugin including an MCP server no file declares, for instance — `scripts/validate-services.sh` catches this before commit). Everything else — a fetch that fails, a kit declaring access, an entity awaiting its bundle — is reported, not fatal.

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
| the pages | `/admin/configuration` (Platform home), `/admin/sync` (Code sync: sources, export & import), `/admin/sync/import/{stage}` (preview), and each owner page's `?tab=sync`; reads for the console tier, actions for administrators |
| the API | `/api/public/admin/sync/status`, `…/planes/{id}/drift`, `…/planes/{id}/apply`, `…/planes/{id}/export`, `…/sources/refresh`, `…/export.zip`, `…/import`, `…/import/{stage}/apply`, `DELETE …/import/{stage}` |
| the component | `partials/components/sync-plane.hbs` + `js/components/sp-sync-plane.js`, built by `handlers/ssr/sync_plane/` for any plane from disk or from uploaded text |
| the inventory | `repositories/sync/inventory.rs` — every kind the configuration page lists |
| the planes | `extensions/web/admin/src/repositories/sync/` — one `impl SyncPlane` and one registry line per plane |
| the state | table `sync_state`, one row per plane |
| the kit template and registry | `deploy/kit/` — runbook, publish workflow, sidecars, `known-kits.json` |
| the exporter | `just kit-export <marketplace> <dir>` — writes a kit and proves it re-imports identically |
| the pin | `just services-pin <kit> <digest \| channel> [profile]` |
