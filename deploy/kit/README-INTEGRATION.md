# Kit integration runbook

A **kit** is a GitHub repository that owns one marketplace in Anthropic
marketplace format — `.claude-plugin/marketplace.json`, `plugins/<id>/` with
`.claude-plugin/plugin.json` and `skills/<kebab-id>/SKILL.md` — plus two small
systemprompt sidecars. Its CI publishes a signed services bundle to GHCR on
every release; the instance composes that bundle with this repository's
`services/` tree at boot and shows it on `/admin/sync`. No redeploy — and no restart — of the instance is needed to
ship a kit: **Import** recomposes, reconciles entitlement and refreshes the
inventory in the running process.

Every kit is seeded from this directory so all kits have the same shape by
construction. The kits are listed in `known-kits.json`; this instance ships
with none, so the list is empty until Dynapps publishes its first kit. A plugin
may additionally carry `package.json` + lockfile (Claude Code runs `npm ci` on
the client), declared `scripts`, and `plugin.json` `dependencies`, including
cross-marketplace ones (`allowCrossMarketplaceDependenciesOn` in
marketplace.json plus `external_marketplaces` in the marketplace sidecar).

## What a kit may and may not contain

| may | may not |
|---|---|
| `.claude-plugin/marketplace.json` (every plugin entry needs a `category`) | an `access:` block anywhere — **who reaches a kit is declared in this repository**, `services/access-control/rules.yaml`, as `marketplace/<id>` with `owner: bundle:<kit>` |
| `.claude-plugin/systemprompt.yaml` (`schema: 1`, title, visibility, `mcp_servers` by id) | MCP server definitions — servers are defined on the instance (`services/mcp/`) and referenced by id |
| `plugins/<id>/…` skills, rules, hooks, references, scripts | a marketplace, plugin or skill id that also exists in `services/` — composition fails boot naming both |
| `plugins/<id>/.claude-plugin/systemprompt.yaml` (category, `mcp_servers`) | `config/`, `gateway/`, `access-control/` directories — only the base source carries those |

A kit release therefore can never widen who sees it. Ownership is decided
by id at composition: the kit owns its marketplace, plugins and skills; this
repository owns everything else, every entitlement included.

## Seeding a kit from `services/`

```bash
just kit-export dynapps-<team>-dev ../<team>-claude-kit
cp -r deploy/kit/.github ../<team>-claude-kit/
cp -r deploy/kit/tools ../<team>-claude-kit/
cp deploy/kit/README-INTEGRATION.md ../<team>-claude-kit/
```

The exporter writes the marketplace, its plugins and their skills in kit
shape (no access block) plus a starter `.systempromptignore`, then re-imports the result with core's strict
importer and diffs it against `services/` — it fails if the round trip loses
anything. Commit the kit, then remove the exported ids from `services/`
**after** the first bundle is pinned (see below); until then the ids are
served from `services/` and `known-kits.json` lists the kit as `planned`.

## The sanitation gate

`tools/sanitize-kit.py` runs before core's importer, so a bad tree is reported as the
file and line that causes it rather than a byte offset, and every finding lands as a
GitHub annotation on the pull request. It is copied from `deploy/kit/tools/` like the
workflow — edited here, never in a kit.

It exists because core's importer is deliberately permissive: `description` is the
only frontmatter key it requires, so everything else degrades quietly. A skill with no
`title` is listed as its de-hyphenated id; one with no `display_category` groups under
"General" and sorts last on the public skills page; a `references/` path the skill does
not carry simply never reaches the bundle. The gate makes each of those visible.

Errors block the release: frontmatter that does not parse or carries no `description`,
a skill directory with no `SKILL.md` (core does not import it, so nothing it holds
ships), a cited `references/` file the skill does not carry, a path climbing out of the
skill directory (composition renames skill directories to snake_case, so a relative
path to a sibling never resolves), an `access:` block, a plugin with no category, two
skills covering one topic, an id that does not carry the prefix
`tools/kit-sanitize.json` declares for that plugin — ids are claimed globally at
composition, so a generic `devops` collides with another kit at boot and neither
starts. Warnings name shape drift: a `name` that disagrees with the directory (the
directory is the real id), missing `title`/`tags`/`category`/`display_category`, and a
`metadata.version` that is already released.

The gate also applies core's dev-file filter, so a skill that cites a file the bundle
will not carry is an error here rather than a missing file on a client (see below).

```bash
python3 tools/sanitize-kit.py            # report
python3 tools/sanitize-kit.py --strict   # exit 1 on any error, as CI runs it
```

## Dev-only files

Nothing a skill's author keeps for development reaches a client. Core drops, inside
every skill: a `README.md` at the skill root (any case), any `tests/`, `test/`,
`fixtures/` or `__tests__/` directory, and any `*.test.*` or `*.spec.*` file. It does
so at import, at managed capture, and again when it lays a revision out for a client.

On top of those, `.systempromptignore` at the kit root names the kit's own dev-only
paths in gitignore syntax: `#` comments, `*`/`?`/`**`, a trailing `/` for a directory,
a pattern containing `/` anchored to the kit root, and `!` to re-include. The ignore
file is consulted before the defaults one directory level at a time, so `!fixtures/`
ships a skill's fixtures; nothing under an excluded directory comes back. A malformed
pattern fails the import. `just kit-export` writes a starter file:

```
*.snap
coverage/
node_modules/
__pycache__/
.DS_Store
```

Removing dev files changes what a plugin carries, so its bundle digest (and therefore
the stamped plugin version) moves once the first time a kit is imported by a core that
filters them.

## Keys and secrets

| where | secret | value |
|---|---|---|
| kit repository | `BUNDLE_SIGNING_SEED` | `systemprompt core services keygen` — the base64 seed. Keep the printed public key. |
| kit repository | `DYNAPPS_RELEASE_TOKEN` | fine-grained PAT, Contents: read on this repository — the kit CI downloads the server binary (and packs the base from `services/`) from here while it is private |
| kit repository (optional) | `SYSTEMPROMPT_ADMIN_TOKEN`, `SYSTEMPROMPT_API_URL` | an instance admin PAT and the gateway URL, so a release imports itself |
| kit repository (optional) | `DYNAPPS_STATS_PAT` (+ `SYSTEMPROMPT_API_URL`) | a PAT whose owner has console access; `tools/kit-stats.sh` writes the run record per kit version into each release's notes and attaches `kit-stats.json`, and `kit-stats.yml` refreshes the latest release daily. The PAT is accepted on `GET /admin/export/{dataset}` only |
| this repository | `deploy/kit/known-kits.json` `public_key`, `pull`, `channel` | the public half of the kit's seed; `public` when the GHCR package is public (no pull secret anywhere); the channel tag the CI moves |
| instance profile secrets | `ghcr_pull_token` | `<github-user>:<read:packages PAT>` — only for a kit whose package is private (`pull: private` in `known-kits.json`; `services-pin` then writes `auth_secret: ghcr_pull_token`) |
| this repository (Actions) | `DYNAPPS_BUNDLE_SIGNING_SEED` | signs the **base** bundle `release.yml` publishes as `ghcr.io/systempromptio/systemprompt-dynapps-base:<version>` |

The image is pushed with the workflow's own `GITHUB_TOKEN` to
`ghcr.io/<owner>/<repo>`; no packages PAT is needed. Make the package public
after the first push if the kit is public; a private kit requires every
instance that composes it to carry `ghcr_pull_token`. Another instance
follows `docs/kits-on-another-instance.md`.

## Release flow

**The version is the trigger.** `metadata.version` in `marketplace.json` names the
release; nothing else has to be done by hand.

1. Bump `metadata.version` in the PR that changes the kit, and merge it into `main`.
2. `publish-bundle.yml` runs two jobs. `validate` (every push **and every pull
   request**) sanitizes the tree → imports → validates against the base (the
   release asset, or `services/` packed from this repository's `next`).
   `publish` then reads `metadata.version`: if `vX.Y.Z` is already tagged it
   writes a notice and stops green, so a merge never republishes under a
   shipped version; otherwise it packs and signs → publishes
   `ghcr.io/<owner>/<kit>:vX.Y.Z` and moves `:stable` to the same digest →
   creates the tag and the GitHub release → prints the **digest** → asks the
   instance to import. A hand-cut release still publishes, for a tag made
   outside this flow.
3. In this repository, once: `just services-pin <kit> stable` makes the profile
   follow the channel (or `just services-pin <kit> sha256:<digest>` to pin one
   upload; a pinned profile answers the CI's import with `changed: false`
   until re-pinned).
4. If the CI has no admin token, press **Import sources** on `/admin/sync`
   (or `POST /api/v1/admin/services/refresh`). Core recomposes, projects
   entitlement and refreshes the inventory in place; nothing restarts.
5. `/admin/sync` shows the kit's row as **in step** with the new active
   digest, the catalog lists the marketplace, and the access-control plane
   shows the `owner: bundle:<kit>` entity as governed instead of *awaiting its
   bundle*.

## Rollback

`just services-pin <kit> <previous digest>` and Import. The bundle cache keeps
the last two content-addressed trees per source, so a rollback is a re-point,
not a download. `on_fetch_failure` in the profile decides what a failed fetch
does: `use_bundled` boots without the kit (first rollout), `use_last_good`
keeps serving the previous composition (steady state), `fail_closed` refuses
to boot.

## Hashes, and which one means what

| hash | where it comes from | what it proves |
|---|---|---|
| bundle `digest` (`sha256:…`) | the registry, per pushed artifact | *which upload* — this is what the profile pins |
| bundle `content_hash` | `bundle.json`, SHA-256 over sorted `path\0sha256` | *which tree* — the same tree pushed twice has one content hash |
| base `tree_hash` | `/admin/sync` computes it over `services/` with the same algorithm | equals the base bundle's `content_hash` at this release |
| `composed_hash` | SHA-256 over ordered `name\0content_hash` of every source | *which composition* — reconcile pending when ≠ `last_reconciled_hash` |
| plane `declared_hash` | SHA-256 over the canonical declaration (rules.yaml → keys, access, why) | what the database was last made equal to, recorded in `sync_state` |
