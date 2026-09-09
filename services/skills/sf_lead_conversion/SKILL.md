---
name: sf-lead-conversion
description: Use when implementing or reviewing Lead Conversion in Salesforce.
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

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

# Skill: Lead Conversion

Patterns for persisting Lead identity, mapping custom fields onto Account / Contact / Opportunity, and gating conversion eligibility.

## Pattern A — Persist the Lead Id on the converted Opportunity

Useful when an external system (data warehouse, BI) needs to join converted records back to the originating Lead.

### Why a formula field on Lead?

The Lead's standard `Id` cannot be used directly as a source in `LeadConvertSettings` mapping — the mapping requires a **custom field** as the source. A formula field solves this:

| Object | Field | Type | Definition |
|---|---|---|---|
| `Lead` | `Lead_ID_Custom__c` | Formula (Text) | `CASESAFEID(Id)` — always 18-character, case-safe, auto-computed |
| `Opportunity` (or Account / Contact) | `Lead_ID__c` | Text(18) | Receives the value during conversion; `null` for non-converted records |

### Why a Text field (not Lookup or Formula) on the destination?

1. It receives the string value from the formula during conversion.
2. It must remain `null` for records created without lead conversion (a Lookup or Formula would not satisfy this).
3. Downstream consumers use it as a string join key; no Salesforce relationship semantics are needed.

### `LeadConvertSettings` mapping

```xml
<objectMapping>
    <inputObject>Lead</inputObject>
    <outputObject>Opportunity</outputObject>
    <mappingFields>
        <inputField>Lead_ID_Custom__c</inputField>
        <outputField>Lead_ID__c</outputField>
    </mappingFields>
</objectMapping>
```

### FLS

- Source formula field on Lead: read-only by definition (formulas are not editable). Grant **Read** to the personas that need traceability.
- Destination text field on Opportunity / Account / Contact: **Read + Edit** to admin / integration users; **Hidden** from end users if it is purely an integration field.

### Page layouts

If the field is purely for integration, **exclude it from all page layouts**. Otherwise add it to the System Information section.

### Deployment order

1. Deploy the Lead formula field (`Lead_ID_Custom__c`).
2. Deploy the destination Text field (`Lead_ID__c`).
3. Deploy `LeadConvertSettings` (mapping depends on both fields existing).
4. Deploy profile / permission set FLS.
5. Deploy Apex tests.

### QA

- **AT-01:** Convert a Lead — assert the destination field equals `CASESAFEID(Lead.Id)` (18 chars).
- **AT-02:** Create the destination record without conversion — assert the field is `null`.
- **AT-03:** Query as a non-privileged user (`System.runAs`) — assert FLS denies read.

## Pattern B — Map a custom field on Lead to a custom field on the converted record

Same `LeadConvertSettings` mechanism. Both source (custom field on Lead) and destination (matching-type custom field on Account / Contact / Opportunity) must exist before deploying the mapping.

## Pattern C — Set the converted Opportunity's record type and close date from a Custom Metadata Type

For projects that convert leads into different Opportunity record types based on a key from the Lead (product, segment, region, etc.).

### Schema (suggested — adapt to your project)

`<Source_Key_To_Record_Type>__mdt`:

| Field | Type | Purpose |
|---|---|---|
| `Source_Key__c` | Text | Source value used to key the mapping (e.g. a `Product2.Type` value) |
| `Opportunity_Record_Type_Developer_Name__c` | Text (required) | Target Opportunity record type developer name |
| `Close_Date_Offset_Months__c` | Number (required) | Months to add to `Date.today()` for `CloseDate` |

### Apex (in `OpportunityTriggerHandler.beforeInsert`)

1. Load the CMT mappings into a `Map<String, <Cmt>__mdt>` keyed by source value.
2. For converted Opportunities (identified by a populated Lead-Id field, e.g. `Lead_ID__c`) with a populated source-key field (e.g. a `<Source_Key__c>` lookup):
   - Read the source key value (or look it up via the related record).
   - Look up the mapping; set `RecordTypeId` to the matching record type and `CloseDate = Date.today().addMonths(offset)`.
3. Bulk-safe: query related records, record types, and CMT once outside the loop.

## Pattern D — Conversion eligibility guard

Use a CMT-driven configurable rule engine (see `the installed sf-custom-metadata-driven-config skill`) so admins can change the rules without a deploy:

- Rule developer name: e.g. `Lead_Conversion`.
- Conditions reference `Account.Status__c != Inactive`, `Contact.Status__c != Inactive`, etc.
- Apex `LeadConversionEvaluator` exposes `@AuraEnabled` (for an LWC banner on the Lead record page) and `@InvocableMethod` (for a Flow).
- A `LeadTrigger` -> `LeadConversionGuard` runs in `before update`: when `IsConverted` flips to `true` and the rule evaluates `false`, `addError` to block conversion with a user-friendly message.
- The LWC banner on the Lead record page calls the same evaluator so the user sees the warning before clicking **Convert**.

## Pattern E — Re-run assignment rules after in-line edits

If your project assigns Leads on insert via assignment rules and you also want re-assignment after certain in-line edits:

- See `the installed sf-assignment-rule-reexecution skill`.
