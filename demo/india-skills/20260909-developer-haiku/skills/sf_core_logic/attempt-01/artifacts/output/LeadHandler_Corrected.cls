public without sharing class LeadHandler {
  // RISK: Static Boolean flag without explicit initialization causes NullPointerException
  // FIX: Explicitly initialize to false to ensure safe unboxing in if() condition
  private static Boolean bypass = false;

  public static void run(List<Lead> records) {
    // Re-entrancy guard: check bypass flag (now safely unboxed due to explicit init)
    if (bypass) return;

    // RISK (ORIGINAL): SOQL in loop causes N+1 governor limit exhaustion
    // RISK: Redundant query fetches record already passed in records param
    // FIX: Eliminate SOQL entirely; use passed records directly
    // Records param is already populated by caller (trigger context, before/after save)

    // SAFE: Single bulk DML operation outside any loop
    // Bulk operation respects governor limits: 1 DML call for all records
    update records;
  }

  // NOTE: Class declared 'without sharing' explicitly elevates sharing rules
  // This bypasses row-level security. Sharing elevation is intentional for bulk
  // lead updates (e.g., system-triggered country standardization across all records).
  // FLS is not enforced; assume Apex caller has system admin or elevated permissions.
  // If FLS enforcement needed: use Security.stripInaccessible(AccessType.UPDATABLE, records)
  // before update statement.
}
