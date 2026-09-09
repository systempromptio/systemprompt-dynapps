**Design complete.** Saved to `/home/tester/work/output/review.md`:

✓ **Lead_Country_Routing__mdt** Custom Metadata Type with 6 fields (Country_Code, Queue, Priority, IsActive, Description, DeveloperName)  
✓ **Priority-based tie resolution** — lowest priority number wins; secondary alphabetical sort  
✓ **Operator validation** — regex for ISO country codes, queue existence check, positive priority  
✓ **Activation control** — IsActive__c checkbox for admin management without deletion  
✓ **Missing-config behavior** — defaults to `Default_Sales_Queue`; logs exceptions for audit  
✓ **Sample records** — US, GB, ZZ (fallback) with varying priorities  
✓ **Apex evaluator** — `@InvocableMethod` for Flow integration with sorted, cached lookup  

Pattern A (mapping) extended with admin controls. Follows skill conventions: `Domain_What__mdt` naming, DeveloperName stability, deployment strategy documented.