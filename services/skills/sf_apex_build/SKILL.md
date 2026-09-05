# SF Apex Build

Scaffolds the Apex for one Salesforce Core story: one trigger per object, a handler with discrete before/after methods, any service class, and a test class that reaches at least 90% meaningful coverage including a bulk case. It ends with the exact `sf apex run test` command and the coverage figure the story is expected to report, so the result is checkable in the sandbox rather than argued in review.

## Use when

- `sf_story_intake` routed the story to `Apex_Architect`, or the user asks for a trigger, handler, batch, queueable, schedulable, invocable, or callout.
- Declarative options were evaluated and rejected in the story's `design-note.md`. If that evaluation is missing, run `sf_declarative_build` first.

## Inputs

| Input | Required | Notes |
|---|---|---|
| `stories/<STORY-ID>/design-note.md` | yes | Source of object and field API names and the declarative evaluation. |
| Object API name(s) | yes | Exactly as in the project schema. |
| Existing trigger and handler for the object | yes | Read `force-app/main/default/triggers/` and `classes/` first; there is one trigger per object. |
| `TestDataFactory` (or equivalent) | yes | If absent, create it; every test fixture goes through it. |
| Sandbox alias | no | Placeholder `<sandbox-alias>` if unknown. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "<Object> trigger handler existing automation order of execution", "top_k": 5, "source_types": ["project_docs"]}` and `{"query": "<Object> active validation rules required fields", "top_k": 5, "source_types": ["project_docs"]}`; then `{"query": "<STORY-ID> related tickets <Object>", "top_k": 5, "source_types": ["project_tickets"]}`. Cite hits in the handler's class doc comment and in `design-note.md`. If unavailable, say so and read `docs/PROJECT_SCHEMA.md` and the existing test classes for validation-rule defaults.
2. **Confirm the trigger inventory.** `ls force-app/main/default/triggers/ | grep -i <Object>`. If a trigger exists, extend its handler; never add a second trigger for the object.
3. **Design the handler.** Write the method table before code: `beforeInsert`, `beforeUpdate`, `afterInsert`, `afterUpdate` as needed; for each, the collections it builds, the single SOQL and single DML outside loops, and the recursion guard (`@TestVisible private static Boolean bypassForRecursion = false;`).
4. **Choose the bypass.** Default to a custom permission checked with `FeatureManagement.checkPermission('<Object>_<Action>_Bypass')` and a matching permission set; a profile-name check is allowed only for a hard System Administrator rule and must be documented.
5. **Write the code** with these non-negotiables: `public with sharing class`; `Security.stripInaccessible(AccessType.UPDATABLE, records)` or `WITH SECURITY_ENFORCED` before DML and on queries; no SOQL or DML in loops; collect IDs in loops, query once with a `Map<Id, SObject>`; no hardcoded IDs, usernames, or profile names; static Boolean flags initialised to `false`; `@InvocableMethod` request shape documented if a Flow will call it.
6. **Write the test class** `<Handler>Test`: `@TestSetup` through `TestDataFactory` satisfying every active validation rule found in step 1; positive, negative, and bulk (`201` records) methods; assertions on derived state (`Assert.areEqual(expected, actual, message)`), never `System.assert(true)`; `HttpCalloutMock` or the Stub API for any callout; a method that exercises the bypass permission via `System.runAs`.
7. **Self-check against the sixteen rules** by running `sf_validate_rules` over the new files. Fix every `error` before continuing.
8. **Predict coverage.** Count lines per method and state the expected percentage per class. The bar is 90% on every touched class, not the org average.
9. **Update the story folder** in the same change: `design-note.md` (components, bulkification note, security approach, bypass permission, `Last updated`), `deployment.md` (deploy order: fields → custom permission → permission set → classes → trigger), `qa-test-request.md` (the `AT-nn` table mapping each test method to a scenario, plus the run command). Update `docs/PROJECT_SCHEMA.md` if any field was added.
10. **Emit the command** for the user to run after deploy:
    ```bash
    sf apex run test --class-names <Handler>Test --code-coverage --result-format human --target-org <sandbox-alias> --wait 10
    ```
    State the expected outcome: all methods pass, each touched class ≥ 90%.

## Result

Files under `force-app/main/default/`: `triggers/<Object>Trigger.trigger` (+ `-meta.xml`), `classes/<Object>TriggerHandler.cls`, any service class, `classes/<Object>TriggerHandlerTest.cls`, the custom permission and permission set when a bypass exists. The story folder is updated. The final message contains the run command and a per-class expected coverage table. Measure: the command exits 0 and the coverage column for every new class reads ≥ 90.

## Guardrails

- One trigger per object, no logic in the trigger body. Extending an existing handler is the norm; a second trigger is a defect.
- No Apex without its test class in the same change; no coverage claim without a bulk method.
- `with sharing` unless the design note documents why `without sharing` is required and who approved it.
- Never hardcode IDs, sandbox aliases, employee or customer names, or profile names other than `System Administrator` inside the sanctioned bypass.
- Never deploy, commit, or open a PR. Present the diff; a write needs an explicit human YES in the current turn.
- If the knowledge bank or the schema doc cannot confirm a field, stop and ask; do not scaffold against a guessed API name.
- A story whose design note shows no declarative evaluation is sent back to `sf_declarative_build` first.
