# MRP Production — Odoo 18.0
<!-- verified: v18.0 @ 2026-06 -->

## 1. The Big Picture — Manufacturing Through Stock Moves

MRP operates entirely through `stock.move` records. Every BOM component and finished product is a move.

```
mrp.production (Manufacturing Order)
│
├── move_raw_ids: One2many → stock.move
│   Component moves: Stock/Components → Production Location
│   Linked via: move.raw_material_production_id = MO
│
├── move_finished_ids: One2many → stock.move
│   Finished product + byproducts: Production Location → Stock/Finished
│   Linked via: move.production_id = MO
│
├── workorder_ids: One2many → mrp.workorder
│   Operations with time tracking and quality checks
│
└── procurement_group_id: Many2one → procurement.group
    Groups all related MOs (original + backorders)
```

MRP doesn't have its own inventory system. It creates stock.move records that go through the standard stock flow (`_action_confirm` → `_action_assign` → `_action_done`).

## 2. Manufacturing Order — Core Model

### State Machine

```
draft → confirmed → progress → to_close → done
                                            ↓
                                          cancel
```

| State | Trigger | Stock Effect |
|-------|---------|-------------|
| `draft` | MO created | Moves in draft, no reservation |
| `confirmed` | `action_confirm()` | Moves confirmed, components reserved (if at_confirm) |
| `progress` | Any component picked OR workorder started OR qty_producing > 0 | Components being consumed |
| `to_close` | All workorders done OR qty_producing >= product_qty | Ready to finalize |
| `done` | `button_mark_done()` → `_post_inventory()` | All moves done, quants updated |
| `cancel` | `action_cancel()` | All moves cancelled |

### Key Fields (Stock-Related)

| Field | Type | Purpose |
|-------|------|---------|
| `product_id` | M2O → product.product | What to manufacture |
| `product_qty` | Float | Quantity to produce |
| `bom_id` | M2O → mrp.bom | Bill of Materials (normal type only) |
| `location_src_id` | M2O → stock.location | Where components come FROM |
| `location_dest_id` | M2O → stock.location | Where finished products go TO |
| `location_final_id` | M2O → stock.location | Ultimate destination (from procurement) |
| `picking_type_id` | M2O → stock.picking.type | Operation type (code=`mrp_operation`) |
| `move_raw_ids` | O2M → stock.move | Component consumption moves |
| `move_finished_ids` | O2M → stock.move | Finished product + byproduct moves |
| `lot_producing_id` | M2O → stock.lot | Lot/serial being produced |
| `qty_producing` | Float | Current production quantity |
| `qty_produced` | Float (computed `_get_produced_qty`) | Total already produced |
| `procurement_group_id` | M2O → procurement.group | Groups related MOs |
| `move_dest_ids` | O2M → stock.move | Downstream moves (e.g., to SO delivery) |

### reservation_state (Component Availability)

| State | Meaning |
|-------|---------|
| `confirmed` | Waiting for components (not all reserved) |
| `assigned` | All components reserved — ready to produce |
| `waiting` | Waiting for another operation |

Depends on `bom.ready_to_produce`: `'all_available'` = need ALL reserved, `'asap'` = need only 1st operation's components.

## 3. Bill of Materials (mrp.bom)

### BOM Types

| Type | Code | Behavior |
|------|------|----------|
| **Manufacture** | `'normal'` | Creates MO. Components consumed, product produced. |
| **Kit (Phantom)** | `'phantom'` | No MO. Components exploded into parent document (SO, MO, picking). |
| **Subcontracting** | `'subcontract'` | Sent to external manufacturer. Components may be supplied. |

### Key Fields

| Field | Type | Purpose |
|-------|------|---------|
| `product_tmpl_id` | M2O → product.template | Product this BOM is for |
| `product_qty` | Float | Base quantity (1 BOM = N units of product) |
| `type` | Selection | normal / phantom / subcontract |
| `bom_line_ids` | O2M → mrp.bom.line | Components |
| `byproduct_ids` | O2M → mrp.bom.byproduct | Byproducts |
| `operation_ids` | O2M → mrp.routing.workcenter | Operations/work centers |
| `consumption` | Selection | `flexible` / `warning` / `strict` |
| `ready_to_produce` | Selection | `all_available` / `asap` |
| `produce_delay` | Integer | Manufacturing lead time (days) |
| `days_to_prepare_mo` | Integer | Advance days to create MO |

### BOM Line (mrp.bom.line)

| Field | Type | Purpose |
|-------|------|---------|
| `product_id` | M2O → product.product | Component |
| `product_qty` | Float | Qty per BOM set |
| `operation_id` | M2O → mrp.routing.workcenter | Which operation consumes it |
| `manual_consumption` | Boolean | If True, qty must be manually confirmed |
| `child_bom_id` | M2O (computed) | Sub-BOM if component has its own BOM |

### Byproduct (mrp.bom.byproduct)

| Field | Type | Purpose |
|-------|------|---------|
| `product_id` | M2O | Byproduct produced |
| `product_qty` | Float | Expected qty per BOM set |
| `cost_share` | Float | % of production cost allocated |
| `operation_id` | M2O | Which operation produces it |

## 4. BOM Explosion — `explode()` Method

Recursively expands multi-level BOMs into a flat component list.

```
BOM: Table (qty=1)
├── Wood Frame (qty=4) → has Normal sub-BOM
│   ├── Wood Plank (qty=1)
│   └── Screws (qty=10)
├── Leg Kit (qty=4) → has Phantom sub-BOM (Kit)
│   ├── Plastic Cap (qty=1)
│   └── Metal Base (qty=1)
└── Finish (qty=0.5L)

explode(Table, quantity=2):
├── Wood Frame: 8 units → sub-BOM preserved (separate MO if route=Manufacture)
│   ├── Wood Plank: 8
│   └── Screws: 80
├── Plastic Cap: 8 → EXPLODED from Leg Kit phantom
├── Metal Base: 8 → EXPLODED from Leg Kit phantom
└── Finish: 1L
```

**Rules:**
- **Phantom BOMs** are "transparent" — components merge into parent's component list
- **Normal sub-BOMs** preserved as sub-assemblies (can trigger separate MOs via MTO)
- Quantities scale: `line_qty = parent_qty * bom_line.product_qty`
- Variant filtering: lines can be limited to specific product variants

## 5. Stock Moves in Manufacturing

### Fields Added to stock.move by MRP

| Field | Type | Purpose |
|-------|------|---------|
| `raw_material_production_id` | M2O → mrp.production | Links to MO (component move) |
| `production_id` | M2O → mrp.production | Links to MO (finished move) |
| `bom_line_id` | M2O → mrp.bom.line | Which BOM line created this |
| `byproduct_id` | M2O → mrp.bom.byproduct | If this is a byproduct move |
| `operation_id` | M2O → mrp.routing.workcenter | Which operation |
| `workorder_id` | M2O → mrp.workorder | Which work order |
| `unit_factor` | Float (computed) | Consumption scaling factor |
| `should_consume_qty` | Float (computed) | Expected consumption for current qty_producing |
| `manual_consumption` | Boolean | From BOM line |
| `order_finished_lot_id` | M2O → stock.lot | Lot being produced |
| `cost_share` | Float | Cost allocation % (byproducts) |

### Location Logic

```python
# Component moves (move_raw_ids):
location_id = MO.location_src_id          # FROM: Components location (usually Stock)
location_dest_id = product.property_stock_production  # TO: Virtual Production location

# Finished product moves (move_finished_ids):
location_id = product.property_stock_production  # FROM: Virtual Production location
location_dest_id = MO.location_dest_id           # TO: Finished goods location (usually Stock)
```

The **Production location** (virtual, usage=`production`) is the intermediate. Components enter it, finished products leave it. No real stock — it's a routing abstraction.

### Unit Factor & Consumption Scaling

```python
unit_factor = product_uom_qty / (MO.product_qty - MO.qty_produced)

# Example: MO produces 10 tables, component needs 20 screws
# unit_factor = 20 / (10 - 0) = 2.0
# If qty_producing = 5:
#   should_consume_qty = (5 - 0) * 2.0 = 10 screws
```

### MRP Overrides on stock.move

All in `mrp/models/stock_move.py`:

| Method | Override | Purpose | Line |
|--------|----------|---------|------|
| `_action_confirm()` | Explodes phantom BOMs (kits) before confirming | Kit moves → component moves | `:527` |
| `_action_done()` | Explodes remaining kits (`is_kits` & not draft/cancel) | Catch-all for unexploded kits | `:533` |
| `_action_assign()` | Writes `production_id` + `workorder_id` onto `move_line_ids` | Traceability | `:519` |
| `_action_cancel()` | Cancels MO if ALL its `move_raw_ids` are cancelled (unless `skip_mo_check`) | Cascade cancel | `:600` |
| `_should_bypass_reservation()` | Kits bypass reservation (components reserve, not kit) — `or product_id.is_kits` | Phantom BOM handling | `:540` |
| `action_explode()` | Skips move if its product == MO product, or no phantom BoM found | Phantom expansion guard | `:543` |

## 6. Manufacturing Flow — Complete Sequence

### 1-Step Manufacturing

```
action_confirm():
  ├── _get_moves_raw_values() → create component moves from BOM
  ├── _get_moves_finished_values() → create finished + byproduct moves
  ├── move_raw_ids._action_confirm() → reserve components
  └── move_finished_ids._action_confirm()

[Operator produces]

button_mark_done() (mrp_production.py:2109) → _post_inventory() (:1802):
  ├── move_raw_ids picked=True → _action_done() → quants decreased
  ├── move_finished_ids.quantity = qty_producing
  ├── move_finished_ids picked=True → _action_done() → quants increased
  ├── move_line traceability: consume_line_ids linked
  └── move_dest_ids._trigger_assign() → chain to delivery
```

### 2-Step Manufacturing (Pick + Manufacture)

```
WH config: manufacture_steps = 'pbm'
Creates: pbm_type_id (Pick Before Manufacturing) + pbm_loc_id

Flow:
  Stock → pbm_loc_id (Pick components, internal transfer)
    ↓
  pbm_loc_id → Production (Manufacturing, MO consumes from pre-production)
    ↓
  Production → Stock (Finished products to stock)
```

### 3-Step Manufacturing (Pick + Manufacture + Store)

```
WH config: manufacture_steps = 'pbm_sam'
Creates: pbm_type_id + sam_type_id + pbm_loc_id + sam_loc_id

Flow:
  Stock → pbm_loc_id (Pick components)
    ↓
  pbm_loc_id → Production (MO consumes)
    ↓
  Production → sam_loc_id (Post-production staging)
    ↓
  sam_loc_id → Stock (Store finished products)
```

## 7. Backorder — Partial Production

When `qty_producing < product_qty` (`_split_productions`, `mrp/models/mrp_production.py:1870`, `MrpProduction._get_backorder_mo_vals`):

```python
_split_productions():
  1. Calculate: backorder_qty = product_qty - qty_producing
  2. Create new MO with backorder_qty
  3. Split move_raw_ids proportionally (backorder gets scaled-down qty)
  4. Split move_finished_ids proportionally
  5. Both MOs share same procurement_group_id
  6. Original MO: product_qty = qty_producing
  7. Backorder confirmed + assigned (if at_confirm)
```

Backorder MO keeps the same `procurement_group_id` for traceability.

## 8. Unbuild — Reverse Manufacturing

`mrp.unbuild` reverses a BOM: finished product → components.

```
Unbuild Order:
  product_id = Table, bom_id = Table BOM, lot_id = LOT001
  mo_id = MO/00123 (optional — for lot traceability)

action_unbuild():
  1. Create consume moves: Table (Stock → Production)
  2. Create produce moves: Components (Production → Stock)
  3. If mo_id set: trace back to original MO move lines for lot matching
  4. Link consume/produce move_lines for traceability
  5. All moves → done
```

**With lot traceability:** If unbuilding a specific MO, the system reads the original MO's move lines to determine which lots of components to return.
