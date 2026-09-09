> Astound marketplace adaptation: Historical upstream project template, not Claude Code installation instructions or facts about your org. Cursor paths and agent rosters describe the source kit and are not installed by this marketplace. Use the installed sf-salesforce-project-context skill and the current repository instructions for this client.

# Project Schema

> **TEMPLATE.** This file is the **single source of truth** for the project's custom objects and fields. Fill it in as the project grows. Every PR that adds or changes a custom object / field **must** update this file in the same change.

The agent reads this file before generating any metadata, Apex, LWC, or Flow that touches custom objects so it uses the exact API names that exist in the org.

---

## How to use this file

1. List **only** custom objects and custom fields that this project owns (do not duplicate the Salesforce standard schema).
2. For each object, list its custom fields with API name, type, length / picklist values, and a one-line purpose.
3. List relationships (lookups, master-details, metadata relationships).
4. List record types and their purpose.
5. Keep API names exactly as they appear in metadata (case-sensitive `__c`, `__mdt`, `__r` suffixes).

---

## Custom objects

### `<Object_Api_Name__c>` — `<Friendly Label>`

**Purpose:** _<one-line business purpose>_

**Sharing model:** _<Public Read/Write | Public Read Only | Private | Controlled by Parent>_

**Record types:**

| Developer Name | Label | Purpose |
|---|---|---|
| `<RT_Dev_Name>` | `<Label>` | _<purpose>_ |

**Custom fields:**

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|
| `<Field__c>` | `<Label>` | Text / Picklist / Number / Lookup / Formula / etc. | _<length, precision, picklist values, related object>_ | _<one-line purpose>_ |

**Relationships:**

| API Name | Type | Related Object | Cascade |
|---|---|---|---|
| `<Lookup_Field__c>` | Lookup | `<Other_Object__c>` | Set Null on delete |

---

## Custom fields on standard objects

Where this project adds custom fields to standard objects, list them per object.

### `Lead`

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|
| `<Field__c>` | `<Label>` | _<type>_ | _<details>_ | _<purpose>_ |

### `Account`

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|

### `Contact`

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|

### `Opportunity`

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|

### `Case`

| API Name | Label | Type | Length / Values | Purpose |
|---|---|---|---|---|

---

## Custom Metadata Types (configuration)

| API Name | Purpose | Key fields |
|---|---|---|
| `<Cmt_Name__mdt>` | _<what configuration this drives>_ | `<Field1__c>`, `<Field2__c>`, ... |

For each CMT, list the **records** that ship in source (seed values) — note where admins are expected to manage these in production.

---

## Custom Permissions

| API Name | Used by | Purpose |
|---|---|---|
| `<Object_Action_Bypass>` | _<Apex class / Flow that checks it>_ | _<bypass scope>_ |

---

## Last updated

_<YYYY-MM-DD>_
