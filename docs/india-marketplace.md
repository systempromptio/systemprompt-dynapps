# India development marketplace is a fetched bundle

`astound-india-dev` (Astound Digital — India Development), its plugins
`astound-india-sfcore` (38 skills) and `astound-india-ba` (16 skills) and their
skills are not defined in `services/`. Since 2026-09-18 they are authored in
[Astound-Digital/sfscore-claude-india-kit](https://github.com/Astound-Digital/sfscore-claude-india-kit)
in Claude Code marketplace format, published by that repository's CI as a
signed services bundle on GHCR (private package), and composed by this instance
— the profiles follow the kit's `stable` channel, so a release is one
**Import** on `/admin/sync` away. Kit contract and runbook:
`deploy/kit/README-INTEGRATION.md`; another instance: `docs/kits-on-another-instance.md`; registry: `deploy/kit/known-kits.json`
(kit `india`); model: `/documentation/services-sync`; the developer walkthrough from an
edit to the agent: `/documentation/update-a-skill`.

Since v3.3.0 (2026-09-22) the kit publishes on **merge to `main`**, not on a hand-cut
release: `metadata.version` in `marketplace.json` names the tag, CI validates every
pull request first, and a merge that leaves the version alone publishes nothing. Every
sfcore skill carries the `sf-` prefix declared in the kit's `tools/kit-sanitize.json`.

## What stays here

Everything the bundle references by id: the MCP servers `atlassian` and
`salesforce` (`services/mcp/`), the `india-devs` group
(`services/web/config/groups.yaml`, directory group `Systemprompt-Core`) —
**and the marketplace's access**. `marketplace/astound-india-dev` is declared
in `services/access-control/rules.yaml` with `owner: bundle:india` and
`allow: group: [india-devs]`; the kit carries no `access:` block and its CI
refuses one. Composition is by id: never re-add the marketplace, plugin or
skill ids here — boot names both sources and fails.

An ordinary member with only the India group grant receives this development
marketplace. Additional grants can legitimately add content: the `storefront`
project grants Europe access and the `knowledge_worker` role grants Cowork
access. Admin access must be checked separately; an admin-only run cannot
prove ordinary developer isolation.

## Profile

```yaml
services:
  sources:
    - name: india
      oci:
        reference: ghcr.io/astound-digital/sfscore-claude-india-kit:stable
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["a8Xa0VL5BFVauW8NsUmVktmzj8G+9+qt6//UBZH4X0o="]
  cache_dir: /app/services-cache
  on_fetch_failure: use_last_good
```

Written by `just services-pin india stable [profile]`. `ghcr_pull_token` is
`<github-user>:<read:packages PAT>` in the profile's `secrets.json`. To hold
one upload: `just services-pin india sha256:<digest>`; rollback is the same
command with the previous digest, then Import.

## Upstream sources

The skills were imported from `mruia-astound/sf-dev-agent` (Salesforce Core)
and `nbohra-astound/mcp-ba-platform` (BA). The importer, the provenance
inventory and the acceptance harnesses moved with the content into the kit
repository under `tools/`; they still describe the pre-kit `services/` layout
and are re-pointed there before the next upstream sync (`tools/KIT-NOTE.md`).

## Authentication and live acceptance

[Connector configuration](integrations/centralized-mcp.md) describes application
registrations, callbacks and account authorization. Provider credentials belong in
Systemprompt's credential layer. OpenCode receives bridge configuration; skills
never handle provider tokens. Missing authorization fails a live check.

Run the repository unit and database integration suites for imported membership,
OAuth response validation, grant isolation and one-time callback state. Local
mock responses exercise the same OAuth HTTP code with fixture credentials.
These checks do not establish successful authorization against a real provider.

The real-account harness uses a fresh client home, pinned OpenCode 1.18.29,
installer enrollment, exact marketplace/plugin/skill/connector sets, and real
MCP tool discovery. Its final model turn must execute authenticated reads through
all three connectors, backed by correlated database audit rows:

```bash
python3 tools/e2e-opencode.py   # in the kit repository \
  --gateway http://localhost:8080 \
  --pat-file /secure/path/india-systemprompt-pat \
  --model <configured-gateway-model>
```

For the Astound live endpoint use `https://sp-dev.astound.digital` and provide that deployment's
audit database through `SYSTEMPROMPT_DATABASE_URL`. The PAT is the real India
user's Systemprompt credential, not a provider token. Output defaults to
`.build/india-e2e/`; restrict access to transcripts because reads may contain
provider data. A failed enrollment, missing grant, empty tool list, model failure
or absent audit evidence exits nonzero. No skip counts as a live pass.

## Deployment and later access changes

Use the repository's preflight and deployment gates and the branch-specific
instructions in `CLAUDE.md`. The `production` profile and `just deploy` target
the test environment at `astound.systemprompt.io`, not the separate live Astound
endpoint. Verify downloaded bridge bytes from the endpoint actually under test.

After deployment rerun the live harness and verify the deployed version and
source commits. Keep the previous image/config revision available for rollback.
Retest parent access, group/role combinations and project grants when changing
marketplace restrictions.

## Claude Code skill matrix

`tools/e2e-claude-code.py` (kit repository) prepares all 34 starting prompts and acceptance
rubrics and runs each scenario in a fresh Docker client against the live Astound
endpoint. It supports admin enrollment, Haiku cost/timing evidence, reviewed
Atlassian follow-ups, local Salesforce fixtures and explicitly labelled local
patched retests. Salesforce/GitHub live integrations are not substituted with
mocks or counted as passes. See `tools/README.md` in the kit repository.

Keep run transcripts and customer account diagnostics in private operational
storage. Report fixture checks and live connector acceptance separately.
