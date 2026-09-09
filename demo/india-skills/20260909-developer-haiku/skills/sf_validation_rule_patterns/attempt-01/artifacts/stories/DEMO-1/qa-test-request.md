# QA Test Request — DEMO-1

## Validation Rule: Email_Required_When_Qualified

**Purpose:** Enforce Lead.Email population when Status transitions to Qualified.

**Formula:**
```
AND(
    ISPICKVAL(Status, "Qualified"),
    ISBLANK(Email)
)
```

**Error Configuration:**
- **Error Display Field:** Email
- **Error Message:** "Email is required when Lead status is Qualified. Please enter a valid email address."

---

## Manual Test Cases

### MT-01: Positive — Email populated, Status = Qualified (PASS)
- **Precondition:** Lead with FirstName=Test, LastName=Lead, Company=TestCo
- **Action:** Set Status → Qualified, Email → test@example.com
- **Expected:** Record saves without validation error

### MT-02: Positive — Email populated, Status = Open - Not Contacted (PASS)
- **Precondition:** Lead with FirstName=Test, LastName=Lead, Company=TestCo
- **Action:** Set Status → Open - Not Contacted, leave Email empty
- **Expected:** Record saves without validation error

### MT-03: Negative — Email blank, Status = Qualified (FAIL)
- **Precondition:** Lead with FirstName=Test, LastName=Lead, Company=TestCo
- **Action:** Set Status → Qualified, leave Email blank
- **Expected:** Validation error fires on Email field: "Email is required when Lead status is Qualified. Please enter a valid email address."

### MT-04: Negative — Email blank, transition to Qualified (FAIL)
- **Precondition:** Lead with Status = Open - Not Contacted, Email empty
- **Action:** Update Status → Qualified (trigger status change)
- **Expected:** Validation error fires on Email field blocking the transition
