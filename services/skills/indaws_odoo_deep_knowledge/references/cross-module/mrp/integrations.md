# MRP Integrations — Odoo 18.0

> Subsystem/topic map for MRP. Gotcha IDs use prefix **MG** — full text in `cross-module/mrp/gotchas.md` (MG1–MG29). End-to-end MTO walkthrough: see `playbooks/make_to_order.md` (do not duplicate it here). v19 deltas: §17 below + `cross-module/mrp/v19_changes.md`.

## 9. Subcontracting (module: mrp_subcontracting)

### Flow

```
1. PO confirmed to subcontractor
   ↓
2. Receipt move created (Supplier → Stock)
   ↓
3. _action_confirm() detects subcontract BOM
   ↓
4. Move location changed: Supplier → Subcontracting Location
   ↓
5. picking._subcontracted_produce() creates MO:
   - subcontractor_id = partner
   - location_src_id = subcontracting location
   - Components reserved in subcontracting location
   ↓
6. Subcontractor records components (portal or auto)
   ↓
7. Receipt validated → MO marked done → finished product in stock
```

### Special Locations

```
WH
├── Stock
├── Subcontracting Location (company.subcontracting_location_id)
│   Components staged here for subcontractor
└── Partner-specific subcontracting locations
    (partner.property_stock_subcontractor)
```

### Key Fields

| Model | Field | Purpose |
|-------|-------|---------|
| `stock.move` | `is_subcontract` | Boolean — marks receipt as subcontracted |
| `mrp.bom` | `type='subcontract'` | BOM type for subcontracting |
| `mrp.bom` | `subcontractor_ids` | M2M → partners who can subcontract |
| `mrp.production` | `subcontractor_id` | M2O → partner (for portal access) |
| `stock.warehouse` | `subcontracting_to_resupply` | Boolean — enable component supply |
| `stock.warehouse` | `subcontracting_route_id` | Route for "Resupply Subcontractor on Order" |
| `res.partner` | `property_stock_subcontractor` | Partner-specific subcontracting location |

### Component Supply Modes

| Mode | Config | Flow |
|------|--------|------|
| **Subcontractor buys** | No resupply | PO → Receipt. Subcontractor handles components. |
| **We supply** | `subcontracting_to_resupply = True` + MTO on components | PO → Component transfer (Stock → Subcontracting Loc) → Receipt |
| **Dropship to subcontractor** | `Dropship Subcontractors` on warehouse | PO to vendor → Vendor ships directly to subcontractor |

### Constraints

- Subcontract BOMs **cannot have operations** (no work orders) — MG7
- Subcontract BOMs **cannot have byproducts** — MG7
- Subcontractor can record components via portal (limited write access) — MG14 (only `move_line_raw_ids`, `lot_producing_id`, `subcontracting_has_been_recorded`, `qty_producing`, `product_qty` are writable)

> Gotchas: receipt move location changes silently Supplier → Subcontracting Location at `_action_confirm()` (MG8). The MO is created by `picking._subcontracted_produce()` with `subcontractor_id` set.

## 10. MRP Accounting (module: mrp_account)

### Production Cost Calculation

```python
mrp.production._cal_price(consumed_moves):
  1. component_cost = SUM(consumed_moves.stock_valuation_layer_ids.value)
  2. operation_cost = SUM(workorders.duration * workcenter.costs_hour)
  3. extra_cost = MO.extra_cost * finished_qty
  4. total_cost = component_cost + operation_cost + extra_cost

  # Allocate between finished product and byproducts
  5. For byproducts: cost = total_cost * byproduct.cost_share / 100
  6. For main product: cost = total_cost - SUM(byproduct_costs)
  7. finished_move.price_unit = main_product_cost / finished_qty
```

Source: `mrp_account/models/mrp_production.py:40`, `MrpProduction._cal_price` (`_cal_price`). Gotchas: cost = components + operations (`duration × costs_hour`) + `extra_cost` (MG10 — zero-duration WO undervalues); `extra_cost` does NOT post a separate journal line, it folds into the single finished SVL (MG21); the materials/labor/overhead breakdown report is EE-only `mrp_account_enterprise` (MG22). v19: `_cal_price` signature/semantics changed (§17).

### WIP Accounting (Work In Progress)

For `real_time` valuation:
```
1. Component consumed → SVL created → Journal entry:
   Dr. WIP Account (production)    $X
      Cr. Stock Valuation               $X

2. Labor posted → _post_labour():
   Dr. WIP Account (production)    $Y
      Cr. Labour Account                $Y

3. Finished product produced → SVL created:
   Dr. Stock Valuation             $(X+Y)
      Cr. WIP Account (production)      $(X+Y)
```

WIP account = `product.property_stock_production` location's valuation accounts. These entries exist **only with `valuation='real_time'`** — manual-valuation companies get no WIP visibility (MG11). Account routing overrides live in `mrp_account/models/stock_move.py` (`_should_force_price_unit`, `_get_src_account`/`_get_dest_account`).

### Cost Share in Byproducts

If main product has 80% cost_share and byproduct has 20%:
```
Total production cost = $1000
Main product cost = $1000 * 80% = $800
Byproduct cost = $1000 * 20% = $200
```

### Subcontracting Cost (module: mrp_subcontracting_account)

When components are supplied to the subcontractor:

```
Basic flow (no component supply):
  Receipt SVL value = PO price to subcontractor
  → product cost = what you paid the subcontractor

With component supply:
  1. Components shipped → SVL OUT (component cost leaves your stock)
  2. Product received → SVL IN at PO price
  3. mrp_subcontracting_account ADJUSTS the receipt SVL:
     adjusted_value = PO_price + SUM(component_SVL_values)
     → Finished product cost = subcontractor fee + component cost
```

**Example:** Components shipped: $500 (screws + metal), Subcontractor fee (PO): $300 → Finished product enters stock at $800.

Without `mrp_subcontracting_account`, finished product enters at $300 only (PO price). Component cost is "lost" in the WIP account.

> Gotcha (MG26): the receipt SVL credit is **split** into a service-cost line (PO fee) and a component-cost line — not one line. With `cost_method='standard'`, the gap between PO fee and standard cost posts to the price-difference account. Source: `mrp_subcontracting_account/models/stock_move.py:15`, `StockMove._generate_valuation_lines_data`.

## 11. Kit Handling in Sales (sale_mrp)

When SO contains a kit (phantom BOM) product:

```
SO confirmed → stock.move created for kit product
  ↓
move._action_confirm() → action_explode()
  ↓
Kit detected (product.is_kits = True, phantom BOM)
  ↓
BOM exploded → component moves created
  ↓
Original kit move DELETED
  ↓
Component moves assigned to delivery picking
```

Kit products never appear in pickings. Only their components are delivered. Kit cost distributed to components via `cost_share` on BOM lines.

> Gotchas: phantom BOMs explode at `stock.move._action_confirm()`, not at SO/MO creation — the kit move is deleted (MG2). Cost goes to components by `cost_share`, NOT proportionally — unset `cost_share` dumps all cost on one component (MG9). Kits bypass reservation (components reserve instead) via `_should_bypass_reservation` (MG2 family). Kits support cross-UoM components within the same UoM category (MG23). Source: `sale_mrp/models/sale_order_line.py:21-77`, `SaleOrderLine._compute_qty_to_deliver`, `mrp/models/stock_move.py:543`, `class StockMove`.

## 12. Procurement → Manufacturing

### Route: "Manufacture"

```
stock.rule with action='manufacture'
  ↓
_run_manufacture():
  1. _get_matching_bom() → find applicable BOM
     Priority: explicit bom_id > orderpoint.bom_id > _bom_find()

  2. Search existing DRAFT MO to merge (same product/BOM/company)
     IF found → increase qty via change.production.qty wizard
     IF not → create new MO

  3. _prepare_mo_vals(): set product, qty, BOM, locations, dates
     - date_start = procurement.date_planned - bom.produce_delay
     - location_src_id from picking_type defaults
     - move_dest_ids links to downstream (sales delivery, etc.)

  4. Batch splitting: if values['batch_size'] set, split into chunks

  5. Auto-confirm if _should_auto_confirm_procurement_mo()
```

Source: `mrp/models/stock_rule.py:45`, `StockRule._run_manufacture` (`_run_manufacture`), `:123` (`_get_matching_bom`), `:155` (`_prepare_mo_vals`), `:39` (`_should_auto_confirm_procurement_mo`). Gotchas: merges into an existing draft MO via `change.production.qty` instead of creating a new one (MG3); the MO is created as SUPERUSER so `create_uid` is the admin, not the actor (MG25, `:79-80`); MTO-launched MO links its finished move's `move_dest_ids` to the SO delivery and may stay `draft` if it doesn't auto-confirm (MG18 + playbook STKG67).

### BOM Selection by Warehouse (picking_type filter)

`_bom_find()` accepts `picking_type` parameter. Each warehouse has its own `manu_type_id` (picking type code=`mrp_operation`). A BOM with `picking_type_id` set only matches that warehouse.

```
BOM A: Product X, picking_type_id = WH-Madrid/Manufacturing → only for Madrid
BOM B: Product X, picking_type_id = WH-Barcelona/Manufacturing → only for Barcelona
BOM C: Product X, picking_type_id = False → fallback for any warehouse
```

Resolution: `_bom_find()` filters `picking_type_id in (picking_type, False)` and then picks `search(order='sequence, product_id, id', limit=1)` (`mrp/models/mrp_bom.py:351-377`, `MrpBom._bom_find_domain`). **It does NOT prefer the picking_type-specific BOM over the `False` fallback** — selection is purely by `sequence` (MG27). To force per-warehouse BOMs, give the specific BOMs a lower `sequence` than the generic fallback. Enables **different BOMs per warehouse** without custom code, provided the sequence is set correctly.

### Multi-Level BOM Procurement

```
SO: 10 Tables
  ↓ Manufacture route
MO: 10 Tables (BOM: 4 Wood Frames + 40 Screws + ...)
  ↓
Wood Frame has Manufacture route + MTO:
  move_raw (Wood Frame) triggers new procurement
    ↓
  MO: 40 Wood Frames (sub-MO)
    ↓
  Screws have Buy route:
    move_raw (Screws) triggers _run_buy()
      ↓
    PO: 800 Screws to vendor
```

Recommended approach (from docs): **Reordering rules (0/0/1) instead of MTO** for sub-assemblies — more flexible, allows inventory reassignment (MG12 — a cancelled parent MO with MTO locks sub-assembly stock). Cancelling an MO cascades to its stock moves but NOT to the triggering SO or component POs (MG16).

## 13. Manufacturing Steps — Warehouse Configuration

### Picking Types Auto-Created

| Picking Type | Code | Steps | Purpose |
|-------------|------|-------|---------|
| `manu_type_id` | `mrp_operation` | All | Manufacturing operation |
| `pbm_type_id` | `internal` | 2, 3 | Pick Before Manufacturing |
| `sam_type_id` | `internal` | 3 | Stock After Manufacturing |

### Locations Auto-Created

| Location | Steps | Purpose |
|----------|-------|---------|
| `pbm_loc_id` | 2, 3 | Pre-production staging (components picked here) |
| `sam_loc_id` | 3 | Post-production staging (finished products before storage) |

### Route & Rules Auto-Created

```
manufacture_to_resupply = True creates:
  Route: "Manufacture" (warehouse-selectable)
    Rule: action='manufacture', picking_type=manu_type_id

manufacture_steps = 'pbm' adds:
  Rule: Pull from Stock → pbm_loc_id (pbm_type_id)
  MO consumes from pbm_loc_id instead of Stock

manufacture_steps = 'pbm_sam' adds:
  Rule: Pull from Stock → pbm_loc_id
  Rule: Push from sam_loc_id → Stock (sam_type_id)
  MO produces to sam_loc_id instead of Stock
```

> Gotchas: with `pbm`, confirming an MO creates a pick-before-manufacturing transfer first and the MO shows "Waiting" until that pick is validated (MG13). With `pbm_sam`, the finished product lands in `sam_loc_id` and is **not** in Stock / available for delivery until the Store Finished Products (SAM) transfer is validated — `move_dest_ids._trigger_assign()` (MG18) fires only after the SAM transfer is done (MG28).

## 14. Operations & Work Orders (CE vs Enterprise)

### CE (Community) — Operations in BOM

In CE, `mrp.routing.workcenter` records in `operation_ids` specify work center assignment, which components are consumed per operation (`bom_line.operation_id`), and duration estimates.

**CE limitations:** No standalone work order records (`mrp.workorder` exists but empty without Enterprise), no time tracking per operation, no quality checks, no employee assignment, no Gantt/planning view, no Shop Floor. Operations in CE are metadata — they document the process but don't drive execution. Consequence for costing: CE produces zero operation cost unless a workcenter `costs_hour` and WO duration exist (MG10), and analytic distribution in CE applies uniformly to all raw/finished moves — no per-component split (MG20).

### Enterprise (mrp_workorder) — Full Work Orders

### Model: mrp.workorder

| Field | Type | Purpose |
|-------|------|---------|
| `production_id` | M2O → mrp.production | Parent MO |
| `workcenter_id` | M2O → mrp.workcenter | Where work happens |
| `duration` | Float (computed) | Actual time spent |
| `duration_expected` | Float | Expected time |
| `state` | Selection | pending/waiting/ready/progress/done/cancel |
| `check_ids` | O2M → quality.check | Quality checks during operation |
| `employee_ids` | M2M → hr.employee | Workers assigned |

### Work Center (mrp.workcenter)

| Field | Type | Purpose |
|-------|------|---------|
| `costs_hour` | Float | Hourly cost (for costing calculation) |
| `capacity` | Float | Units processed per cycle |
| `time_start` / `time_stop` | Float | Setup/teardown time |
| `oee_target` | Float | OEE target % |

### Quality Integration

Work orders can have quality checks (from `quality_control`): at start, during (per unit or batch), or at end of operation. Links to `stock.move.line` via `quality_check_ids`.

## 15. Enterprise: Master Production Schedule (mrp_mps)

Forecast-based production planning. Generates MOs and POs based on demand forecasts.

### Key Fields

| Field | Type | Purpose |
|-------|------|---------|
| `product_id` | M2O | Product to schedule |
| `warehouse_id` | M2O | Target warehouse |
| `forecast_ids` | O2M → mrp.production.schedule.demand | Demand per period |
| `forecast_target_qty` | Float | Safety stock target |
| `min_to_replenish_qty` / `max_to_replenish_qty` | Float | Order constraints |
| `replenish_trigger` | Selection | manual / automated / never |
| `batch_size` | Float | MO splitting size |

### Replenishment Flow

```
MPS.action_replenish():
  1. Filter schedules by replenish_trigger != 'never'
  2. Skip phantom BOMs (kits — components procured, not kit)
  3. Create Procurement objects for replenish qty
  4. Run procurement.group.run() with skip_lead_time=True
  5. Mark forecasts as procurement_launched=True
```

Source: `odoo_enterprise/18/mrp_mps/models/mrp_mps.py::action_replenish` (`action_replenish`), `:276` (`skip_lead_time=True`). Gotchas: MPS skips phantom BOMs to avoid double procurement — forecasts on a kit generate nothing, schedule the components instead (MG15, `:219`); MPS-launched MOs/POs ignore `produce_delay`/supplier lead time and plan at the period start, so they may already be late vs manual reordering rules (MG29).

### Indirect Demand

MPS calculates demand cascading through BOM levels:
```
MPS: 100 Tables → needs 400 Wood Frames
  ↓
MPS: 400 Wood Frames → needs 400 Planks + 4000 Screws
  ↓
MPS: auto-generates schedules for sub-assemblies
```

## 16. Key Method Reference

### mrp.production

| Method | Purpose |
|--------|---------|
| `action_confirm()` | Confirm MO, create/confirm moves, trigger scheduler |
| `_get_moves_raw_values()` | Generate component move dicts from BOM |
| `_get_moves_finished_values()` | Generate finished + byproduct move dicts |
| `button_mark_done()` | Validate production, handle backorders |
| `_post_inventory()` | Finalize all moves (consume + produce) |
| `_split_productions()` | Create backorders for partial production |
| `action_assign()` | Reserve components (depends on `bom.ready_to_produce`) |
| `_cal_price()` | Calculate finished product cost (mrp_account) — MG10/MG21, `mrp_account/mrp_production.py:40`, `MrpProduction._cal_price` |
| `action_cancel()` | Cancel MO + its moves, but NOT triggering SO/POs — MG16 |

> `button_mark_done` → `_post_inventory` (`mrp/mrp_production.py:1802`, `MrpProduction`) creates SVLs and calls `_cal_price`; backorders from `_split_productions` share `procurement_group_id` (MG4), `unit_factor` scaling accumulates rounding across backorders (MG6).

### mrp.bom

| Method | Purpose |
|--------|---------|
| `explode(product, qty)` | Recursively expand BOM to flat component list |
| `_bom_find(products, picking_type, company_id, bom_type)` | Find applicable BOM — orders by `sequence`, not picking_type specificity (MG27), `mrp/mrp_bom.py:362`, `MrpBom` |

### stock.rule (mrp extension)

| Method | Purpose |
|--------|---------|
| `_run_manufacture(procurements)` | Create MOs from procurement (as SUPERUSER, MG25) — `mrp/stock_rule.py:45`, `StockRule._run_manufacture` |
| `_get_matching_bom(product, company, values)` | BOM selection (bom_id > orderpoint.bom_id > `_bom_find`) — `:123` |
| `_prepare_mo_vals(procurement, bom)` | Build MO values dict — `:155` |
| `_should_auto_confirm_procurement_mo(p)` | Whether the MO auto-confirms (else stays draft) — `:39` |

### stock.move (mrp extension)

| Method | Purpose |
|--------|---------|
| `action_explode()` | Expand phantom BOM moves into component moves — MG2, `mrp/stock_move.py:543`, `StockMove` |
| `_should_bypass_reservation()` | Kits bypass reservation (components reserve instead) — `:540` |
| `_action_confirm(merge, merge_into)` | Explodes phantom before confirming — `:527` |
| `_action_cancel()` | Cancels the MO if all raw moves are cancelled — `:600` |

## 17. v19 Changes (mrp)

Verified against `odoo19/addons/mrp` + `mrp_account`. Migration checklist: `cross-module/mrp/v19_changes.md`.

### Domain / API

- **Domain API**: `expression.AND/OR` → `Domain.AND/OR`; `from odoo.fields import Domain, Command` (`mrp/models/stock_rule.py:8`, la linea de imports). Custom overrides of `_run_manufacture`/`_get_matching_bom`/`_bom_find_domain` that build domains via `odoo.osv.expression` must be ported.
- **Move filter domains**: `scrapped=False` → `('location_dest_usage', '!=', 'inventory')` on `move_raw_ids`/`move_finished_ids` (`mrp/models/mrp_production.py:207`, `MrpProduction.company_id` y `mrp/models/mrp_production.py:211`, `MrpProduction.qty_produced`). Any §16 method or report filtering moves by `scrapped` breaks.
- **UoM domains**: category-based filtering → computed `allowed_uom_ids` M2M on `mrp.production`.

### Multi-lot production (`lot_producing_id` → `lot_producing_ids`)

- `mrp.production.lot_producing_id` (M2O) is replaced by **`lot_producing_ids` (M2M)** (`mrp/models/mrp_production.py:120`, `MrpProduction.date_deadline`) — multi-lot production. The finished move now receives `move.lot_ids = order.lot_producing_ids.ids` at `_post_inventory` (`:~1907`). Impact: code/reports reading `lot_producing_id` on MOs, finished moves or quality checks break — both a data and code migration. The MG14 subcontractor-writable field list still references `lot_producing_id` conceptually but maps to the M2M in v19.

### `_cal_price` semantics (mrp_account)

- v18: `total_cost = - sum(consumed_moves.stock_valuation_layer_ids.value) + work_center_cost + extra_cost` (reads SVLs off the moves, with a sign flip).
- v19: `total_cost = sum(move.value for move in consumed_moves) + ...` and an **early return** sets `finished_move.price_unit = standard_price` for non-fifo/average products (`mrp_account/models/mrp_production.py:57-76`, `MrpProduction._cal_price`). `consumed_moves` is still a `stock.move` recordset, but cost now comes from `move.value` directly. Impact: overrides of `_cal_price` that walked `stock_valuation_layer_ids` must switch to `move.value`; standard-cost products no longer compute a derived finished cost.

### New grouping models

- **`mrp.production.group`** / `production_group_id` (M2O) on `mrp.production` — group multiple MOs (parent/child). Does not change the base flow but queries keyed only on `procurement_group_id` may now need `production_group_id` too.
- **`stock.reference`** inheritance (`mrp/models/stock_reference.py`): `reference_ids` M2M links MOs to stock references (document linking).

### Scrap

- `mrp/models/stock_scrap.py` reworked around `_should_check_available_qty`/`_create_scrap_move`/`do_replenish` — scrap-from-MO replenishment path differs; verify custom scrap overrides against v19.

### Categories / security domains

- Module category `'Manufacturing/Manufacturing'` → `'Supply Chain/Manufacturing'`.
- User domains `groups_id` → `all_group_ids`.

## Flujo variantes × atributos × kits × fabricación (VXK) — preguntas de flujo

> Cómo se enganchan entre sí variantes, atributos, valores, kits y BoM en el día a día de un fabricante. Verificado contra `mrp`, `sale_mrp`, `mrp_account`, `stock` v18 (+v19 estable salvo lo indicado). Cada fila = pregunta de flujo con el mecanismo y el ancla.

### A. ¿Qué BoM se aplica a cada variante?

**VXK1. Tengo un producto con tallas/colores. ¿Puedo tener una BoM distinta por variante?** — SÍ · `mrp.bom.product_id` (variante) vs `product_tmpl_id` (todas las variantes) · Una BoM con `product_id` set aplica solo a esa variante; sin él, a todas las del template.

**VXK2. Si defino una BoM genérica (template) y otra específica de la variante "Rojo", ¿cuál gana al fabricar Rojo?** — DEPENDE DEL `sequence` (NO "siempre la de variante") · `_bom_find` order `sequence, product_id, id` (`mrp/models/mrp_bom.py:362-395`, `MrpBom._bom_find`) · A **igual `sequence`**, la variante-específica (`product_id` no nulo) va antes que la genérica (`product_id` NULL, NULLS LAST) y gana. Pero **el `sequence` manda**: una BoM genérica con `sequence` menor gana sobre la variante-específica (en el bucle multi-producto, si el template ya está en `bom_by_product_tmpl` la BoM de variante posterior se descarta). Verificado 2026-07-15.

**VXK3. ¿Cómo elige Odoo la BoM al crear la MO desde una venta MTO?** — `_get_matching_bom` → `_bom_find(product_id, bom_type='normal')` (`mrp/models/stock_rule.py:123-128`, `StockRule._get_matching_bom`) · Prioridad: `values['bom_id']` explícito → BoM del orderpoint → `_bom_find` de la variante concreta.

**VXK4. ¿Puedo tener varias BoM para la misma variante y elegir según el almacén?** — SÍ · `_bom_find_domain` filtra por `picking_type_id` (`mrp/models/mrp_bom.py:351-359`, `MrpBom._bom_find_domain`) · Una BoM con `picking_type_id` set solo aplica en ese tipo de operación/almacén; con `sequence` se prioriza.

**VXK5. ¿Y por compañía (multi-company)?** — SÍ · `_bom_find_domain` filtra `company_id` (`:353-354`) · BoM con `company_id=False` es global; con compañía set, solo esa.

### B. Componentes y operaciones condicionados por variante/atributo

**VXK6. En una BoM compartida por todas las variantes, ¿puedo hacer que un componente solo entre para la talla XL?** — SÍ · `mrp.bom.line.bom_product_template_attribute_value_ids` + `_skip_bom_line` (`mrp/models/mrp_bom.py:715`, `MrpBomLine._skip_bom_line`) · La línea se salta si la variante no matchea sus PTAV (`_match_all_variant_values`, `mrp/product.py:384`, `ProductProduct._match_all_variant_values`).

**VXK7. ¿Puedo condicionar también una operación (ruta) o un subproducto a una variante?** — SÍ · `operation_ids`/`byproduct_ids.bom_product_template_attribute_value_ids` (`mrp/models/mrp_bom.py:181`, `MrpBom._check_bom_lines`) · Mismo mecanismo de skip por variante: operaciones vía `mrp_routing._skip_operation_line` (`:167`) y by-products vía `_skip_byproduct_line` (`mrp/models/mrp_bom.py:836`, `MrpByProduct._skip_byproduct_line`), ambos delegando en `mrp.bom._skip_for_no_variant` (`mrp/models/mrp_bom.py:569`, `MrpBom._skip_for_no_variant`). Verificado 2026-07-15.

**VXK8. Con una sola BoM "matricial" (líneas condicionadas por atributo), ¿evito crear una BoM por variante?** — SÍ (patrón recomendado) · `bom_product_template_attribute_value_ids` · Es la alternativa a proliferar BoMs: una BoM con líneas que aparecen/desaparecen según la variante fabricada.

**VXK9. Si cambio los valores posibles del atributo, ¿se limpian las condiciones de las líneas?** — NO por ese cambio en sí · `_onchange...` (`mrp/models/mrp_bom.py:107-122`, `class MrpBom`, `:226-247`) · Cambiar solo los valores posibles del atributo en la plantilla NO dispara limpieza. Lo que SÍ resetea: cambiar `product_id`/`product_tmpl_id` de la propia BoM (onchange `_onchange_product_id` `:107-122` y `@api.onchange('product_tmpl_id')` `:226-247`) pone a False TODOS los `bom_product_template_attribute_value_ids` de líneas/operaciones/by-products (no solo los incompatibles), avisando 'will permanently reset all previously encoded variant-related data'. Verificado 2026-07-15.

### C. Atributos `no_variant` que afectan a la fabricación

**VXK10. Tengo un atributo "Grabado" que NO crea variantes. ¿Puede aun así añadir un componente/operación al fabricar?** — SÍ · `_skip_for_no_variant` + `never_attribute_values` (`mrp/models/mrp_bom.py:569`, `MrpBom._skip_for_no_variant`) · Los valores no_variant elegidos en la venta viajan como `never_product_template_attribute_value_ids` y activan/saltan líneas condicionadas por ese valor.

**VXK11. ¿Cómo llega ese valor no_variant desde la línea de venta hasta la explosión de la BoM?** — vía procurement · `procurement.values['never_product_template_attribute_value_ids']` → `explode(..., never_attribute_values=...)` (`mrp/models/stock_rule.py:272`, `ProcurementGroup.run`, `mrp/models/stock_move.py:580`, `StockMove.action_explode`) · Se propaga hasta `_skip_bom_line`.

**VXK12. Un valor no_variant con precio extra, ¿se cobra sin crear variante?** — SÍ · `_get_no_variant_attributes_price_extra` (`product/models/product_product.py:777`, `ProductProduct._get_no_variant_attributes_price_extra`) · El extra se suma a la línea de venta por contexto, sin materializar `product.product`.

### D. Configurador de venta → variante dinámica → MO (MTO)

**VXK13. Con un catálogo enorme configurable, ¿puedo NO pre-generar variantes y crearlas al vender?** — SÍ · `create_variant='dynamic'` + `_create_product_variant` · El configurador crea la variante bajo demanda al confirmar el SO (VARCAP11).

**VXK14. Esa variante creada al vender, ¿arrastra la BoM correcta a la MO?** — SÍ · `_bom_find(variant)` en `_run_manufacture` · La MO busca la BoM de esa variante concreta (variant-specific si existe, si no la del template).

**VXK15. ¿El flujo venta→fabricación de una variante configurada es MTO nativo?** — SÍ · `stock.rule` manufacture (`_run_manufacture`) · Ver [[playbook make_to_order]]; la variante dispara procurement→MO.

**VXK16. Si la combinación elegida está excluida, ¿puede llegar a fabricarse?** — NO · `_is_combination_possible` (`product/models/product_template.py:1080`, `ProductTemplate._is_combination_possible`) · El configurador no deja confirmar combinaciones excluidas; no se crea variante ni MO.

### E. Kits que contienen variantes / kits de productos fabricados

**VXK17. ¿Un kit puede contener variantes concretas (talla M + color Rojo)?** — SÍ · `bom_line.product_id` = `product.product` (variante) · Las líneas del kit apuntan a variantes concretas.

**VXK18. ¿Un componente de un kit puede ser a su vez un producto fabricado (con su MO)?** — SÍ · `explode` + rutas por componente · Al explotar el kit, cada componente sigue su ruta: si es manufacture, dispara su propia MO (VARCAP/KITCAP25).

**VXK19. ¿Un kit puede contener otro kit que contiene variantes?** — SÍ · `explode` recursivo (`mrp/models/mrp_bom.py:396`, `MrpBom.explode`, `_bom_find bom_type='phantom'`) · La explosión es recursiva y resuelve la BoM phantom de cada componente.

**VXK20. Al vender un kit de variantes, ¿se reabastece cada variante-componente por separado?** — SÍ · procurement por componente tras `explode` (`mrp/models/stock_rule.py:272`, `ProcurementGroup.run`) · Cada variante-componente dispara sus reglas (buy/manufacture/MTO).

**VXK21. ¿Puedo condicionar los componentes de un kit según la variante del propio kit?** — SÍ · phantom BoM + `bom_product_template_attribute_value_ids` · Un kit que es a su vez un template con variantes puede tener líneas condicionadas (kit "Pack" en Rojo lleva componente distinto que en Azul).

### F. Coste: roll-up de variante ← BoM y coste de kit

**VXK22. ¿Puedo calcular el coste de una variante fabricada a partir de su BoM?** — SÍ · `_set_price_from_bom` / botón "Compute Price from BoM" (`mrp_account/models/product.py:44`, `button_bom_cost` `:35`) · Recorre la BoM (componentes + operaciones) y fija `standard_price` de la variante.

**VXK23. ¿El coste desde BoM respeta las líneas condicionadas por variante?** — SÍ · `action_bom_cost` busca BoM por variante o template (`mrp_account/product.py:39-40`, `ProductProduct.action_bom_cost`) · Calcula por variante usando su BoM aplicable.

**VXK24. En un kit, ¿cómo se reparte el coste real entre componentes al vender?** — NO por `cost_share` · En un kit (BoM phantom) NO existe reparto por `cost_share`: cada componente conserva su propio coste (estándar/medio). Al vender/facturar, la BoM se explota en movimientos de stock por componente y el COGS del kit se obtiene sumando el coste real de cada componente (`product._compute_average_price` sobre sus stock moves) ponderado por su cantidad en la BoM y dividido por `bom.product_qty` (`sale_mrp/account_move.py`). El campo `cost_share` (%) es exclusivo de `mrp.bom.byproduct` y solo distribuye el coste de FABRICACIÓN entre subproductos, NO entre componentes de un kit (KITCAP15). Verificado 2026-07-15.

**VXK25. ¿El `standard_price` del producto kit refleja la suma de sus componentes?** — NO (automático) · KITCAP37 · No se mantiene solo; el valor real está en los componentes. "Precio/coste desde BoM" del kit ≠ nativo fiable.

**VXK26. ¿Coste distinto por variante y por compañía a la vez?** — SÍ · `standard_price` company_dependent en `product.product` (PRG3) · Cada variante tiene su coste por compañía; roll-up desde BoM se hace por variante.

### G. Decisión de diseño: proliferación de variantes vs kit vs BoM matricial

**VXK27. ¿Cuándo uso variantes y cuándo un kit?** — VARIANTES = mismo producto vendible con diferencias (talla/color, 1 SKU por combinación); KIT = un producto que se entrega descompuesto en varios. No son intercambiables (KITCAP1, KITCAP31).

**VXK28. Mi configurable genera 5000 combinaciones. ¿Qué evita el colapso?** — `dynamic` (no pre-genera) o BoM matricial · Límite `always`=1000 (VARCAP7); pasar a `dynamic` + una BoM con líneas condicionadas evita 5000 `product.product` y 5000 BoMs.

**VXK29. ¿Una BoM matricial (líneas condicionadas) reemplaza a tener N BoMs por variante?** — SÍ (recomendado) · `bom_product_template_attribute_value_ids` (VXK8) · Menos mantenimiento; una sola BoM cubre todas las variantes.

**VXK30. ¿Un "pack" que el cliente configura (elige 1 de N) es kit o combo?** — COMBO (no kit) · `product.combo` (v18) · Elegir opciones a precio combo = combo; kit = componentes fijos siempre entregados (KITCAP31).

### H. Cambios con producción en curso

**VXK31. Cambio la BoM de una variante con MOs abiertas. ¿Qué pasa con esas MOs?** — SE MARCAN "outdated" · `mrp.production.is_outdated_bom` (`mrp/models/mrp_production.py:263`, `class MrpProduction`) · Las MOs existentes NO se recalculan solas; se marcan como BoM desactualizada para revisión.

**VXK32. ¿Puedo cambiar los atributos de un producto ya fabricado/vendido sin romper el historial?** — SÍ (patrón) · archivar PTAL + crear nueva (VAG1, VARCAP30) · Las variantes viejas se archivan; SO/MO/stock antiguos siguen apuntando a ellas.

**VXK33. Cambio las líneas de un kit ya usado en entregas. ¿Afecta a lo ya explotado?** — NO (solo futuro) · aviso onchange (`mrp/models/mrp_bom.py:204-207`, `class MrpBom`, KITCAP33) · Las explosiones ya hechas no cambian; Odoo avisa si hay stock.move con `bom_line_id` del kit.

**VXK34. Gestión formal de cambios de BoM/variante (aprobaciones).** — NATIVO (EE) · `mrp_plm` `mrp.eco` (MRPCAP12) · ECO para versionar BoM con workflow; en CE es cambio directo + marca outdated.

### I. Trazabilidad y reporting a través de variantes y kits

**VXK35. ¿Puedo trazar lote/serie de los componentes de un kit vendido?** — SÍ · move lines de componentes (KITCAP35) · La trazabilidad es por componente (el kit no tiene lote propio).

**VXK36. ¿El forecast/reabastecimiento distingue por variante?** — SÍ · orderpoint por `product.product` · Cada variante tiene su propia previsión/regla. Matiz kit: el **MPS omite** los phantom (KITCAP27, MG15), pero un **orderpoint sobre un kit SÍ se soporta** explotándolo a componentes (KITCAP26 — corregido).

**VXK37. ¿La cantidad entregada de un kit de variantes se calcula bien en la venta?** — SÍ (cuello de botella) · `sale_mrp._compute_qty_delivered` (`sale_mrp/sale_order_line.py:33`, `SaleOrderLine._compute_qty_delivered`) · El kit cuenta como entregado según el componente menos servido (KITCAP9).

### J. Diferencias v18 → v19 (flujo)

**VXK38. ¿Cambia algo del flujo variante×kit×BoM en v19?** — CASI NO · diff mayormente limpio · `_bom_find`, `_skip_bom_line`, `cost_share`, `_set_price_from_bom` idénticos. OJO: `explode` y `_run_manufacture` SÍ cambian — adoptan la nueva API UoM v19 (`uom.round()`/`uom.compare()` en vez de `float_round`/`float_compare`) y `_run_manufacture` implementa la novedad `mrp.bom.batch_size`/`enable_batch_size` (MOs autogeneradas por lotes, KITCAP39). Otra novedad: `is_dynamically_created` (VARCAP14). Constraints a `models.Constraint`. Verificado 2026-07-15.

---

## Cross-references

- `cross-module/mrp/gotchas.md` — MG1–MG29 (source of truth for MRP gotchas).
- `cross-module/mrp/production.md` — MO model, state machine, explode(), backorders, 1/2/3-step.
- `cross-module/mrp/v19_changes.md` — full v18→v19 breaking-change checklist.
- `playbooks/make_to_order.md` — end-to-end MTO walkthrough (stages, failure modes, config). Do not duplicate.
- `functional/stock/gotchas.md` — STKG9, STKG21, STKG23 (procurement, MTO route, SO cancellation).
- `functional/sale/integrations.md` — SAM1–SAM6 (sale_mrp / kit), SAS1 (qty_delivered).
- `enterprise/quality/{gotchas,integrations}.md` — QG* (checks on MO/WO).
- `enterprise/mrp_workorder/*.md` — WO, capacity, Shop Floor, tablet.
