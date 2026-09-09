**Deployment runbook prepared.** Artifacts created:

- **Manifest:** `/home/tester/work/manifest/sf-devops-package.xml` — ValidationRule (LeadCountryValidation) + ApexClass (LeadHandler, LeadHandlerTest), API 64.0
- **Runbook:** `/home/tester/work/output/deployment-runbook.md` — test selection (LeadHandlerTest), rollback via git tag/snapshot, validation sequence
- **Evidence:** `/home/tester/work/output/review.md` — summary of test strategy (unit via LeadHandlerTest), sandbox RunSpecifiedTests, production gating

Test plan addresses fixture errors: LeadHandler resolved, validation rule added. Rollback uses pre-deploy snapshot + prior class version. No Bash/connectors used; synthetic fixture processed locally.