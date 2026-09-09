# Validation Rule Exercise — Evidence Summary

## Scenario: Email_Required_When_Qualified

**Task:** Create metadata for requiring Lead.Email when Status is Qualified, attach error to Email field, add positive/negative MT cases.

---

## Artifacts Created

### 1. Validation Rule Metadata
**Path:** `/home/tester/work/output/Email_Required_When_Qualified.validationRule-meta.xml`

**Content:**
- **Rule Name:** `Email_Required_When_Qualified`
- **Active:** Yes
- **Error Condition Formula:** Pattern 1 (Required field when status transitions)
  ```
  AND(ISPICKVAL(Status, "Qualified"), ISBLANK(Email))
  ```
- **Error Display Field:** `Email` (field-level attachment per skill convention)
- **Error Message:** "Email is required when Lead status is Qualified. Please enter a valid email address."
  - Message is actionable: tells user what to fix and why

---

### 2. QA Test Request with Manual Test Cases
**Path:** `/home/tester/work/stories/DEMO-1/qa-test-request.md`

**Coverage:**

**Positive Cases (PASS):**
- MT-01: Email populated, Status = Qualified → no error
- MT-02: Email blank, Status = Open - Not Contacted → no error (rule inactive)

**Negative Cases (FAIL):**
- MT-03: Email blank, Status = Qualified → error fires on Email field
- MT-04: Status transition to Qualified without Email → error blocks save

---

## Validation Rule Design Notes

- **Schema Alignment:** Lead.Status picklist supports "Qualified" value per PROJECT_SCHEMA.md
- **Formula Pattern:** Pattern 1 (status-triggered required field) from skill authoring guide
- **Field Mapping:** Error attached to `Email` (the failing field), not top-of-page
- **No New Fields Invented:** Uses standard Lead.Email and Lead.Status fields

---

## Operations Status

✅ Metadata XML created  
✅ QA test cases added  
✅ Evidence summary generated  
❌ No Salesforce org or GitHub writes performed (fixture exercise, local only)
