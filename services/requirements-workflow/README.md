# Requirements workflow demo

The Europe storefront marketplace includes four requirements skills and the
Atlassian, GitHub, Salesforce, Knowledge Bank and requirements workflow MCP
servers. Connected-account permissions continue to control source access.

The fictional Linden & Rowe migration dataset is labelled `[DEMO]` and
`demo-spa-migration`. Its 20 Confluence source pages live below
https://astounddigital.atlassian.net/wiki/spaces/EAICOMM/pages/1374322821.
Its 60 Jira issues are EAICOMM-35 through EAICOMM-94. Charles Tyrwhitt is a
public visual reference, not the fictional client's identity or a source of
asserted private implementation facts.

## Runtime

Build using `just requirements-build`. The web approval routes also require
the main server to be rebuilt using the repository's coordinated `just build`.
Apply configuration through the normal services loader. The workflow MCP
listens on port 5031 and installs its additive schema in the instance database.
Knowledge Bank installs its source-export schema in its configured database.
The standalone workflow binary registers the same group/project dimensions as
the web gateway, so Europe membership is enforced at both boundaries.

For a non-container installation, install both dependency sets:

```sh
npm ci --prefix services/requirements-workflow --ignore-scripts
npm ci --prefix services/requirements-workflow/upstream/.project/eval --ignore-scripts
```

Provision a dedicated random 32-byte base64 secret named
`requirements_workflow_signing_seed` in each deployment's existing secret store.
Preserve existing keys. The browser process reads this key to sign approvals;
the MCP tools expose no signing or approval operation. Retain old public keys
in `requirements_workflow_keys` when rotating the seed.

The evaluator requires `bedrock_api_key` (or supported AWS environment
credentials) and uses the pinned upstream generator/judge pair. The upstream
configuration uses us-east-1 for these synthetic fixtures. This demo source
selector must be replaced with an engagement-specific manifest and appropriate
model routing before using real client data. A missing evaluator credential or
failed projection is an error, never a successful validation.

`destination.json` is the demo Confluence publication parent. Runtime tokens,
seed resume state and working bundles belong in the ignored
`storage/requirements-workflow/` directory and are excluded from container builds.

## Workflow

1. Run `sync_knowledge` for Jira and Confluence with `rebuild: true` on the first
   run after this upgrade. Wait for `knowledge_sync_status` to report completion.
2. `start_requirements_workflow` freezes the caller-readable 60-issue/20-page
   programme. Supply `delivery_scope: functional`; integration work must be split.
3. Submit and validate discovery, design, proposal and grooming in that order.
   Every mutation uses the current `expected_revision`.
   Validation returns a persisted job; poll `get_requirements_workflow` for its
   status and the saved artifact verdict. Worker expiry or a stale revision
   never unlocks the next phase. Repeated requests reuse an active job.
4. Request the grooming review. The workflow owner signs in to
   `/admin/requirements/<workflow-id>`, examines the evidence, documents and
   validation results, and approves or requests changes in the browser.
5. After approval, submit and validate specs, generate the FSD scaffold, preserve
   its generated story/acceptance sections, and submit and validate the FSDs.
6. Request a separate publication review. After that browser approval,
   `publish_requirements` writes discovery and FSD documents to Confluence and
   reads them back. `verify_requirements_publication` detects subsequent edits.

The review screen can download the complete JSON evidence/artifact bundle.
Approvals bind the exact evidence, rubric commit, artifacts and validation
reports. Editing an upstream artifact invalidates its downstream artifacts and
approvals. Immutable revision events retain prior content. Browser decisions use
a session-bound, revision-bound, expiring nonce and a same-origin POST.

Reviews are currently restricted to the workflow owner so a project role alone
cannot reveal another user's private source snapshot. Shared reviewer delegation
requires checking every source grant for the proposed reviewer.

Publication reserves an attempt before sending a create request. An ambiguous
network outcome requires reconciliation against the exact Confluence title;
it does not automatically send another create. Readback must have the same canonical Markdown rendering as the approved body;
Confluence escaping such as `\[DEMO\]` is allowed, while changed text, links,
formatting, tables or code are rejected. The exact fetched body hash is retained
for subsequent tamper checks. A conversion that changes the rendering stops
verification and must be investigated before claiming publication success.

## Reproducibility and checks

`provenance.json` pins the upstream source commit. Its files are vendored without
changing the contracts; the Astound adapter lives in
`scripts/requirements-workflow/validate.mjs`. The store-locator golden supplies
the upstream structural example, while the frozen demo sources are authoritative
for project facts. Upstream goldens are never treated as migration evidence.

```sh
just requirements-check
just requirements-test
# Set SYSTEMPROMPT_TEST_DATABASE_URL to a maintenance test database first.
just requirements-integration
bash scripts/validate-services.sh
python3 scripts/requirements-workflow/test_evaluator.py
```

Rebuild the fixture JSON with `scripts/requirements-workflow/build_fixtures.py`.
The seeder accepts a gateway URL and a token file, discovers the actual connected
Atlassian tool contracts, and resumes from its manifest. Keep that manifest
between runs. It refuses to overwrite records without the demo ownership marker.

Local checks do not establish deployment. The test environment is
`https://astound.systemprompt.io`; the separate live endpoint is
`https://sp-dev.astound.digital`. Verify each after deploying its actual hosting
target. An image registry push alone does not deploy the live endpoint.
For the existing Fly test profile, run
`python3 scripts/requirements-workflow/prepare_fly.py production` before the
normal deployment build; the generated profile image otherwise lacks Node and
the evaluator dependencies. This changes local build inputs only.
