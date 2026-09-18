> **Pinned by channel.** The profiles follow the kit's `stable` tag
> (`just services-pin sfnext stable`), so every release the kit's CI publishes
> is one **Import** on `/admin/sync` away. `deploy/kit/known-kits.json` lists
> the kit as `pinned` once the first release is live.
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
Operator runbook (keys, tokens, releases, rollback): `README-INTEGRATION.md` in the kit
(a copy of `deploy/kit/README-INTEGRATION.md`).

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
        reference: ghcr.io/astound-digital/sfnext-claude-dev-kit:stable
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["<kit public key>"]
  cache_dir: /app/services-cache
  on_fetch_failure: use_last_good
```

`ghcr_pull_token` lives in `secrets.json` as `<github-user>:<read:packages PAT>`
and is pushed as environment by `cloud deploy` like every other secret. The
local profile follows the same channel.

## Bumping

The profile follows the `stable` channel, so a kit release needs no edit here:
press **Import sources** on `/admin/sync` (or
`POST /api/v1/admin/services/refresh`). To hold one upload instead:

```bash
just services-pin sfnext sha256:<digest printed by the kit's publish run>
```

Rollback is the same command with the previous digest. `GET
/api/v1/admin/services/status` shows the active digest and provenance per
source; `use_last_good` keeps serving the previous composition when a fetch
fails and reports the failure there. The package is private: the instance
pulls with the `ghcr_pull_token` secret (`<github-user>:<read:packages PAT>`).
