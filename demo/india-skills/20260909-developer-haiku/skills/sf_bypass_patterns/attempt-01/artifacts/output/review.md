# Bypass Patterns Scenario Review

## Issues Found

**LeadHandler.cls (Original)**
- Line 2: Nullable static Boolean `bypass` causes null-reference exception in `if (bypass)` check
- No custom permission framework—hardcoded flag is not admin-manageable

## Corrections Applied

1. **LeadHandler.cls (Corrected)**
   - Renamed to `bypassForRecursion` with `@TestVisible` and initialized to `false` (line 2)
   - Added `isBypassed()` guard method using `FeatureManagement.checkPermission('Lead_Country_Update_Bypass')`
   - Wrapped core logic in try/finally to ensure recursion flag reset
   - Test bypass: check `Test.isRunningTest() && bypassForRecursion` before permission check

2. **Custom Permission Metadata**
   - Created: `Lead_Country_Update_Bypass.customPermission-meta.xml`
   - Assignable to users and integration accounts without code changes

3. **Permission Set (Least-Privilege)**
   - Created: `Lead_Country_Update_Bypass.permissionset-meta.xml`
   - Contains only the custom permission, no other privileges

## Artifacts Saved
- `/home/tester/work/output/LeadHandler.cls`
- `/home/tester/work/output/Lead_Country_Update_Bypass.customPermission-meta.xml`
- `/home/tester/work/output/Lead_Country_Update_Bypass.permissionset-meta.xml`
