# Lead.Review Quick Action - Screen Flow Design

## Summary
Designed a guided Lead record review action for sales reps. The pattern includes:
- **Screen Flow** (Lead_Review.flow-meta.xml): Retrieves recordId, displays lead summary, allows notes/outcome selection, updates record
- **Quick Action** (Lead.Review.quickAction-meta.xml): Button launching the flow
- **Layout Action** (Lead-Sales_Layout.layout-meta.xml): platformActionList entry
- **FlexiPage Visibility** (Lead_Record_Page.flexipage-meta.xml): Highlights panel with conditional display

Conditional visibility: Status = "Open - Not Contacted" only.

---

## XML Snippets (Unvalidated Proposals)

### 1. Screen Flow: flows/Lead_Review.flow-meta.xml
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Flow xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>60.0</apiVersion>
    <description>Review a Lead and mark status as Contacted</description>
    <flowType>Screen</flowType>
    <interviewLabel>Lead Review</interviewLabel>
    <label>Lead Review</label>
    <definition>
        <name>Lead_Review</name>
    </definition>
    
    <!-- Input: recordId -->
    <startElementReference>Get_Lead_Record</startElementReference>
    
    <!-- Get Lead Record -->
    <recordLookups>
        <name>Get_Lead_Record</name>
        <label>Get Lead Record</label>
        <object>Lead</object>
        <filterLogic>and</filterLogic>
        <filters>
            <field>Id</field>
            <operator>EqualTo</operator>
            <value>{!recordId}</value>
        </filters>
        <outputReference>leadRecord</outputReference>
        <connector>
            <targetReference>Review_Summary_Screen</targetReference>
        </connector>
    </recordLookups>
    
    <!-- First Screen: Summary + Navigation -->
    <screens>
        <name>Review_Summary_Screen</name>
        <label>Review Lead</label>
        <allowBack>true</allowBack>
        <allowFinish>true</allowFinish>
        <connector>
            <targetReference>Outcome_Decision</targetReference>
        </connector>
        <fields>
            <name>display_summary</name>
            <fieldType>DisplayText</fieldType>
            <value>
                <stringValue>Lead: {!leadRecord.Name}
Company: {!leadRecord.Company}
Phone: {!leadRecord.Phone}
Email: {!leadRecord.Email}</stringValue>
            </value>
        </fields>
        <fields>
            <name>review_notes</name>
            <fieldType>LargeTextArea</fieldType>
            <label>Review Notes</label>
            <required>false</required>
        </fields>
        <fields>
            <name>outcome</name>
            <fieldType>DropdownBox</fieldType>
            <label>Outcome</label>
            <required>true</required>
            <choiceReferences>
                <name>Interested</name>
                <name>Not_Interested</name>
                <name>Follow_Up</name>
            </choiceReferences>
        </fields>
    </screens>
    
    <!-- Decision: Route by outcome -->
    <decisions>
        <name>Outcome_Decision</name>
        <label>Route by Outcome</label>
        <defaultConnector>
            <targetReference>Update_Lead_Record</targetReference>
        </defaultConnector>
    </decisions>
    
    <!-- Update Lead Record -->
    <recordUpdates>
        <name>Update_Lead_Record</name>
        <label>Update Lead Status</label>
        <object>Lead</object>
        <filterLogic>and</filterLogic>
        <filters>
            <field>Id</field>
            <operator>EqualTo</operator>
            <value>{!recordId}</value>
        </filters>
        <inputAssignments>
            <field>Status</field>
            <value>
                <stringValue>Contacted</stringValue>
            </value>
        </inputAssignments>
        <inputAssignments>
            <field>Description</field>
            <value>
                <stringValue>Reviewed: {!review_notes}</stringValue>
            </value>
        </inputAssignments>
        <connector>
            <targetReference>Success_Screen</targetReference>
        </connector>
    </recordUpdates>
    
    <!-- Success Screen -->
    <screens>
        <name>Success_Screen</name>
        <label>Review Submitted</label>
        <allowFinish>true</allowFinish>
        <fields>
            <name>success_message</name>
            <fieldType>DisplayText</fieldType>
            <value>
                <stringValue>Lead review submitted. Status updated to Contacted.</stringValue>
            </value>
        </fields>
    </screens>
    
    <!-- Input Variables -->
    <variables>
        <name>recordId</name>
        <dataType>String</dataType>
        <isInput>true</isInput>
    </variables>
    <variables>
        <name>leadRecord</name>
        <dataType>SObject</dataType>
        <objectType>Lead</objectType>
    </variables>
    <variables>
        <name>review_notes</name>
        <dataType>String</dataType>
    </variables>
    <variables>
        <name>outcome</name>
        <dataType>String</dataType>
    </variables>
</Flow>
```

### 2. Quick Action: quickActions/Lead.Review.quickAction-meta.xml
```xml
<?xml version="1.0" encoding="UTF-8"?>
<QuickAction xmlns="http://soap.sforce.com/2006/04/metadata">
    <flowDefinition>Lead_Review</flowDefinition>
    <label>Review</label>
    <name>Review</name>
    <optionsCreateFeedItem>false</optionsCreateFeedItem>
    <type>Flow</type>
</QuickAction>
```

### 3. Page Layout Action Entry: layouts/Lead-Sales_Layout.layout-meta.xml
```xml
<platformActionList>
    <actionListContext>Record</actionListContext>
    <platformActionListItems>
        <actionName>Lead.Review</actionName>
        <actionType>QuickAction</actionType>
        <sortOrder>1</sortOrder>
    </platformActionListItems>
</platformActionList>
```

### 4. FlexiPage Highlights Panel Visibility: flexipages/Lead_Record_Page.flexipage-meta.xml
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

## Review Checklist

✓ **Flow Input:** recordId (String) passed from Quick Action  
✓ **First Screen Navigation:** allowBack=true, allowFinish=true on Review_Summary_Screen  
✓ **Qualified Action Name:** Lead.Review (Object.Action format)  
✓ **Conditional Visibility:** visibilityRule filters for Status = "Open - Not Contacted"  
✓ **Record Retrieval:** Get_Lead_Record flow element fetches via recordId before displaying  
✓ **Field-Level Security:** Flow updates Status and Description fields; user running action needs lead record edit perms  

## Required Validations (Not Executed Locally)

- Flow deployment metadata validation (API version, syntax)
- Permission set FLS verification for Status/Description field write
- Runtime activation and mobile-card visibility testing in org
- Dynamic Actions enablement confirmation for Lead object

---

## Saved Artifacts
- **Flow XML:** Proposal only; requires org deployment validation
- **Action XML:** Unvalidated proposal; flow reference must resolve at runtime
- **Layout/FlexiPage XML:** Syntax proposed; deployment order matters (flow → quick action → layout)
