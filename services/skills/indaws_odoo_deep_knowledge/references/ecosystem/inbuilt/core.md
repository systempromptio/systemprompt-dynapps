<!-- verified: v18.0 @ 2026-06 (source $PROJECTS_ROOT/vertical-odoo-inbuilt; 15-claim sample vs source, 2 corrected: model count + IBG17) -->
# inBuilt — Construction Industry Vertical

## Overview
**16 modules**, ~4,800 lines Python, **7 custom models** (+ 2 abstract mixins + 4 wizards), extends 17+ Odoo models.
Vertical for construction: progressive billing (certifications), measurement-based pricing,
FIEBDC3 budget import, and project profitability analysis.

Depends on **indaws_core** (layers 1-3: indaws_base, indaws_sale, indaws_account, indaws_product).

---

## Module Architecture (6 Layers)

### Layer 1 — Foundation
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_base` | `indaws_base` | Auto-install foundation, category registration |
| `inbuilt_base_sale` | `indaws_base_sale` | Config toggles: measurement, FIEBDC3, sale_project |
| `inbuilt_base_project` | `indaws_base_project` | Config toggle: project_certification |

### Layer 2 — Core Sales
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_sale` | `inbuilt_base_sale` | `description_large` on SOL |
| `inbuilt_sale_margin` | `inbuilt_base_sale`, `indaws_sale_margin` | Indirect cost % on purchase price, real margin |

### Layer 3 — Measurement
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_sale_measurement` | `inbuilt_base_sale`, `indaws_sale_order_line_edit`, `indaws_sale_order_type` | Physical dimensions → qty |

### Layer 4 — Project Integration
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_sale_project` | `sale_project`, `indaws_sale`, `inbuilt_sale` | SO→Project wizard, hierarchical tasks |
| `inbuilt_sale_margin_project` | `inbuilt_sale_margin`, `inbuilt_sale_project` | 30+ computed fields: budget vs real margins |
| `inbuilt_purchase_sale_margin_project` | above + `indaws_purchase_sale_margin_project` | PO costs in project margin |
| `inbuilt_sale_project_stock_material` | `indaws_project_sale_margin_stock_material` | Bridge: materials on tasks |
| `inbuilt_sale_margin_project_stock_material` | above | Bridge: material margin |
| `inbuilt_sale_project_mrp_stock_material` | `indaws_project_mrp_stock_material` | Bridge: MRP BOM on tasks |

### Layer 5 — Certification & Billing
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_project_certification` | `inbuilt_base_project`, `inbuilt_sale_project`, `indaws_sale_order_line_code`, `indaws_account_retention` | Progressive invoicing |
| `inbuilt_project_certification_measurement` | above + `inbuilt_sale_measurement` | Measurement→certification audit trail |

### Layer 6 — Data Import
| Module | Depends | Purpose |
|--------|---------|---------|
| `inbuilt_fiebdc3_import` | `indaws_sale_order_line_code`, `inbuilt_sale`, `inbuilt_sale_margin` | BC3 budget import |
| `inbuilt_fiebdc3_import_material` | `indaws_project_sale_margin_stock_material`, `inbuilt_fiebdc3_import` | BC3→stock materials |

---

## Custom Models

### `sale.measurement`
Dimension-based quantity: `partial = units × length × width × height`.
One2many from `sale.order.line.measurement_ids`.
Used in quotation phase. Locks `product_uom_qty` when measurements exist.

### `project.certification`
**State machine**: `draft → invoiced → cancel`
Main record for progressive billing. Key fields:
- `certified_amount`, `certified_percent`, `certified_before_amount` (cumulative)
- `retention_percent` — optional withholding (0-100%)
- `billing_type`: `origin` (cumulative + adjustments) or `current` (delta only)
- `invoice_presentation`: `consolidated` | `all_lines` | `certified_lines`

### `project.certification.line`
Hierarchical (parent/child via `section.line.mixin`). Links to `sale.order.line`.
Key fields: `quantity_certified`, `certified_percent`, per-line amounts.

### `project.certification.measurement`
Snapshot of `sale.measurement` data at certification time → audit trail.
Prevents post-certification quantity manipulation.

### `project.cost.summary`
Aggregated cost breakdown by category (`sale.order.line.cost.category`) at project level.
Computed from analytic lines + stock moves + timesheets.

### `fiebdc3.job`
**State machine**: `draft → queued → running → done | failed`
Manages async BC3 import. Cron-triggered (`trigger_run()`). Progress tracking via `percentage`.

### `fiebdc3.percentage.map`
Maps BC3 percentage codes (e.g., `%GG`, `%0*`) to Python expressions.
Uses `safe_eval` with context: `code`, `bc3`, `siblings`, `base`, `material_base`.

---

## Key Workflows

### 1. SO → Project → Certification → Invoice
```
sale.order (manual or FIEBDC3 import)
  → inbuilt.project.create wizard (chapter/part/subpart tasks)
    → project.project + project.task hierarchy
      → project.certification (select lines via wizard)
        → project.certification.line (set certified quantities)
          → account.move (invoice) + optional retention move
```

### 2. Certification Billing Modes

**Origin mode**: Invoice shows cumulative total minus previous certifications.
Creates adjustment line referencing `certification_previous_billing_product_id`.

**Current mode**: Invoice shows only current period delta. No adjustments.

### 3. Retention
Optional per-project. Creates separate `account.move` with `is_retention_move=True`.
Modes: `unique_line` (one retention move per certification) or `per_invoice`.

### 4. Measurement → Certification
If `inbuilt_project_certification_measurement` installed:
- Certification lines can have `certification_measurement_ids`
- Certified quantity auto-computed from measurements
- Manual qty editing blocked when measurements exist

### 5. FIEBDC3 Import
BC3 file → `fiebdc3.job` (queued) → cron processes → creates SO with:
- Hierarchical lines (chapters/parts/items via `indaws_sale_order_line_code`)
- Cost matrix from percentage maps (labor, materials, machinery, subcontract)
- UOM mapping (BC3 units → Odoo uom.uom)
- Product creation/matching

---

## Odoo Models Extended

| Model | Fields/Logic Added | By Module |
|-------|-------------------|-----------|
| `sale.order` | project creation, FIEBDC3 params, indirect cost margin | sale_project, fiebdc3, sale_margin |
| `sale.order.line` | `measurement_ids`, `description_large`, cost categories | sale, measurement, certification |
| `project.project` | `sale_order_ids`, certification config, 30+ margin computed fields | sale_project, margin_project, certification |
| `project.task` | `parent_sol_id`, material cost tracking | sale_project, margin, purchase |
| `account.move` | `certification_id`, `is_retention_move`, `retention_percent` | certification |
| `account.analytic.line` | cost summary aggregation | margin_project |
| `indaws.sale.order.type` | measurement flag, project creation options | measurement, sale_project |
| `res.company` | certification products, FIEBDC3 config, indirect cost % | multiple |
| `res.config.settings` | all toggles (6 modules add settings) | multiple |
| `product.template` | FIEBDC3 code link | fiebdc3_import |
| `uom.uom` | BC3 UOM mapping | fiebdc3_import |

---

## Indaws Core Dependencies (Critical)

| Indaws Module | Used For |
|---------------|----------|
| `indaws_base` | Foundation |
| `indaws_base_sale` / `indaws_sale` | Sale order extensions |
| `indaws_sale_margin` | Cost categories, margin engine |
| `indaws_sale_order_line_code` | Hierarchical line codes (1, 1.1, 1.1.1) |
| `indaws_sale_order_line_edit` | Inline line editing |
| `indaws_sale_order_type` | Order type classification |
| `indaws_account_retention` | Retention/withholding on invoices |
| `indaws_base_project` / `indaws_project` | Project foundation |
| `indaws_project_sale_margin_stock_material` | Material cost on projects |
| `indaws_project_mrp_stock_material` | MRP BOM on project tasks |
| `indaws_purchase_sale_margin_project` | PO cost→project margin |

---

## Security

- **No custom groups** — uses standard Odoo groups (`sale.group_sale_salesman`, `project.group_project_user`, etc.)
- **Multi-company rules** on: `project.certification`, `project.certification.line`, `sale.measurement`
- **ACL files** in: certification, fiebdc3_import, sale_project, measurement modules

---

## Budget vs Real — Data Flow Direction (CRITICAL)

The margin engine in `inbuilt_sale_margin_project` is a **one-way dashboard**, NOT a sync mechanism. Two independent data sources are compared, and the gap is shown as deviation:

```
┌──────────────────────────┐         ┌──────────────────────────┐
│ Budgeted                 │         │ Real                     │
│ sale.order.line.cost     │         │ account.analytic.line    │
│ (descompuestos)          │         │ (timesheets, vendor      │
│                          │         │  bills, picking entries, │
│                          │         │  MOs)                    │
└────────┬─────────────────┘         └────────┬─────────────────┘
         │                                    │
         ▼                                    ▼
   sale_cost_summary_ids               project_cost_summary_ids
         │                                    │
         └────────────────┬───────────────────┘
                          ▼
                cost_deviation =
                sale_direct - real_direct
                (Summary tab — read-only)
```

**No write-back exists.** Real costs (e.g., a supplier invoice for machine rental imputed to the project's analytic account) appear as **real cost** and as **deviation**, but the SOL's `cost_ids` are NEVER updated automatically. To reflect a real cost in the budget, edit the SOL manually — `indaws_sale_order_line_edit` (in the vertical's dep tree) permits this on confirmed orders.

### Edge case: Real-cost categorization

`inbuilt_purchase_sale_margin_project` overrides `_set_default_cost_category` on `account.analytic.line` to assign `cost_category_id` based on the line's origin (vendor_bill→`is_purchase`, picking_entry/MO→`is_material`, timesheet→`is_time`). If no category exists with the expected flag, the cost falls into "Other" with NO visual alert. See **IBG72** (analytic with mismatched account_id is silently invisible).

### Edge case: Multi-currency

`sale_cost_summary_ids` aggregates raw subtotals across SOs without converting currencies (IBG69). On a project with SOs in EUR and USD, totals are arithmetically wrong. The model has `currency_id` related to the SO, not the project — but the aggregate ignores it.

---

## Certification State Machine — Hidden Behaviors

The state of `project.certification` is **computed** from `move_ids.state`:

```python
@api.depends("move_ids.state")
def _compute_state(self):
    for cert in self:
        active_invoices = cert.move_ids.filtered(
            lambda m: not m.is_retention_move and m.state in ('draft', 'posted')
        )
        cert.state = 'invoiced' if active_invoices else 'draft'
```

Implications:
- **Draft invoices count as "invoiced"** (IBG30) — cancel or unlink the draft before re-editing the certification.
- Cancelling all invoices reverts the certification to `draft` automatically.
- `action_set_draft` / `action_cancel` are blocked by `_constrains_state_with_invoices` while any non-retention invoice is in draft/posted.

### Edge case: Intermediate cancellation

Cancelling certification N (mid-sequence) does NOT recompute `certified_before_amount` of N+1, N+2... because the depends is `@api.depends("project_id")` only (IBG29). To force a refresh, write any value back to `project_id` on each affected certification.

### Edge case: Mixing SOs in one project

The `Add Lines` wizard enforces consistency across selected SOs:
- Same `currency_id`
- Same `fiscal_position_id`
- Same `partner_invoice_id`

With `delete_existing_lines=False`, it validates against pre-existing certification lines too (IBG35).

---

## Measurement → Certification Snapshot Lifecycle

`project.certification.measurement` is created **in the `create` of the certification line**, copying every `sale.measurement` of the SOL. It's a one-shot snapshot, NOT a live link.

Consequences:
- **Adding** measurements to the SOL after certification creation does NOT propagate (IBG63). The line is marked `sol_has_measurements=True` (blocking manual edits) but new measurements have no snapshot.
- **Editing** dimensions of an already-certified measurement updates `partial` (related, store=True) but does NOT revalidate `quantity_certified` (IBG64). Possible to end up with `quantity_certified > partial`.
- **Deleting** a `sale.measurement` cascades to `project.certification.measurement` regardless of certification state (IBG65). Audit trail lost.

---

## FIEBDC3 Import — Transaction Semantics

The job runs **without rollback**:

```python
# inbuilt_fiebdc3_import/models/fiebdc3_job.py
# Comment: "Deliberate transaction commits — refactor to a job-queue pattern is out of scope"
env.cr.commit()  # at job start
for chunk in chunks:
    process(chunk)
    env.cr.commit()  # per chunk (default 100, ir.config_parameter override)
except:
    env.cr.commit()  # before raising
    raise
```

Implications:
- A failed job leaves processed chunks persisted in BD. Re-queueing merges by code (IBG39).
- Two simultaneous imports on the same SO have a race window between job creation and the `fiebdc3_job_id` recompute (IBG40).
- The cron is event-driven via `_trigger()` (IBG42). If disabled, jobs queue forever with no watchdog.

### Edge case: Re-import semantics

Re-import on an existing SO is **merge by code**, not replace. Lines not in the new BC3 stay (orphan); `cost_ids` and `measurement_ids` on matched lines are wiped and recreated (IBG39).

### Edge case: Ephemeral product reuse

If the import finds an existing ephemeral product (`product.template.fiebdc3_sale_id != False`) matching by `default_code`, it reuses it. But the product's `fiebdc3_sale_id` still points to the FIRST SO that created it. Deleting that first SO cascades the product (IBG44), breaking the second SO's references.

---

## Project Creation — Sync Semantics

`inbuilt.project.create` wizard updates tasks **in-place** via `sale.order.line.task_id`:

| Update on re-run | Behavior |
|------------------|----------|
| Task name `[code] description` | **Overwritten** (re-composed from current SOL) |
| Task description | **NOT overwritten** (only written on CREATE) |
| `allocated_hours` | **Overwritten** (recomputed from `cost_ids.is_time`) |
| `parent_id` | **Updated** if SOL's `parent_id` changed |
| Task on deleted SOL | **Orphaned** (no cascade, no cleanup) |

Asymmetry between name (synced) and description (not synced) is undocumented in the code — surprises users (IBG50).

---

## Retention — Manual Release Required

`indaws_account_retention` covers:
- Creation of retention `account.move` on `_post` ✓
- Cancellation on factura cancel ✓
- Unlink on factura delete ✓
- **Release back to customer** ✗ (NO automatic flow — IBG73)

End-of-project workflow:
1. Generate manual reverse journal entry: debit receivable, credit retention account.
2. Reconcile with the customer's final payment.

Changing `retention_percent` on a posted invoice does NOT update the existing retention move (IBG74).
