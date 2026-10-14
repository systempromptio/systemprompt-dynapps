<!-- verified: v18.0 @ 2026-06 (stock_landed_costs CE; anclas verificadas vs fuente) -->
# Landed Costs — Gotchas (Odoo 18.0 CE)

Comportamientos ocultos. Núcleo en [`core.md`](core.md); compras/mrp/contabilidad en
[`integrations.md`](integrations.md) (LCG15+). **v18; v19 refactoriza el motor — ver [`v19_changes.md`](v19_changes.md).**

**LCG1. No se puede cancelar tras validar.** `button_cancel` (`stock_landed_costs/models/stock_landed_cost.py:98`, `StockLandedCost.button_cancel`) lanza
`UserError` si está `done`. La única reversión es crear un **landed cost NEGATIVO** que lo contrarreste.
Irreversible por diseño.

**LCG2. Solo `draft` se puede validar.** `_check_can_validate` (`:322`); no hay re-validación.

**LCG3. Solo productos FIFO/AVCO entran en el reparto.** `get_valuation_lines` (`:230`): los moves de
producto `standard` (coste estándar) se **ignoran**; si ninguno cualifica → `UserError` ("only for
products with FIFO or average costing method").

**LCG4. El asiento contable solo se crea si la valoración es `real_time`.** `button_validate`
(`:179`): productos con `valuation != 'real_time'` (manual/periódica) hacen `continue` y **NO** generan
`account.move`, pero **sí** ajustan el coste FIFO/AVCO. Un LC puede quedar `done` sin `account_move_id`.

**LCG5. El `standard_price` sube prorrateando solo el stock REMANENTE.** `cost_to_add =
(remaining_qty/move_qty) * additional_landed_cost` (`:127`). La porción ya vendida no incrementa el
coste del producto — va a COGS.

**LCG6. Si toda la mercancía ya salió, no se ajusta el coste.** `remaining_qty=0` → no se crea SVL ni
sube el standard_price; el coste de lo vendido se imputa a gasto vía `qty_out` (`:182`).

**LCG7. `_check_sum` bloquea la validación si las líneas no cuadran.** (`:330`) La suma de
`additional_landed_cost` debe igualar `amount_total` y cada cost line su reparto, o `UserError`.

**LCG8. `compute_landed_cost` BORRA y regenera los adjustment lines** (`:250`): cualquier edición
manual de los ajustes se pierde al recomputar (y `button_validate` recomputa si están vacíos).

**LCG9. Valoración por lote: el reparto se subdivide por lote.** Con `lot_valuated` (`:136`) se ajusta
el `standard_price` de cada lote; lotes con `remaining_qty=0` se saltan.

**LCG10. El SVL del LC NO impacta el price_unit del receipt si viene de factura.**
`_should_impact_price_unit_receipt_value` (`stock_landed_costs/models/stock_valuation_layer.py:12`, `StockValuationLayer._should_impact_price_unit_receipt_value`) devuelve False cuando hay
`vendor_bill_id` — evita doble contabilización con el coste de la factura.

**LCG11. La reconciliación solo en anglosajón con factura posteada.** `reconcile_landed_cost` (`:214`)
solo reconcilia `stock_input` si `vendor_bill_id` está `posted` Y `company.anglo_saxon_accounting`.

**LCG12. El producto LC debe ser tipo servicio.** `stock_landed_costs/models/account_move.py:78`, `AccountMoveLine._onchange_is_landed_costs_line` fuerza `is_landed_costs_line=False`
si no es `type='service'`; no se puede quitar `landed_cost_ok` ni cambiar el tipo si ya está en un apunte.

**LCG13. `split_method='equal'` reparte por nº de líneas, NO por importe.** (`:290`) `value =
price_unit / total_line`. Los métodos `by_*` requieren que la base (qty/weight/volume/cost) sea ≠0 o
caen al reparto equitativo.

**LCG14. Dropshipping desvía el débito a la cuenta de GASTO.** `_create_accounting_entries` (`:427`):
si el move es dropship, débito a `expense` en vez de `stock_valuation`. Crédito = `account_id` de la
cost line o `property_stock_account_input_categ_id` (UserError si falta).
