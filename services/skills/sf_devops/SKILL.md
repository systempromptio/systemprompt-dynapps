---
name: sf-devops
description: Use when implementing or reviewing DevOps (Domain Knowledge) in Salesforce.
---

## Systemprompt execution contract

Workspace access installs these skills. The `atlassian`, `salesforce`, and
`github` connectors are delivered separately after the signed-in user connects
and verifies each provider account. An installed skill does not prove a connector
is ready. Local file tasks can run without provider connections. For remote
operations, use only the currently advertised tools and input schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

# Skill: DevOps (Domain Knowledge)

Use this when working with source, orgs, and CI/CD.

## Expertise areas

- **SFDX:** Project structure, `sfdx-project.json`, source format, scratch orgs.
- **Salesforce CLI (`sf`):** Deploy, retrieve, validate, test, org management.
- **GitHub Actions** (or equivalent): Workflows for validate, test, deploy, and governance (Jira / docs / audit).
- **Environments:** Dev, UAT, PreProd, Prod; JWT auth for CI; environment approvals.

## Key patterns

- Validate before deploy; run tests in CI with a defined test level (e.g. `RunLocalTests` for release).
- No secrets in repo; use GitHub Secrets / Environments or a vault.
- Tagged releases and approval gates for production deploys.
- Prefer a `manifest/<story-id>-package.xml` for stories that touch many metadata types — avoid the brittle `--metadata Type1,Type2` joined-argument syntax.
- For sandbox-only validation deploys where unrelated org rules cause `RunSpecifiedTests` rollbacks, use `--test-level NoTestRun` (and document why in the story's `deployment.md`).

## Useful CLI snippets

Deploy a single source path to an org alias:

```bash
sf project deploy start \
  --source-dir force-app/main/default/objects/<Object>/fields/<Field>.field-meta.xml \
  --target-org <alias>
```

Deploy via manifest:

```bash
sf project deploy start \
  --manifest manifest/<story-id>-package.xml \
  --target-org <alias> \
  --test-level NoTestRun \
  --ignore-conflicts \
  --wait 15
```

Run a specific test class:

```bash
sf apex run test \
  --class-names <TestClassName> \
  --result-format human \
  --code-coverage \
  --target-org <alias> \
  --wait 10
```

Retrieve an existing component before editing (so you do not clobber unrelated permissions, e.g. on a profile):

```bash
sf project retrieve start \
  --metadata Profile:Admin \
  --target-org <alias>
```


## Required deployment review checks

For trigger or sharing changes and production promotion, state RunLocalTests
explicitly. RunSpecifiedTests is not a substitute for that requirement in this
skill. Use the supplied API version and exact component names; mark unknown
metadata as proposed. Include a validation step, retained test/deploy evidence
and a rollback plan. Local fixture output is an unexecuted runbook; do not claim
validation or deployment has run. Save the actual package manifest alongside the
runbook and list the saved paths.
