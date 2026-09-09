**Artifact saved**: `/home/tester/work/output/review.md`

**Deliverables**:

1. **Lead Field Section** — Country field in FlexiPage header, `uiBehavior=readonly`, visible only when `Status="Open - Not Contacted"` via `visibilityRule` equality check.

2. **Conditional Quick Action** — `Lead.Qualify_Lead` in page layout, visibility filtered on same Status condition.

3. **Metadata Snippets** — XML following Patterns A (read-only field) and D (visibility filters) from skill guidance.

4. **Permission QA Cases** (4 scenarios) — Status visibility, FLS enforcement, feature gate check. All marked **NOT RUN** per fixture constraints.

5. **Unsupported Operations**:
   - Dynamic Actions org enablement unverifiable without Salesforce connector.
   - Quick action existence assumed; action deployment depends on org.

**Schema used**: Fixture Status values ("Open - Not Contacted" | "Qualified"), Lead standard fields. No record types, users, or Dynamic Actions support known.