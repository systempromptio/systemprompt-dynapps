# App + Utility Bar Repair Plan

## Issues Identified

1. **Incomplete CustomApplication**: `fixtures/broken-app.xml` lacks `navType`, `uiType`, `formFactors`, `tabs`
2. **Missing UtilityBar FlexiPage**: Referenced as `Demo_UtilityBar` but file absent
3. **Profile Visibility**: Set to `false`; app invisible to users

## Repair Actions

### 1. Complete Demo.app-meta.xml
```xml
<CustomApplication xmlns="http://soap.sforce.com/2006/04/metadata">
    <label>Demo</label>
    <navType>Standard</navType>
    <uiType>Lightning</uiType>
    <formFactors>Large</formFactors>
    <formFactors>Small</formFactors>
    <tabs>standard-home</tabs>
    <tabs>standard-Account</tabs>
    <tabs>standard-Lead</tabs>
    <utilityBar>Demo_UtilityBar</utilityBar>
</CustomApplication>
```

### 2. Create Missing Demo_UtilityBar.flexipage-meta.xml
```xml
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
        </itemInstances>
    </flexiPageRegions>
</FlexiPage>
```

### 3. Profile Visibility (Proposed Fragment)
Add to each profile (retrieve full profile before edit):
```xml
<applicationVisibilities>
    <application>Demo</application>
    <default>false</default>
    <visible>true</visible>
</applicationVisibilities>
```

## Deployment Dependency
- App references: `Demo_UtilityBar`
- FlexiPage name match: ✓ Aligned

## Unsupported Operations
- Full profile retrieval: Unavailable (no Salesforce connector)
- App visibility verification: Blocked (local fixture only)

## Saved Artifacts
This review.md — evidence summary only. XML snippets are proposals; no files written to metadata source.
