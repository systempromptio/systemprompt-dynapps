---
name: sf-profile-permissionset-patterns
description: Use when implementing or reviewing Profile and Permission Set patterns
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

# Skill: Profile and Permission Set patterns

Hard-won patterns for managing profiles, permission sets, permission set groups, and app visibility from source.

## Hierarchy and what lives where

| Container | Contains | Does **not** contain |
|---|---|---|
| Profile | Object permissions, FLS, app visibility, tab visibility, page-layout assignment, user permissions, record type visibility | Permission set group membership |
| Permission Set | Object permissions, FLS, app visibility, tab visibility, user permissions, custom permissions, record type visibility | Profile assignment |
| Permission Set Group | Member permission sets only | `<objectPermissions>` (object access lives on the member sets) |
| Custom Permission | A named flag the running user either has or does not have | Any actual privilege on its own (your code / config consumes it) |

## App Launcher visibility (very common pitfall)

A custom application appears in the App Launcher only when **every relevant profile** sets `<applicationVisibilities>` `visible=true` for that app.

```xml
<applicationVisibilities>
    <application>My_Custom_App</application>
    <default>false</default>
    <visible>true</visible>
</applicationVisibilities>
```

### Pitfalls

- A minimal profile file (only the new field / app entries) deployed over the org default can silently set `visible=false` or wipe other settings. **Retrieve the full profile first**, especially `Admin`, then edit and deploy.
- The **System Administrator** profile is large; deploying a partial Admin profile is a common cause of "the app exists but does not appear in the launcher."

### Recipe

```bash
sf project retrieve start --metadata Profile:Admin --target-org <alias>
# edit force-app/main/default/profiles/Admin.profile-meta.xml — add the applicationVisibilities block
sf project deploy start --source-dir force-app/main/default/profiles/Admin.profile-meta.xml --target-org <alias>
```

## CustomApplication + UtilityBar FlexiPage

The app's `<utilityBar>` value references a FlexiPage developer name; both must deploy together.

```xml
<!-- My_Custom_App.app-meta.xml -->
<utilityBar>My_Custom_App_UtilityBar</utilityBar>

<!-- My_Custom_App_UtilityBar.flexipage-meta.xml -->
<FlexiPage ...>
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
        </flexiPageRegions>
    </itemInstances>
</FlexiPage>
```

Deploy both source files in the same command (or a single manifest).

## Salesforce permission dependencies

Salesforce enforces prerequisite chains. Common ones:

- **Modify All** on an object implies **Delete** and **View All** on that object.
- **`ManageUsers`** requires **`DelegatedTwoFactor`** and **`MonitorLoginHistory`**.
- **`ViewAllData`** has a long chain of dependent system permissions.
- Some permissions (e.g. `AllowForecasting`, `ExportReports`) are **not accepted as `<userPermissions>`** in some orgs; they must be granted in Setup or via the corresponding feature license.

### Recipe

When generating permission sets from a spreadsheet (Y/N grids), apply a **dependency normalization** pass after merging cells: walk a known table of "if granted X, also grant Y, Z" and add the implied permissions, otherwise the deploy fails with `unknown user permission` or `permission requires <other>`.

## Permission Set Group

```xml
<!-- My_Persona.permissionsetgroup-meta.xml -->
<PermissionSetGroup ...>
    <description>Persona X — combined permissions</description>
    <label>My Persona</label>
    <permissionSets>My_Persona_Object_Access</permissionSets>
    <permissionSets>My_Persona_System_Access</permissionSets>
</PermissionSetGroup>
```

Object permissions go on the member sets, not the group.

## Naming conventions

| Component | Convention |
|---|---|
| Permission set (object access) | `<Persona>_Object_Access` |
| Permission set (system / tabs / user perms) | `<Persona>_System_Access` or `<Default>_Access_for_<Persona>` |
| Permission set group | `<Persona>` |
| Custom application | `<Org>_<Function>` (e.g. `My_Sales`) |
| Utility bar FlexiPage | `<App>_UtilityBar` |
| Custom permission (bypass) | `<Object>_<Action>_Bypass` |

## Deployment order

1. Custom Permissions.
2. Permission Sets.
3. Permission Set Groups.
4. Profiles (can be combined with step 2 if your pipeline deploys the whole tree).
5. Custom Applications + their UtilityBar FlexiPages (together).
6. After deploy, assign the relevant permission set groups to users in Setup (or via a separate `sf data import` / data step).

## Page layout assignment after deploy

Salesforce often defaults new record types to the master layout for any profile that wasn't included in the metadata change. After deploy, walk **Setup -> Object Manager -> <Object> -> Page Layout Assignment** and confirm each profile uses the intended layout. Note this in the runbook.
