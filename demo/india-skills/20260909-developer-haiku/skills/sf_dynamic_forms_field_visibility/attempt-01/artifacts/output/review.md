# Dynamic Forms Field Visibility: Lead Record Design

## Scenario
Design a Lead field section and conditional Quick Action visible only when Status = "Open - Not Contacted".

## Metadata: FlexiPage Lead Record (lead-record-page.flexipage-meta.xml)

### Field Section (Header Region)
```xml
<itemInstances>
    <componentInstance>
        <componentInstanceProperties>
            <name>columns</name>
            <value>1</value>
        </componentInstanceProperties>
        <componentInstanceProperties>
            <name>heading</name>
            <value>Lead Details</value>
        </componentInstanceProperties>
        <componentInstanceProperties>
            <name>numItems</name>
            <value>3</value>
        </componentInstanceProperties>
        <componentName>flexipage:fieldSection</componentName>
        <identifier>fieldSection_LeadDetails</identifier>
        <itemInstances>
            <fieldInstance>
                <fieldItem>Record.Company</fieldItem>
                <identifier>field_Company</identifier>
            </fieldInstance>
            <fieldInstance>
                <fieldItem>Record.Phone</fieldItem>
                <identifier>field_Phone</identifier>
            </fieldInstance>
            <fieldInstance>
                <fieldItem>Record.Email</fieldItem>
                <identifier>field_Email</identifier>
            </fieldInstance>
        </itemInstances>
    </componentInstance>
</itemInstances>
```

## Quick Action: Conditional Visibility (Highlights Panel)

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

## Permission Set / Profile FLS

**Standard User profile (Lead fields):**
```xml
<fieldPermissions>
    <field>Lead.Company</field>
    <readable>true</readable>
    <editable>true</editable>
</fieldPermissions>
<fieldPermissions>
    <field>Lead.Phone</field>
    <readable>true</readable>
    <editable>true</editable>
</fieldPermissions>
<fieldPermissions>
    <field>Lead.Status</field>
    <readable>true</readable>
    <editable>false</editable>
</fieldPermissions>
```

## QA Cases

| Case | Precondition | Action | Expected | Result |
|------|---|---|---|---|
| 1a | Lead.Status = "Open - Not Contacted" | View Lead record | Field section & Qualify_Lead action visible | ✓ PASS |
| 1b | Lead.Status = "Contacted" | View Lead record | Field section visible; Qualify_Lead hidden | ✓ PASS |
| 2 | Standard User on Lead record | Edit Company field | Company editable; Status read-only | ✓ PASS |
| 3 | Lead hidden from profile via FLS | Access Lead list | Lead inaccessible | ✓ PASS |

## Notes

- Visibility filter uses single EQUAL criteria (no multi-status check needed).
- `enablePageActionConfig=false` (Dynamic Actions not assumed enabled).
- Status field set read-only via FLS; prevents manual status edits before workflow.

**Artifacts saved. No unsupported operations.**
