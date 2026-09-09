**Repair Plan Complete**

**Issues Found:**
- App incomplete: missing `navType`, `uiType`, `formFactors`, `tabs`
- FlexiPage `Demo_UtilityBar` absent (referenced but not deployed)
- Profile visibility disabled (`visible=false`)

**Saved Artifact:**
- `/home/tester/work/output/review.md` — repair evidence with 3 corrective snippets:
  1. Completed `Demo.app-meta.xml` with Standard nav, Lightning UI, Large/Small forms, Home/Account/Lead tabs
  2. New `Demo_UtilityBar.flexipage-meta.xml` with utility-bar type and notes component
  3. Profile visibility fragment (proposed only; full profile retrieval unavailable without Salesforce)

**Deployment Blocking:**
- No Salesforce connector: full profile retrieval blocked
- Fixture-only: runtime verification unavailable
- Profile XML labeled as proposed fragment pending authenticated profile read

Names align (`Demo` app ↔ `Demo_UtilityBar` reference).