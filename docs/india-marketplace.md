# India development marketplace

The `india-devs` group receives `astound-india-dev`, containing
`astound-india-ba` (16 skills) and `astound-india-sfcore` (18 skills).
Both plugins inherit marketplace access for every role. Group exclusions keep
the baseline, Europe and Cowork marketplaces out of the India client while
preserving their grants for other groups. Work attribution remains `core`.

The inventory in `india-skills-inventory.json` records source commits, original
file checksums, destination IDs and adapted-file checksums. Source repositories
are sibling checkouts outside this repository. Reproduce the import with:

```bash
python3 scripts/india/import-skills.py --source-root /var/www/html
```

The importer replaces the previous India skill selection, explicitly enumerates
all imported skills, and preserves Salesforce project templates under the
project-context skill's references directory. Templates are not facts about the
user's Salesforce org. Review upstream changes before repeating an import.

BA platform persistence is adapted to `.astound/ba/` workspace files. MCP writes
use advertised schemas and retain the source skills' review requirements.
Notification and scheduling skills produce drafts and proposals only. Unsupported
provider capabilities and authentication failures are reported explicitly.

## Authentication and live acceptance

`credentials.txt` describes the application registrations, callbacks and account
authorization required locally and in production. Provider credentials belong in
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

For production use `https://astound.systemprompt.io` and provide the deployment's
audit database through `SYSTEMPROMPT_DATABASE_URL`. The PAT is the real India
user's Systemprompt credential, not a provider token. Output defaults to
`.build/india-e2e/`; restrict access to transcripts because reads may contain
provider data. A failed enrollment, missing grant, empty tool list, model failure
or absent audit evidence exits nonzero. No skip counts as a live pass.

Run `scripts/e2e-marketplaces.sh --hosts opencode` as well for other-audience
regressions. Its knowledge-bank model cells do not cover India's new connectors;
the India harness above is required separately.

## Deployment and later access changes

Use the repository's preflight and deployment gates. This checkout currently
uses published core 0.47.0 with inactive local patches: use `just deploy` for that
configuration. Only a patch-active checkout uses `just deploy-next` with a clean,
committed core source. Never vendor unrelated working-tree changes.

After deployment rerun the live harness and verify the deployed version and
source commits. Keep the previous image/config revision available for rollback.
Project-based marketplace access and narrower plugin roles are future changes;
add parent-access and cross-role tests before enabling those restrictions.
