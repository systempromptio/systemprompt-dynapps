---
name: sf-bypass-patterns
description: Use when implementing or reviewing Bypass patterns (custom permissions,
  profile checks, static flags) in Salesforce.
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

# Skill: Bypass patterns (custom permissions, profile checks, static flags)

How to let some users (or contexts) skip a guard rule cleanly.

## Default: Custom Permission via Permission Set (preferred)

A **Custom Permission** is a named flag the running user either has or does not have. Assign it via a permission set so admins can grant it to humans **and** integration users without touching code.

### Files

```
force-app/main/default/customPermissions/<Object>_<Action>_Bypass.customPermission-meta.xml
force-app/main/default/permissionsets/<Object>_<Action>_Bypass.permissionset-meta.xml
```

### Apex check

```apex
public with sharing class SomeGuard {
    @TestVisible private static Boolean bypassForTest = false;

    public static Boolean isBypassed() {
        if (Test.isRunningTest() && bypassForTest) {
            return true;
        }
        return FeatureManagement.checkPermission('My_Object_My_Action_Bypass');
    }
}
```

In the trigger handler:

```apex
if (SomeGuard.isBypassed()) {
    return;
}
```

### Why custom permission, not profile name?

- Assignable to integration users without changing their profile.
- Admin-controlled; no code change to add a new bypassed user.
- Survives org refreshes and profile renames.
- Easy to test: assign the permission set to a `TestUser` in `setup()`.

## Profile-based bypass (only when truly necessary)

When the bypass should apply to **System Administrator** as a hard rule (not configurable), it can be a profile-name check. Keep it narrow.

```apex
private static Boolean isAdmin() {
    return [SELECT Profile.Name FROM User WHERE Id = :UserInfo.getUserId() LIMIT 1]
            .Profile.Name == 'System Administrator';
}
```

Combine with the custom permission check:

```apex
if (isAdmin() || SomeGuard.isBypassed()) {
    return;
}
```

Avoid hardcoding any other profile name (e.g. "Sales") in Apex; project-specific profile names belong in `docs/PROJECT_SCHEMA.md` and are cleaner expressed via custom permissions.

## Static flag re-entrancy / test bypass

Static `Boolean` flags are useful for re-entrancy guards (a trigger updating its own object causing recursive fires) and for tests. **Always initialize to `false`** — unboxing a null `Boolean` in `if (flag)` causes runtime errors.

```apex
public class MyHandler {
    @TestVisible private static Boolean bypassForRecursion = false;

    public static void run(List<MyObject__c> records) {
        if (bypassForRecursion) { return; }
        bypassForRecursion = true;
        try {
            // ... do work that may trigger the same trigger again ...
        } finally {
            bypassForRecursion = false;
        }
    }
}
```

## "Block edits when status = X" pattern (closed-record guard)

A common requirement: most users cannot change a record once a terminal status / stage is set, but a privileged persona may.

```apex
public static void enforceClosedGuard(List<Opportunity> oldList, List<Opportunity> newList) {
    if (isAdmin() || SomeGuard.isBypassed()) {
        return;
    }
    Map<Id, Opportunity> oldMap = new Map<Id, Opportunity>(oldList);
    Schema.DescribeSObjectResult dsr = Opportunity.SObjectType.getDescribe();
    for (Opportunity n : newList) {
        Opportunity o = oldMap.get(n.Id);
        if (o == null) { continue; }
        if (o.IsClosed != true || n.IsClosed != true) {
            continue; // allow open->closed and closed->reopen, only block edits while closed
        }
        for (String field : n.getPopulatedFieldsAsMap().keySet()) {
            Schema.DescribeFieldResult dfr = dsr.fields.getMap().get(field).getDescribe();
            if (!dfr.isUpdateable()) { continue; }
            if (n.get(field) != o.get(field)) {
                n.addError('Opportunity cannot be edited in this status.');
                break;
            }
        }
    }
}
```

The key rules:

- Compare `old.IsClosed` and `new.IsClosed` so users can still **close** (open -> closed) and **reopen** (closed -> open) without being blocked — only block changes that happen **while the record stays closed**.
- Use `getPopulatedFieldsAsMap()` to limit the comparison to fields the user actually sent in this DML.
- Skip non-updateable fields per describe.


## Required bypass review checks

Keep three concerns separate: a custom permission for an authorized business
bypass, an optional test-only override, and a production recursion guard. Check
the recursion flag in every execution, not only inside `Test.isRunningTest()`;
initialize it to false and reset it in a finally block. Do not leave SOQL or DML
inside loops when adapting a supplied handler.

Include test scenarios for a user with the permission, an ordinary user without
it, and a recursive production call. State which scenarios are only proposed.
Do not claim a test ran without its execution output.
