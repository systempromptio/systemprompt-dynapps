# Deployment Runbook: Lead Validation & Handler

**Source API Version:** 64.0  
**Manifest:** `manifest/sf-devops-package.xml`  
**Components:** ValidationRule (LeadCountryValidation), ApexClass (LeadHandler + LeadHandlerTest)

## Deployment Sequence

1. **Validate deployment** (no tests): Manifest syntax, XML well-formedness, API version compatibility
2. **Deploy to sandbox** with `--test-level RunSpecifiedTests` for LeadHandlerTest
3. **Run apex test**: `LeadHandlerTest` with code coverage (expect ≥75%)
4. **Production deploy** with `--test-level RunSpecifiedTests` after sandbox sign-off

## Test Selection & Ordering

- **Unit Test:** LeadHandlerTest — covers bypass logic, Country field update, SOQL binding
- **Integration test:** Manual smoke test in sandbox: create Lead with Country, trigger handler via flow/quick action, verify update succeeds
- **Validation rule test:** Lead.LeadCountryValidation blocks invalid Country ISO codes

## Rollback Strategy

1. **Pre-deploy:** Snapshot current LeadHandler class (git tag or retrieve)
2. **Post-deploy failure:** Revert manifest to prior version; deploy previous ApexClass + ValidationRule
3. **Runtime issue (bypass needed):** Set `LeadHandler.bypass = true` via Execute Anonymous in sandbox; promote hotfix

## Evidence & Validation

- ✓ Manifest created at `manifest/sf-devops-package.xml`
- ✓ Test class selection: LeadHandlerTest (RunSpecifiedTests mode)
- ✓ No test-level conflicts for sandbox deployment
- ✓ Rollback via prior tag/branch snapshot available
- ✓ Deployment errors fixture resolved: LeadHandler class exists, validation rule added

**Deployment command (sandbox validation):**
```
sf project deploy start \
  --manifest manifest/sf-devops-package.xml \
  --target-org sandbox \
  --test-level RunSpecifiedTests \
  --tests LeadHandlerTest \
  --wait 15 \
  --json
```
