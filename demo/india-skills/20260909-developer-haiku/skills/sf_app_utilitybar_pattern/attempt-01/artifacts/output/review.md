# Demo App Utility Bar Pattern — Repair Plan

## Issues Identified
1. **Missing UtilityBar FlexiPage**: App references `Demo_UtilityBar` but the flexipage file is absent
2. **Profile visibility = false**: Application not visible to users in App Launcher

## Repair Actions

### 1. Create UtilityBar FlexiPage
**File:** `force-app/main/default/flexipages/Demo_UtilityBar.flexipage-meta.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<FlexiPage xmlns="http://soap.sforce.com/2006/04/metadata">
    <masterLabel>Demo Utility Bar</masterLabel>
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

### 2. Fix Application Configuration
**File:** `force-app/main/default/applications/Demo.app-meta.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<CustomApplication xmlns="http://soap.sforce.com/2006/04/metadata">
    <label>Demo</label>
    <navType>Standard</navType>
    <uiType>Lightning</uiType>
    <formFactors>Large</formFactors>
    <formFactors>Small</formFactors>
    <tabs>standard-home</tabs>
    <tabs>standard-Account</tabs>
    <utilityBar>Demo_UtilityBar</utilityBar>
</CustomApplication>
```

### 3. Add Profile Visibility
**File:** `force-app/main/default/profiles/Admin.profile-meta.xml` (add to existing profile)

```xml
<applicationVisibilities>
    <application>Demo</application>
    <default>false</default>
    <visible>true</visible>
</applicationVisibilities>
```

## Deploy
```bash
sf project deploy start \
  --source-dir force-app/main/default/applications \
  --source-dir force-app/main/default/flexipages \
  --source-dir force-app/main/default/profiles \
  --target-org <alias>
```
