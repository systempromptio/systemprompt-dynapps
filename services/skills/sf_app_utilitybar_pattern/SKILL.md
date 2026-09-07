---
name: sf-app-utilitybar-pattern
description: Use when implementing or reviewing Custom Application + Utility Bar pattern
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

# Skill: Custom Application + Utility Bar pattern

A small, deploy-tight pattern for delivering a custom Lightning app with a utility bar.

## Files

| File | Purpose |
|---|---|
| `force-app/main/default/applications/<App>.app-meta.xml` | The CustomApplication: tabs, navType, formFactors, `<utilityBar>` reference |
| `force-app/main/default/flexipages/<App>_UtilityBar.flexipage-meta.xml` | UtilityBar FlexiPage referenced by the app |
| `force-app/main/default/profiles/<Profile>.profile-meta.xml` | `<applicationVisibilities>` for `<App>` (`visible=true`) per profile |

## CustomApplication skeleton

```xml
<CustomApplication ...>
    <label>My App</label>
    <navType>Standard</navType>
    <uiType>Lightning</uiType>
    <formFactors>Large</formFactors>
    <formFactors>Small</formFactors>
    <tabs>standard-home</tabs>
    <tabs>standard-Account</tabs>
    <tabs>standard-Opportunity</tabs>
    <!-- ... ordered list of tabs ... -->
    <utilityBar>My_App_UtilityBar</utilityBar>
</CustomApplication>
```

## Tab API names (common reference)

| UI label | Tab API name |
|---|---|
| Home | `standard-home` |
| Accounts | `standard-Account` |
| Contacts | `standard-Contact` |
| Leads | `standard-Lead` |
| Opportunities | `standard-Opportunity` |
| Campaigns | `standard-Campaign` |
| Campaign Members | `standard-Campaign` (no separate member tab in most orgs) |
| Products | `standard-Product2` |
| Quotes | `standard-Quote` |
| Reports | `standard-report` |
| Dashboards | `standard-Dashboard` |
| Tasks | `standard-Task` |
| Calendar | `standard-Event` |
| Forecasts | `standard-Forecasting3` |

For a custom-object tab, use `<Namespace>__<Object>__c` (e.g. `My_Custom_Object__c`).

For an "all tabs visible" permission-set entry (Sales Operations Advanced Access pattern), iterate `SELECT Name FROM TabDefinition WHERE IsAvailableInAloha = true` for the target org and emit each as a `<tabSettings>` of `Visible`.

## UtilityBar FlexiPage skeleton

```xml
<FlexiPage ...>
    <masterLabel>My App Utility Bar</masterLabel>
    <type>UtilityBar</type>
    <flexiPageRegions>
        <name>utilityItems</name>
        <type>Region</type>
        <itemInstances>
            <componentInstance>
                <componentName>notes:utilityBarNoteList</componentName>
            </componentInstance>
            <componentInstance>
                <componentName>runtime_sales_todo_list:unifiedToDoListAuraWrapper</componentName>
            </componentInstance>
        </itemInstances>
    </flexiPageRegions>
</FlexiPage>
```

## Profile visibility

Add to each profile that should see the app (do not forget Admin):

```xml
<applicationVisibilities>
    <application>My_App</application>
    <default>false</default>
    <visible>true</visible>
</applicationVisibilities>
```

**Always retrieve the full profile (especially `Admin`) before editing**; partial profile files can clobber unrelated permissions.

## Deploy

```bash
sf project deploy start \
  --source-dir force-app/main/default/applications \
  --source-dir force-app/main/default/flexipages \
  --source-dir force-app/main/default/profiles \
  --target-org <alias>
```

## Post-deploy

- Setup -> App Manager -> confirm the app is listed.
- App Launcher -> confirm the app appears for each profile (use **Login As** to check).
- If the app exists but the launcher does not show it for a profile, the profile's `<applicationVisibilities>` `visible` is `false` — re-retrieve, fix, redeploy.
