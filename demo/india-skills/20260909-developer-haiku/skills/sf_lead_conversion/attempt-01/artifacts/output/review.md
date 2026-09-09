# Lead Conversion Design — Synthetic Schema (API 64.0)

## Field Mapping

| Lead Field | Account Field | Contact Field | Opportunity Field | Type | Notes |
|---|---|---|---|---|---|
| `Lead_ID_Custom__c` | — | — | `Lead_ID__c` | Formula→Text(18) | Traceability: `CASESAFEID(Id)` |
| `Company` | `Name` | — | — | Text | Account lookup or creation |
| `FirstName` / `LastName` | — | `FirstName` / `LastName` | — | Text | Contact creation required |
| `Email` | — | `Email` | — | Email | Unique constraint check |
| `Phone` | — | `Phone` | — | Phone | Contact phone |
| `Country` | `BillingCountry` | — | — | Text | Account standard field |
| `Status` | — | — | — | Picklist | Validation before conversion |

## Repeat-Conversion Handling

**Standard SFDC behavior:** `Lead.IsConverted = true` prevents re-conversion. No custom guard needed unless:
- Requirement: Allow re-conversion after record merge or undelete.
- **Action:** Use `LeadTrigger` to detect `IsConverted` update; log audit trail; block if Account/Contact/Opportunity record exists.

## Required Values Before Conversion

| Field | Validation |
|---|---|
| `Company` | Non-blank (triggers Account creation) |
| `Email` | Non-blank; deduplicate against Contact.Email |
| `FirstName` + `LastName` | Both non-blank for Contact creation |
| `Status` | Must not be "Invalid" or "Do Not Convert" (custom CMT rule) |

## Assignment Implications

1. **Lead Assignment (Pre-Conversion):** Existing lead assignment rules apply; no change needed.
2. **Account Assignment:** None (Accounts created via conversion inherit no auto-assignment).
3. **Contact Assignment:** None (Contacts created via conversion inherit no auto-assignment).
4. **Opportunity Assignment:** Use `OpportunityTrigger.beforeInsert` + CMT mapping (Pattern C) to set owner based on Account region or product type.

## QA Scenarios

- **QA-01:** Convert Lead with all required fields → verify `Lead_ID__c` on Opportunity equals `CASESAFEID(Lead.Id)` (18 chars).
- **QA-02:** Attempt conversion with missing `Company` → assert error "Company required".
- **QA-03:** Duplicate email against existing Contact → assert merge or error per policy.
- **QA-04:** Convert via Apex `LeadConvert` with `doNotCreateOpportunity=true` → verify Account/Contact created, Opportunity not.
- **QA-05:** Manually create Opportunity after Lead conversion → verify `Lead_ID__c` remains null (non-converted path).

## Org Metadata Still Needed

- **Custom fields:** `Lead.Lead_ID_Custom__c` (formula), `Opportunity.Lead_ID__c` (text), `Contact.Email` uniqueness setting.
- **LeadConvertSettings:** XML mapping Lead_ID_Custom__c → Lead_ID__c.
- **Custom Metadata Type:** `Lead_Conversion_Rules__mdt` for status validation.
- **Profiles/Permission Sets:** FLS read on Lead_ID_Custom__c; read+edit on Lead_ID__c (admins only).
- **Apex (missing):** `OpportunityTrigger`, `OpportunityTriggerHandler.beforeInsert`, `LeadConversionEvaluator` (CMT-driven rule engine).
- **Page Layouts:** Hide Lead_ID__c from Opportunity standard view; expose in System Information section for System Admin.

## Evidence Summary

Input files analyzed:
- `fixtures/project-brief.md`: Sales Cloud Lead review, API 64.0, synthetic only.
- `fixtures/LeadHandler.cls`: Current handler logic (without sharing) updates Country on Lead.
- `fixtures/deployment-errors.txt`: References missing Demo_UtilityBar, broken flows (not directly related to Lead conversion).
- `fixtures/interview.md`: Rounding rules (not lead-conversion-scoped).

**Unsupported Operations:** None (local fixture scenario, no Salesforce org connection required).
