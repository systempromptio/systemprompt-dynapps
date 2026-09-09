# Lead Assignment Rule Re-execution QA Cases

## AT-01: Insert Lead Routes to Assignment Rule User
**Precondition:** Active assignment rule routes Leads with Country='India' to User INDIA_SALES_QUEUE  
**Steps:**
1. Insert Lead with Country='India', required fields (LastName, Company, Phone)
2. Verify OwnerId equals INDIA_SALES_QUEUE Id

**Expected Result:** Lead is assigned per active rule on insert  
**Pass/Fail:** [   ]

---

## AT-02: Country Change Triggers Re-assignment via Invocable Flow
**Precondition:** Lead exists with Country='USA', OwnerId=USA_SALES_QUEUE. Active rule routes Country='India' to INDIA_SALES_QUEUE.  
**Steps:**
1. Update Lead Country from 'USA' to 'India' via inline API call (no assignment rule UI checkbox)
2. Trigger Lead_After_Save_Automation flow manually or via Platform Event
3. Flow calls LeadAssignmentInvocable.runAssignmentRules with Record.Id
4. Verify OwnerId changed to INDIA_SALES_QUEUE

**Expected Result:** OwnerId re-routed to INDIA_SALES_QUEUE per new Country  
**Pass/Fail:** [   ]

---

## AT-03: Non-Key-Field Update Does Not Trigger Re-assignment
**Precondition:** Lead assigned to USA_SALES_QUEUE, assignment rule keys on Country field only  
**Steps:**
1. Update Lead.Description (field not in rule logic)
2. Flow entry condition guards: only run when Country changed (use `$Record.Country != $Record__Prior.Country`)
3. Verify flow does not execute

**Expected Result:** OwnerId remains unchanged; no re-assignment  
**Pass/Fail:** [   ]

---

## AT-04: Bulk Update — Multiple Country Changes
**Precondition:** 50 Leads with mixed Countries; assignment rules active  
**Steps:**
1. Bulk update 50 Leads changing Country via API (no UI checkbox)
2. Apex batch or async job calls LeadAssignmentInvocable with all 50 IDs
3. Verify all 50 Leads re-assigned per new Country

**Expected Result:** All 50 Leads updated in single Database.update call; all routed correctly  
**Pass/Fail:** [   ]

---

## AT-05: Recursion Guard — No Infinite Loop
**Precondition:** Flow has entry condition; static flag prevents recursive re-run  
**Steps:**
1. Insert Lead; triggers assignment rule → Lead has OwnerId
2. Flow: Check if `$Record.OwnerId != null AND $Record.Country != $Record__Prior.Country`
3. Verify flow runs exactly once per Country change, not repeatedly

**Expected Result:** Single assignment rule execution; no recursion error  
**Pass/Fail:** [   ]

---

## AT-06: Validation Rules Satisfied During Re-assignment
**Precondition:** Active validation rule requires Country is not null  
**Steps:**
1. Insert Lead with Country='India'
2. Database.update via LeadAssignmentInvocable re-executes rule
3. Verify Lead passes validation (Country != null)

**Expected Result:** Update succeeds; validation errors are NOT thrown  
**Pass/Fail:** [   ]
