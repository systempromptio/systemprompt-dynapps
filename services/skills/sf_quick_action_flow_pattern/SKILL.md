---
name: sf-quick-action-flow-pattern
description: Use when implementing or reviewing Quick Action launching a Screen Flow
  in Salesforce.
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

# Skill: Quick Action launching a Screen Flow

A clean way to deliver a guided record action (Accept / Reject, Approve, Resolve, Submit-with-reason) without writing Apex.

## Anatomy

| Component | File | Purpose |
|---|---|---|
| Screen Flow | `flows/<Object>_<Action>.flow-meta.xml` | The guided UI: input `recordId`, screens, decisions, record updates, success message |
| Quick Action | `quickActions/<Object>.<Action>.quickAction-meta.xml` | The button users click; references the flow by API name |
| Page Layout | `layouts/<Object>-<Layout>.layout-meta.xml` | `platformActionList` includes `<Object>.<Action>` (qualified name) |
| FlexiPage (record page) | `flexipages/<Object>_Record_Page_*.flexipage-meta.xml` | Highlights panel `actionInstance` with visibility filters |

## Screen Flow skeleton

- **Type:** Screen Flow.
- **Inputs:** `recordId` (`Input` available, type matches the object).
- **First screen:** `allowBack=true` or `allowFinish=true` (Salesforce requires at least one).
- **Logic:** Screens, decisions, record updates as needed.
- **Output / success:** A final screen with success text, or close after a `Update Records` step.

## Quick Action skeleton

```xml
<QuickAction ...>
    <flowDefinition>My_Object_Reject</flowDefinition>
    <label>Reject</label>
    <name>Reject</name>
    <optionsCreateFeedItem>false</optionsCreateFeedItem>
    <type>Flow</type>
</QuickAction>
```

File name: `quickActions/MyObject.Reject.quickAction-meta.xml`.

## Page Layout: include the action

```xml
<platformActionList>
    <actionListContext>Record</actionListContext>
    <platformActionListItems>
        <actionName>MyObject.Reject</actionName>
        <actionType>QuickAction</actionType>
        <sortOrder>0</sortOrder>
    </platformActionListItems>
    <!-- ... other actions ... -->
</platformActionList>
```

Add to **every** layout where the action should appear (one per record type, typically).

## FlexiPage highlights panel: surface the action

In the Highlights Panel of the Lightning record page, add an `actionInstance` per action with visibility filters so it only appears in the right state. See `the installed sf-dynamic-forms-field-visibility skill` Pattern D.

```xml
<actionInstance>
    <actionName>MyObject.Reject</actionName>
    <visibilityRule>
        <criteria>
            <leftValue>{!Record.Status__c}</leftValue>
            <operator>EQUAL</operator>
            <rightValue>Pending</rightValue>
        </criteria>
    </visibilityRule>
</actionInstance>
```

Set `numVisibleActions` on the highlights panel to a number that fits all conditional actions.

## Common pitfalls

- **Flow not found at runtime:** the quick action references the flow API name; deploy the flow before the quick action.
- **Action not visible in the highlights panel:** the FlexiPage's `actionInstance` is missing or filtered out. Check the visibility rule.
- **Action not visible in the action bar (Salesforce Mobile / cards):** the page **layout's** `platformActionList` is missing the qualified `Object.Action` entry.
- **`enablePageActionConfig=true`:** requires Dynamic Actions for the object to be enabled in Setup; default to `false` unless confirmed.
- **FLS on fields the flow updates:** users running the action need edit on those fields. Add to the persona's permission set / profile FLS in the same change.
