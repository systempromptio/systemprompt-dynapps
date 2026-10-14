<!-- verified: v18.0 @ 2026-06 (stock_landed_costs CE + mrp/subcontracting bridges; anclas verificadas vs fuente) -->
# Landed Costs (`stock_landed_costs`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **CE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (LCG*).

**LCCAP1. ¿Repartir costes adicionales (flete/aduana/seguro) sobre el coste de productos recibidos?** — **NATIVO** · baja · `stock_landed_cost.py::compute_landed_cost`/`button_validate`. Es el propósito del módulo.

**LCCAP2. ¿Métodos de reparto?** — **NATIVO** · baja · `SPLIT_METHOD`: `equal`, `by_quantity`, `by_weight`, `by_volume`, `by_current_cost_price`. Cinco; no hay reparto por importe de otra línea.

**LCCAP3. ¿Crear el coste en destino desde la factura del transportista?** — **NATIVO** · baja · `account_move.py::button_create_landed_costs` (líneas `is_landed_costs_line=True`).

**LCCAP4. ¿Aplicar a recepciones de compra?** — **NATIVO** · baja · `target_model='picking'`; `_get_targeted_move_ids = picking_ids.move_ids`.

**LCCAP5. ¿Aplicar a órdenes de fabricación?** — **NATIVO** (módulo `mrp_landed_costs`, auto_install) · baja · `target_model='manufacturing'` + `mrp_production_ids` (sobre `move_finished_ids`).

**LCCAP6. ¿Funciona con valoración manual o requiere automática + FIFO/AVCO?** — **CONFIG** · media · `get_valuation_lines` exige `cost_method in ('fifo','average')`; con valoración manual ajusta SVL/standard_price pero **NO** genera asiento (solo `real_time`). Coste estándar excluido.

**LCCAP7. ¿Ajusta la valoración de stock (SVL) y el asiento contable?** — **NATIVO** · media · `button_validate` crea `stock.valuation.layer` (`stock_landed_cost_id`) + `account.move`; ajusta `remaining_value` y `standard_price`.

**LCCAP8. ¿Revertir/cancelar un landed cost ya validado?** — **NO (directo)** · baja · `button_cancel` lo impide si `done`; la vía soportada es crear un **landed cost negativo** que lo contrarreste.

**LCCAP9. ¿Coste en destino en subcontratación?** — **NATIVO** (módulo `mrp_subcontracting_landed_costs`, auto_install) · media · reparte vía el picking de la recepción subcontratada (flujo `picking` estándar).

**LCCAP10. ¿Usar peso/volumen del producto para el reparto?** — **NATIVO** · baja · `get_valuation_lines` toma `weight = product.weight*qty` / `volume = product.volume*qty` + métodos `by_weight`/`by_volume`.
