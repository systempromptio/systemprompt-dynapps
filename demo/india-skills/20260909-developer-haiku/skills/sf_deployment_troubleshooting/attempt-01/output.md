**Repair Runbook Complete**

Diagnosed 5 deployment errors from the synthetic fixture. Root causes: missing flow metadata, orphaned UtilityBar reference, profile visibility misconfiguration, unmet permission dependencies, and a hanging flow screen.

**Repair sequence** (dependency-ordered):
1. Deploy Flow `Lead_Review` with fixed screen config
2. Create UtilityBar `Demo_UtilityBar`
3. Add prerequisite permissions to PermissionSet
4. Retrieve and fix Profile `applicationVisibilities` to `true`
5. Deploy QuickAction (now all dependencies exist)

**Verification:** Manifest validation, test deploy with `--test-level NoTestRun`, confirm app launcher visibility and flow completion.

**Saved:** `/home/tester/work/output/review.md` — includes rollback strategy and evidence summary. No connectors invoked (local synthetic exercise only).