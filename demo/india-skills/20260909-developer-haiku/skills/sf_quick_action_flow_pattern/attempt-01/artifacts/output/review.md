# Lead.Review Screen Flow Action – Design Summary

**Object:** Lead | **Action:** Review | **API:** Lead_Review  
**Visibility:** Status = "Open - Not Contacted" | **Input:** recordId

---

## Screen Flow: Lead_Review.flow-meta.xml

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Flow xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>64.0</apiVersion>
    <description>Review a Lead record and update status</description>
    <interviewLabel>Lead Review</interviewLabel>
    <isTemplate>false</isTemplate>
    <label>Lead Review</label>
    <processType>Flow</processType>
    <recordUpdates>
        <name>UpdateLeadStatus</name>
        <label>Update Lead Status</label>
        <locationX>200</locationX>
        <locationY>400</locationY>
        <inputAssignments>
            <field>Status</field>
            <value>
                <stringValue>Qualified</stringValue>
            </value>
        </inputAssignments>
        <object>Lead</object>
        <inputReference>recordId</inputReference>
    </recordUpdates>
    <screens>
        <name>ReviewScreen</name>
        <label>Review Lead</label>
        <locationX>200</locationX>
        <locationY>150</locationY>
        <allowBack>true</allowBack>
        <allowFinish>true</allowFinish>
        <fields>
            <name>DisplayLeadInfo</name>
            <fieldText>{!$Record.FirstName} {!$Record.LastName} from {!$Record.Company}</fieldText>
            <fieldType>DisplayText</fieldType>
        </fields>
        <nextOrFinishScreen>UpdateLeadStatus</nextOrFinishScreen>
    </screens>
    <start>
        <locationX>50</locationX>
        <locationY>50</locationY>
        <connector>
            <targetReference>ReviewScreen</targetReference>
        </connector>
    </start>
    <variables>
        <name>recordId</name>
        <dataType>String</dataType>
        <isInput>true</isInput>
        <isOutput>false</isOutput>
    </variables>
</Flow>
```

---

## Quick Action: Lead.Review.quickAction-meta.xml

```xml
<?xml version="1.0" encoding="UTF-8"?>
<QuickAction xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>64.0</apiVersion>
    <description>Review and qualify a Lead</description>
    <flowDefinition>Lead_Review</flowDefinition>
    <label>Review</label>
    <name>Review</name>
    <optionsCreateFeedItem>false</optionsCreateFeedItem>
    <type>Flow</type>
</QuickAction>
```

---

## Page Layout: platformActionList snippet

```xml
<platformActionList>
    <actionListContext>Record</actionListContext>
    <platformActionListItems>
        <actionName>Lead.Review</actionName>
        <actionType>QuickAction</actionType>
        <sortOrder>0</sortOrder>
    </platformActionListItems>
</platformActionList>
```

---

## FlexiPage Highlights Panel: actionInstance with visibility

```xml
<actionInstance>
    <actionName>Lead.Review</actionName>
    <visibilityRule>
        <criteria>
            <leftValue>{!Record.Status}</leftValue>
            <operator>EQUAL</operator>
            <rightValue>Open - Not Contacted</rightValue>
        </criteria>
    </visibilityRule>
</actionInstance>
```

---

## Deployment Order
1. Deploy Screen Flow (Lead_Review) first
2. Deploy Quick Action (Lead.Review)
3. Update Page Layout with platformActionList
4. Update FlexiPage with actionInstance visibility filter

**Status:** All components designed with recordId input, first-screen navigation, and conditional visibility. Ready for deployment.
