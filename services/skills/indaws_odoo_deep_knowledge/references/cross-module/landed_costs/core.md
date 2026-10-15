<!-- verified: v18.0 base + v19.0 diffs @ 2026-06 (stock_landed_costs CE; anclas verificadas vs fuente v18 y v19) -->
# Landed Costs — Core Reference (Odoo 18.0 CE)

**Costes en destino** (`stock_landed_costs`, CE): repartir costes adicionales (flete, aduana, seguro)
sobre el coste de productos recibidos/fabricados, ajustando su valoración. Cruza stock + account +
purchase + mrp. **v19 refactoriza el motor** (delega la valoración en `stock.move`) — ver
[`v19_changes.md`](v19_changes.md). Este core describe v18.

## stock.landed.cost — `stock_landed_cost.py`

| Campo | Notas |
|-------|-------|
| `state` (`:54`) | `draft` / `done` ("Posted") / `cancel`. El validado es **`done`**, no "validated". |
| `cost_lines` (`:44`) | O2M `stock.landed.cost.lines` — los costes a repartir (manual o desde factura). |
| `valuation_adjustment_lines` (`:47`) | O2M `stock.valuation.adjustment.lines` — generadas por `compute_landed_cost`, no se editan. |
| `picking_ids`/`target_model` (`:37`) | en CE `target_model` solo `'picking'`; moves objetivo = `picking_ids.move_ids`. |
| `account_journal_id` (`:63`) | required; default `company.lc_journal_id` o el `property_stock_journal`. |

`button_validate` (`:104`): computa si falta, valida `_check_sum`, crea SVLs adicionales, ajusta
`standard_price` AVCO/FIFO, crea el `account.move` (si hay líneas) y reconcilia → `state='done'`.
`compute_landed_cost` (`:248`) regenera los adjustment lines y reparte cada `price_unit` por su `split_method`.

## stock.landed.cost.lines (`:355`)

`product_id` (required), `price_unit` ("Cost"), `account_id` (cuenta de crédito del asiento),
`split_method` (`:357`, **5 valores**: `equal`/`by_quantity`/`by_current_cost_price`/`by_weight`/`by_volume`).

## stock.valuation.adjustment.lines (`:382`)

`cost_line_id`, `move_id`, `additional_landed_cost` (el reparto), `former_cost` ("Original Value"),
`final_cost` (= former + additional), bases `weight`/`volume`/`quantity`.

## Producto y valoración

- `product.template.landed_cost_ok` + `split_method_landed_cost` (`stock_landed_costs/models/product.py:12`, `class ProductTemplate`): marca el producto
  (servicio) como coste en destino y su método de reparto por defecto.
- `product.cost_method` (`categ_id.property_cost_method`): el LC SOLO opera sobre moves con
  `cost_method in ('fifo','average')` — el coste **estándar se ignora**.
- `stock.valuation.layer.stock_landed_cost_id`: el LC crea SVLs **adicionales** (qty=0) sobre el SVL
  del move y suma a su `remaining_value`.
- `account.move.landed_costs_ids` + `button_create_landed_costs` (`stock_landed_costs/models/account_move.py:21`, `AccountMove.button_create_landed_costs`): crea el LC
  desde la factura del proveedor (líneas `is_landed_costs_line=True`).

Trampas en [`gotchas.md`](gotchas.md) (LCG*); compras/mrp/contabilidad en
[`integrations.md`](integrations.md) (LCG15+); capacidades en [`capabilities.md`](capabilities.md).
Valoración base en [[stock]].
