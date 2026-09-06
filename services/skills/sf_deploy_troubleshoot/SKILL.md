# SF Deploy Troubleshoot

Diagnoses a Salesforce deploy that hangs, rolls back, fails validation, or lands without the expected result, and returns the root cause with an ordered `sf` CLI command sequence to recover, including the rollback path. It works from the actual CLI output and the org's Deployment Status, not from guesses, and it records the corrected commands in the story's `deployment.md` so the next deploy of that story does not repeat the incident.

## Use when

- `sf project deploy start` sits at `Pending` or redraws progress past its `--wait`.
- The deploy reports "whole deploy rolled back", an unknown user permission, a permission dependency, a missing FlexiPage, or a component that deployed but is not visible.
- The user asks which test level to use, whether to use a manifest or `--metadata`, or how to roll back.

## Inputs

| Input | Required | Notes |
|---|---|---|
| Exact command that was run | yes | With every flag. |
| CLI output or Deployment Status text | yes | The error message, component failures, and failing test names. |
| Target org alias and type | yes | Sandbox, UAT, PreProd, or Production. Production changes the recovery path. |
| Story ID | no | Enables the `deployment.md` update. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "deployment troubleshooting <first line of the error>", "top_k": 5, "source_types": ["project_docs"]}` and `{"query": "<STORY-ID> deploy failed <component>", "top_k": 5, "source_types": ["project_tickets"]}`. Cite hits that match a known incident. If unavailable, say so and continue with the symptom table below.
2. **Classify the symptom** against this table and name the row in the diagnosis:

| Symptom | Root cause | Recovery |
|---|---|---|
| Hangs or loops | `--metadata Type1,Type2` joined argument; the org queue; a `RunSpecifiedTests` rollback the CLI keeps waiting on | switch to `--manifest`; `--wait 15` or higher; pick the right test level |
| Whole deploy rolled back | one failing test under `RunSpecifiedTests` or `RunLocalTests` rolls back everything, including non-Apex | read the test result in Setup → Deployment Status; sandbox: `NoTestRun` then run the touched class separately; unrelated failing test: fix the `TestDataFactory` default |
| "Unknown user permission" / "requires X" | permission dependency chain not normalised (`ManageUsers` needs `DelegatedTwoFactor` + `MonitorLoginHistory`; Modify All implies Delete + View All); some permissions are not deployable | add the implied permissions; grant the non-deployable ones in Setup and remove them from source |
| App missing from App Launcher | profile `applicationVisibilities` `visible=false`, or a partial profile clobbered the org | `sf project retrieve start --metadata Profile:<Name>`, set `visible=true`, redeploy the full profile |
| Utility bar app fails | `CustomApplication` deployed without its `UtilityBar` FlexiPage | deploy the FlexiPage in the same manifest |
| Action not on the action bar | FlexiPage `actionInstance` missing, or layout `platformActionList` lacks `Object.Action` | add to every relevant layout and the FlexiPage |
| "Cannot specify a default on: Opportunity" | `<default>` on a stage in BusinessProcess | remove the `<default>` element |
| Field in source, missing in org / picklist value ignored | standard value set vs field metadata; CSV scientific notation (`4.00E+01`) | update the right value set first; normalise the source value |
| Flow inactive after deploy | pipeline deploys flows as Draft | activate as a post-deploy step |

3. **Choose the test level** and state why: sandbox validation of non-Apex or when an unrelated test fails → `NoTestRun` with the reason written in `deployment.md`; a small Apex change → `RunSpecifiedTests --tests <Class>`; triggers, sharing, auth, integrations, or any UAT / PreProd / Production promotion → `RunLocalTests`.
4. **Choose manifest over `--metadata`** whenever more than one type is involved. Generate `manifest/<STORY-ID>-package.xml` listing every dependent component, dependencies first.
5. **Write the ordered command sequence.** Retrieve before edit, validate, deploy, verify, and the rollback:
   ```bash
   sf project retrieve start --metadata Profile:Admin --target-org <alias>
   sf project deploy validate --manifest manifest/<STORY-ID>-package.xml --target-org <alias> --test-level RunLocalTests --wait 60
   sf project deploy start --manifest manifest/<STORY-ID>-package.xml --target-org <alias> --test-level NoTestRun --ignore-conflicts --wait 15
   sf apex run test --class-names <TouchedTestClass> --code-coverage --result-format human --target-org <alias> --wait 10
   sf project deploy report --job-id <id> --target-org <alias>
   ```
   Rollback: redeploy the previous commit of the same manifest (`git stash` is not a rollback); for destructive changes use `--pre-destructive-changes` only inside the approved destructive workflow; for Production, quick-deploy the last validated package and open an incident note.
6. **Verify** with the observable that matches the symptom: Deployment Status shows Succeeded, the app appears in the App Launcher for the affected profile, the flow is Active, the test run reports ≥ 90% on touched classes.
7. **Record.** Update `stories/<STORY-ID>/deployment.md` with the corrected commands, the test-level reason, and the post-deploy steps; add the incident to the design note if the cause was a design gap (missing dependency, hand-written profile).

## Result

A diagnosis naming the symptom row and root cause, a numbered command sequence the user can run top to bottom, a rollback command, and the verification observable. Measure: after the sequence, `sf project deploy report` shows `Succeeded` and the verification observable holds; `deployment.md` matches the commands actually run.

## Guardrails

- Never run a deploy, validate, or destructive command yourself; produce the sequence and wait for the human to run it or say YES.
- Production and release branches use `RunLocalTests` or stricter. Never propose `NoTestRun` outside a sandbox.
- No Production deploy without an approved release tag, environment approval, and a written rollback.
- Never hand-write a profile; retrieve it first.
- Never delete metadata or use destructive changes outside the approved destructive-changes workflow.
- If the CLI output is not provided, ask for it; do not diagnose from the symptom name alone.
