# Execution Protocol

## STEP 0: Data Gathering
1. Display activation output message (see `references/activation-prompt.md`)
2. Use AskUserQuestion for any missing parameters
3. Validate inputs (paths exist, versions valid)
4. Summarize configuration before starting

## STEP 1: Module Scanning
Use `odoo-module-scanner` skill to analyze each module. Read the module inventory from its structured output.

## STEP 2: Classification
**Load the appropriate strategy file based on migration type:**

| Migration Type | Reference File |
|---------------|----------------|
| 1:1 | `references/migration-1to1.md` |
| Refactoring | `references/migration-refactor.md` |
| Refactoring_No_Studio | `references/migration-refactor-no-studio.md` |

Apply classification rules from the loaded strategy.

## STEP 3: Version Analysis
**Consult version references for Standard classification:**
- `references/odoo-versions/version-impact-summary.md` — effort multipliers by module type
- `odoo-version-api-reference` skill → `diffs/v{TARGET}-diff.md` — detailed API changes, breaking changes, code examples
- Source code (via symlinks if available) → `base/{module}/models/*.py` — diff actual model changes

Check if functionality is now native in target version.

## STEP 4: Hour Calculation
Consult `odoo-effort-estimator` skill for hours-per-task-type and risk-contingency tables.

Apply:
1. Base hours from estimator
2. Code quality adjustments (if deep analysis enabled)
3. Risk contingency based on project size

## STEP 5: Database Migration (Optional)
If `INCLUDE_DB_MIGRATION` is Yes:
- Load `references/database-migration.md`
- Calculate database upgrade hours
- Apply risk contingency

## STEP 6: Functional Testing (Optional)
If `INCLUDE_FUNCTIONAL_TESTING` is Yes:
1. For each module classified in STEP 2, determine complexity (Low/Medium/High based on LOC)
2. Consult `odoo-effort-estimator` skill → Category `FUNCTIONAL_TESTING`
3. Calculate base hours per module using complexity level
4. Apply risk contingency based on project size
5. Generate functional testing table in report

## STEP 7: Unit Testing (Optional)
If `INCLUDE_UNIT_TESTING` is Yes:

**Scope: ONLY modules classified as `🔴 Custom` in STEP 2.**

For each Custom module:
1. **Detect existing tests**: Search for `tests/` directory, count `test_*.py` files, measure LOC in test files
2. **Measure code LOC**: Count lines of Python code (excluding `tests/`, `__manifest__.py`, `__init__.py`)
3. **Estimate current coverage** using heuristic:
   ```
   coverage_estimate = LOC_tests / (LOC_code × 0.6)
   ```
   Factor 0.6: not all code requires equal test density (boilerplate, imports, simple accessors are excluded from effective coverage calculation)
4. **Classify module**:
   - **Coverage >= 80%**: Mark as "Sufficient" → only estimate test migration/adaptation hours
   - **Coverage < 80%**: Calculate gap to 90% target → estimate test migration + new test development
   - **Coverage = 0% (no tests/)**: Full test development from scratch
5. **Calculate hours using LOC-proportional model** (see `references/ai-assistance-and-risk.md` for AI factors):

   **Test Migration (adapting existing tests to new version):**
   ```
   migration_hours = BASE_MIGRATION_HOURS × AI_MIGRATION_FACTOR (0.70)
   ```
   BASE_MIGRATION_HOURS from `odoo-effort-estimator` skill → `Test Migration (Existing)`

   **Coverage Gap Filling (writing new tests):**
   ```
   gap_LOC = LOC_code × (gap_percentage / 100)
   manual_hours = gap_LOC × RATE_PER_LOC
   ai_hours = max(manual_hours × AI_GAP_FACTOR (0.35), MINIMUM_HOURS)
   ```
   RATE_PER_LOC and MINIMUM_HOURS from `odoo-effort-estimator` skill → `Coverage Gap Filling`

6. **Apply risk contingency** based on project size
7. Generate unit testing table in report (see `templates/migration-report.md` Section 4)

## STEP 8: Generate Report
Write report to `OUTPUT_PATH` using the template in `templates/migration-report.md`.
Render in `LANGUAGE`. All sections marked as conditional must be included/excluded based on user parameters.
