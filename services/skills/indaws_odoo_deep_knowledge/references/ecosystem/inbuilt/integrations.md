# inBuilt — Odoo Integration Map

## Integration Summary

inBuilt touches **6 core Odoo domains** + indaws_core. Every integration point
is a potential source of bugs when upgrading Odoo or indaws modules.

```
                    ┌──────────────┐
                    │  FIEBDC3     │
                    │  BC3 Import  │
                    └──────┬───────┘
                           │ creates
                    ┌──────▼───────┐
          ┌─────────│  sale.order  │─────────┐
          │         │  + SOL lines │         │
          │         └──────┬───────┘         │
          │                │ wizard           │ margin
          │         ┌──────▼───────┐   ┌─────▼─────┐
          │         │  project     │   │ indaws_    │
          │         │  .project    │   │ sale_margin│
          │         └──────┬───────┘   └─────┬─────┘
          │                │                 │
    ┌─────▼─────┐   ┌─────▼──────┐   ┌──────▼──────┐
    │ sale.      │   │ project.   │   │ project.    │
    │measurement │   │ task       │   │ cost.summary│
    └─────┬─────┘   └─────┬──────┘   └─────────────┘
          │               │
    ┌─────▼──────────────▼──────┐
    │   project.certification    │
    │   + certification.line     │
    │   + certification.measure  │
    └─────────────┬──────────────┘
                  │ creates
           ┌──────▼───────┐
           │ account.move  │
           │ (invoice)     │
           │ + retention   │
           └───────────────┘
```

---

## 1. Sale ↔ inBuilt

### Fields added to `sale.order`
- `description_large` (Html) — rich text per SOL
- Project creation fields (via mixin)
- Indirect cost % margin (company-level)
- FIEBDC3 import params (JSON)

### Fields added to `sale.order.line`
- `measurement_ids` (One2many → `sale.measurement`)
- `has_measurements` (Boolean, computed)
- Cost category links (for FIEBDC3 decomposition)

### Key interaction
- SOL `product_uom_qty` is **overridden** when measurements exist (computed from `sum(measurement.partial)`)
- SOL hierarchy (parent/child via `indaws_sale_order_line_code`) drives project task creation AND certification line structure

---

## 2. Project ↔ inBuilt

### Fields added to `project.project`
- `sale_order_ids` (One2many back-ref)
- `project_certification` (Boolean — enable certifications)
- `certification_billing_type` (Selection: origin/current)
- `certification_product_id` (Many2one → product for invoice lines)
- `certification_previous_billing_product_id` (for origin adjustments)
- `retention_percent` (Float, 0-100)
- 30+ computed margin fields (budget vs real: amounts, costs, deviations)

### Fields added to `project.task`
- `parent_sol_id` (Many2one → sale.order.line that created it)

### Key interactions
- Project creation from SO via wizard with hierarchy options
- Task hierarchy mirrors SO line hierarchy (chapter→part→subpart)
- Margin computation chains: SO lines → analytic lines → task costs → project totals

---

## 3. Account ↔ inBuilt

### Fields added to `account.move`
- `certification_id` (Many2one → project.certification)
- `is_retention_move` (Boolean)
- `retention_percent` (Float)

### Invoice creation flow
1. `project.certification.create_invoice()` → `account.move.create()`
2. Invoice lines from certification lines (3 presentation modes)
3. Optional: adjustment line for previous certifications (origin mode)
4. Optional: retention move (separate `account.move`)
5. Auto-post if configured

### Retention mechanics
- Separate invoice document (NOT a line on main invoice)
- Customer pays main invoice immediately, retention invoice on project completion
- Uses `indaws_account_retention` module for base retention logic

---

## 4. Stock/MRP ↔ inBuilt

### Via bridge modules (3 modules)
- `inbuilt_sale_project_stock_material` → material costs on project tasks
- `inbuilt_sale_margin_project_stock_material` → material margins
- `inbuilt_sale_project_mrp_stock_material` → BOM explosion on tasks

### Key interactions
- Stock moves create cost entries in `project.cost.summary`
- MRP BOMs explode into component materials per task
- Valuation method (AVCO/FIFO/Standard) affects project cost accuracy

---

## 5. Analytic ↔ inBuilt

### `account.analytic.line` extended
- Cost summary aggregation by cost category
- Links to project.task and project.project for margin computation

### Key interactions
- Timesheets → analytic lines → project cost (labor)
- Expenses → analytic lines → project cost (other)
- Stock moves → analytic lines → project cost (materials)
- All feed into `project.cost.summary` and margin computed fields

---

## 6. HR/Timesheet ↔ inBuilt

### Indirect dependency (via project)
- Employee timesheets on project tasks → `account.analytic.line`
- Timesheet cost = `employee.hourly_cost × unit_amount`
- Feeds into project margin: budgeted hours (from SO) vs actual hours (from TS)

---

## 7. indaws_core Dependencies (Critical — Field-Level Detail)

### 7.1. `indaws_sale_order_type` (model: `indaws.sale.order.type`)

**Fields ADDED by inBuilt to this model:**
- `create_project` (Boolean) — added by `inbuilt_sale_project`
  - Used in `sale.order._compute_show_project_and_task_button()` (4 references)
  - Used in `sale.order.action_create_project()` to trigger custom wizard
  - Used in `project.project` domain filters
- `measurements` (Boolean) — added by `inbuilt_sale_measurement`
  - Propagated to `sale.order.line` via `related` field `has_measurements`
  - Controls whether measurement tab/calculations are active

**Views inherited:**
- `indaws_sale_order_type.view_indaws_sale_order_type_form` — both modules place fields in group `name="sale_secondary_group"`

**Risk**: If `indaws_sale_order_type` renames the form view or removes the group, both inbuilt modules break.

### 7.2. `indaws_sale_margin` (cost category engine)

**How inBuilt uses it:**
- `inbuilt_sale_margin` extends margin computation: adds `indirect_purchase_price` on top of `purchase_price_total` from indaws
- `inbuilt_sale_margin_project` reads `cost_summary_ids` from `sale.order` (computed by indaws_sale_margin)
- `inbuilt_fiebdc3_import` dynamically checks `_get_module_installed('indaws_sale_margin')` to decide if cost categories are available

**Security group referenced:**
- `indaws_sale_margin.group_sale_advanced_costs` — used in `inbuilt_sale_margin_project` views to control visibility of margin summary page

**Views inherited:**
- `indaws_sale_margin.view_order_form` — adds fields after group `name="margin"`
- `indaws_sale_margin.sale_order_line_cost_category_view_form` — FIEBDC3 adds import-specific fields

**Risk**: Changes to `purchase_price_total` computation, `cost_summary_ids` structure, or the security group break margin analysis.

### 7.3. `indaws_sale_order_line_code` (hierarchical line codes)

**How inBuilt uses it:**
- `inbuilt_project_certification` uses `sale.order.line.parent_id` and `line_child_ids` to build hierarchical certification lines
- `inbuilt_fiebdc3_import` uses line codes (1, 1.1, 1.1.1) for chapter/part/item structure from BC3 files
- Certification wizard propagates parent/child selection bidirectionally using these fields

**Risk**: If the hierarchy model changes (field rename, relation type change), certification import AND FIEBDC3 import both break.

### 7.4. `indaws_account_retention` (retention/withholding)

**How inBuilt uses it:**
- `inbuilt_project_certification` delegates retention invoice creation to indaws_account_retention
- Migration v18.0.2.1.0 moved retention field ownership FROM inbuilt TO indaws_account_retention
- Certification reads `retention_percent` and calls `_prepare_retention_move()` from indaws

**Risk**: Changes to retention flow (journal, account assignment, move creation) directly affect certification invoicing.

### 7.5. Bridge modules (dependency only, no code)

| inBuilt Module | indaws Dependency | Purpose |
|----------------|-------------------|---------|
| `inbuilt_sale_project_stock_material` | `indaws_project_sale_margin_stock_material` | Ensures material costs propagate to project tasks |
| `inbuilt_sale_project_mrp_stock_material` | `indaws_project_mrp_stock_material` | Ensures BOM components propagate to project tasks |
| `inbuilt_purchase_sale_margin_project` | `indaws_purchase_sale_margin_project` | Ensures PO costs appear in project margin |

These are pure auto-install bridges — no code, just dependency wiring. Risk is low unless the indaws module is removed.

### 7.6. Field-Level Dependency Map

| Field | Model | Source | Used By | Breaks If |
|-------|-------|--------|---------|-----------|
| `create_project` | `indaws.sale.order.type` | inbuilt_sale_project | SO project button, wizard | Field removed/renamed |
| `measurements` | `indaws.sale.order.type` | inbuilt_sale_measurement | SOL measurement tab | Field removed/renamed |
| `order_type_id` | `sale.order` | indaws_sale_order_type | Both above (navigation) | Relation type change |
| `parent_id` | `sale.order.line` | indaws_sale_order_line_code | Certification hierarchy | Hierarchy model change |
| `line_child_ids` | `sale.order.line` | indaws_sale_order_line_code | Certification wizard propagation | Field removed |
| `cost_summary_ids` | `sale.order` | indaws_sale_margin | Project margin aggregation | Model/field rename |
| `purchase_price_total` | `sale.order` | indaws_sale_margin | Indirect cost computation | Compute logic change |
| `group_sale_advanced_costs` | `res.groups` | indaws_sale_margin | View visibility | XML-ID removed |
| `retention_percent` | `project.project` | indaws_account_retention | Certification invoicing | Field relocated |

### Upgrade risk zones (summary)
- `indaws_sale_margin` changes to cost category model → breaks FIEBDC3 import mapping + project margins
- `indaws_sale_order_line_code` changes to line hierarchy → breaks certification line import + FIEBDC3
- `indaws_account_retention` changes to retention flow → breaks certification invoice creation
- `indaws_sale_order_type` view changes → breaks measurement and project creation UI
- Any model field rename in indaws → breaks computed field chains in margin_project (30+ fields)

---

## 8. Cross-Module Data Flow (End-to-End)

### Construction Project Lifecycle in Odoo
```
1. BUDGET PHASE
   BC3 file → fiebdc3.job → sale.order (with lines, costs, measurements)

2. QUOTATION PHASE
   sale.order → adjust margins → add measurements → send to client

3. PROJECT START
   sale.order → wizard → project.project + project.task (hierarchy)

4. EXECUTION PHASE
   project.task → stock.picking (materials)
                → hr_timesheet (labor hours)
                → purchase.order (subcontract/materials)

5. CERTIFICATION (monthly/milestone)
   project.certification → select SO lines → set certified qty
                        → create invoice (+ retention)

6. MARGIN ANALYSIS
   project.project form → 30+ fields showing:
     - Budgeted vs real: revenue, costs, margins
     - By category: labor, materials, machinery, subcontract, overhead
     - Deviations: cost overrun, revenue shortfall

7. PROJECT CLOSE
   Final certification (100%) → release retention → archive project
```
