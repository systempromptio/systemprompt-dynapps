**Design Complete.** Saved comprehensive Lead conversion design to `/home/tester/work/output/review.md`.

**Deliverables:**
- **Field Mapping:** Lead ID traceability (formula → text), Company/Contact/Opportunity bindings.
- **Repeat-Conversion:** Standard `IsConverted` block; audit trail optional if re-conversion allowed.
- **Required Values:** Company, Email, FirstName + LastName; Status picklist validation via CMT.
- **Assignment:** Leads pre-assigned by rules; Accounts/Contacts inherit none; Opportunities mapped by CMT (region/product).
- **QA:** 5 scenarios covering success, missing fields, duplicates, no-opp flag, non-converted paths.
- **Metadata Gaps:** Formula field, text field, LeadConvertSettings XML, CMT rule engine, Apex trigger handlers, profiles/FLS, page layouts.

All operations completed locally; no Salesforce connector or external calls required.