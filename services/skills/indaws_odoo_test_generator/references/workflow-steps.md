# Workflow Steps — Detailed Reference

Read this file when starting a test generation session. Follow the steps in order.

---

## STEP 0: Information Gathering [STOP]

**NEVER proceed without collecting ALL required information.**

**Pre-fill from registry:** Before asking, check if `desarrollos/PROJECT_REGISTRY.md` exists in the project root. If found, extract: Odoo version (frontmatter `odoo_version`), custom module paths, and dependency tree. Skip asking for info already in the registry.

Use `AskUserQuestion` tool to gather remaining information:

### Question 1: Source Code Location

- **Module path:** Where is the module to test? — _skip if registry lists the module_
- **Odoo version:** Which version? (14.0-19.0) — _skip if in registry_
  - **CRITICAL:** Different versions have different test APIs (see [version-differences.md](version-differences.md))

### Question 2: Test Scope

Ask which test types to generate. See [test-types.md](test-types.md) for the full table.
If user doesn't specify, recommend: **Unit Tests + Form/Onchange Tests** as baseline.
If module inherits 2+ flow models (SOL, POL, AML, stock.move, stock.rule), **also recommend E2E Flow Tests**.

### Question 3: Test Strategy

- **Option A:** "Generate tests for specific methods/features" → User specifies which methods. Generates targeted tests for those methods only. No coverage target — the goal is surgical testing of specific logic.
- **Option B:** "Analyze the module and suggest a test plan" → Enter Plan Mode, scan module, propose a test plan with priorities. Generates tests after user approves the plan. Reports estimated LOC coverage as advisory info but does not enforce a threshold.
- **Option C:** "Generate a complete test suite with coverage verification" → Full module analysis + test generation + automatic coverage execution with `coverage.py`. **Target: >= 80% line coverage.** If coverage falls short, identifies uncovered lines and generates additional tests (max 2 iterations). Requires a working Odoo environment.

### Question 4: Reference Paths (Optional)

- **Odoo CE Path:** e.g., `$ODOO_SRC/odoo18/addons`
- **Odoo EE Path:** e.g., `$ODOO_ENTERPRISE_SRC/18`
- **Existing tests path:** Does the module already have tests?

### Question 5: Execution Environment (Option C only)

Only ask if user selected Option C:
- **Odoo-bin path:** e.g., `$ODOO_SRC/odoo-bin` or Docker container name
- **Test database:** e.g., `testdb` (a disposable database for running tests)
- **Environment:** Local or Docker?
- **Config file (Docker):** e.g., `/etc/odoo/odoo.conf`

**STOP HERE** — Wait for user's response before continuing.

---

## STEP 0.5: Plan Mode (OPTIONAL — If Option B or C)

Use `EnterPlanMode` to analyze the module and design a test plan.

**Component Discovery:** Use **odoo-module-scanner** skill if available, or run grep searches from [detection-matrix.md](detection-matrix.md) Phase 5:

```
_name=, _inherit=          → Model inventory
compute=                   → Computed fields
@api.constrains            → Constraints
@api.onchange              → Onchanges
def action_                → Workflow methods
def create|write|unlink    → CRUD overrides
raise UserError            → Error paths
@http.route                → Controllers
ir.rule|group_             → Security
extends Component          → JS components
_prepare_invoice_line      → Cross-document flow (E2E)
```

**Apply Decision Flowchart** from detection-matrix.md to map components → test types.

**Select Test Base Class** based on `depends`:
- `sale` → `SaleCommon` | `account` → `AccountTestInvoicingCommon`
- `stock` → `TestStockCommon` | `mail` → `MailCommon`
- Only `base` → `TransactionCase`

**Present Test Plan:** file paths, class names, base classes, test methods, priorities.
Exit Plan Mode for approval.

---

## STEP 1: Code Analysis (2-Phase Pattern Loading)

**Phase 1 — Scan & Detect** (no pattern files loaded yet):
Run Detection Matrix scans via Grep/Glob.

**Phase 2 — Load Only Needed Patterns:**
Read [patterns/INDEX.md](patterns/INDEX.md) and load ONLY matching pattern files.
`01-transaction-case.md` is always loaded. All others conditional.

**Classify Components** using the detection rules:
- Computed fields → unit test | Constraints → constraint test | Onchanges → Form test
- CRUD overrides → CRUD test | Workflows → workflow test | Controllers → HTTP test
- See detection-matrix.md for full classification including advanced patterns (mocks, i18n, e2e, etc.)

**Propose Test Plan** → **STOP** — Wait for user confirmation.

---

## STEP 2: Test Generation

Generate test files in this order. See [test-templates.md](test-templates.md) for code templates.

1. `tests/__init__.py`
2. `tests/common.py` — Shared test data (if 2+ test files)
3. `tests/test_<model>.py` — Unit tests (TransactionCase)
4. `tests/test_<model>_form.py` — Form/onchange tests
5. `tests/test_controllers.py` — HTTP tests (HttpCase)
6. `tests/test_tour.py` — Tour tests (HttpCase + JS)
7. `tests/test_security.py` — Access rights + record rules
8. `tests/test_flow_e2e.py` — E2E flow tests
9. `tests/test_performance.py` — Query count benchmarks

---

## STEP 3: Test Data Strategy

1. Use `setUpClass` for data shared across tests in a class
2. Use `Command` API (v16+) for inline record creation
3. Use `cls.env.ref()` for existing demo/data records
4. Avoid depending on demo data — create all test data explicitly
5. Use `new_test_user()` for test users with groups
6. Disable mail in setup context for performance:
   ```python
   ctx = {'tracking_disable': True, 'mail_create_nolog': True,
          'mail_create_nosubscribe': True, 'no_reset_password': True}
   cls.env = cls.env(context=dict(cls.env.context, **ctx))
   ```

---

## STEP 4: Assertion Strategy

| What to Test | Assertion Method |
|---|---|
| Single field value | `assertEqual(record.field, expected)` |
| Multiple records | `assertRecordValues(records, [dicts])` |
| Record exists | `assertTrue(record)` |
| Exception raised | `with self.assertRaises(UserError):` |
| Exception + continue | `with self.assertRaises(X), self.cr.savepoint():` |
| Float comparison | `assertAlmostEqual(a, b, places=2)` |
| SQL query count | `with self.assertQueryCount(admin=N):` |

---

## STEP 5: Final Validation

1. **Structure:** `__init__.py` imports all test modules, all classes have `@tagged`, all methods start with `test_`
2. **Independence:** Each test runs alone, `setUpClass` creates all data
3. **Tags:** `@tagged('post_install', '-at_install')` for most tests
4. **Version:** Verify APIs match target version via **odoo-version-api-reference** skill
5. **No anti-patterns:** No `time.sleep()`, no demo data deps, no hardcoded IDs, no `cr.commit()`
6. **Gotchas check:** Read [test-gotchas.md](test-gotchas.md) and verify no generated test hits a known gotcha
7. **Present summary** using this format:

```
## Test Generation Summary

| Metric | Value |
|--------|-------|
| Files created | N |
| Test classes | N |
| Test methods | N |
| Test types | Unit, Form, Security, ... |
| Estimated LOC (tests) | N |
| Module LOC (code) | N |

### Files
| File | Class | Methods | Type |
|------|-------|---------|------|
| tests/test_model.py | TestModel | 5 | Unit |
| tests/test_model_form.py | TestModelForm | 3 | Form |
| ... | ... | ... | ... |

### Run Command
<pre>./odoo-bin -d testdb --test-tags /module -u module --stop-after-init</pre>
```

**If Option C:** Continue to STEP 6.
**If Option A/B:** Done. Present summary and stop.

---

## STEP 6: Coverage Execution (Option C Only)

This step runs automatically for Option C. It requires the execution environment info gathered in STEP 0 (Question 5).

### 6.1 Install coverage

```bash
pip install coverage 2>/dev/null
```

### 6.2 Run tests with coverage

**Local:**
```bash
coverage run --source=<module_path> <odoo_bin_path> \
  -d <test_db> \
  --test-tags /<module_name> \
  -u <module_name> \
  --stop-after-init \
  --no-http
```

**Docker:**
```bash
docker exec <container> pip install coverage 2>/dev/null
docker exec <container> coverage run --source=<module_path_in_container> \
  /path/odoo-bin \
  -d <test_db> \
  --test-tags /<module_name> \
  -u <module_name> \
  --stop-after-init \
  --no-http \
  --xmlrpc-port=18199 \
  -c <config_file>
```

### 6.3 Evaluate results

**If tests FAIL:** Stop. Report the errors. Do NOT proceed to coverage analysis — fix the failing tests first, then re-run.

**If tests PASS:** Get the coverage report:

```bash
coverage report --include="<module_path>/*.py" --omit="*/tests/*"
```

### 6.4 Check coverage threshold

**If coverage >= 80%:** PASS. Present the final report (see format below) and stop.

**If coverage < 80%:** Get the missing lines:

```bash
coverage report -m --include="<module_path>/*.py" --omit="*/tests/*"
```

Analyze the `Missing` column to identify uncovered lines. Generate additional tests targeting those specific lines/branches. Then re-run STEP 6.2-6.4.

**Maximum 2 iterations.** If after 2 rounds coverage is still < 80%, present the report with the achieved coverage and flag the uncovered areas for the developer.

### 6.5 Final Coverage Report

```
## Coverage Report

| Metric | Value |
|--------|-------|
| Line coverage | XX% |
| Target | 80% |
| Status | PASS / FAIL |
| Iterations | N |

### Per-file breakdown
| File | Stmts | Miss | Cover | Missing lines |
|------|-------|------|-------|---------------|
| models/model.py | 120 | 15 | 87% | 45-48, 92 |
| models/wizard.py | 40 | 2 | 95% | 33 |
| ... | ... | ... | ... | ... |

### Uncovered areas (if coverage < 80%)
- `test_website/models/model.py:45-48`, `TestModel._search_get_detail` — error handling branch in `action_validate()`
- `test_website/models/model.py:92`, `TestModelExposed._description` — edge case in `_compute_total()` when qty=0
```
