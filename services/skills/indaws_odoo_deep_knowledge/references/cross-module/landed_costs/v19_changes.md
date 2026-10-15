<!-- verified: v19.0 @ 2026-06 (diff stock_landed_costs v18→v19 vs odoo19 source; anclas verificadas) -->
# Landed Costs — Cambios v19

**Refactorización profunda del motor de valoración** (verificado contra
`$ODOO_SRC/odoo19/addons/stock_landed_costs/`). Los 3 modelos núcleo y las selections (`split_method`,
`state`, `target_model`) **no cambian**; lo que cambia es CÓMO se aplica el coste: v19 abandona crear
SVLs propias y delega en el nuevo motor de valoración de `stock.move`.

## Cambio central

- v18: `button_validate` creaba `stock.valuation.layer` manualmente, prorrateaba `remaining_qty`,
  manejaba lotes y actualizaba `standard_price`.
- v19: `button_validate` (`stock_landed_costs/models/stock_landed_cost.py:103`, `class StockLandedCost`) **elimina todo eso** y solo construye el asiento;
  al final llama `cost.valuation_adjustment_lines.move_id._set_value()` (`:152`). El coste adicional se
  inyecta en el valor del movimiento vía los nuevos `stock.move._get_landed_cost()` /
  `_get_value_from_extra()`.

## Modelos / campos eliminados

- **Eliminado** `models/stock_valuation_layer.py` (la extensión con `stock_landed_cost_id` y el override
  `_should_impact_price_unit_receipt_value`).
- `stock.landed.cost.stock_valuation_layer_ids` (O2M) **eliminado**.
- `vendor_bill_id` gana `index='btree_not_null'`; `cost_id` pasa a `index=True` en las líneas;
  `account_id` de la cost line pierde el `domain=[('deprecated','=',False)]`.

## Métodos

- **Eliminados**: `reconcile_landed_cost`, `action_view_stock_valuation_layers`,
  `StockMove._get_stock_valuation_layer_ids`, y en `account_move.py` el override `_post` (que llamaba
  reconcile), `_get_stock_valuation_layers`, `_eligible_for_cogs`.
- **Nuevos**: `StockMove._get_landed_cost(at_date=None)`, `_get_value_from_extra(quantity, at_date)`.
- **Cambian firma**: `_create_accounting_entries(self, remaining_qty)` (antes `(move, qty_out)`);
  `_create_account_move_line(credit, debit, remaining_qty)`. Se elimina la rama "already out /
  anglo-saxon expense".
- `get_valuation_lines`: `former_cost` pasa de sumar SVLs a `move._get_value()`.
- `onchange_product_id`: la cuenta por defecto pasa de `stock_input` a **`expense`**; igual en
  `account_move.py` (línea LC `stock_input`→`expense`).

## Impacto

Módulos que dependan de `stock.landed.cost.stock_valuation_layer_ids`, `reconcile_landed_cost`,
`StockMove._get_stock_valuation_layer_ids`, o del override `account.move._post`, **rompen en v19**. El
comportamiento funcional (reparto por split_method, ajuste de valoración) se mantiene, pero la
mecánica contable y de SVL es nueva.
