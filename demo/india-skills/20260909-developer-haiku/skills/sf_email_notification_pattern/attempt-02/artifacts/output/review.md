# Email Notification Design: Lead Queue Routing

## Configuration Summary

**Scenario:** Lead assigned to Sales_Queue triggers branded email notification.

**Components Created:**
1. **Letterhead** (`Astound_Letterhead.letter-meta.xml`) — Branded header with corporate blue (#0071C5)
2. **Email Folder** (`Lead_Notifications-meta.xml`) — Container for templates
3. **Email Template** (`Lead_Routed_Notification.email-meta.xml` + `.email`) — Text template with merge fields for Lead.FirstName, Lead.Company, Lead.Phone, Lead.Id
4. **Flow** (`Lead_Assignment_Notification.flow-meta.xml`) — Record-triggered after-save on OwnerId change, guards on Queue name and email presence

## Recipient Selection
- **Field:** Lead.Email
- **Guard:** Non-null check prevents silent failures for records without email
- **Cross-object:** Owner.Queue.QueueName verification ensures queue-specific routing

## Transaction Behavior
- **Insert:** If created with Sales_Queue owner and email present → sends
- **Update:** OwnerId change to Sales_Queue (entry condition triggers only on OwnerId change) → sends
- **Idempotence:** Re-assigning to Sales_Queue fires again (intended; no deduplication)

## Negative Tests
- **MT-02:** Lead assigned to user (non-queue) → no email
- **MT-04:** Lead assigned to Sales_Queue, email blank → no email, no error (decision guards)
- **MT-05:** Lead assigned to different queue → no email

## Artifacts Saved
- `/home/tester/work/output/Astound_Letterhead.letter-meta.xml`
- `/home/tester/work/output/Lead_Notifications-meta.xml`
- `/home/tester/work/output/Lead_Routed_Notification.email-meta.xml`
- `/home/tester/work/output/Lead_Routed_Notification.email`
- `/home/tester/work/output/Lead_Assignment_Notification.flow-meta.xml`

**Note:** Design is synthetic/unsent. No connectors used; no Salesforce deployment executed.
