---
name: sf-assignment-rule-reexecution
description: Use when implementing or reviewing Re-running default assignment rules
  after in-line edits in Salesforce.
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

# Skill: Re-running default assignment rules after in-line edits

By default, Lead / Case **Assignment Rules** run on insert and (optionally) on update via the UI checkbox "Use Active Assignment Rule." Inline edits and API updates do not re-run them. This pattern adds a clean, declarative re-execution path.

## Pattern

A small invocable Apex method that takes a list of record Ids, sets the DML option to use the default assignment rule, and updates the records. Call it from an after-save record-triggered Flow.

## Apex

```apex
public with sharing class LeadAssignmentInvocable {

    public class Request {
        @InvocableVariable(required=true)
        public Id leadId;
    }

    @InvocableMethod(label='Run Lead Assignment Rules' description='Triggers default assignment rules for the provided Lead IDs')
    public static void runAssignmentRules(List<Request> requests) {
        Set<Id> leadIds = new Set<Id>();
        for (Request r : requests) {
            if (r.leadId != null) { leadIds.add(r.leadId); }
        }
        if (leadIds.isEmpty()) { return; }

        List<Lead> leads = [SELECT Id FROM Lead WHERE Id IN :leadIds];
        if (leads.isEmpty()) { return; }

        Database.DMLOptions opts = new Database.DMLOptions();
        opts.assignmentRuleHeader.useDefaultRule = true;
        Database.update(leads, opts);
    }
}
```

> The `@InvocableMethod(label='...' description='...')` annotation parameters must be on **one line** in Apex source.

## Flow

| Property | Value |
|---|---|
| API name | `Lead_After_Save_Automation` (or your project's convention) |
| Label | Lead After Save Automation |
| Trigger | Lead - after save - created or updated |

Single path: **Start -> Apex action `LeadAssignmentInvocable` (input `leadId` <- `{!$Record.Id}`)**.

## Why a separate after-save flow

- **Order of execution:** assignment rules normally fire after triggers and before workflow rules; using `Database.DMLOptions.assignmentRuleHeader` from your own DML re-runs them cleanly without changing the rule itself.
- **Granular control:** add entry conditions to the Flow to limit when re-assignment runs (e.g. only when a specific field changes, or only when ownership has not been overridden manually).

## Cautions

- Do not call this from a `before` trigger or a before-save flow; assignment rules need an after-save context.
- Test with **all active validation rules** in the org. The invocable performs a real `Database.update`, which means tests must produce records that satisfy every active validation rule. Use a `TestDataFactory` to centralise required-field defaults (address, contact method, project-required custom fields).
- Recursion: if the assignment rule's update fires the same after-save flow again with no entry-condition guard, you will hit a recursion limit. Guard the flow's entry condition (e.g. only run when `OwnerId` does **not** equal a "do not re-assign" sentinel queue, or use a static flag in Apex).

## QA

- **AT-01:** Insert a Lead that the assignment rule routes to a specific user / queue -> assert `OwnerId` equals the expected value.
- **AT-02:** Update a Lead's `Country` (or whatever field the rule keys on) -> assert `OwnerId` re-routes per the new value.
- **AT-03:** Update a field the rule does **not** key on -> assert `OwnerId` is unchanged.
