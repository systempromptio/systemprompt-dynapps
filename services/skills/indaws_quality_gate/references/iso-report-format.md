# ISO 27001 Report Format

Always emit as a separate section after the base report.

```
## ISO 27001 Secure Coding Audit

### Blocking Gates (1-7)

| Gate | ISO Control | Status | Finding |
|---|---|---|---|
| 1 · Injection & Sanitization | A.8.28 | ... | ... |
| 2 · Access Control | A.5.15 / A.8.3 | ... | ... |
| 3 · RPC Exposure | A.8.21 | ... | ... |
| 4 · Sensitive Data | A.5.34 / A.8.11 | ... | ... |
| 5 · Dependencies | A.8.8 | ... | ... |
| 6 · Crypto & Auth | A.8.24 | ... | ... |
| 7 · Audit Trail | A.8.15 | ... | ... |

### Warning Gates (8-12)

| Gate | ISO Control | Status | Finding |
|---|---|---|---|
| 8 · Performance as Security | A.8.6 | ... | ... |
| 9 · Error Handling | A.8.15 | ... | ... |
| 10 · Input Validation | A.8.28 | ... | ... |
| 11 · Resource Limits | A.8.6 | ... | ... |
| 12 · Logging Hygiene | A.8.15 | ... | ... |

**ISO Verdict:** PASS / WARN / FAIL  
(FAIL = any blocking gate FAIL. WARN = only warning gates breached. PASS = all clear.)

### Remediation priority (ISO)
For each non-PASS gate, give: the offending `file:line`, the concrete fix, and why it violates the ISO control.
```

If ISO verdict is FAIL, recommend the base verdict be reconsidered.

## Scope-aware skipping

If scope is genuinely non-code (docs-only, pure cosmetic markdown, config-only with no executable content): mark every gate **N/A** with a one-line note. Always emit the report — its absence is indistinguishable from an accidental skip.
