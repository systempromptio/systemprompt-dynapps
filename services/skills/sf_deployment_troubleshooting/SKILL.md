---
name: sf-deployment-troubleshooting
description: Use when implementing or reviewing Deployment troubleshooting in Salesforce.
---

## Systemprompt execution contract

The marketplace provisions authenticated `atlassian`, `salesforce`, and `github`
MCP connectors for the signed-in user. Use their advertised tools and input
schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

# Skill: Deployment troubleshooting

Lessons from real deploy incidents. Use when a deploy fails, hangs, or "looks stuck."

## Symptoms and root causes

### Deploy "hangs" or loops

- **Wrong CLI usage.** `--metadata FlexiPage,Layout,Profile` as one joined argument is invalid in modern `sf` and the CLI may never validate components correctly. **Use a manifest** (`--manifest manifest/<story>-package.xml`) or one `--metadata Type:Member` per component.
- **Org queue.** Deploys stay `Pending` until Salesforce processes them; the CLI redraws progress until `--wait` minutes elapse. Use `--wait 15` (or higher) and accept the wait.
- **`RunSpecifiedTests` rollback.** If any specified test fails, Salesforce rolls back the whole deploy and nothing lands in the org. The CLI may keep waiting. Pick the right test level for the change (see below).

### "Whole deploy rolled back" with no clear reason

- A single failing test in `RunSpecifiedTests` (or `RunLocalTests`) rolls back the whole package — even non-Apex changes. Check the test result in Setup -> Deployment Status.
- If the failing test is **unrelated to your change** (e.g. an existing test fails because the org now requires a custom field your story did not touch), use `--test-level NoTestRun` for sandbox validation deploys and run the relevant test class separately afterwards. Document this in the story's `deployment.md`.

### "Unknown user permission" or "permission requires <other>"

- Salesforce enforces permission dependency chains (e.g. `ManageUsers` requires `DelegatedTwoFactor` + `MonitorLoginHistory`; `Modify All` implies `Delete` + `View All`). Add the implied permissions to the permission set / profile. See `.cursor/skills/Profile_PermissionSet_Patterns.md`.
- Some permissions (e.g. `AllowForecasting`, `ExportReports`, `ManageMfaInUserInterface`) are not accepted as `<userPermissions>` in some orgs. Grant them in Setup or via a feature license; do not add them to the bundle.

### "App exists but does not appear in the App Launcher"

- The profile's `<applicationVisibilities>` for the app has `visible=false`, or the profile file in source is partial and clobbered the org default. **Retrieve the full profile, fix `visible=true`, redeploy.** See `.cursor/skills/Profile_PermissionSet_Patterns.md`.

### "Action not visible in the highlights panel" / "Action not on the action bar"

- The FlexiPage `actionInstance` is missing or filtered out -> add / fix the visibility rule.
- The page **layout's** `platformActionList` does not include the qualified `Object.Action` -> add it to every relevant layout.

### "Cannot specify a default on: Opportunity" (and similar)

- Some objects' BusinessProcess metadata does not allow `<default>` on individual stage `<values>` (Opportunity is one). Omit the `<default>` element on each stage; the default is set elsewhere.

### "Field is in source but missing in the org" / picklist value not honored

- Picklist value sets are governed by the **standard value set** (for standard picklists) or by the field metadata (for custom picklists). Update the right place; deploy the value set first if you are renaming or replacing.
- If the org uses Excel/scientific notation in a CSV source (e.g. `4.00E+01`), normalize to the literal value (`04E1`) before generating metadata. Source CSV typos commonly become deployment failures.

## Recipes

### Sandbox deploy that should ignore unrelated test failures

```bash
sf project deploy start \
  --manifest manifest/<story-id>-package.xml \
  --target-org <sandbox-alias> \
  --test-level NoTestRun \
  --ignore-conflicts \
  --wait 15
```

Then run the affected test classes separately:

```bash
sf apex run test \
  --class-names <TouchedTestClass> \
  --result-format human \
  --code-coverage \
  --target-org <sandbox-alias> \
  --wait 10
```

### Production-style deploy

```bash
sf project deploy validate \
  --manifest manifest/<release>-package.xml \
  --target-org <prod-or-staging-alias> \
  --test-level RunLocalTests \
  --wait 60

# After validation succeeds, quick-deploy in your release pipeline.
```

### Retrieve before edit (avoid clobber)

```bash
sf project retrieve start --metadata Profile:Admin --target-org <alias>
sf project retrieve start --metadata FlexiPage:<Name> --target-org <alias>
```

### Diff a single file deploy

```bash
sf project deploy start \
  --source-dir force-app/main/default/objects/<Object>/fields/<Field>.field-meta.xml \
  --target-org <alias>
```

## Pre-deploy checklist (when in doubt)

- [ ] Manifest is up to date and includes all dependent metadata.
- [ ] Dependent objects / fields / record types deploy **before** the components that reference them.
- [ ] Profiles were retrieved (not hand-written) before edit.
- [ ] Permission set dependencies normalized.
- [ ] Test class data satisfies all active validation rules in the target org.
- [ ] Story's `deployment.md` reflects the actual command(s) used.
