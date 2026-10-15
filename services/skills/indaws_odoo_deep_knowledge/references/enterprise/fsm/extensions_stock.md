# FSM — Stock integration (`industry_fsm_stock` + `_repair`)

Adds real inventory management on FSM tasks: pickings created from material SOLs, lot/serial tracking via wizard, multi-warehouse logistics, auto-validation on `fsm_done`, no backorder by default.

**Modules:**
- `industry_fsm_stock` (2882 py lines) — material → picking → stock move.
- `industry_fsm_repair` (46 py lines) — access bridge so FSM users can view repair-flagged pickings without `repair.group_repair_user`.

**Dependencies:** `industry_fsm_sale`, `sale_stock`.

---

## 1. Task extensions

Source: `industry_fsm_stock/models/project_task.py`.

| Field | Type | Notes |
|-------|------|-------|
| `stock_move_customer_product_count` | Integer, computed | Count of `stock.move` ending at customer location for SOLs without `task_id`. |

`material_line_product_count` and `material_line_total_price` are inherited from `industry_fsm_sale` and recomputed when stock-driven SOLs change.

---

## 2. SOL → procurement → picking

### `sale.order.line._action_launch_stock_rule()` override
`industry_fsm_stock/models/sale_order_line.py:12-62`, `SaleOrderLine._prepare_procurement_values`:

- When SOL belongs to an FSM task **and** trigger comes from product catalog (context `industry_fsm_stock_set_quantity=True`) or tracking wizard (`industry_fsm_stock_tracking=True`):
  - `warehouse_id` forced to the user's default (`user.property_warehouse_id`).
  - Procurement runs immediately, creating `stock.move` + `stock.move.line` with `fsm_lot_id` attached when tracking is set.
- Qty increased → appends to the last move_line or creates a new one.
- Qty decreased → decrements move_line.quantity; unlinks move_line when it hits zero.

### Warehouse selection (precedence)
1. `user.property_warehouse_id` (technician's default).
2. First warehouse by sequence for the company.

**No per-task warehouse field.** A technician serving from a non-default warehouse must either be reassigned or have their user default updated.

### Auto-validation on `fsm_done`

`industry_fsm_sale/models/project_task.py:175`, `Task._compute_material_line_totals` — `action_fsm_validate()` override (calls `super()` then `_validate_stock()`, `:36-121`):
1. `_validate_stock()` — filters pickings via `is_fsm_material_picking()`:
   - Pickings whose moves end at customer location.
   - Linked to SOLs with `task_id` matching or already `fsm_done`.
2. `move.quantity` set to `product_uom_qty` (forces full delivery) on lines 109-114.
3. `picking.button_validate(skip_sms=True, cancel_backorder=True)` (line 115) — **no backorder is created**. Partial picks are silently truncated.
4. Recursive validation (line 120-121) — push-rule generated downstream pickings (2-step / 3-step delivery) are walked and validated as well.

### Lot-aware reservation
`industry_fsm_stock/models/stock_move.py:12-48`, `StockMove._update_reserved_quantity` overrides `_update_reserved_quantity()`:
- If `product.tracking='lot'` and SOL has `fsm_lot_id` set: prioritises reserving from that specific lot.
- If `product.tracking='serial'`: matches each serial in `fsm_lot_id` to its own move_line (quantity=1.0 per quant).

### Constraint on under-delivery
Lines 18-32 — if total `stock.move` qty < ordered qty on SOL when validating, raises an activity-style exception via `_log_decrease_ordered_quantity()`.

---

## 3. Lot / serial tracking wizard

`industry_fsm_stock/wizard/fsm_stock_tracking.py` (~222 lines). Transient model.

### Entry
`product.action_assign_serial()` (lines 98-148) — triggered when the technician tries to set quantity for a tracked product. Creates wizard with:
- `tracking_line_ids` — pending lines (no `fsm_lot_id` or qty not yet delivered).
- `tracking_validated_line_ids` — already-delivered move_lines (readonly).

### Form behaviour (`fsm_stock_tracking_views.xml`)
- Editable list, `lot_id` domain filtered to `('product_id', '=', product_id)`.
- Serial: `quantity` field hidden (always 1).
- Lot: `quantity` editable.
- `is_same_warehouse` computed — if the underlying move is on a **different** warehouse than the user's default, the row is muted and readonly.

### `generate_lot()` submit (lines 106-186)
1. Creates new SOLs for new lots (line 113-134).
2. Updates `fsm_lot_id` on existing SOLs if lot changed.
3. **Syncs intermediate pickings** (2-step / 3-step deliveries): creates / removes `stock.move.line` to mirror the lot changes (line 136-178).
4. **If warehouse mismatch** (intermediate picking on a different WH than the technician's default): **cancels the moves** instead of updating them (line 156-158). Effectively orphans the lot assignment on intermediate stages.

### Mobile tablet behaviour
- Context `fsm_task_id` is passed; product pre-populated.
- Technician picks a lot from quants available **at the technician's warehouse only**.
- Save → `qty_delivered` updated, `move_line.quantity` set, downstream pickings recursively validated.

---

## 4. Quantity decrease / returns

### `product._inverse_fsm_quantity()` (lines 90-96)
Wraps the parent with `industry_fsm_stock_set_quantity=True` context. Before writing, checks `quantity_decreasable_sum` (sum of `move.product_uom_qty` for moves not yet done, per WH + task).

If new `fsm_quantity < quantity_decreasable_sum`: raises `UserError` "cannot decrease below delivered amount".

### Return flow (manual only)
- No FSM-driven return UI.
- Workarounds:
  1. Negative SOL on the SO (manual or return wizard).
  2. Return picking via Inventory app (`stock.return.picking` wizard).
- Returns are **not** linked back to the task.

---

## 5. `industry_fsm_repair` — access bridge only

Minimal module (46 py lines). Purpose: let FSM users **view** FSM-created `stock.picking` without holding `repair.group_repair_user`.

- `industry_fsm_repair/security/ir.model.access.csv` — FSM group gets READ on stock.picking; repair group keeps full CRUD.
- `industry_fsm_repair/views/stock_picking_views.xml:8-14`, `repair_view_picking_form_no_repair_access` — hides repair-specific header buttons (`action_repair_return`, `is_repairable`, `repair_ids`) when the picking is viewed by an FSM-only user.
- `industry_fsm_repair/models/stock_move.py:9-24`, `StockMove.action_open_reference_no_repair` — `action_open_reference_no_repair()`: returns picking form if accessible, falls back to `stock.move` form on AccessError.

**There is no FSM → repair workflow.** A repair order needs to be created manually; no auto-sync between `task.fsm_done` and `repair.order.state`.

---

## 6. Inventory rules

### Reservation policy
- Initial `stock.move` created in `draft` state on SO confirm.
- `picking.action_confirm()` → moves transition to `confirmed` → `_update_reserved_quantity` runs with the FSM lot-aware override.
- For FSM-driven SOLs, reservation happens at SOL creation time (via the catalog wizard), not at SO confirm time.

### Backorder
**Default: cancel_backorder=True.** Partial deliveries do not generate backorders. A 10-unit SOL with only 5 picked finishes with `qty_delivered=5` and `state='done'`, no follow-up picking.

### Validation order on `fsm_done`
1. Timer check (`action_fsm_validate`).
2. Worksheet/signature checks (if `industry_fsm_report` installed).
3. Material check — `_prepare_materials_delivery()`.
4. Stock validation — `_validate_stock()`.
5. SO lock (`sale_order.action_lock()` unless `fsm_create_sale_order` context).

---

## 7. Multi-warehouse caveats

### 2-way / 3-way deliveries
- SO confirms with `warehouse_id = SO.warehouse_id` (typically picked from partner/company default).
- Intermediate transfers (output zone, packing) created by routing rules.
- Technician's warehouse may differ → tracking wizard detects mismatch (`is_same_warehouse` flag), greys out the row, and on save **cancels the intermediate moves** rather than syncing the lot.

### Implication
For organisations with 2-step delivery, ensure technicians' default warehouse equals the SO warehouse, **or** customise routing rules so intermediate moves are created on the technician's warehouse.

---

## 8. Tests of interest

- `industry_fsm_stock/tests/test_fsm_stock.py` — material catalog → SO → picking → validate flow.
- `industry_fsm_sale/tests/test_multicompany.py:142-165`, `TestMultiCompany.test_task` — cross-company SO confirmation; warehouse selection.

---

## 9. Cross-references

- Sale-side mechanics (catalog UI, SOL pricing) → [[extensions_sale]].
- Picking validation order with worksheet → [[extensions_report]].
- v19 changes (`Domain.OR` etc.) → [[v19_changes]].
- Backorder silent truncation, return manual workflow → [[gotchas]].
