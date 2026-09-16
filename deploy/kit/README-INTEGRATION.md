# Kit integration runbook

A **kit** is a GitHub repository that owns one marketplace in Anthropic
marketplace format — `.claude-plugin/marketplace.json`, `plugins/<id>/` with
`.claude-plugin/plugin.json` and `skills/<kebab-id>/SKILL.md` — plus two small
systemprompt sidecars. Its CI publishes a signed services bundle to GHCR on
every release; the instance composes that bundle with this repository's
`services/` tree at boot and shows it on `/admin/sync`. No redeploy of the
instance is needed to ship a kit.

Every kit is seeded from this directory so all kits have the same shape by
construction. The two kits are listed in `known-kits.json`.

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
just kit-export astound-india-dev ../sfnext-claude-india-kit
cp -r deploy/kit/.github ../sfnext-claude-india-kit/
cp deploy/kit/README-INTEGRATION.md ../sfnext-claude-india-kit/
```

The exporter writes the marketplace, its plugins and their skills in kit
shape (no access block), then re-imports the result with core's strict
importer and diffs it against `services/` — it fails if the round trip loses
anything. Commit the kit, then remove the exported ids from `services/`
**after** the first bundle is pinned (see below); until then the ids are
served from `services/` and `known-kits.json` lists the kit as `planned`.

## Keys and secrets

| where | secret | value |
|---|---|---|
| kit repository | `BUNDLE_SIGNING_SEED` | `systemprompt core services keygen` — the base64 seed. Keep the printed public key. |
| kit repository | `GHCR_PUSH_TOKEN` | a PAT with `write:packages` on `Astound-Digital` |
| kit repository (optional) | `SYSTEMPROMPT_ADMIN_TOKEN`, `SYSTEMPROMPT_API_URL` | an instance admin PAT and the gateway URL, to ask the instance to refresh after publishing |
| this repository | `deploy/kit/known-kits.json` `public_key` | the public half of the kit's seed |
| instance profile secrets | `ghcr_pull_token` | `<github-user>:<read:packages PAT>` |
| this repository (Actions) | `ASTOUND_BUNDLE_SIGNING_SEED` | signs the **base** bundle `release.yml` publishes as `ghcr.io/astound-digital/systemprompt-astound-base:<version>` |

## Release flow

1. Merge into the kit; `git tag vX.Y.Z && git push --tags`; create the GitHub release.
2. `publish-bundle.yml` imports → validates against the base bundle of the
   pinned Astound release → packs and signs → publishes to
   `ghcr.io/astound-digital/<kit>:vX.Y.Z` → prints the **digest**.
3. In this repository: `just services-pin <kit> sha256:<digest>` rewrites the
   profile's `services.sources[]` entry (inserting the block if absent).
   Production pins by digest only; the local profile may follow a tag.
4. `just deploy`, or on a running instance press **Refresh sources** on
   `/admin/sync` (or `POST /api/v1/admin/services/refresh?restart=true`).
   The process restarts on the new composition and entitlement reconciles at
   that boot.
5. `/admin/sync` shows the kit's row as **in step** (pinned = active, composed
   = reconciled), the catalog lists the marketplace, and the access-control
   plane shows the `owner: bundle:<kit>` entity as governed instead of
   *awaiting its bundle*.

## Rollback

`just services-pin <kit> <previous digest>` and refresh. The bundle cache keeps
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
| `composed_hash` | SHA-256 over ordered `name\0content_hash` of every source | *which composition* — restart pending when ≠ `last_reconciled_hash` |
| plane `declared_hash` | SHA-256 over the canonical declaration (rules.yaml → keys, access, why) | what the database was last made equal to, recorded in `sync_state` |
