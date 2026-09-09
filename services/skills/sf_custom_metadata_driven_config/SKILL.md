---
name: sf-custom-metadata-driven-config
description: Use when implementing or reviewing Custom Metadata-driven configuration
  in Salesforce.
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

# Skill: Custom Metadata-driven configuration

Use this when a business rule, mapping, or eligibility check is likely to change without a code deploy, or when the same engine should evaluate many similar conditions.

## When to use

- **Mapping tables:** source value -> target value, key-tuple -> result (e.g. `(Segment, Sub-Segment, Application Code) -> Application Id`).
- **Eligibility / condition rules:** an expression like `1 AND 2` referencing atomic conditions stored as records.
- **Per-record-type / per-environment defaults** that should not require a code change to update.
- Anything an admin should be able to tune in production without involving an engineer.

## Anti-patterns (when *not* to use)

- A handful of constants that almost never change -> use Apex constants or a custom setting (hierarchical) instead.
- Logic that needs millisecond-fast lookup at very high volume -> consider Custom Settings (cached) or Platform Cache.
- Anything containing secrets or PII -> never. Use Named Credentials / vault.

## Pattern A — Simple key -> value mapping

A single Custom Metadata Type (CMT) where each record carries the source key fields and the result.

| Field on `<Mapping>__mdt` | Type | Purpose |
|---|---|---|
| `<Key1>__c` | Text | First lookup key |
| `<Key2>__c` | Text | Second lookup key (if composite) |
| `Result__c` | Text / Number | The mapped value |

In a **before-save Flow** on the source object:

1. **Entry filter:** all key fields are populated.
2. **Get Records** on `<Mapping>__mdt` with equality on each key field.
3. **Decision:** if a record was found, set `$Record.<TargetField>` to `Get_Result.Result__c`.

In Apex, query the CMT once outside loops; cache results in a `Map<String, <Mapping>__mdt>` keyed by the composite key.

## Pattern B — Configurable rule engine ("logic + atomic conditions")

Useful when admins need to express "Account Status != Inactive AND Contact Status != Inactive" without changing code. Shape your CMTs as a parent "rule set" with one or more child "atomic conditions."

### Schema (suggested names — adapt to your project)

`<Rule_Logic>__mdt` (parent rule set):

| Field | Type | Purpose |
|---|---|---|
| `DeveloperName` | (built-in) | Stable key for callers (e.g. `Lead_Conversion`) |
| `Condition_Logic__c` | Text | Boolean expression referencing condition `Order__c`s, e.g. `1 AND 2`, `(1 AND 2) OR 3` |

`<Rule_Condition>__mdt` (atomic conditions):

| Field | Type | Purpose |
|---|---|---|
| `Logic_Link__c` | Metadata Relationship -> `<Rule_Logic>__mdt` | Parent rule set |
| `Order__c` | Number | The slot referenced by `Condition_Logic__c` |
| `Object_Name__c` | Text | API name of the object the condition reads (e.g. `Account`, `Contact`) |
| `Field_API__c` | Text | Field API name on that object (e.g. `Status__c`) |
| `Operator__c` | Picklist | `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `startsWith`, `endsWith` |
| `Field_Value__c` | Text | Expected value |

### Apex evaluator (sketch)

- `@AuraEnabled` for LWC, `@InvocableMethod` for Flow.
- Public method `evaluate(String logicDeveloperName, Map<String, SObject> context)`:
  1. Load logic + conditions for the given `logicDeveloperName`.
  2. For each condition, read the field from the relevant context record and apply `Operator__c` against `Field_Value__c` -> Boolean per `Order__c`.
  3. Substitute the booleans into `Condition_Logic__c` and evaluate (a small expression parser, or a constrained set of supported expressions).
  4. Return the final boolean (and optional reasons for false conditions).

### Trigger guard

Pair the evaluator with a `BeforeUpdate` guard (e.g. `LeadConversionGuard`) that calls `evaluate` and `addError` if the result is `false`. Surface the same evaluator to a UI banner LWC so the user sees the warning before they submit.

## Conventions

- CMT API names: `<Domain>_<What>__mdt` (e.g. `Country_Region_Mapping__mdt`).
- Records named with a stable, human-readable `DeveloperName` — sequential (`MAP_0001`, `MAP_0002`, ...) for high-volume mappings or business-meaningful names (`Account_Status_Must_Not_Be_Inactive`) for rule conditions.
- Always document the CMT and its records in `docs/PROJECT_SCHEMA.md`.
- For mapping tables loaded from a CSV / spreadsheet, keep the source file under version control (e.g. `data/<mapping>.csv`) and a regeneration script under `scripts/` listed in `.forceignore` so the CLI does not treat scripts as metadata.

## Deployment

- Deploy the CMT object first, then the records.
- Records are deployed as `<Cmt>.<DeveloperName>.md-meta.xml` files.
- If admins manage the records in production, document in the runbook that the **records** in source are seed values; environments diverge after admin edits.


## Required evaluator review checks

State the supported comparison operators and reject unsupported or malformed
configuration explicitly. Sort rules by ascending priority and a stable unique
name; retain the first matching rule. Do not overwrite the winning rule with a
later entry when populating a map. Make the no-match result explicit without
assuming a queue or custom object exists in the target org.

Save proposed tests covering a match, no match, equal-priority tie, and invalid
configuration. Review the code against those examples before returning it. Label
metadata names as proposed when they have not been confirmed in the project.
