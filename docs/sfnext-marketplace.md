> **Not pinned in this release.** The profiles ship with no `services.sources`
> block, so no remote bundle is fetched at boot. Everything below still works —
> publish a bundle and `just services-pin sfnext <digest>` inserts the block.
> The instance-wide picture — sources, planes, hashes, the refresh button —
> is `/admin/sync`; model and runbook: `/documentation/services-sync` and
> `deploy/kit/README-INTEGRATION.md`.

# The Europe development marketplace is a fetched bundle

`astound-europe-dev` (Astound Digital — Europe Development), its plugins
`astound-europe-storefront` and `astound-europe-delivery`, and their ten
skills are not defined in `services/`. They are authored in
[Astound-Digital/sfnext-claude-dev-kit](https://github.com/Astound-Digital/sfnext-claude-dev-kit)
in Claude Code marketplace format, published by that repository's CI as a
signed services bundle on GHCR, and fetched by this instance at boot.

Design: [Three Artifacts, One Instance](https://claude.ai/code/artifact/3e53a7ee-d5b7-4934-a8a0-ecaa4498e173).
Operator runbook (keys, tokens, releases, rollback): `INTEGRATION.txt` in the kit.

## What stays here

Everything the bundle references by id: MCP servers (`services/mcp/`),
groups and roles (`services/access-control/`, `services/web/config/groups.yaml`),
AI providers, gateway routes — **and the marketplace's access**. Who reaches
`astound-europe-dev` is declared in `services/access-control/rules.yaml` here
as `marketplace/astound-europe-dev` with `owner: bundle:sfnext`; the kit
carries no `access:` block (one it carries anyway is ignored as a declaration
and flagged on `/admin/sync`). Until the bundle is pinned the entity shows as
*awaiting its bundle*; once active it is governed like every internal one.

Composition is by id. If a marketplace, plugin or skill id appears both in
`services/` and in the bundle the boot fails naming both sources, so never
re-add those ids here. `tests/integration/admin-config` and
the bundle manifest (`_bundled`) records the split.

## Profile

`.systemprompt/profiles/production/profile.yaml`:

```yaml
services:
  sources:
    - name: sfnext
      oci:
        reference: ghcr.io/astound-digital/sfnext-claude-dev-kit@sha256:<digest>
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["<kit public key>"]
  cache_dir: /app/services-cache
  on_fetch_failure: use_bundled
```

`ghcr_pull_token` lives in `secrets.json` as `<github-user>:<read:packages PAT>`
and is pushed as environment by `cloud deploy` like every other secret. The
local profile follows a tag instead of a digest.

## Bumping

```bash
just services-pin sfnext <sha256 digest printed by the kit's publish run>
just deploy        # or Refresh sources on /admin/sync (POST /api/v1/admin/services/refresh?restart=true)
```

Rollback is the same command with the previous digest. `GET
/api/v1/admin/services/status` shows the active digest and provenance per
source; `use_bundled` means a failed fetch boots without the Europe workspace
and reports the failure there, which is the intended first-rollout behaviour
on a stateless machine. Switch to `use_last_good` once a fetch has succeeded.
