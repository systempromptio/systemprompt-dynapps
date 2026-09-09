# Dynamic Forms Field Visibility — Lead Review Scenario
**Fixture**: India Skills Fixture (API 64.0) | **Date**: 2026-09-09  
**Scenario**: Conditional Lead field section + Quick Action visible only when Status = "Open - Not Contacted"

---

## Design: Lead Field Section (FlexiPage Header)

**Metadata**: Lead record page (`Lead_Record_Page.flexipage-meta.xml`)

```xml
<itemInstances>
    <fieldInstance>
        <fieldInstanceProperties>
            <name>uiBehavior</name>
            <value>readonly</value>
        </fieldInstanceProperties>
        <fieldItem>Record.Country</fieldItem>
        <identifier>fieldCountryReadonly</identifier>
        <visibilityRule>
            <criteria>
                <leftValue>{!Record.Status}</leftValue>
                <operator>EQUAL</operator>
                <rightValue>Open - Not Contacted</rightValue>
            </criteria>
        </visibilityRule>
    </fieldInstance>
</itemInstances>
```

**Field**: Country (standard Lead field)  
**Behavior**: Read-only in header; visible only when Status = "Open - Not Contacted"

---

## Design: Conditional Quick Action

**Metadata**: Lead page layout (`Lead_Layout.layout-meta.xml`)

```xml
<actionInstance>
    <actionName>Lead.Qualify_Lead</actionName>
    <visibilityRule>
        <criteria>
            <leftValue>{!Record.Status}</leftValue>
            <operator>EQUAL</operator>
            <rightValue>Open - Not Contacted</rightValue>
        </criteria>
    </visibilityRule>
</actionInstance>
```

**Action**: Qualify_Lead (hypothetical quick action)  
**Visibility**: Enabled only when Status = "Open - Not Contacted"  
**⚠️ Dynamic Actions Support**: UNKNOWN — Verify in Lightning App Builder under Setup → Salesforce Mobile App → Dynamic Actions. If not enabled, deploy will fail. Conditional design requires confirmation.

---

## Permission QA Cases

| Case | Status | Expected Visibility | Test Result |
|------|--------|-------------------|-------------|
| **Q1**: User with FLS `Country.readable=true, editable=false` views Lead with Status="Open - Not Contacted" | Open - Not Contacted | Field section shown, read-only; Qualify_Lead action shown | **NOT RUN** — Local fixture only |
| **Q2**: User with FLS `Country.readable=false` views same Lead | Open - Not Contacted | Field section hidden (FLS blocks read); Qualify_Lead action shown | **NOT RUN** — FLS enforcement requires org |
| **Q3**: User with FLS `Country.readable=true, editable=true` views Lead with Status="Qualified" | Qualified | Field section hidden (status condition fails); Qualify_Lead action hidden | **NOT RUN** — Conditional visibility logic requires runtime |
| **Q4**: Admin views Lead; dynamic action support disabled | Open - Not Contacted | Field section shown, read-only; Qualify_Lead action **NOT** shown (requires Dynamic Actions feature) | **NOT RUN** — Feature flag check requires org |

---

## Unsupported Operations

- ❌ **Dynamic Actions verification**: Cannot confirm org-level enablement of Dynamic Actions for Lead object without Salesforce connector.  
- ❌ **Action deployment**: Quick action `Lead.Qualify_Lead` assumed to exist; not verified against fixture.  
- ❌ **Record type filtering**: Fixture supplies no record types; single layout assumed.

---

## Evidence & Summary

**Saved artifacts**: This file (`output/review.md`)  
**Schema source**: `/home/tester/work/docs/PROJECT_SCHEMA.md` (Lead standard fields; Status: "Open - Not Contacted" | "Qualified")  
**Fixture brief**: `/home/tester/work/fixtures/project-brief.md`  

**Metadata patterns applied**:
1. **Pattern D** (visibility filters) — Status equality check on action instance
2. **Pattern A** (read-only field) — `uiBehavior=readonly` + visibilityRule in field section

**Next steps for deployment**:
1. Confirm quick action `Lead.Qualify_Lead` exists in target org.
2. In Lightning App Builder, verify Dynamic Actions is enabled for Lead under Setup → Salesforce Mobile App.
3. Test conditional visibility in browser with Status = "Open - Not Contacted" and Status = "Qualified".
4. Validate FLS (`Country` readable/editable) for assigned profile/permission sets.
