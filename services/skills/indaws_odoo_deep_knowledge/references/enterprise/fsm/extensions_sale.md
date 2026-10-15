# FSM — Sale integration (`industry_fsm_sale` + `_report` + `_subscription`)

Bridges between Field Service and the sale chain. Adds SO ↔ task wiring, employee-rate pricing, materials-as-SOL, customer signature on report, recurring tasks from subscriptions, and warranty handling.

**Modules:**
- `industry_fsm_sale` (4064 py lines) — core SO ↔ task wiring.
- `industry_fsm_sale_report` — worksheet template propagation from product, "Sign Report" buttons.
- `industry_fsm_sale_subscription` (32 py lines) — subscription product filtering on FSM task materials.

**Dependencies:** `industry_fsm`, `sale_timesheet_enterprise`, `project_sale_subscription`.

---

## 1. `product.template` extensions

Source: `industry_fsm_sale/models/product_template.py:11`, `ProductTemplate.project_id`.

- `project_id` domain restricted to FSM projects with `allow_billable=True`. If `service_policy='delivered_timesheet'`, project must also `allow_timesheets`.
- Service products with `service_tracking IN ('task_new_project', 'task_in_project', 'task_global_project')` auto-create FSM tasks on SO confirmation.

### Worksheet template auto-binding
Source: `industry_fsm_sale_report/models/product_template.py:15-39`, `ProductTemplate._compute_worksheet_template_id`.

- On `create()`: if `allow_worksheets`, `service_tracking IN ('task_global_project','task_new_project')`, and project is FSM → product's `worksheet_template_id` is auto-computed from project default.
- On `write()`: recomputed if `service_tracking` or `project_id` changed.
- Onchange on `project_id` (lines 36-39): auto-fills `worksheet_template_id` from project setting.

Task created from this product inherits the template via `default_worksheet_template_id` context (`industry_fsm_sale_report/models/sale_order_line.py:10-19`, `SaleOrderLine._timesheet_create_task`).

---

## 2. SO → FSM task lifecycle

### `sale.order.line._timesheet_create_task()`
Standard `sale_project` mechanism. FSM override at `industry_fsm_sale/models/sale_order.py:79-83`, `SaleOrderLine._timesheet_create_task_prepare_values` (`_timesheet_create_task_prepare_values`): sets `partner_id = order.partner_shipping_id` instead of `order.partner_id` (FSM consumes the **delivery address**, not the invoicing one).

### `sale.order.action_confirm()`
Override at `industry_fsm_sale/models/sale_order.py:28-37`, `SaleOrder.action_confirm`: posts a `mail.message_origin_link` chatter message linking SO → task. No record state change.

### Task creation from FSM task (reverse direction)
`industry_fsm_sale/models/project_task.py:494-512`, `Task._fsm_create_sale_order` — `_fsm_create_sale_order()` creates an SO from an FSM task:
- `partner_id` = task partner.
- `company_id` = task company (fallback to partner company if task has none).
- `origin` = `"{Project} - {Task}"`.
- Context flag `fsm_create_sale_order=True` suppresses standard SO lock on confirm (see §6).

### Task ↔ SO link via partner
`_compute_sale_order_id()` (lines 239-255): task auto-binds to an SO if the task's `partner_id` matches **either** `SO.partner_id` OR `SO.partner_shipping_id` (line 252-254). This is broader than the default `sale_project` heuristic.

### `_compute_partner_id()` override
Lines 257-265 — task partner inherits `SO.partner_shipping_id` when group `account.group_delivery_invoice_address` is enabled (i.e. shipping address differs from invoicing). Auto-syncs if SO shipping address is updated after task linked.

---

## 3. Pricing — `task_rate` vs `employee_rate`

Computed on `project.project` (`industry_fsm_sale/models/project_project.py:83-91`, `Project._compute_pricing_type`):
- `employee_rate` when `sale_line_employee_ids` is set on the project.
- `task_rate` otherwise.

### task_rate flow (project_task.py:650-673)
- Single SOL created using `timesheet_product_id` from project.
- Qty = sum of billable timesheets.
- Price unit from SO pricelist (`_get_product_price(product, qty)`) or product list price.
- `under_warranty=True` → `price_unit=0`.

### employee_rate flow (project_task.py:525-649) — 7-step algorithm
1. Group timesheets by `employee_id`.
2. For each employee, look up `project.sale.line.employee.map` (employee + project) to find their billing product + price.
3. Build a dict `{(product_id, price_unit, uom_id): [timesheet_ids]}`.
4. Timesheets from unmapped employees fall back to project default `timesheet_product_id` (if `task SOL` doesn't exist yet).
5. Classify existing service SOLs by `(product_id, price_unit)` to merge.
6. Link default-product SOL to its timesheets.
7. Create/update one SOL per unique `(product, price, uom)` combination.

UoM conversion (lines 616-620): timesheet UoM converted to mapping UoM if categories match, using `HALF-UP` rounding.

### `project.sale.line.employee.map`
- `timesheet_product_id` — must be service, delivery policy, timesheet policy.
- `price_unit` — editable for FSM (line 18), normally readonly elsewhere.
- Currency inherited from `timesheet_product_id.currency_id` (line 44).

### Multi-currency
- Task `pricelist_id.currency_id` (computed from SO.pricelist or partner.pricelist) → fallback `company_id.currency_id`.
- Tax-inclusive pricing via `product._get_tax_included_unit_price()` (lines 625-632) — applies fiscal position, date_order from SO.

---

## 4. Material / Product as SOL

### Adding a product on the task (catalog kanban)
Controller `industry_fsm_sale/controllers/catalog.py:8-42`, `CatalogControllerFSM.product_catalog_get_order_lines_info`:
- `product_catalog_get_order_lines_info()` — passes `fsm_task_id` to scope SOL lookups and restricts to task's `allowed_company_ids`.
- `product_catalog_update_order_line_info()` — calls `product.set_fsm_quantity(qty)`. Returns `price_unit`. If product requires tracking (lot/serial), invokes serial-assignment wizard.

### `product.product.set_fsm_quantity(qty)`
Lines 123-152 — the workhorse:
1. Ensures an SO exists via `_fsm_ensure_sale_order()`.
2. **Returns silently if `sale_order.locked=True`** (line 131-132) — user sees no error.
3. For manual service products (`service_type='manual'`): sets `qty_delivered = qty` immediately.
4. For consumables (`type='consu'`): updates qty on the SOL only if SO not locked.

### Material line totals (project_task.py:159-184)
- `material_line_product_count` — count of material (non-timesheet, non-task-SOL) SOLs on task.
- `material_line_total_price` — sum of `price_total`.
- Excludes timesheet product **and** all products used in `project.sale.line.employee.map`.

### `_prepare_materials_delivery()` (project_task.py:702-715)
Called by `action_fsm_validate()`. Sets `qty_delivered = product_uom_qty` for all material SOLs on the task (`:714-715`). Skipped for `service_policy='delivered_milestones'`. **Overridden by `industry_fsm_stock`** to create stock moves instead — without that module installed, this method blindly marks materials delivered (no real inventory impact).

---

## 5. `action_fsm_validate()` override (project_task.py:449)

Calls super (timer + worksheet checks), then for billable tasks with timesheets/materials:
1. `_fsm_ensure_sale_order()` — create SO if missing.
2. If `allow_timesheets`: `_fsm_create_sale_order_line()` for timesheets.
   - **Skipped if `under_warranty AND` only materials** (line 462).
3. Confirm SO if `draft`/`sent`.
4. `_prepare_materials_delivery()` for material SOLs.

### Invoice button visibility (project_task.py:186-209)
- Primary "Create Invoice" visible when: `fsm_done AND allow_billable AND sale_order_id AND invoice_status IN ('upselling','to invoice')`.
- Hidden if `under_warranty AND no_billable_products`.
- Secondary button: partial invoicing logic.

### Invoice wizard hook
`industry_fsm_sale/wizard/sale_make_invoice_advance.py:10-19`, `SaleAdvancePaymentInv._create_invoices` — invoice creation wizard receives `industry_fsm_message_post_task_id` context (dict mapping SO.id → [task_ids]). Posts the resulting invoice link to each task via chatter.

---

## 6. Warranty handling

`under_warranty` (Boolean) field on `project.task` (`industry_fsm_sale/models/project_task.py:44-47`, `Task.under_warranty`).
- View-visible only when `is_fsm AND allow_material`.

### Pricing impact
`_compute_under_warranty_on_existing_sols` (lines 675-690): when toggled, updates **all invoiceable SOLs** on the task:
- `under_warranty=True` → `price_unit=0, technical_price_unit=0`.
- `under_warranty=False` → recomputed from product list price / pricelist.

### Validation impact
- `action_fsm_validate()` skips timesheet SOL creation if `under_warranty AND` no billable products (`_has_no_billable_products`, `:156`; checked at `:462`).
- Material SOLs still created with `price_unit=0` (line 625).

---

## 7. Portal extensions

`industry_fsm_sale/controllers/portal.py:16-92`, `CustomerPortal._task_get_page_view_values`:
- `_task_get_page_view_values()` extended to include quotation links in `task_link_section`.
- New routes:
  - `/my/projects/<int:project_id>/task/<int:task_id>/quotes` — list of SOs linked to task. Defaults to sortby='date'.
  - `/my/projects/<int:project_id>/task/<int:task_id>/invoices` — invoices from task's SO.

---

## 8. SO lock behaviour

`industry_fsm_sale` does **not** override any `_should_be_locked` / locking method on `sale.order` in v18 — SO locking follows standard `sale` behaviour (the "Lock Confirmed Sales" setting / `auto_lock`). The FSM-relevant lock effect is on the consuming side: `product.product.set_fsm_quantity()` (`industry_fsm_sale/models/product_product.py:123`, `ProductProduct.set_fsm_quantity`) returns **False** early when `task.sale_order_id.locked` is true (`industry_fsm_sale/models/product_product.py:131`, `ProductProduct.set_fsm_quantity`) — the catalog "+"/"−" buttons silently do nothing on a locked SO (FSMG25). The task also reads `sale_order_id.locked` to hide the qty buttons (`industry_fsm_sale/models/project_task.py:434`, `Task.action_fsm_view_material`) and to gate post-validate updates (`industry_fsm_sale/models/project_task.py:682`, `Task.write`).

---

## 9. Subscription bridge — `industry_fsm_sale_subscription`

Source: `industry_fsm_sale_subscription/models/project_task.py:10-13`, `Task.action_fsm_view_material`. Minimal module (32 py lines).

- `action_fsm_view_material()` domain extended with `[('recurring_invoice', '=', False)]` — **prevents technicians from adding subscription products as one-shot materials** on FSM tasks.

### Recurring FSM tasks
Subscription renewal cron creates new task via standard `project_sale_subscription` mechanism, **not** via `project.task.recurrence`. The two mechanisms (recurrence vs subscription) coexist:
- `industry_fsm_sale/models/project_task.py:717-723` — `ProjectTaskRecurrence._get_sale_line_id()` returns `False` for FSM tasks, so each recurrence creates its own SO chain.
- `industry_fsm_sale/models/project_task.py:692-700`, `Task` — FSM non-recurring task `copy()` resets `sale_order_id`, `sale_line_id` to allow clean re-binding.

---

## 10. Tests of interest

- `industry_fsm_sale/tests/test_industry_fsm_sale_flow.py:19-106`, `TestFsmFlowSale.test_invoicing_flow` — full catalog → SO → timesheet → validate → invoice.
- `industry_fsm_sale/tests/test_industry_fsm_employee_rate.py:24-156`, `TestIndustryFsmEmployeeRate.test_fsm_employee_rate` — employee_rate routing with 3 mappings and unmapped fallback.
- `industry_fsm_sale/tests/test_industry_fsm_task.py:127-150`, `TestIndustryFsmTask.test_warranty_status_updates_sale_order_line_price` — warranty toggle flipping SOL `price_unit` 0 ↔ list_price.
- `industry_fsm_sale/tests/test_industry_fsm_task.py:12-46`, `TestIndustryFsmTask.test_partner_id_follows_so_shipping_address` — `partner_id` syncing from `SO.partner_shipping_id` when permission group present.

---

## 11. Cross-references

- Stock moves, lots, picking validation → [[extensions_stock]].
- Customer signature + "Send Report" buttons → [[extensions_report]].
- Subscription recurring task cron timing → [[integrations]].
- Worksheet template per-product → [[worksheet]].
- v19 changes (class renames, import shifts) → [[v19_changes]].
- Warranty + locked SO + double-task creation traps → [[gotchas]].
