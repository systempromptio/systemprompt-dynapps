# India development marketplace

> **Becoming a kit.** The India marketplace is being moved into its own
> repository (`Astound-Digital/sfnext-claude-india-kit`, listed as `planned`
> in `deploy/kit/known-kits.json`) so Nishit's and Jay's teams edit it in
> Anthropic marketplace format and a release publishes a signed bundle.
> Seed it with `just kit-export astound-india-dev ../sfnext-claude-india-kit`;
> the runbook is `deploy/kit/README-INTEGRATION.md`. Access stays here:
> `marketplace/astound-india-dev` in `rules.yaml` gains `owner: bundle:india`
> when the ids leave `services/`. Model: `/documentation/services-sync`.

The `india-devs` group receives `astound-india-dev`, containing
`astound-india-ba` (16 skills) and `astound-india-sfcore` (18 skills).
Both plugins inherit marketplace access. An ordinary member with only the
India group grant receives this development marketplace. Additional grants can
legitimately add content: the `storefront` project grants Europe access and the
`knowledge_worker` role grants Cowork access. There is no India deny rule on
Europe or Cowork. Admin access must be checked separately; an admin-only run
cannot prove ordinary developer isolation. Work attribution remains `core`.

The inventory in [`skills-inventory.json`](../scripts/india/skills-inventory.json) records source commits, original
file checksums, destination IDs and adapted-file checksums. Source repositories
are sibling checkouts outside this repository. Reproduce the import with:

```bash
python3 scripts/india/import-skills.py --source-root /path/to/source/checkouts
```

For a BA-only refresh, add `--suite ba`; `--suite sf` selects Salesforce and the
default remains both suites. A selective import preserves the other suite's
files and provenance. Source skill directories must exist and have no
uncommitted changes. Plugin versions are authored in their descriptors and are
not reset by imports.

The September 11 BA refresh follows `nbohra-astound/mcp-ba-platform` branch
`feature/deduplicate-tools-unified-auth` at commit
`8e25a2e940bc62dd7b38f6ef3666ae073f2e8bba`. All 16 skills include the standalone
rules and 13 explicit review gates. Approval covers concrete proposals rather
than unavailable BA confirmation tokens; Jira relationship fields and content
formats follow advertised connector schemas. RTM updates require reviewed rows
and confirmed Jira success, with tracker failures reported separately.

The September 15 BA refresh advances the same branch to commit
`5efee126f6ffcc01393402ef5680a4deea1fe1c7`. Upstream replaced every Jira
wiki-markup section with Markdown-style formatting (`### **Heading**`,
`**Bold Text**`, `1.` lists) and extended story drafting with Related Stories,
Work Sequencing relationship types and the ADA final-step rule. The importer now
keeps that Markdown style in the Jira remote-write row (translated to the
advertised schema, ADF when required) and no longer labels the Markdown example
descriptions as wiki-only; only the Confluence wiki-table sections keep that
label. Plugin `astound-india-ba` is 2.2.0 and marketplace `astound-india-dev`
is 3.2.0.

The [astound-ba-skills repository](https://github.com/systempromptio/astound-ba-skills)
is now the **public kit-integration proof** (`deploy/kit/known-kits.json`, kit
`astound-ba`): two dummy plugins, nothing proprietary, published as a signed
services bundle on every release and imported on `/admin/sync` in place. It no
longer carries the BA skills; those remain here, delivered from `services/`,
until the India BA cutover kit (`known-kits.json` entry `india`, still
`planned`) is seeded with `just kit-export`. The dummy ids (`ba_kit_hello`,
`ba_kit_node_smoke`) never collide with the BA skill ids, so both can be served
at once.

The importer replaces the previous India skill selection, explicitly enumerates
all imported skills, and preserves Salesforce project templates under the
project-context skill's references directory. Templates are not facts about the
user's Salesforce org. Review upstream changes before repeating an import.

BA platform persistence is adapted to `.astound/ba/` workspace files. MCP writes
use advertised schemas and retain the source skills' review requirements.
Notification and scheduling skills produce drafts and proposals only. Unsupported
provider capabilities and authentication failures are reported explicitly.

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
python3 scripts/india/e2e-opencode.py \
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

Run `scripts/e2e-marketplaces.sh --hosts opencode` as well for other-audience
regressions. Its knowledge-bank model cells do not cover India's new connectors;
the India harness above is required separately.

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

`scripts/india/e2e-claude-code.py` prepares all 34 starting prompts and acceptance
rubrics and runs each scenario in a fresh Docker client against the live Astound
endpoint. It supports admin enrollment, Haiku cost/timing evidence, reviewed
Atlassian follow-ups, local Salesforce fixtures and explicitly labelled local
patched retests. Salesforce/GitHub live integrations are not substituted with
mocks or counted as passes. See [the runbook](../scripts/india/README.md).

Keep run transcripts and customer account diagnostics in private operational
storage. Report fixture checks and live connector acceptance separately.
