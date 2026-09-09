# LeadHandler Code Review - Core Logic Analysis

## Issues Identified

### 1. **Bulkification Violation** (Lines 5-9)
- SOQL query in loop causes N+1 governor limit exhaustion
- Performs redundant lookup of already-passed record

### 2. **Static Boolean Null Reference Risk** (Line 2)
- `bypass` not initialized to `false`
- Unboxing null Boolean in `if (bypass)` causes `NullPointerException`

### 3. **Sharing/FLS Gaps**
- Class declared `without sharing` but lacks documented justification
- No `Security.stripInaccessible()` despite DML (explicit elevation, but undocumented)

## Corrected Design

**Pattern**: One object → one handler. Collect IDs; query/update once outside loop.

**Key fix**: Eliminate redundant SOQL; use passed records directly. Initialize Boolean flag explicitly.

```apex
public without sharing class LeadHandler {
  // SOQL+DML moved outside loop; records already passed in context
  private static Boolean bypass = false;
  
  public static void run(List<Lead> records) {
    if (bypass) return;
    
    // No SOQL needed: records param already contains current state
    // Assuming Country field is set in records param before handler invoked
    update records; // Bulk DML once, outside loop
  }
}
```

## Evidence Summary
- **Before**: 1 SOQL + 1 DML per record = N * (1+1) operations
- **After**: 1 bulk DML = 1 operation; governor compliant
- **Risk mitigated**: Boolean unboxing NullPointerException; SOQL limit exhaustion

