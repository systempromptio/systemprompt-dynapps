# Test Detection Matrix

Systematic algorithm to analyze ANY Odoo module and determine exactly which tests to generate.

---

## Phase 1: Module Scanning

Run these searches on the module path to build a component inventory.

### 1.1 Manifest Analysis

```
Read: __manifest__.py
Extract:
  - depends        → Determines available base test classes and test data
  - data files     → Identifies XML files to check for views, actions, security
  - assets         → Identifies JS files needing JS tests
  - external_dependencies → May need mocking in tests
```

**Decision:**
- `depends` includes `mail` → Tests can use `MailCommon`, `MockEmail` mixins
- `depends` includes `website` → Likely needs tour tests for public pages
- `depends` includes `portal` → Needs portal access token tests
- `depends` includes `sale`/`purchase`/`account` → Use their Common classes as test base
- `assets.web.assets_backend` has JS → Needs JS component tests
- `external_dependencies.python` → Needs mock/patch for external libs

### 1.2 Model Discovery

```
Grep pattern: _name\s*=\s*['"]
Grep pattern: _inherit\s*=\s*['"]
Grep pattern: _inherits\s*=\s*\{
```

**For each model found, classify:**

| Pattern Found | Model Type | Primary Test Type |
|---------------|-----------|-------------------|
| `_name = 'x'` without `_inherit` | **New model** | Full CRUD + business logic tests |
| `_name = 'x'` with `_inherit = 'x'` | **Extension** (same model) | Tests for added/modified behavior only |
| `_inherit = 'y'` without `_name` | **Delegation inherit** | Tests for overridden methods |
| `_name = 'x'` with `_inherit = 'y'` | **New model from parent** | Full tests + verify parent behavior |
| `_inherits = {'y': 'y_id'}` | **Prototype inherit** | Full tests + delegation field tests |
| `_transient = True` | **Wizard** | Form tests + execute action tests |

---

## Phase 2: Component Detection

For each model file, search for these patterns and map to test types.

### 2.1 Field Analysis

```
Grep patterns per file:
  fields.Char|fields.Text|fields.Integer|fields.Float|fields.Boolean
  fields.Many2one|fields.One2many|fields.Many2many
  fields.Selection|fields.Date|fields.Datetime
  fields.Binary|fields.Html|fields.Monetary
```

**Detection rules:**

| Field Pattern | Test Required | Priority |
|---------------|---------------|----------|
| `compute='_compute_X'` | Unit test: verify computation | HIGH |
| `compute='_compute_X', store=True` | Unit test + trigger dependency test | HIGH |
| `compute='_compute_X', inverse='_inverse_X'` | Unit test: compute + inverse | HIGH |
| `related='path.to.field'` | Unit test: verify propagation | MEDIUM |
| `default=lambda self:` | Form test: verify default | MEDIUM |
| `required=True` | Constraint test: empty value | LOW |
| `readonly=True, states={...}` | Workflow test: field editability per state | MEDIUM |
| `tracking=True` | Chatter tracking test (if mail in depends) | **HIGH** (si el módulo añade tracking a propósito) |
| `groups='group.xml_id'` | Security test: field visibility | MEDIUM |
| `currency_field='currency_id'` (Monetary) | Unit test: multi-currency computation | MEDIUM |

### 2.2 Method Analysis

```
Grep patterns per file:
  @api.depends
  @api.constrains
  @api.onchange
  @api.model_create_multi|def create
  def write
  def unlink
  def action_
  def _compute_
  def _inverse_
  def _check_
  def _onchange_
  def _default_
  def _search_
  @staticmethod
  def _generate_
  def _build_
```

**Detection rules:**

| Method Pattern | Test Type | Test Strategy | Priority |
|----------------|-----------|---------------|----------|
| `@api.depends('field')` + `_compute_X` | **Unit** | Set dependency → assert computed value | HIGH |
| `@api.constrains('field')` + `_check_X` | **Unit** | Invalid data → assertRaises(ValidationError) | HIGH |
| `@api.onchange('field')` + `_onchange_X` | **Form** | Form class: set field → assert dependent fields | HIGH |
| `def create(self, vals_list)` override | **Unit** | Create record → verify custom logic executed | HIGH |
| `def write(self, vals)` override | **Unit** | Write record → verify custom logic | HIGH |
| `def unlink(self)` override | **Unit** | Unlink → verify pre/post conditions | MEDIUM |
| `def action_<name>(self)` | **Unit/Workflow** | Call action → assert state change | HIGH |
| `def _cron_<name>(self)` | **Unit** | Call method → assert side effects | MEDIUM |
| `raise UserError` (inside any method) | **Unit** | Trigger condition → assertRaises(UserError) | HIGH |
| `raise ValidationError` (inside method) | **Unit** | Invalid input → assertRaises + savepoint | HIGH |
| `self.env['ir.attachment'].create` | **Unit** | Verify attachment created | LOW |
| `self.message_post(` | **Unit** | Verify message posted / tracking values (`MailCommon`) | **HIGH** (si el módulo publica mensajes/tracking) |
| `request.env` / `http.request` | **HTTP** | Controller test with url_open | HIGH |
| `.sudo()` calls | **Security** | Verify sudo is necessary, test without | MEDIUM |
| `self.env.user` / `self.env.uid` | **Security** | Test with different users | MEDIUM |
| `@staticmethod` | **Unit** | Call directly on class, no record needed | MEDIUM |
| `def _generate_` (code/template gen) | **Unit** | Verify output contains expected fragments | HIGH |
| `def _build_` (tree/graph builders) | **Unit** | Verify structure, test edge cases | HIGH |

### 2.2b Module-Level Analysis

```
Grep patterns at module level (outside class definitions):
  ^[A-Z_]+\s*=\s*\{          → Dict constants
  ^[A-Z_]+\s*=\s*\[          → List constants
  ^[A-Z_]+\s*=\s*set\(       → Set constants (explicit)
  ^[A-Z_]+\s*=\s*\{$         → Multi-line set/dict constants
  Serialized\(                → Serialized (JSON) fields
  fields\.Json\(              → JSON fields (v17+)
  json\.dumps\|json\.loads    → JSON operations in code
  safe_eval\|test_python_expr → Dynamic code execution
```

**Detection rules:**

| Pattern | Test Type | Test Strategy | Priority |
|---------|-----------|---------------|----------|
| Module-level dict constant | **Unit** | Validate structure, keys, value types | MEDIUM |
| Module-level set constant | **Unit** | Validate no overlaps with related sets, entries exist in registry | MEDIUM |
| `Serialized` field | **Unit** | Test returns dict, test empty case | MEDIUM |
| `fields.Json` field | **Unit** | Test roundtrip write/read, test empty | MEDIUM |
| Text field with `json.dumps`/`json.loads` | **Unit** | Test roundtrip, test invalid JSON handling | HIGH |
| `safe_eval` / `test_python_expr` | **Unit** | Verify generated code passes validation | HIGH |
| Idempotent operations (generators, importers, syncs) | **Unit** | Run twice, assert no duplication | HIGH |

### 2.2c Cross-Document Flow Detection (E2E Tests)

```
Grep patterns to detect cross-document data propagation:
  _prepare_invoice_line           → SO line → invoice line
  _prepare_account_move_line      → PO line → invoice line
  _get_stock_move_values          → SO/PO → stock.move
  _compute_qty_invoiced           → invoice → SO/PO reconciliation
  _compute_qty_delivered          → picking → SO reconciliation
  _prepare_base_line_for_taxes    → custom tax/amount computation
```

**Detection rules:**

| Pattern | Test Type | Test Strategy | Priority |
|---------|-----------|---------------|----------|
| Module inherits **2+ models** from {sale.order.line, purchase.order.line, account.move.line, stock.move, stock.rule} | **E2E flow** | Full document cycle test | **MUST HAVE** |
| Override of `_prepare_invoice_line` | **E2E flow** | SO → invoice → verify propagated fields | **MUST HAVE** |
| Override of `_prepare_account_move_line` | **E2E flow** | PO → invoice → verify propagated fields | **MUST HAVE** |
| Override of `_get_stock_move_values` | **E2E flow** | SO → picking → verify propagated fields | **MUST HAVE** |
| Override of `_compute_qty_invoiced` | **E2E flow** | Full cycle → verify qty reconciliation | **MUST HAVE** |
| Override of `_prepare_base_line_for_taxes_computation` | **E2E flow** | Create doc → verify amounts → invoice → verify amounts match | **MUST HAVE** |
| Module touches pricing/qty in **both** order line AND invoice line | **E2E mixed** | Order with special + normal products | **MUST HAVE** |

**Key principle:** If custom data flows from document A to document B, a unit test
of `_prepare_X` is necessary but NOT sufficient. An E2E test must verify the data
survives the full cycle (create → confirm → process → invoice → post → reconcile).

### 2.3 Controller Detection

```
Grep patterns:
  @http.route
  class.*http.Controller
  type=['"]json['"]
  type=['"]http['"]
  auth=['"]public['"]
  auth=['"]user['"]
  auth=['"]none['"]
```

**Detection rules:**

| Controller Pattern | Test Type | Test Strategy | Priority |
|--------------------|-----------|---------------|----------|
| `auth='public'` | **HTTP** | Test without auth + with auth | HIGH |
| `auth='user'` | **HTTP** | Test authenticated + unauthorized | HIGH |
| `auth='none'` | **HTTP** | Test direct access | MEDIUM |
| `type='json'` | **HTTP** | JSON-RPC payload test | HIGH |
| `type='http'` | **HTTP** | GET/POST with url_open | HIGH |
| Portal routes (`/my/...`) | **HTTP** | Token access + portal user | HIGH |
| `website=True` | **Tour** | End-to-end UI test | MEDIUM |
| `csrf=False` | **Security** | Flag as security concern | HIGH |

### 2.4 View Analysis (for Tour Tests)

```
Grep patterns in XML:
  <form
  <tree|<list
  <kanban
  <calendar
  <pivot|<graph
  <search
  <wizard
  <menuitem
  t-call=['"]web.login['"]
  website.layout
```

**Tour test triggers:**

| View Pattern | Needs Tour? | Reason |
|--------------|-------------|--------|
| Complex form with state machine + buttons | **YES** | Verify full UI workflow |
| Wizard with multiple steps | **YES** | Verify wizard step navigation |
| `website=True` pages | **YES** | Public-facing pages need E2E |
| Portal pages with token access | **YES** | Portal UX verification |
| Simple CRUD form/list | **NO** | Unit tests are sufficient |
| Report QWeb template | **NO tour** | Se cubre con test de render (§2.4b), no con tour |

### 2.4b Report Analysis (QWeb render tests)

```
Grep patterns:
  <record model=['"]ir.actions.report['"]      (XML)
  <report                                        (XML shortcut tag)
  _name = ['"]report\.                           (modelo report.<módulo>.*)
  def _get_report_values                         (contexto de informe propio)
```

**Report test triggers:**

| Report Pattern | Test Type | Reason | Priority |
|----------------|-----------|--------|----------|
| Módulo añade/override un `ir.actions.report` + plantilla QWeb | **Report render** (`_render_qweb_html`) | Un `t-field`/campo inválido solo peta al imprimir | **HIGH** |
| Modelo `report.<módulo>.*` con `_get_report_values` | **Unit** del método + **Report render** | La construcción de datos es lógica tuya | **HIGH** |
| Layout PDF crítico | **Smoke PDF** (`_render_qweb_pdf`, requiere wkhtmltopdf) | Valida cadena completa; lento | LOW |

Patrón completo: `patterns/23-qweb-report.md`.

### 2.5 Security File Analysis

```
Read: security/ir.model.access.csv
Read: security/*.xml (record rules)
Grep: <record.*model="ir.rule"
Grep: group_
```

**Detection rules:**

| Security Pattern | Test Type | Test Strategy | Priority |
|------------------|-----------|---------------|----------|
| Custom groups defined in XML | **Security** | Test each group's access level | HIGH |
| `ir.model.access.csv` with restricted perms | **Security** | Test CRUD per group | HIGH |
| Record rules with domain | **Security** | Test record visibility per user | HIGH |
| Multi-company rules (`company_id`) | **Security** | Multi-company record isolation | HIGH |
| `groups="base.group_system"` on fields | **Security** | Verify field visibility | MEDIUM |
| No security files at all | **Security** | Flag as missing! Create basic test | CRITICAL |

### 2.6 JavaScript Detection

```
Glob: static/src/**/*.js
Glob: static/src/**/*.xml (OWL templates)
Grep: extends Component|class.*Component
Grep: extends AbstractField|extends Widget
Grep: registry.category
Grep: useService|useEnv
```

**Detection rules:**

| JS Pattern | Test Type | Framework | Priority |
|------------|-----------|-----------|----------|
| OWL Component class | **JS Unit** | QUnit (v16-17) / Hoot (v18) | HIGH |
| Custom widget (field) | **JS Unit** | QUnit / Hoot | HIGH |
| Custom action | **JS Unit** | QUnit / Hoot | MEDIUM |
| Service registration | **JS Unit** | Mock service test | MEDIUM |
| RPC calls (`this.orm.call`) | **JS Unit** | Mock server test | MEDIUM |
| Only template overrides | **NO** | Visual only, tour covers it | LOW |

---

## Phase 3: Test Plan Generation

### 3.1 Decision Algorithm

After scanning, apply this priority algorithm:

```
FOR EACH model IN module:
    IF model has computed fields:
        → ADD unit test file: test_<model>_compute.py  [HIGH]
    IF model has constraints:
        → ADD to unit test: constraint test methods    [HIGH]
    IF model has onchanges:
        → ADD form test file: test_<model>_form.py     [HIGH]
    IF model has action_ workflow methods:
        → ADD workflow tests to unit test              [HIGH]
    IF model has CRUD overrides:
        → ADD CRUD tests to unit test                  [HIGH]
    IF model has state field + buttons in views:
        → CONSIDER tour test for full flow             [MEDIUM]

FOR EACH controller IN module:
    → ADD HTTP test file: test_controllers.py          [HIGH]
    IF controller has auth='public' or website=True:
        → ADD tour test                                [MEDIUM]

IF module defines custom security groups:
    → ADD security test file: test_security.py         [HIGH]

IF module adds/overrides an ir.actions.report (QWeb) OR a report.<module>.* model:
    → ADD report render test file: test_report.py      [HIGH]
    (assert _render_qweb_html no lanza + aparece el dato que aporta el módulo; patterns/23)

IF module has JS components:
    → ADD JS test files in static/tests/               [MEDIUM]

IF module inherits 2+ models from {sale.order.line, purchase.order.line, account.move.line, stock.move, stock.rule}:
    → ADD E2E flow test file: test_flow_e2e.py         [MUST HAVE]
    IF module adds conditional behavior (some products special, some normal):
        → ADD mixed product E2E test                   [MUST HAVE]

IF module overrides _prepare_invoice_line OR _prepare_account_move_line:
    → ADD E2E flow test: full cycle with invoice        [MUST HAVE]

IF module overrides _compute_qty_invoiced:
    → ADD E2E flow test: verify reconciliation          [MUST HAVE]

IF module is performance-critical (bulk operations, crons):
    → ADD performance test file: test_performance.py   [LOW]

FOR EACH model file IN module:
    IF file has module-level constants (UPPER_CASE dicts/sets):
        → ADD constants validation tests               [SHOULD HAVE]
    IF file has @staticmethod methods:
        → ADD static method tests (call on class)      [SHOULD HAVE]
    IF file has _generate_* or _build_* methods:
        → ADD code generation output tests             [MUST HAVE]
    IF file has safe_eval / test_python_expr:
        → ADD safe_eval compatibility tests            [MUST HAVE]
    IF model has JSON/Serialized fields:
        → ADD JSON roundtrip + error handling tests    [SHOULD HAVE]
    IF model has idempotent operations (import, sync, generate):
        → ADD idempotency tests (run twice)            [SHOULD HAVE]
```

### 3.2 File Naming Convention

| Component | File Name | Class Name |
|-----------|-----------|------------|
| Shared data | `tests/common.py` | `<Module>Common` |
| Model unit tests | `tests/test_<model_snake>.py` | `Test<ModelCamel>` |
| Form/onchange tests | `tests/test_<model_snake>_form.py` | `Test<ModelCamel>Form` |
| Controller tests | `tests/test_controllers.py` | `Test<Module>Controllers` |
| Tour tests | `tests/test_tour.py` | `Test<Module>Tour` |
| Security tests | `tests/test_security.py` | `Test<Module>Security` |
| Report render tests | `tests/test_report.py` | `Test<Module>Report` |
| E2E flow tests | `tests/test_flow_e2e.py` | `Test<Flow>FlowE2E` |
| Performance tests | `tests/test_performance.py` | `Test<Module>Performance` |
| JS tours | `static/tests/tours/<tour_name>.js` | N/A |
| JS unit tests | `static/tests/<component>.test.js` | N/A (v18) |

### 3.3 Test Method Naming

| What is Tested | Method Name Pattern | Example |
|----------------|--------------------|---------|
| Computed field | `test_compute_<field>` | `test_compute_margin_percent` |
| Computed field edge case | `test_compute_<field>_<case>` | `test_compute_margin_percent_zero_total` |
| Constraint valid | `test_check_<constraint>_valid` | `test_check_dates_valid` |
| Constraint invalid | `test_check_<constraint>_invalid` | `test_check_dates_invalid` |
| Onchange | `test_onchange_<trigger_field>` | `test_onchange_partner_id` |
| Create | `test_create_<scenario>` | `test_create_basic` |
| Write | `test_write_<scenario>` | `test_write_state_change` |
| Workflow action | `test_action_<action_name>` | `test_action_confirm` |
| Full workflow | `test_<workflow>_flow` | `test_sale_to_invoice_flow` |
| Access right | `test_<group>_<operation>` | `test_user_cannot_delete` |
| Record rule | `test_rule_<rule_name>` | `test_rule_own_records_only` |
| Controller route | `test_<method>_<route_desc>` | `test_get_portal_order` |
| Tour | `test_<flow>_tour` | `test_checkout_tour` |
| Performance | `test_<operation>_query_count` | `test_search_orders_query_count` |

---

## Phase 4: Common Test Base Selection

### 4.1 Which Common Class to Inherit

Based on the module's `depends`, select the best available common class:

| Module Depends On | Recommended Test Base | Import |
|--------------------|-----------------------|--------|
| `sale` | `SaleCommon` | `from odoo.addons.sale.tests.common import SaleCommon` |
| `account` | `AccountTestInvoicingCommon` | `from odoo.addons.account.tests.common import AccountTestInvoicingCommon` |
| `stock` | `StockCommon` (v18) or custom | `from odoo.addons.stock.tests.common import TestStockCommon` |
| `purchase` | `PurchaseTestCommon` | `from odoo.addons.purchase.tests.common import PurchaseTestCommon` |
| `mail` | `MailCommon` or `MockEmail` | `from odoo.addons.mail.tests.common import MailCommon` |
| `website` | `HttpCaseWithUserDemo` | `from odoo.addons.website.tests.test_base_url import TestBaseUrl` |
| `helpdesk` (enterprise) | `HelpdeskCommon` | `from odoo.addons.helpdesk.tests.common import HelpdeskCommon` |
| `sale_subscription` (enterprise) | `TestSubscriptionCommon` | `from odoo.addons.sale_subscription.tests.common_sale_subscription import TestSubscriptionCommon` |
| Only `base` | `TransactionCase` | `from odoo.tests.common import TransactionCase` |

### 4.2 When to Create Custom Common

Create `tests/common.py` when:
- Module has **2+ test files** that share setup data
- Module creates **new models** that need test records
- Module extends models with fields that need consistent test values

Skip `tests/common.py` when:
- Module has only **1 test file**
- Module only extends views (no model changes)
- Parent common class already provides sufficient data

---

## Phase 5: Grep Commands for Automated Detection

Run these searches to build the component inventory automatically:

```bash
# Models: new and inherited
grep -rn "_name\s*=" --include="*.py" <module_path>/models/
grep -rn "_inherit\s*=" --include="*.py" <module_path>/models/

# Computed fields
grep -rn "compute=" --include="*.py" <module_path>/models/

# Constraints
grep -rn "@api.constrains" --include="*.py" <module_path>/models/
grep -rn "_sql_constraints" --include="*.py" <module_path>/models/

# Onchanges
grep -rn "@api.onchange" --include="*.py" <module_path>/models/

# Workflow actions
grep -rn "def action_" --include="*.py" <module_path>/models/

# CRUD overrides
grep -rn "def create\|def write\|def unlink\|def copy" --include="*.py" <module_path>/models/

# Exceptions raised
grep -rn "raise UserError\|raise ValidationError\|raise AccessError" --include="*.py" <module_path>/

# Controllers
grep -rn "@http.route" --include="*.py" <module_path>/controllers/

# Security groups
grep -rn "category_id\|group_" --include="*.xml" <module_path>/security/

# JavaScript components
grep -rn "extends Component\|class.*Component" --include="*.js" <module_path>/static/

# State fields (workflow candidates)
grep -rn "fields.Selection.*state\|selection=.*draft.*confirm" --include="*.py" <module_path>/models/

# Cron jobs
grep -rn "ir.cron\|_cron_" --include="*.py" --include="*.xml" <module_path>/

# QWeb reports (render tests)
grep -rn "ir.actions.report\|<report\|_get_report_values" --include="*.xml" --include="*.py" <module_path>/

# External API calls (need mocking)
grep -rn "requests\.\(get\|post\|put\|delete\)\|urllib\|http\.client" --include="*.py" <module_path>/

# Static methods (testable without records)
grep -rn "@staticmethod" --include="*.py" <module_path>/models/

# Code generation methods (need output validation)
grep -rn "def _generate_\|def _build_" --include="*.py" <module_path>/models/

# Module-level constants (dicts, sets, lists)
grep -rn "^[A-Z_]\+\s*=\s*[{\[\(]" --include="*.py" <module_path>/models/

# JSON/Serialized fields
grep -rn "Serialized\|fields\.Json\|json\.dumps\|json\.loads" --include="*.py" <module_path>/models/

# Dynamic code execution (safe_eval)
grep -rn "safe_eval\|test_python_expr" --include="*.py" <module_path>/models/

# Cross-document flow methods (E2E test candidates)
grep -rn "_prepare_invoice_line\|_prepare_account_move_line" --include="*.py" <module_path>/models/
grep -rn "_get_stock_move_values" --include="*.py" <module_path>/models/
grep -rn "_compute_qty_invoiced\|_compute_qty_delivered" --include="*.py" <module_path>/models/
grep -rn "_prepare_base_line_for_taxes_computation" --include="*.py" <module_path>/models/

# Count inherited flow models (if 2+, E2E is mandatory)
grep -rn "_inherit.*sale.order.line\|_inherit.*purchase.order.line\|_inherit.*account.move.line\|_inherit.*stock.move\b\|_inherit.*stock.rule" --include="*.py" <module_path>/models/
```

---

## Quick Decision Flowchart

```
START: Received an Odoo module to test
  │
  ├─ Has models with computed fields?
  │    YES → Unit tests (test_<model>.py)              [MUST HAVE]
  │
  ├─ Has @api.constrains or SQL constraints?
  │    YES → Constraint tests in unit test file        [MUST HAVE]
  │
  ├─ Has @api.onchange methods?
  │    YES → Form tests (test_<model>_form.py)         [MUST HAVE]
  │
  ├─ Has action_* workflow methods?
  │    YES → Workflow tests in unit test file           [MUST HAVE]
  │
  ├─ Has CRUD overrides (create/write/unlink)?
  │    YES → CRUD tests in unit test file              [MUST HAVE]
  │
  ├─ Has controllers (@http.route)?
  │    YES → HTTP tests (test_controllers.py)          [MUST HAVE]
  │
  ├─ Has custom security groups or record rules?
  │    YES → Security tests (test_security.py)         [SHOULD HAVE]
  │
  ├─ Has complex UI workflow (state machine + views)?
  │    YES → Tour test (test_tour.py + JS)             [SHOULD HAVE]
  │
  ├─ Has JavaScript components?
  │    YES → JS tests (static/tests/)                  [SHOULD HAVE]
  │
  ├─ Has bulk/batch operations or crons?
  │    YES → Performance tests (test_performance.py)   [NICE TO HAVE]
  │
  ├─ Has external API calls?
  │    YES → Mock tests with patch()                   [MUST HAVE]
  │
  ├─ Has module-level constants (dicts/sets)?
  │    YES → Constants validation tests                [SHOULD HAVE]
  │
  ├─ Has JSON/Serialized fields?
  │    YES → JSON roundtrip + invalid handling tests   [SHOULD HAVE]
  │
  ├─ Has code generation methods (_generate_*)?
  │    YES → Output validation + safe_eval tests       [MUST HAVE]
  │
  ├─ Has idempotent operations (generators, importers)?
  │    YES → Run-twice-no-duplicate tests              [SHOULD HAVE]
  │
  ├─ Has @staticmethod helpers?
  │    YES → Direct call tests (no record needed)      [SHOULD HAVE]
  │
  └─ Inherits 2+ flow models (SOL, POL, AML, stock.move, stock.rule)?
     OR overrides _prepare_invoice_line / _prepare_account_move_line?
     OR overrides _compute_qty_invoiced?
       YES → E2E flow tests (test_flow_e2e.py)          [MUST HAVE]
             + Mixed product test if conditional behavior [MUST HAVE]
```
