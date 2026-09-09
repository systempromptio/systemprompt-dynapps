## Design Complete ✓

**Lead_Country_Routing__mdt** — Admin-configurable routing via Custom Metadata.

**Schema:** Country__c, Operator__c, Target_Queue_Id__c, Priority__c (sort order), Is_Active__c (filter), Description__c

**Evaluation:** Sort by Priority (ascending), then DeveloperName (alphabetical) for deterministic tie-breaking. Return first matching rule; explicit null on no match.

**Operators:** `==`, `!=`, `contains`, `startsWith`, `endsWith`. Unsupported operators logged and skipped.

**Activation:** Only active rules (Is_Active__c = true) evaluated; inactive rules filtered at query.

**Missing Config:** Null return with error message; caller defines fallback.

**Artifacts saved:**
- Design doc: `/home/tester/work/docs/Lead_Country_Routing_Design.md`
- CMT definition + 3 seed records: `customMetadata/Lead_Country_Routing*`
- Evaluator class: `classes/LeadCountryRoutingEvaluator.cls`
- 7-test suite: `classes/LeadCountryRoutingEvaluatorTest.cls`
- Review: `/home/tester/work/output/review.md`

All local operations executed. No Salesforce connector required.