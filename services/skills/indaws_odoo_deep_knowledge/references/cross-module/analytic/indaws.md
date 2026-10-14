# Indaws Analytic Modules — Odoo 18.0

## Core Analytic Modules

### 1. indaws_pos_analytic — POS Analytic Distribution
**Path:** `indawsodoo/18/indaws_pos_analytic/`
**Deps:** `indaws_base_pos`
**Purpose:** Adds POS config dimension to analytic distribution rules.

**What it does:**
- Extends `account.analytic.distribution.model` with `pos_config_id` (Many2one to `pos.config`)
- Filters distribution models by active POS config in context
- `_get_distribution()` and `_get_applicable_models()` overridden to scope by POS

**Solves:** ANG22 (POS closing entry has no analytic). Allows different analytic distributions per POS shop.
**Enabled via:** `module_indaws_pos_analytic` checkbox in `indaws_base_pos` settings.

### 2. indaws_sale_order_type_analytic — Sale Type Distribution
**Path:** `indawsodoo/18/indaws_sale_order_type_analytic/`
**Deps:** `account`, `analytic`, `sale`, `indaws_sale_order_type`

**What it does:**
- Extends `account.analytic.distribution.model` with `sale_type_id` (Many2one to `sale.order.type`)
- Extends `sale.order.line._compute_analytic_distribution()` to include sale type in matching params
- Allows targeting specific sale order types for auto-fill rules

**Use case:** "All B2B orders → Project: Wholesale", "All B2C orders → Project: Retail"

### 3. indaws_purchase_sale_margin_project — Project Costing
**Path:** `indawsodoo/18/indaws_purchase_sale_margin_project/`
**Deps:** `project_purchase`, `indaws_sale_margin`
**Auto-install:** True

**What it does:**
- Adds `task_id` + `cost_category_id` to `account.analytic.line`
- Adds same fields to `account.move.line`
- Overrides `_prepare_analytic_distribution_line()` to propagate task and cost category

**Use case:** Per-task cost tracking with categorization (materials, travel, subcontracting).

## Extension Modules (Analytic Propagation)

### 4. indaws_account_invoice_exchange — Exchange Invoice Analytic
**Path:** `indawsodoo/18/indaws_account_invoice_exchange/`
**What:** Copies `analytic_distribution` from original line to cancellation lines in exchange invoices.
**Method:** `account.move._get_offset_line_vals()`

### 5. indaws_purchase_account — Advance Payment Analytic
**Path:** `indawsodoo/18/indaws_purchase_account/`
**What:** Copies `analytic_distribution` from PO line to advance payment invoice lines.
**Method:** `purchase_make_invoice_advance` wizard.

### 6. indaws_account_move_merge — Merge Key Includes Analytic
**Path:** `indawsodoo/18/indaws_account_move_merge/`
**What:** Includes `analytic_distribution` in invoice merge key. Invoices with different distributions are NOT merged.
**Method:** `account.move._get_invoice_line_key_cols()`

### 7. indaws_project_stock_material — Stock → Project Analytic
**Path:** `indawsodoo/18/indaws_project_stock_material/`
**What:** Assigns `project_id` to stock moves for analytic accounting.
**Methods:** `_assign_picking_values()`, `_get_new_picking_values()` propagate project_id.

### 8. indaws_project_sale_margin_stock_material — Project Cost Analytic
**Path:** `indawsodoo/18/indaws_project_sale_margin_stock_material/`
**What:** Overrides `stock.move._prepare_analytic_lines()` for custom project-based cost calculations.

### 9. indaws_hr_timesheet_attendance — Attendance → Analytic
**Path:** `indawsodoo/18/indaws_hr_timesheet_attendance/`
**What:** Extends `hr.attendance` with `timesheet_id` (Many2one to `account.analytic.line`). Creates/updates analytic lines from check-in/check-out.

## View Extensions

### 10. indaws_sale_purchase — Analytic in Invoice Lines Tree
**Path:** `indawsodoo/18/indaws_sale_purchase/`
**What:** Shows `analytic_distribution` widget in `account.move.line` tree view.

### 11. indaws_sale_line_print — Print + Analytic
**Path:** `indawsodoo/18/indaws_sale_line_print/`
**What:** Adds `can_print` field alongside `analytic_distribution` in invoice line forms.

## Summary

| Module | Type | Key Feature |
|--------|------|------------|
| `indaws_pos_analytic` | Core | POS config → distribution rules |
| `indaws_sale_order_type_analytic` | Core | Sale type → distribution rules |
| `indaws_purchase_sale_margin_project` | Core | Task + cost category on analytic lines |
| `indaws_account_invoice_exchange` | Propagation | Analytic survives invoice exchanges |
| `indaws_purchase_account` | Propagation | Analytic on advance payments |
| `indaws_account_move_merge` | Integrity | Merge respects analytic differences |
| `indaws_project_stock_material` | Propagation | Stock moves → project analytic |
| `indaws_project_sale_margin_stock_material` | Propagation | Custom stock → project cost analytic |
| `indaws_hr_timesheet_attendance` | Integration | Attendance → timesheet analytic lines |
| `indaws_sale_purchase` | View | Analytic widget in invoice lines |
| `indaws_sale_line_print` | View | Print flag with analytic |

**Pattern:** All modules work WITH standard Odoo's `analytic_distribution` field — none replaces the core mechanism. They extend matching criteria (POS config, sale type) or ensure propagation through flows Odoo misses (exchange invoices, advance payments, stock materials).
