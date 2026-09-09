# Lead Routing Email Notification Design

## Scenario
Sales Cloud Lead routing to **SalesQueue_Distribution** queue; notify Lead contact when assignment occurs.

## Configuration Artifacts

### 1. Letterhead: `force-app/main/default/letterhead/Sales_Letterhead.letter-meta.xml`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Letterhead xmlns="http://soap.sforce.com/2006/04/metadata">
    <available>true</available>
    <backgroundColor>#FFFFFF</backgroundColor>
    <bodyColor>#FFFFFF</bodyColor>
    <description>Sales Cloud branded letterhead</description>
    <footer>
        <backgroundColor>#333333</backgroundColor>
        <height>20</height>
    </footer>
    <header>
        <backgroundColor>#0071C5</backgroundColor>
        <height>40</height>
    </header>
    <name>Sales Letterhead</name>
    <topLine>
        <color>#0071C5</color>
        <height>1</height>
    </topLine>
</Letterhead>
```

### 2. Email Folder: `force-app/main/default/email/Lead_Notifications-meta.xml`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<EmailFolder xmlns="http://soap.sforce.com/2006/04/metadata">
    <name>Lead_Notifications</name>
    <publicFolderAccess>PublicReadAndWriteAccess</publicFolderAccess>
</EmailFolder>
```

### 3. Email Template: `force-app/main/default/email/Lead_Notifications/Lead_Assignment.email-meta.xml`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<EmailTemplate xmlns="http://soap.sforce.com/2006/04/metadata">
    <available>true</available>
    <encodingKey>UTF-8</encodingKey>
    <folderName>Lead_Notifications</folderName>
    <letterhead>Sales_Letterhead</letterhead>
    <name>Lead Assignment Notification</name>
    <relatedEntityType>Lead</relatedEntityType>
    <subject>Your Inquiry Has Been Routed: {{{Lead.Company}}}</subject>
    <type>text</type>
    <uiType>Aloha</uiType>
</EmailTemplate>
```

### 4. Template Body: `force-app/main/default/email/Lead_Notifications/Lead_Assignment.email`
```
Dear {{{Lead.FirstName}}},

Your inquiry for {{{Lead.Company}}} has been received and assigned to our sales team.

Lead ID: {{{Lead.Id}}}
Status: {{{Lead.Status}}}
Lead Source: {{{Lead.LeadSource}}}

A representative will contact you shortly.

Best regards,
Sales Team
```

### 5. Flow: `force-app/main/default/flows/Lead_RoutingNotification.flow-meta.xml`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Flow xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>64.0</apiVersion>
    <decisions>
        <name>Check_Is_Queue</name>
        <label>Is SalesQueue and Has Email?</label>
        <locationX>250</locationX>
        <locationY>200</locationY>
        <defaultConnectorLabel>No Match</defaultConnectorLabel>
        <rules>
            <name>Queue_And_Email</name>
            <conditionLogic>and</conditionLogic>
            <conditions>
                <leftValueReference>$Record.Owner:Queue.QueueName</leftValueReference>
                <operator>EqualTo</operator>
                <rightValue>SalesQueue_Distribution</rightValue>
            </conditions>
            <conditions>
                <leftValueReference>$Record.Email</leftValueReference>
                <operator>IsNull</operator>
                <rightValue>false</rightValue>
            </conditions>
            <connector>
                <targetReference>Send_Notification</targetReference>
            </connector>
            <label>Queue Match + Email</label>
        </rules>
    </decisions>
    <description>Route Lead assignment notification to SalesQueue_Distribution owner</description>
    <environments>Default</environments>
    <interviewLabel>Lead Routing Notification</interviewLabel>
    <label>Lead_RoutingNotification</label>
    <processMetadataValues>
        <name>BuilderType</name>
        <value>
            <stringValue>LightningFlowBuilder</stringValue>
        </value>
    </processMetadataValues>
    <processType>AutoLaunchedFlow</processType>
    <recordTriggerType>CreateAndUpdate</recordTriggerType>
    <recordUpdates>
        <name>Send_Notification</name>
        <label>Send Email</label>
        <locationX>450</locationX>
        <locationY>200</locationY>
        <actionType>EmailAlert</actionType>
        <inputAssignments>
            <name>emailTemplate</name>
            <value>
                <stringValue>Lead_Notifications.Lead_Assignment</stringValue>
            </value>
        </inputAssignments>
        <inputAssignments>
            <name>emailAddresses</name>
            <value>
                <elementReference>$Record.Email</elementReference>
            </value>
        </inputAssignments>
    </recordUpdates>
    <start>
        <locationX>50</locationX>
        <locationY>50</locationY>
        <connector>
            <targetReference>Check_Is_Queue</targetReference>
        </connector>
    </start>
    <status>Active</status>
    <triggerObject>Lead</triggerObject>
    <triggerType>RecordAfterSave</triggerType>
    <entryCriteria>
        <formula>
            OR(
                ISNEW(),
                ISCHANGED($Record.OwnerId)
            )
        </formula>
    </entryCriteria>
</Flow>
```

## Recipient Selection
- **Primary:** `Lead.Email` (required, guarded by flow condition `Email != blank`)
- **Failsafe:** Email is validated non-null before flow execution; missing email silently skips send (no error logged)

## Transaction Behavior
- **Insert with target Queue:** Sends notification immediately post-save
- **Update owner to target Queue:** Sends notification on change detection
- **Update owner to different queue:** No send (entry condition: `ISCHANGED(OwnerId)` false)
- **Blank email:** Decision exits cleanly without send or error

## Negative Tests (NOT RUN — local fixture only; delivery unverified)

| Test | Scenario | Expected | Status |
|------|----------|----------|--------|
| **MT-01** | Insert Lead assigned to SalesQueue_Distribution + valid email | Email sent to Lead.Email | NOT RUN |
| **MT-02** | Insert Lead assigned to different queue | No email sent | NOT RUN |
| **MT-03** | Update Lead owner to SalesQueue_Distribution | Email sent to Lead.Email | NOT RUN |
| **MT-04** | Lead assigned to queue with blank email | No send, no error, flow exits Decision | NOT RUN |
| **MT-05** | Invalid email format (syntax error) | Send attempted; failure logged by Salesforce platform | NOT RUN |
| **MT-06** | Recipient address bounces (delivery failure) | Email queued; bounce handled by Email Service; no flow error | NOT RUN |

## Fault Paths
- **Guard:** Email blank → Decision returns "No Match" → flow terminates (no send action invoked)
- **Failure (not guard):** Email valid but undeliverable → Email Service queues message; flow completes successfully; bounce is platform responsibility
- Distinction: Guard prevents send invocation; failure occurs during delivery outside flow scope

## Unsupported Operations (Local Fixture)
- Email delivery cannot be verified in synthetic fixture
- Salesforce queue name resolution requires live org metadata
- Flow syntax validation requires deployment to target org
- Letterhead rendering and merge field substitution are platform-time operations

## Artifacts Saved
- Letterhead, email folder, template (body + metadata), flow all defined above
- Configuration complete; no deployment attempted
- No email sent or claimed delivered
