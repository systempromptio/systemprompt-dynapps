---
name: sf-dynamic-forms-field-visibility
description: Use when implementing or reviewing Dynamic Forms and Field Visibility
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

# Skill: Dynamic Forms and Field Visibility

How to surface fields where you want them seen and lock them down where they should not be edited, using FlexiPages, page layouts, and profile FLS together.

## When to use which lever

| Goal | Use |
|---|---|
| Hide a field from a specific persona | Profile / Permission Set FLS (`readable=false`) |
| Show a field but make it read-only on the record page | Dynamic Forms field on FlexiPage with `uiBehavior=readonly`, plus FLS `editable=false` if it must be read-only everywhere |
| Remove the **Change Record Type** action | `<excludeButtons>ChangeRecordType</excludeButtons>` on the page layout |
| Keep a field readable on Lightning but hide on the layout's Detail tab | Remove from the layout's System Information section; surface via Dynamic Forms field section |

## Pattern A — Read-only field in the record page header

For a field like `RecordType` that the user should see but never change inline:

1. Open the FlexiPage for the record page in App Builder (or edit the `*.flexipage-meta.xml`).
2. Add a Field Section in the **header region** (just under the Highlights Panel).
3. Place the field inside; set `uiBehavior=readonly`.
4. Remove the same field from the **System Information** field section on the Detail tab so it cannot be edited there.

```xml
<itemInstances>
    <fieldInstance>
        <fieldInstanceProperties>
            <name>uiBehavior</name>
            <value>readonly</value>
        </fieldInstanceProperties>
        <fieldItem>Record.RecordTypeId</fieldItem>
        <identifier>fieldRecordTypeReadonly</identifier>
    </fieldInstance>
</itemInstances>
```

## Pattern B — Disable Change Record Type on a page layout

```xml
<platformActionList>
    <actionListContext>Record</actionListContext>
    ...
</platformActionList>
<excludeButtons>ChangeRecordType</excludeButtons>
```

This hides the action from Lightning Experience and the Salesforce Mobile App for layouts where the metadata is applied. Keep it on a separate "admin layout" where Change Record Type should remain available for support work.

## Pattern C — `enablePageActionConfig`

`enablePageActionConfig=true` lets the FlexiPage drive header actions per page. **It requires Dynamic Actions for the object to be enabled** under Setup -> Salesforce Mobile App -> Dynamic Actions. If the org has not enabled it, deploy fails with a not-very-helpful message. Default to `false` unless you have confirmed the setup.

## Pattern D — Action visibility filters in the highlights panel

When you add quick actions to the highlights panel (e.g. an Accept / Reject pair), restrict them with visibility filters so they only appear in the right state:

```xml
<actionInstance>
    <actionName>MyObject.Accept_Action</actionName>
    <visibilityRule>
        <criteria>
            <leftValue>{!Record.Some_Flag__c}</leftValue>
            <operator>EQUAL</operator>
            <rightValue>true</rightValue>
        </criteria>
        <criteria>
            <leftValue>{!Record.Status__c}</leftValue>
            <operator>NE</operator>
            <rightValue>Accepted</rightValue>
        </criteria>
        <criteria>
            <leftValue>{!Record.Status__c}</leftValue>
            <operator>NE</operator>
            <rightValue>Rejected</rightValue>
        </criteria>
    </visibilityRule>
</actionInstance>
```

Two `NE` checks for "anything other than Accepted / Rejected" are equivalent to "status is blank" when the picklist only has those two values; document this in the design note.

## Pattern E — Mirror layout actions across record types

If the same quick action should appear on multiple page layouts (one per record type), add it to the `platformActionList` on each layout file with the qualified `Object.ActionName` form.

## Field Level Security (FLS) — quick reference

| Goal | Profile / PS XML |
|---|---|
| Hidden | `readable=false`, `editable=false` |
| Read-only | `readable=true`, `editable=false` |
| Read + write | `readable=true`, `editable=true` |

Note: a formula field is always read-only; you only set `readable`. FLS for required fields cannot be set to `readable=false`.


## Required visibility review checks

Use only statuses and fields supplied by the current project; do not add a new
status or treat an example field/action as existing org metadata. If Dynamic
Actions support is unknown, label action visibility as a conditional design that
requires verification in Lightning App Builder; do not present it as deployed.
Field visibility is not a replacement for FLS or record access.

For a local fixture exercise, every QA result must say NOT RUN. Give expected
outcomes separately. Never print PASS ticks for proposed UI tests. Include the
unknown platform support and verification step in the saved summary.
