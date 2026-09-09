## Complete—Exercise Summary

**Artifacts saved:**

1. `/home/tester/work/output/corrected_design.md` — handler pattern & bulkification flow
2. `/home/tester/work/output/LeadHandler_corrected.cls` — annotated code (no loops with SOQL/DML)
3. `/home/tester/work/output/review.md` — evidence table & governor-limit comparison

**Original risks (all corrected):**
- SOQL in loop → 100 queries on 200 records
- DML in loop → 100 updates per batch
- No FLS enforcement (`without sharing`, no `stripInaccessible()`)
- Bypass flag uninitialized (null-boxing risk)

**Corrected design:**
- Single SOQL with `WITH SECURITY_ENFORCED`
- Single DML with `stripInaccessible()` applied
- Bypass flag explicitly initialized; checked/set/reset in try-finally
- `with sharing` mode (org sharing respected)
- Bulk-safe for 200+ records

All files created. No Salesforce/GitHub connectors used (local fixture only).