# Deployment Repair Runbook

**Scenario:** India Skills Fixture (Synthetic) — Sales Cloud Lead Review
**API Version:** 64.0
**Date:** 2026-09-09

---

## Diagnosed Errors

1. QuickAction `Lead.Review` references flow `Lead_Review` (not found)
2. CustomApplication `Demo` references `Demo_UtilityBar` (missing)
3. Profile: `applicationVisibilities` has `visible=false` for demo app
4. PermissionSet: missing prerequisite permissions
5. Flow screen: neither `allowBack` nor `allowFinish` enabled (flow will hang)

---

## Repair Sequence (Dependency-Ordered)

### Phase 1: Foundation (Deploy First)
**1. Create Flow `Lead_Review`**
- File: `force-app/main/default/flows/Lead_Review.flow-meta.xml`
- Fix: Add missing screen elements with `allowFinish=true`, remove hanging state
- Verify: `sf project list metadata --metadata Flow:Lead_Review --target-org <alias>`
- Rollback: Delete file, redeploy

**2. Create UtilityBar `Demo_UtilityBar`**
- File: `force-app/main/default/flexipages/Demo_UtilityBar.flexipage-meta.xml`
- Fix: Add to CustomApplication metadata
- Verify: App launcher shows utility bar
- Rollback: Remove file reference

### Phase 2: Dependencies
**3. Deploy PermissionSet with Prerequisites**
- File: `force-app/main/default/permissionsets/CustomSet.permissionset-meta.xml`
- Fix: Add implied permissions (e.g., if `ManageUsers`, add `DelegatedTwoFactor`, `MonitorLoginHistory`)
- Verify: `sf project deploy start --manifest manifest/fix-perms.xml --test-level NoTestRun`
- Rollback: Restore prior version

**4. Fix Profile Application Visibility**
- Retrieve: `sf project retrieve start --metadata Profile:Admin --target-org <alias>`
- Fix: Set `<applicationVisibilities><visible>true</visible></applicationVisibilities>` for Demo app
- Deploy: `sf project deploy start --metadata Profile:Admin --test-level NoTestRun`
- Rollback: Restore retrieved profile

### Phase 3: Integration
**5. Deploy QuickAction (now dependencies exist)**
- File: `force-app/main/default/quickActions/Lead.Review.quickAction-meta.xml`
- Verify: Action visible, flow `Lead_Review` resolves without errors
- Rollback: Delete file

---

## Verification Checklist

- [ ] Flow `Lead_Review` deployed, screen allows finish
- [ ] UtilityBar `Demo_UtilityBar` created and referenced
- [ ] Profile app visibility set to `true`
- [ ] PermissionSet includes all prerequisite permissions
- [ ] QuickAction points to valid flow
- [ ] Test deploy: `--test-level NoTestRun` (no unrelated failures)
- [ ] Sandbox validation passes

---

## Rollback Strategy

**Full Rollback:** Revert manifest to prior committed version, redeploy.
**Partial Rollback:** Remove Phase 3 (QuickAction), redeploy remaining components in reverse order.

---

## Evidence Summary

- **5 errors identified** from deployment-errors.txt
- **Dependency chain:** Flow → UtilityBar, Profile, PermissionSet → QuickAction
- **Root cause:** Missing metadata + incomplete flow screen config + permission dependency chain
- **Repair approach:** Deploy base components first, profile/perms second, actions last
- **No Salesforce/GitHub connectors invoked** (synthetic local exercise)
