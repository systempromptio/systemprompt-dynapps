---
name: sf-email-notification-pattern
description: Use when implementing or reviewing Email notification on record / owner
  / queue change in Salesforce.
---

## Systemprompt execution contract

Workspace access installs these skills. The `atlassian`, `salesforce`, and
`github` connectors are delivered separately after the signed-in user connects
and verifies each provider account. An installed skill does not prove a connector
is ready. Local file tasks can run without provider connections. For remote
operations, use only the currently advertised tools and input schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

Atlassian v2 exposes primary tools plus `discover` and `executeRead`/`executeWrite`.
Inspect live schemas; do not assume legacy Jira or Confluence tool names.

# Skill: Email notification on record / owner / queue change

How to deliver a branded, one-shot email notification triggered by a record change (commonly: ownership changes to a specific Queue) without writing Apex.

## Components

| Component | File | Purpose |
|---|---|---|
| Letterhead | `force-app/main/default/letterhead/<Brand>_Letterhead.letter-meta.xml` | Branded header / footer / colors |
| Email Folder | `force-app/main/default/email/<Folder>-meta.xml` | Folder that holds the template |
| Email Template | `force-app/main/default/email/<Folder>/<Template>.email` + `.email-meta.xml` | The template body and metadata (subject, related entity, letterhead) |
| Record-Triggered Flow | `force-app/main/default/flows/<Object>_<Trigger>.flow-meta.xml` | Detects the change and sends the email |

## Letterhead skeleton

```xml
<Letterhead ...>
    <available>true</available>
    <backgroundColor>#FFFFFF</backgroundColor>
    <bodyColor>#FFFFFF</bodyColor>
    <description>Brand letterhead</description>
    <footer>
        <backgroundColor>#333333</backgroundColor>
        <height>20</height>
    </footer>
    <header>
        <backgroundColor>#0071C5</backgroundColor>
        <height>40</height>
    </header>
    <name>Brand Letterhead</name>
    <topLine>
        <color>#0071C5</color>
        <height>1</height>
    </topLine>
</Letterhead>
```

## Email Template skeleton (text type)

```xml
<EmailTemplate ...>
    <available>true</available>
    <encodingKey>UTF-8</encodingKey>
    <folderName>My_Notifications</folderName>
    <letterhead>Brand_Letterhead</letterhead>
    <name>My Notification</name>
    <relatedEntityType>Lead</relatedEntityType>
    <subject>Regarding your inquiry</subject>
    <type>text</type>
    <uiType>Aloha</uiType>
</EmailTemplate>
```

Body file (`*.email`) uses standard merge fields like `{{{Lead.FirstName}}}`.

## Record-Triggered Flow (after-save)

| Property | Value |
|---|---|
| Object | Lead (or your object) |
| Trigger Type | Record-Triggered, after save |
| Record Trigger | Create and Update |
| Process Type | AutoLaunchedFlow |
| Status | Active |

### Entry conditions

- `OwnerId` `IsChanged` true (only fires on owner change for updates).
- `Owner:Queue.QueueName` equals `<Target Queue Name>` (this is the trick — use the cross-object reference to read the Queue's name).
- `Email` is not blank (guard against records without an email address).

### Elements

1. **Decision** — `Check_Is_Queue`: confirms the owner is the target Queue and the email is not blank (defensive — entry conditions already cover this; the decision keeps the flow readable).
2. **Action** — Send Email using the email template; recipient is the record's email address.

### Behavior

- **Insert** with the target Queue as owner -> sends.
- **Update** that flips ownership to the target Queue -> sends.
- **Update** that does not change owner -> does not send (entry condition).
- **No email on the record** -> guard exits cleanly.

## Deploy

```bash
sf project deploy start \
  --source-dir force-app/main/default/letterhead \
  --source-dir force-app/main/default/email \
  --source-dir force-app/main/default/flows/<Object>_<Trigger>.flow-meta.xml \
  --target-org <alias>
```

Deploy the letterhead first (the email template references it), then the email folder + template, then the flow.

## QA

- **MT-01:** Create a record assigned to the target Queue with an email address — confirm the email is received.
- **MT-02:** Create a record assigned elsewhere — confirm no email.
- **MT-03:** Update a record's owner to the target Queue — confirm the email is sent.
- **MT-04:** Update a record's owner to the target Queue when `Email` is blank — confirm no email and no error.
