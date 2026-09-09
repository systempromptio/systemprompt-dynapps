# Deployment Runbook: Lead Handler & Validation Rule (API 64.0)

## Scope
- **ApexClass:** LeadHandler (bulk handler; country field sync with bypass pattern)
- **ValidationRule:** Lead.Country_Validation (proposed; requires review)
- **CustomApplication:** Demo (missing utility page reference; requires correction)
- **Test Class:** LeadHandler_Test (new; code coverage required)

## Pre-Deployment Validation

**Manifest:** `manifest/lead-handler-64-package.xml`

**Validation Command:**
```bash
sf project validate start \
  --manifest manifest/lead-handler-64-package.xml \
  --target-org [sandbox-alias] \
  --test-level RunLocalTests \
  --wait 10
```

- **Test Level:** RunLocalTests (Apex handler + trigger logic requires local test execution)
- **Expected Coverage:** Minimum 75% on LeadHandler and bypass flow
- **Blocked Metadata:** CustomApplication utility bar reference — flag unresolved FlexiPage

## Deployment Sequence

1. **Apex Class Deployment** (LeadHandler)
   - Deploys with test class LeadHandler_Test
   - Confirms bypass static variable does not break existing logic
   - Validates SOQL query in run() method

2. **Validation Rule Deployment** (Lead.Country_Validation)
   - Deploys after Apex succeeds
   - Blocks invalid country codes (requires mapping)

3. **CustomApplication** (Demo)
   - Conditional: Deploy only after utility page FlexiPage is created
   - Currently marked as unresolved; mark `--ignore-conflicts` if pre-existing

## Rollback Plan

**If validation fails:**
- Rollback is automatic; no destructive changes issued
- No profiles or sharing rules modified

**If test class fails:**
- Revert LeadHandler.cls and re-run tests in org to isolate root cause
- Check bypass flag does not suppress required SOQL updates

**If production deploy fails:**
- Restore from pre-deploy snapshot
- Run `sf project deploy cancel --job-id [id]` if in progress

## Test Evidence Retention

- Store validation job ID and log: `deploy-validation-[timestamp].log`
- Store test code coverage report: `test-coverage-[timestamp].txt`
- Document any RunLocalTests skips with reason in deployment note

## Risks & Notes

- **Query Inefficiency:** LeadHandler performs [SELECT] inside loop; consider refactor post-release
- **Bypass Pattern:** Static bypass flag not guarded by profile-based checks; ensure admin-only access
- **Application:** Utility bar FlexiPage missing; resolve before demo rollout
- **API Version:** 64.0 (Jan 2025); no backwards-compatibility concerns for this metadata

## Sign-Off

Validation run: [pending — local fixture only]  
Deployment: [pending — local fixture only]  
Evidence: [not executed in synthetic environment]
