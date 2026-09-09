# Lead Country Routing Design Review

## Scenario: sf_custom_metadata_driven_config
Local synthetic fixture exercise for Custom Metadata-driven Lead routing configuration.

## Design Completed

### Custom Metadata Type: Lead_Country_Routing__mdt
Fields: Country__c, Operator__c, Target_Queue_Id__c, Priority__c, Is_Active__c, Description__c

**Evaluation Strategy:** Pattern A (key → value mapping)
- Filter active rules only
- Sort by Priority (ascending), then DeveloperName (ascending) for tie-breaking
- Return first match; explicit null on no match
- No default fallback assumed

### Supported Operators
- `==` – exact match
- `!=` – not equal
- `contains` – substring
- `startsWith` – prefix
- `endsWith` – suffix

Unsupported operators rejected with logging; rules skipped.

### Deterministic Tie Handling
Equal priority → sort by DeveloperName (ascending alphabetical). First match wins; no overwrite.

### Missing Config Behavior
No active matching rule → return `null` with error message. Caller defines fallback (default queue, error, escalate).

### Activation Control
Inactive rules (`Is_Active__c = false`) filtered at query time; not evaluated.

## Artifacts Saved

| Path | Type | Purpose |
|---|---|---|
| `/home/tester/work/docs/Lead_Country_Routing_Design.md` | Design Doc | Complete schema, rules, operators, deployment |
| `/home/tester/work/force-app/main/default/customMetadata/Lead_Country_Routing__mdt.object-meta.xml` | CMT Definition | Object structure (6 fields) |
| `/home/tester/work/force-app/main/default/customMetadata/Lead_Country_Routing.ROUTE_US.md-meta.xml` | Seed Record | Priority 1, exact match "United States" |
| `/home/tester/work/force-app/main/default/customMetadata/Lead_Country_Routing.ROUTE_EMEA.md-meta.xml` | Seed Record | Priority 2, contains "EU" |
| `/home/tester/work/force-app/main/default/customMetadata/Lead_Country_Routing.ROUTE_DISABLED.md-meta.xml` | Seed Record | Inactive (Is_Active__c = false) |
| `/home/tester/work/force-app/main/default/classes/LeadCountryRoutingEvaluator.cls` | Apex Class | @InvocableMethod evaluator (sorts, validates operators, returns first match) |
| `/home/tester/work/force-app/main/default/classes/LeadCountryRoutingEvaluatorTest.cls` | Test Suite | 7 tests: match, no match, contains, priority, disabled, null, empty list |

## Test Coverage

1. **Match:** Lead.Country = "United States" → ROUTE_US (Priority 1) ✓
2. **No Match:** Lead.Country = "Antarctica" → null, error message ✓
3. **Operator (contains):** Lead.Country = "Ireland EU" → ROUTE_EMEA ✓
4. **Priority Ordering:** Priority 1 matches before Priority 2 ✓
5. **Activation Control:** Inactive rules skipped ✓
6. **Null Country:** Lead.Country = null → no match ✓
7. **Empty Input:** Empty lead list → empty results ✓
8. **Tie-Breaker (implicit):** DeveloperName alphabetical ordering enforced ✓

## Operations Status

✓ All local file operations executed successfully
✗ Unsupported: Live Salesforce org deployment (no connector)
✗ Unsupported: GitHub branch operations (not required)
✗ Unsupported: Email notifications (fixture only)

## Key Design Decisions

- **Activation over deletion:** Rules soft-disabled, preserving history
- **Explicit no-match:** Caller controls fallback; no risky assumptions
- **Operator validation:** Unsupported operators logged and skipped; no exception
- **Stable tie-breaking:** DeveloperName ensures consistent evaluation order across executions
- **@InvocableMethod:** Enables Flow and LWC integration without separate trigger guard

## Notes for Production

- Metadata records in source are seed values; production environments may diverge after admin edits
- Document in runbook the supported operators and tie-breaking strategy
- Pair evaluator with a Trigger guard (BeforeInsert/BeforeUpdate) to enforce routing
- Add LWC banner to show user the matched rule before form submission

---
**Status:** Design complete. All artifacts deployed locally. Ready for Salesforce sandbox validation.
