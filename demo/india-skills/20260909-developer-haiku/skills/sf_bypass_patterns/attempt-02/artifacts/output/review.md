# Bypass Patterns Review — LeadHandler Refactor

## Issues Fixed

### 1. Nullable Static Boolean (Line 2)
**Original:** `public static Boolean bypass;`  
**Problem:** Uninitialized nullable Boolean causes null-reference runtime error in `if (bypass)`.  
**Fix:** Replaced with `@TestVisible private static Boolean bypassForTest = false;` — initialized to false, marked test-visible for unit testing.

### 2. Hardcoded Bypass Logic
**Original:** Direct static Boolean check without custom permission.  
**Problem:** No admin-configurable bypass mechanism; profile-based checks are inflexible.  
**Fix:** Introduced `isBypassed()` method that checks:
- Test-only override via `bypassForTest` (controlled in unit tests)
- Custom permission `Lead_Update_Country_Bypass` (admin-assigned via permission set)

### 3. Inefficient SOQL (Line 6)
**Original:** Query inside loop for each Lead record.  
**Fix:** Moved SOQL outside loop with Map bulk fetch; filter updates to changed records only.

## Artifacts Created

- **LeadHandler.cls** — Corrected handler with custom permission check and proper flag initialization
- **Lead_Update_Country_Bypass.customPermission-meta.xml** — Named custom permission for admin control
- **Lead_Update_Country_Bypass.permissionset-meta.xml** — Least-privilege permission set for granting bypass access

## Bypass Pattern Applied

✅ Custom Permission: Admin-assignable, no code changes needed to add bypassed users  
✅ Test Override: `bypassForTest = true` in test setup  
✅ Production Guard: Recursive call protection via method guard (not recursive here, but pattern applied)

All files saved to `/home/tester/work/output/`.
