# Apex Test Strategy: Three Scenarios

## Scenario 1: LWC-Only PR Changes
**Test Level:** `RunSpecifiedTests` (impacted only)
**Rationale:** No Apex, no triggers, no security model changes.
**Fixture Requirements:**
- Mock static resources for UI component imports
- Mock callout responses for wired adapters
- No TestDataFactory changes needed

**Evidence to Retain:**
- LWC Jest/Vitest coverage report
- Component screenshot (golden path + error states)
- No Apex test runs required

---

## Scenario 2: Trigger & Sharing Model Change
**Test Level:** `RunLocalTests` (mandatory full suite)
**Rationale:** Trigger and sharing changes affect system-wide data integrity.
**Fixture Requirements:**
- TestDataFactory: Accounts, Contacts, Opportunities with all required fields
- Validation rule compliance for Account > 10 required fields
- RecordTypeId permutations for sharing tests
- User setup with role hierarchy
- Custom permission mocks for feature flags

**Test Classes to Cover:**
- `TriggerHandlerTest` — before/after logic, recursion guards
- `SharingRuleTest` — org-wide defaults, manual sharing, role hierarchy
- `ValidationTest` — trigger + validation rule interactions
- `DataCleanupTest` — account merge, delete cascade

**Evidence to Retain:**
- Test log output (line coverage ≥95% on trigger handler)
- Failing test diagnostics linked to story
- Before/after sharing audit log
- Performance baseline (bulk insert latency)

---

## Scenario 3: Production Promotion (UAT → Prod)
**Test Level:** `RunLocalTests` + **Smoke Test Subset**
**Rationale:** Always run full suite; curated smoke tests catch regressions.
**Fixture Requirements:**
- All TestDataFactory scenarios from Scenario 2
- Prod cloned validation rules & custom settings
- Legacy data edge cases (null fields, orphaned records)
- Pre/post-deployment compare scripts

**Smoke Test Classes (curated):**
- `AccountTriggerSmokeTest` — happy path account creation & update
- `OpportunitySharingSmoke` — sales user sees only their opps
- `BatchJobSmoke` — scheduled batch completes < 2 min
- `IntegrationSmoke` — callout to ERP succeeds with retry

**Evidence to Retain:**
- Full test run output as pipeline artifact
- Coverage delta (per-class) vs previous version
- Performance impact report (CPU time, query rows)
- Deployment checklist & sign-off
- Rollback plan documented if >95% failure rate

---

## Test Data Factory Pattern
```apex
// Centralized defaults for all test classes
TestDataFactory.createAccount(/* overrides */);
TestDataFactory.createContact(accountId, /* overrides */);
```
**Update Rule:** When a new validation rule ships, sweep all factory calls in same PR.

---

## Summary
| Scenario | Test Level | Run Time | Safety | Key Artifact |
|----------|-----------|---------|--------|--------------|
| LWC PR | Specified | ~2 min | Low risk | Component screenshot |
| Trigger PR | LocalTests | ~10 min | High risk | Coverage report + diagnostics |
| Prod Promotion | LocalTests + Smoke | ~15 min | Critical | Performance delta + sign-off |
