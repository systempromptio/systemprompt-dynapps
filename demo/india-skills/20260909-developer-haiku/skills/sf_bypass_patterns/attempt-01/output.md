## Complete

**Issues Identified & Fixed:**

1. **Nullable Static Flag** — Original `public static Boolean bypass;` risks null-reference errors. Corrected: `@TestVisible private static Boolean bypassForRecursion = false;` with explicit initialization.

2. **Hardcoded Bypass** — Replaced with named custom permission `Lead_Country_Update_Bypass` via `FeatureManagement.checkPermission()`, allowing admins to grant bypass without code changes.

3. **Re-entrancy Guard** — Wrapped update logic in try/finally to safely toggle recursion flag.

**Artifacts Saved:**
- `LeadHandler.cls` — Corrected handler with guard method
- `Lead_Country_Update_Bypass.customPermission-meta.xml` — Named permission
- `Lead_Country_Update_Bypass.permissionset-meta.xml` — Least-privilege permission set
- `review.md` — Evidence summary

All files written to `/home/tester/work/output/`.