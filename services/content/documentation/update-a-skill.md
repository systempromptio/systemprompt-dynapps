---
title: "Update a Skill and Ship It"
description: "The developer walkthrough: change a skill in a kit repository, prove it locally, open a pull request, merge, and watch it reach the agents that run it — with the gate that refuses a broken tree before it is merged."
author: "Astound Digital"
slug: "update-a-skill"
keywords: "skill, kit, SKILL.md, frontmatter, sanitize, publish, bundle, digest, stable channel, import, sync, marketplace, plugin, claude code, cowork"
kind: "guide"
public: true
tags: ["documentation", "development", "skills", "operations"]
published_at: "2026-09-22"
updated_at: "2026-09-22"
after_reading_this:
  - "Change a skill in a kit repository and prove it locally with the same binary CI uses"
  - "Read the sanitation gate's annotations and fix what they name"
  - "Ship a change by bumping one version and merging — no tag, no release by hand"
  - "Follow the change from the published digest to the instance and on to the client that loads it"
  - "Tell a kit skill from a base skill, and know which one needs a deploy"
related_docs:
  - title: "Code ↔ Instance: Sources, Planes and Sync"
    url: "/documentation/services-sync"
  - title: "Skills: Lifecycle & Managed Revisions"
    url: "/documentation/skills-lifecycle"
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
---

# Update a skill and ship it

**TL;DR:** Most skills live in a **kit** — a GitHub repository of their own, not in this
platform's repository. Edit `SKILL.md`, run the gate, bump `metadata.version`, open a
pull request, merge. CI publishes a signed bundle and moves the `stable` channel; one
**Import** on `/admin/sync` serves it with no restart and no redeploy. The whole
release is one version number and one merge.

## First: which kind of skill is this?

The two paths differ, and picking the wrong one wastes a day.

| | **Kit skill** | **Base skill** |
|---|---|---|
| lives in | its own repository — `plugins/<plugin>/skills/<skill>/SKILL.md` | this repository — `services/skills/<skill_id>/` |
| examples | every Salesforce Core and business-analysis skill (`sf-…`, `ba-…`), the Europe storefront and delivery skills | the skills of the marketplaces this repository declares — `astound-commons`, `astound-salesforce`, `astound-super-admin` |
| to ship a change | merge a pull request; CI publishes; press Import | deploy the instance |
| takes effect | in place, no restart | at the next deploy |
| who can ship it | anyone with write access to the kit | a platform maintainer |

To find out which you are holding: if the skill id starts with a workspace prefix
(`sf_`, `ba_`) it is a kit skill. Otherwise look for the directory under
`services/skills/` in this repository — if it is not there, it came from a kit. The
Sources table on `/admin/sync` names every kit the instance composes, and
`/admin/configuration` says, for every kind of configuration, which source declares
it.

Everything below is the kit path. The base path is one line: change
`services/skills/<id>/`, commit, and deploy — skills are *served from code*, so there
is no database to reconcile and no Import to press.

## The five minutes before you start

Clone the kit and get the same server binary CI uses, so what passes locally passes
in CI:

```bash
git clone https://github.com/Astound-Digital/<kit-repo>.git
cd <kit-repo>

# the release the kit validates against — the value of ASTOUND_RELEASE in
# .github/workflows/publish-bundle.yml
gh release download "$ASTOUND_RELEASE" --repo Astound-Digital/systemprompt-astound \
  --pattern 'systemprompt-astound-server-*-linux-amd64.tar.gz' --dir ./release
tar -xzf release/systemprompt-astound-server-*-linux-amd64.tar.gz -C release
export PATH="$PWD/$(ls -d release/systemprompt-astound-server-*-linux-amd64)/bin:$PATH"
```

## 1. Change the skill

A skill is one directory. `SKILL.md` is the instruction the model reads; everything
beside it is material the skill can cite.

```text
plugins/astound-india-sfcore/skills/sf-core-logic/
  SKILL.md
  references/        # optional, and only read from here
```

The frontmatter contract — the shape `just kit-export` writes, and the one the gate
measures against:

```yaml
---
name: sf-core-logic          # must equal the directory name
title: Core Logic (Domain Knowledge)
description: Use when implementing or reviewing Apex logic that depends on order of execution.
tags:
  - india
  - salesforce
category: development
display_category: Salesforce Core
---
```

Four rules that are easy to get wrong, each of which the gate will catch:

- **The directory is the id, not `name`.** `plugins/<p>/skills/sf-core-logic/` becomes
  the skill `sf_core_logic`. Renaming the directory renames the skill; changing `name`
  only changes how it is listed.
- **`description` is the only key the platform requires** — and it decides when the
  model reaches for the skill, so write it as *when to use this*, not as a title. Quote
  it if it contains a colon: `Entirely self-contained: no MCP servers required` is
  read as nested YAML and fails the import.
- **A skill may only cite files it carries.** `references/state-machine.md` must exist
  in *that* skill's directory. A path to a sibling skill
  (`../other-skill/references/x.md`) can never work: directories are renamed to
  `snake_case` when the bundle is composed. Share by copying, not by pointing.
- **Keep the plugin's id prefix.** Skill ids are claimed globally across every kit the
  instance composes, so a generic `devops` or `integration` collides with another kit
  and neither instance starts. Each kit declares its prefix in
  `tools/kit-sanitize.json`.

Adding a skill is adding a directory; nothing lists them. Removing one is deleting the
directory.

## 2. Prove it locally

Run the same three stages CI runs, in order. Each takes seconds.

```bash
python3 tools/sanitize-kit.py --strict      # the gate
systemprompt core marketplace import --from . --into ./services --strict
systemprompt core services validate --root ./services --base ./base.tar.gz --strict
```

The gate prints one line per problem, each naming the file, the line and the fix, and
a table in the CI job summary. It exists because the importer is deliberately
permissive — `description` is the only key it demands — so everything else fails
quietly at runtime instead of loudly at build time:

| what it catches | what would otherwise happen |
|---|---|
| frontmatter that does not parse, or has no `description` | the import fails with a byte offset instead of a reason |
| a skill directory with no `SKILL.md` | the directory is not a skill, so **nothing it holds reaches the bundle** — silently |
| a cited `references/` file the skill does not carry | the model is told to read a file that is not there |
| a path climbing out of the skill directory | it can never resolve once directories are renamed to `snake_case` |
| two skills covering one topic | the model has to choose between two answers |
| an id without the plugin's prefix | a boot-time collision with another kit |
| an `access:` block | a kit cannot widen who reaches it; entitlement is declared on the instance |
| a version that is already released | the merge would publish nothing |

Warnings are shape drift rather than blockers: a `name` that disagrees with its
directory, or missing `title`, `tags`, `category` and `display_category`. Those do not
break anything — they make the skill show up under its de-hyphenated id, grouped under
"General", sorted last.

## 3. Bump the version

**The version is the trigger.** In `.claude-plugin/marketplace.json`:

```json
"metadata": { "version": "3.3.0" }
```

Bump it in the same pull request as the change. If the plugin itself changed
materially, bump its `version` too, in both `marketplace.json` and
`plugins/<plugin>/.claude-plugin/plugin.json`.

Forget this and nothing breaks — CI validates, then stops and says
*"metadata.version 3.3.0 is already released, so this push publishes nothing"*. Bump
it and merge again.

## 4. Open the pull request

`validate` runs on the pull request: sanitize → import → compose against the platform's
own `services/` tree. A broken tree is refused here, before it is merged, with
annotations on the offending lines. This is the check to wait for.

Merge when it is green. On merge, `publish` runs and does the rest by itself:

1. packs the tree and signs it with the kit's Ed25519 seed
2. publishes `ghcr.io/<org>/<kit>:v<version>` and moves the `stable` channel to the
   same digest
3. creates the git tag and the GitHub release
4. prints the **digest** in the job summary
5. asks the instance to import, if the kit has been given an admin token

No tag to push, no release to write.

## 5. Serve it on the instance

If the kit has no admin token configured, one button is left. On `/admin/sync`, press
**Import sources**.

This is not a deploy. The running server re-fetches every source, verifies the
signature against the public key it pins for that kit, recomposes the services tree,
projects entitlement and refreshes the skill inventory — in the same request, in the
running process. Nothing restarts.

You do not have to be an administrator: **Import sources** sits in the participant
tier, so anyone who participates in the kit's marketplace can press it for their own
marketplaces.

Then read the row back. The kit is **in step** when the active digest is the one CI
printed. Two states that look like failures and are not:

- **channel** — information, not a warning. The profile follows a tag CI moves on
  every release, which is why one Import is enough.
- **pinned ≠ active** — the profile pins a *digest* rather than the channel, so it
  will not move until someone re-pins it: `just services-pin <kit> <digest>`.

## 6. Verify it, three ways

| question | where |
|---|---|
| Is the instance serving the new bundle? | `/admin/sync` — the kit's row reads *in step* with the new digest |
| Is the skill in the catalogue? | the Skills catalogue, and `systemprompt core skills list` |
| Did the composition actually move? | Analysis → [Versions](/documentation/analysis-versions) — a new generation carrying the new composed hash, with the skills that entered and left |

## 7. Reaching the agents

Publishing a skill does not entitle anyone to it. Two things decide whether a
particular person's client loads it:

- **Entitlement.** Who reaches a marketplace is declared in this repository, not in
  the kit — `services/access-control/rules.yaml`, as `marketplace/<id>` with
  `owner: bundle:<kit>` — and enforced from the database. A kit ships no `access:`
  block and can never widen its own audience. A new skill inside a marketplace someone
  already reaches needs no new grant; a new *marketplace* does. See
  [Access control](/documentation/access-control).
- **Client refresh.** The instance never pushes to a client. The bridge pulls, on a
  schedule; nothing a client already has changes until it does.

### The client pulls — it is not pushed to

The bridge fetches the signed manifest and writes the plugins to disk. It does that:

| trigger | when |
|---|---|
| the scheduled background sync installed with the bridge | **every 30 minutes** — launchd, systemd or Task Scheduler, depending on the platform |
| `astound-bridge sync` | immediately, on demand |
| **Sync now** in the bridge GUI or tray | immediately |
| signing in, first run, or enabling a host | automatically, once |

So an unattended machine is up to half an hour behind a release. That is the number
to quote when someone asks why they cannot see the skill yet.

The bridge compares each file's SHA-256 against what is already installed, so an
unchanged plugin re-syncs without downloading anything, and a changed `SKILL.md`
is fetched and swapped. Plugins the manifest no longer lists are **deleted** — if
someone loses reach to a marketplace, the plugin leaves their disk rather than
merely being hidden.

### A running session never reloads

The bridge writes files; the client reads them when it starts. A `claude` session
that is already open will not pick up a new skill, however many times you sync.

```bash
astound-bridge sync     # fetch the new manifest
# then exit Claude Code and start it again
claude plugin list      # the new skill's plugin, in a new session
```

Cowork is the same: sync, then restart Claude Desktop. Full client instructions are
in [Connect Claude Code](/documentation/connect-claude-code) and
[Connect Cowork](/documentation/connect-cowork).

### "It synced, and I still cannot see it"

Almost always one of these four:

1. **No entitlement.** The filter is *explicit allow* — an item nobody is granted is
   dropped, silently, per user. A new skill inside a marketplace the person already
   reaches is fine; a new **marketplace** needs a rule in
   `services/access-control/rules.yaml`. This is the most common cause by a distance.
2. **The session was not restarted.** See above.
3. **The profile pins a digest, not the channel.** Publishing moves `stable`; a
   digest pin follows nothing until someone re-pins and imports.
4. **The skill is in the marketplace but in no plugin's `skills.include`.** It is
   counted in the manifest and installed nowhere. The sync summary says so
   explicitly — read it rather than guessing.

If you have just changed an entitlement and want to test without waiting, the server
memoises a user's resolved catalogue for up to a minute; `astound-bridge sync --fresh`
bypasses it.

## If something is wrong

**Roll back** by pointing at the previous upload and importing again:

```bash
just services-pin <kit> <previous digest>
```

The cache keeps the last two trees per source, so a rollback is a re-point, not a
download. Re-tagging the channel in the kit works too.

**A failed fetch does not take the instance down.** `on_fetch_failure` in the profile
decides: `use_last_good` keeps serving the previous composition, `use_bundled` boots
without the kit, `fail_closed` refuses to boot. The Sources row shows the fallback and
the error.

**What does fail a boot:** an id present in both this repository and a kit
(composition names both sources and refuses), a signature that does not verify against
the pinned public key, a digest that does not match the pin, or a second source
carrying `config/`, `gateway/` or `access-control/`. The full list, and the model
underneath all of this, is in
[Code ↔ Instance](/documentation/services-sync).
