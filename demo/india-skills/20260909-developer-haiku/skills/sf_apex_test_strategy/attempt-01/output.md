**Test Strategy Created**

Saved artifact: `/home/tester/work/output/review.md`

**Three Scenarios Completed:**

1. **LWC-Only PR:** `RunSpecifiedTests` (impacted only) — no Apex changes, fast feedback
2. **Trigger/Sharing Change:** `RunLocalTests` (full suite) — system-wide impact requires comprehensive coverage with TestDataFactory validation rule sweep
3. **Production Promotion:** `RunLocalTests` + curated smoke subset — mandatory full run + critical-path regression tests, with performance delta tracking

**Evidence Retained:** test logs, coverage reports (≥95% on changed classes), performance baseline, failing diagnostics linked to stories, deployment sign-off.

**Fixture Requirements:** TestDataFactory for Account/Contact/Opportunity defaults, validation rule compliance, user role hierarchy, legacy data edge cases.

Artifact saved locally. No unsupported operations encountered.