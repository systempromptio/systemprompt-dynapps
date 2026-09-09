# LeadHandler Corrected Design

## Pattern: After-Trigger Handler with Bulkification

**Handler signature:** `run(List<Lead> records)` called from `Lead` after-insert/update trigger.

### Key Corrections

1. **Bulkification:** Collect IDs in loop; execute SOQL once outside loop; collect updates; execute DML once.
2. **Authorization:** Apply `Security.stripInaccessible()` before DML to enforce field-level security.
3. **Sharing:** Change to `with sharing` to respect org's sharing rules (unless explicitly elevated with documented authorization).
4. **Recursion flag:** Explicitly initialize `bypass = false`; check, set, reset around DML to prevent infinite loops.

### Corrected Flow

```
1. Initialize bypass flag to false
2. Check bypass flag; return early if true
3. Set bypass to true (prevent re-entrancy)
4. Collect Ids from trigger records in list
5. Execute SOQL once: query all Lead records by Ids
6. Update Country field from trigger records (map or loop over results)
7. Apply Security.stripInaccessible() to enforce FLS
8. Execute update DML once
9. Set bypass to false (always, in finally block or after try)
```

### Governor Limit Safety

- **SOQL queries:** 1 query (not 100 in loop)
- **DML statements:** 1 update (not 100 in loop)
- **Records processed:** Bulk-safe for up to 200 records per transaction
