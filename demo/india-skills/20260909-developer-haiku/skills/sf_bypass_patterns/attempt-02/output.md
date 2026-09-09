**Fixed bypass patterns in LeadHandler:**

**Issues corrected:**
1. **Nullable Boolean (Line 2):** Changed `public static Boolean bypass;` to `@TestVisible private static Boolean bypassForTest = false;` — uninitialized null Boolean would cause runtime errors
2. **No custom permission:** Introduced `isBypassed()` method with named custom permission `Lead_Update_Country_Bypass` instead of hardcoded logic
3. **Inefficient SOQL:** Moved query outside loop, bulk-fetched all records

**Artifacts saved:**
- `/home/tester/work/output/LeadHandler.cls` — Handler with proper initialization and custom permission check
- `/home/tester/work/output/Lead_Update_Country_Bypass.customPermission-meta.xml` — Named permission for admin control
- `/home/tester/work/output/Lead_Update_Country_Bypass.permissionset-meta.xml` — Least-privilege permission set
- `/home/tester/work/output/review.md` — Evidence summary

Custom permission allows admins to grant bypass to integration users without code changes, survives org refreshes, and is easily testable via permission-set assignment.