# SF Declarative Build

Designs the declarative solution for a Salesforce Core story before anyone writes Apex: a record-triggered or screen Flow planned element by element with a fault path on every failable element, validation rules as narrow single-purpose formulas, and a Custom Metadata Type where a business rule will change without a deploy. It produces the plan, the metadata XML stubs, and the deploy order, and it states in writing where declarative stops and Apex must start.

## Use when

- `sf_story_intake` routed the story to `Declarative_Lead`, or the story says "must be filled when", "must follow format", "cannot change after", "automation", "screen flow", "approval", "no-code", "mapping table".
- Any story that has not yet had its declarative-first evaluation recorded, even if Apex is expected to win.

## Inputs

| Input | Required | Notes |
|---|---|---|
| `stories/<STORY-ID>/design-note.md` | yes | Object and field API names, ACs. |
| Existing automation on the object | yes | Flows, triggers, process builders; the plan must not double-fire. |
| Environment-specific values | no | Queue names, record type names, thresholds; these become CMT records, never hardcoded IDs. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "<Object> existing flows automation record-triggered", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<Object> validation rules picklist values record types", "top_k": 5, "source_types": ["project_docs"]}`, and `{"query": "<STORY-ID> <feature> flow validation", "top_k": 5, "source_types": ["project_tickets"]}`. Cite hits in the design note. If unavailable, say so and read `force-app/main/default/flows/` and `objects/<Object>/validationRules/` directly.
2. **Decide the mechanism per AC** and record it in a table: validation rule (single-record invariant, format, required-when-stage), before-save flow (same-record field update, no DML), after-save flow (related records, email, invocable Apex), screen flow (guided user input), CMT (mapping or eligibility likely to change), Apex (cross-object bulk logic, async, callouts, field-by-field prior-value comparison). Apex rows must carry the reason declarative fails.
3. **Plan each Flow element by element.** For every flow: API name `Object_When_What` (e.g. `Opportunity_BeforeSave_Stage_Defaults`), trigger and entry conditions, then an ordered element list `Get / Decision / Assignment / Update / Create / Action / Screen` with inputs, outputs, and the fault path target for every Get, DML, Action, and Subflow. Screen flows: the first screen sets `allowBack` or `allowFinish` to true. Before-save flows never contain DML.
4. **Write each validation rule** with the patterns: required-when-stage `AND(ISPICKVAL(StageName, "X"), ISBLANK(Field__c))` (wrap picklists in `TEXT()`), at-least-one-of, if-A-then-B, email `REGEX(Email, ".+@.+\\..+")`, phone hardening, compound name (`FirstName` cannot be required via metadata). One purpose per rule, named for what it enforces, `errorDisplayField` on the offending field, an error message that says what to fix. Add one `MT-nn` manual case per rule to `qa-test-request.md`.
5. **Design the CMT** when a mapping or eligibility rule belongs to admins: `<Domain>_<What>__mdt` with key fields and `Result__c` (Pattern A), or a `Rule_Logic__mdt` / `Rule_Condition__mdt` pair with `Condition_Logic__c` such as `1 AND 2` and `Order__c`-indexed conditions (Pattern B). Never store secrets or PII in a CMT. Name records with stable `DeveloperName`s and document them in `docs/PROJECT_SCHEMA.md`.
6. **Emit the XML stubs** under `force-app/main/default/`: `flows/<Name>.flow-meta.xml` with `<status>Draft</status>` and the element skeleton, `objects/<Object>/validationRules/<Name>.validationRule-meta.xml` with `<active>true</active>`, `<errorConditionFormula>`, `<errorDisplayField>`, `<errorMessage>`, and `customMetadata/<Cmt>.<DeveloperName>.md-meta.xml` records. Reference no IDs; reference CMT records, custom labels, or named credentials.
7. **Fix the deploy order** and write it to `deployment.md`: fields and picklist values → record types → CMT object → CMT records → validation rules → Apex invocables the flow calls → flows (deployed inactive) → quick actions referencing screen flows → activation as a post-deploy step.
8. **Self-check** by running `sf_validate_rules` over the flow and validation rule files. Fix every `error`.
9. **Update the story folder**: `design-note.md` (mechanism table, fault handling, recursion notes, CMT schema, `Last updated`), `deployment.md`, `qa-test-request.md`. If Apex rows remain, hand off to `sf_apex_build` with the invocable request shape the flow expects.

## Result

A mechanism table in `design-note.md` with one row per AC, a flow plan with a fault path on every failable element, validation rule and CMT XML stubs in `force-app/`, and an ordered deploy list in `deployment.md`. Measure: every failable flow element in the plan has a `faultConnector` target, `grep -c "<validationRule\|<Flow" force-app/main/default -r` matches the stub count, and no stub contains a 15 or 18 character ID literal.

## Guardrails

- Declarative before Apex. An Apex row without a written reason is a defect in the plan.
- No hardcoded IDs, queue names, or environment values in flows or formulas; use CMT, custom labels, or named credentials.
- Every element that can fail has a fault path with a user-readable message; a flow without fault paths does not ship.
- Never activate a flow or deploy anything. Present the plan and stubs; a write needs an explicit human YES in the current turn.
- Do not rename an existing flow to "fix" it; version it.
- Block-edits-after-status across many fields is Apex (prior-value comparison), not a formula; say so and route it.
- Unknown field or picklist value means ask, never guess.
