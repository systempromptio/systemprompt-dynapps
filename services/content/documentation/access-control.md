---
title: "Access Control: Who Reaches What, and Why"
description: "How entitlement works on this instance: the five subject bands and their precedence, the one declarative file every rule lives in, the database that enforces it, and the sync loop that keeps code and console in step."
author: "Astound Digital"
slug: "access-control"
keywords: "access control, entitlement, rules.yaml, roles, groups, projects, marketplace, plugin, skill, mcp server, gateway route, precedence, deny overrides, sync, drift, export"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Read the Access control page and say, for any entity, who reaches it and which band decided"
  - "Predict the resolver's answer for a person from their role, group, project and linked accounts"
  - "Grant a group a tool, make an entity admin-only, or retire a rule — in the file or in the console"
  - "Tell what is enforced (the database) from what is declared (rules.yaml), and reconcile the two in either direction"
related_docs:
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Astound SSO setup"
    url: "/documentation/adfs-sso-setup"
  - title: "Code ↔ Instance: sources, planes and sync"
    url: "/documentation/services-sync"
---

# Access Control: Who Reaches What, and Why

**TL;DR:** Every workspace, plugin, skill, MCP server and model route a signed-in person can reach is decided by rules on that entity. The rules are **declared once**, in `services/access-control/rules.yaml`, each with a stated reason, and **enforced from the database**. A decision walks five bands from narrowest to widest — *person → project → group → linked account → role* — and the first band that names the person decides, with a deny beating an allow inside it. Code and database are compared on every boot and **never silently merged**: `/admin/sync` shows the differences and offers three ways to resolve them.

## What access control decides — and what it does not

Access control answers one question: *may this person reach this entity?* The entities are:

| Kind | Examples on this instance | What "reach" means |
|---|---|---|
| `marketplace` | `astound-commons`, `astound-india-dev` | The workspace appears in Claude Code / Cowork and its plugins can be installed |
| `plugin` | `astound-super-admin`, `astound-sales-uk` | The plugin is offered |
| `skill` | `admin_daily_brief`, `uk_sales_client_prep` | The skill is offered and may run |
| `mcp_server` | `atlassian`, `github`, `google_rag`, `salesforce`, `systemprompt` | The server is listed and its tools may be called |
| `gateway_route` | `claude-star-4203d1`, … | Requests may be routed to that model |

It does **not** decide whether a person can sign in (that is [authentication](/documentation/authentication) and [SSO](/documentation/adfs-sso-setup)), whether a tool call is allowed to proceed once the person is entitled (the [governance chain](/documentation/enterprise-tool-governance)), or what may be said (the [safety scanners](/documentation/enterprise-safety-guardrails)). Those run after entitlement, on every request.

## The model

### Subjects and bands

A rule names a **subject** at one **band**:

| Band | Precedence | Subject is… | Where a person gets it |
|---|---|---|---|
| person | 0 | one account | an override set on the user's Access tab |
| project | 140 | a project id | assigned on the Projects page or mapped from an AD group |
| group | 150 | a group id | mapped from the AD groups the directory asserts at sign-in |
| linked account | 170 | a downstream org id (`salesforce`) | recorded when the person links that org |
| role | 200 | `user`, `admin`, `platform_admin`, `project_manager`, `super_admin`, … | granted by SSO or on the Roles page |

Lower precedence is **narrower**. The ladder is walked from the top.

### How a decision is made

1. **The narrowest band in which the person matches any rule decides.** A band in which nothing names them is skipped entirely — a group rule for a group they are not in has no effect on them, good or bad.
2. **Inside that band a deny beats an allow.**
3. **An entity with any rule is closed to everyone its rules do not name**, unless it is marked `default: open`. The rules are the whole story for that entity.
4. **An entity with no rule at all inherits from its parent**: a skill from its plugin, a plugin from its marketplace. This is why most skills need no rule of their own — they follow the workspace that ships them.

Two consequences worth holding onto:

- A **role grant does not "reopen" a group-gated entity to everyone** — only to the roles it names. `atlassian` is allowed to `admin` *and* to the `india-devs` group; an admin in no group reaches it through the role band, a plain `user` in no group is refused by the closed default.
- A **wider band cannot rescue a narrower deny**. A person-band deny on one account beats every group and role allow that person holds.

### Worked examples from this instance

`mcp_server/atlassian` is closed and allows: role `platform_admin, admin, super_admin` · group `india-devs, europe-devs, uk` · project `storefront`.

| Person | Walk | Result |
|---|---|---|
| role `user`, group `india-devs` | person: nothing · project: nothing · **group: matches allow** | **allowed** (decided by group) |
| role `admin`, no group | person · project · group · linked: nothing · **role: matches allow** | **allowed** (decided by role) |
| role `user`, no group, not on storefront | no band names them | **refused** — the entity is closed |

`mcp_server/salesforce` is closed and allows: role `super_admin` · group `india-devs, europe-devs, uk` · project `storefront` · linked account `salesforce`.

| Person | Walk | Result |
|---|---|---|
| role `user`, group `uk`, **not** linked | group matches allow | **allowed** — the linked-account band only *adds* people; it does not gate the group grant |
| role `user`, no group, **linked** | linked account matches allow | **allowed** (decided by linked account) |
| role `admin`, no group, not linked | no band names them (`admin` is not in the role list) | **refused** |

The **audience grid** at the bottom of `/admin/access-control` runs exactly these walks, for every role, group and project, against every entity; the word in each cell is the band that decided. To check one real person, open their user page and choose **Access**: the same resolver, with their actual roles, groups, projects and links.

## The file: `services/access-control/rules.yaml`

Every rule on the instance is declared in this one file, entity by entity. Marketplaces are entities like any other — there is no second place where a workspace's audience is written.

```yaml
entities:
  - entity: mcp_server/atlassian        # <kind>/<id>
    default: closed                      # open | closed — what an unmatched person gets
    why: >-                              # REQUIRED; becomes the justification on every rule row
      Jira/Confluence for the delivery groups and the storefront project;
      administrators use their own connected Atlassian account.
    allow:                               # band → subjects
      role: [platform_admin, admin, super_admin]
      group: [india-devs, europe-devs, uk]
      project: [storefront]
    deny: {}                             # same shape

  - entity: gateway_route/*              # glob: route ids are generated, never written
    default: open
    why: Every model route is reachable by every signed-in role.
    allow:
      role: [user, admin, platform_admin, project_manager, super_admin]

  - entity: marketplace/astound-sales-uk
    default: closed
    why: UK business team client-intelligence workspace.
    allow:
      group:
        values: [uk]
        why: A band may carry its own reason when it differs from the entity's.
```

Rules of the file, all checked by `scripts/validate-services.sh` in CI and again when the server reads it:

- `why` is required and non-empty. A rule with no reason is a rule nobody can review.
- Every `entity` must exist: plugins, skills, agents and MCP servers in `services/`, marketplaces under `services/marketplaces/`.
- `gateway_route` and `hook` take only the glob `*`, expanded against the live catalog at boot. A hand-written route id names a route that cannot exist.
- Band keys are `role`, `group`, `project`, `salesforce`. Every `group:`/`project:` value must be declared in `services/web/config/groups.yaml`.
- A subject may not be both allowed and denied on the same band of one entity.
- **No `services/marketplaces/*/config.yaml` may carry an `access:` block.** The gate refuses it, so the second truth cannot return.

Per-person overrides (the `user` band) have no place in the file. They belong to one account, are set on that person's Access tab, and are never synced or exported.

## Database and console

The database — `access_control_rules` and `access_control_entities` — is what the resolver reads. It is also what the console edits:

- **A group's Access tab** (`/admin/groups/<id>` → Access) sets that group's band on any entity. Allow or deny asks for a **reason** and refuses to save without one.
- **A person's Access tab** (`/admin/users/<id>` → Access) sets a person-band override. The reason is optional there.
- Every console write is stamped `source = dashboard`, so the Sync page can tell a console decision from a stale row.

The Access control page shows this database, one row per entity: the bands as chips, the reason in full, and a **Code ↔ DB** column saying whether the file agrees. Expand a row for the individual rules and the resolver's outcome in a sentence.

## The sync loop

Code and database are two copies of the same intent. Either may be edited; neither is silently overwritten.

**On boot** the server reads `rules.yaml` and, if the database holds **no** band rules at all (a fresh install), seeds it from the file in one transaction. Otherwise it computes the differences, logs them (`access_control_drift` in the server log), and **writes nothing**. A deploy that adds a rule to the file therefore does not land that rule until an administrator chooses to.

**`/admin/sync`** lists every difference as a line — what the code declares, what the database holds, where the database row came from (`console` or `stale`), and which action would resolve it — and offers three directions:

| Action | Writes | Leaves alone |
|---|---|---|
| **Insert only** | Rules and entity defaults the file declares that the database lacks | Everything that already exists, even if it differs |
| **Overwrite from code** | Inserts the missing, updates rows whose access or reason differs, and **deletes** rows on entities the file names that the file does not declare | Entities the file does not mention; every person-band override |
| **Export from dashboard** | Nothing in the database. Renders it as a `rules.yaml` to copy or download | — |

Both writes run in one transaction that re-reads the tables first, are recorded in the activity log with your name and the counts, and are confirmed in words that name what will be deleted.

**To make the console the source and the code catch up:** edit in the console, press **Export**, replace `services/access-control/rules.yaml` with the result, commit, deploy. The next boot reports *in sync*. Lines beginning `# NOTE:` in the export name what could not be expressed — a band whose rows carry different reasons, or gateway routes whose rule sets differ from one another — and must be resolved by hand.

**To make the code the source and the database catch up:** edit the file, commit, deploy, open the Sync page, choose **Insert only** (additive) or **Overwrite from code** (authoritative).

A note on `stale`: rows written from the console before it stamped its writes carry `source = yaml` and appear as *stale* rather than *console*. Overwrite treats them as any other undeclared row on a declared entity. Re-saving a rule from the console re-stamps it.

The Sync page is instance-wide — access control is one **plane** of it, beside the **sources** every declaration comes from. Each plane records the declared hash it last applied, when, by whom and in which mode, so the card can say *declared `ab12…` · last applied `ab12…` by Ed (overwrite)* or *declaration changed since last apply*. How sources, planes and hashes fit together: [Code ↔ Instance](/documentation/services-sync).

## External kits

A marketplace can arrive as a signed **services bundle** published by another GitHub repository — a *kit* in Anthropic marketplace format — rather than from this repository's `services/`. The Europe development workspace is one. **Kits carry no access.** A kit's sidecar has no `access:` block, and one it carries anyway is ignored as a declaration and flagged on the Sync page. Who reaches a kit's marketplace is declared **here**, like every internal one, with one extra key naming the source it arrives from:

```yaml
  - entity: marketplace/astound-europe-dev
    owner: bundle:sfnext
    default: closed
    why: Europe development workspace, published by sfnext-claude-dev-kit.
    allow:
      group: [europe-devs]
```

`owner: bundle:<name>` lets the entity be declared before the bundle is pinned: until that source is active the Sync page lists it as **awaiting its bundle** rather than failing boot, and the CI gate accepts the id without a local `services/marketplaces/<id>`. Once the bundle is active the entity validates and is governed like any other. A kit release therefore can never widen who sees it — ownership of *content* is the kit's; ownership of *access* is this repository's.

## How-tos

**Grant a group a tool.** Either add the group to the entity's `allow.group` list in `rules.yaml` and sync, or open the group → Access, set the entity to *Allow*, write the reason, Save. Then **Export** if you want the file to carry it.

**Make an entity admin-only.** Declare it with `default: closed` and `allow: { role: [platform_admin, admin, super_admin] }`. Name `platform_admin` beside `admin` always — it is the strictly stronger role, and an entity open to `admin` but closed to `platform_admin` is a hole. Declaring any rule opts the entity out of the marketplace cascade, which is what makes the list restrictive rather than additive.

**Explain one person's access.** User page → Access. Each row says *allow*/*deny* and the band that decided. Compare with the audience grid to see what would change if they joined a group or gained a role.

**Retire a rule cleanly.** Remove it from `rules.yaml`, deploy, Sync → Overwrite. Insert only would leave the old row in place; the Sync page would keep listing it as *stale*.

**Add a subject band.** A new dimension (cost centre, clearance, …) is a provider registered in `extensions/web/admin/src/authz/` and a new key in `BandMap`; no core change.

## Glossary

- **Entity** — the thing reached: marketplace, plugin, skill, MCP server, gateway route.
- **Subject** — who a rule names: a person, project, group, linked org, or role.
- **Band** — the kind of subject, with a precedence number; lower is narrower.
- **Default open / closed** — what an entity gives to a person no rule names. Closed unless declared otherwise.
- **Cascade** — a ruleless entity's inheritance from plugin then marketplace.
- **Declared** — what `rules.yaml` says. **Enforced** — what the database holds.
- **Drift** — any difference between the two: missing in DB, only in DB, differs, default differs.
- **Source** — `yaml` for rows the file placed, `dashboard` for rows the console wrote.

## Caveat: remote services bundles

This instance ships its services tree in the repository. If a profile ever pins a remote services bundle (`services.sources`), core's own boot reconcile also projects that bundle's marketplaces into the same tables from their manifests. Keep `rules.yaml` authoritative: the Sync page will show any such rows as drift, and **Overwrite from code** restores the declared state.
