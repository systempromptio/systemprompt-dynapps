**Exercise Complete.**

### Artifacts Created

1. **Validation Rule Metadata:** `/home/tester/work/output/Email_Required_When_Qualified.validationRule-meta.xml`
   - Formula: `AND(ISPICKVAL(Status, "Qualified"), ISBLANK(Email))`
   - Error attached to Email field
   - Message: "Email is required when Lead status is Qualified. Please enter a valid email address."

2. **QA Test Cases:** `/home/tester/work/stories/DEMO-1/qa-test-request.md`
   - **Positive:** MT-01 (email populated, status qualified), MT-02 (status open, email blank)
   - **Negative:** MT-03 (blank email on qualified), MT-04 (transition to qualified without email)

3. **Evidence Summary:** `/home/tester/work/output/review.md`
   - Design notes, schema alignment, formula rationale

**Design:** Follows Pattern 1 (status-triggered required field). Uses standard Lead.Email and Lead.Status fields. Error message actionable. No unsupported operations.