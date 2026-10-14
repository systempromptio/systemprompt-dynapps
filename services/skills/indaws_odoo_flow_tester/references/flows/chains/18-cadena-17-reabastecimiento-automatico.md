## CADENA 17: Reabastecimiento Automatico
**ID:** CHAIN_INVENTORY_REPLENISHMENT
**Apps:** Inventario + Compras
**Descripcion:** Flujo de reabastecimiento automatico mediante reglas de reordenamiento: configuracion de la regla, ejecucion del scheduler que genera la orden de compra automatica, y recepcion de la mercancia.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| Scheduler run (procurement) | stock.warehouse.orderpoint | purchase.order | `search_records.sh purchase.order '[["origin","like","<orderpoint_name>"]]'` |
| purchase.order.button_confirm | purchase.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` |

### Paso de Variables Entre Flujos
- `{chain.orderpoint_id}` — ID de la regla de reordenamiento
- `{chain.orderpoint_name}` — Nombre de la regla
- `{chain.product_id}` — ID del producto
- `{chain.PO_id}` — ID de la orden de compra generada
- `{chain.PO_name}` — Nombre de la orden de compra
- `{chain.picking_id}` — ID del picking de recepcion

### Precondiciones
- Producto almacenable (type='product') con proveedor configurado (seller_ids)
- Almacen con ruta de compra habilitada (buy route)
- Stock actual del producto por debajo del minimo de la regla (para que el scheduler genere la PO)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | REORDERING_RULE_CREATE | Crear stock.warehouse.orderpoint con product_id, warehouse_id, location_id, product_min_qty, product_max_qty | Regla creada con trigger='auto' |
| 2 | SCHEDULER_RUN | `execute_method.sh procurement.group run_scheduler '[[]]' '{}'` — o forzar directamente: `execute_method.sh stock.warehouse.orderpoint action_replenish '[[<orderpoint_id>]]' '{}'` | Procurement generado |
| 3 | — detectar PO — | `search_records.sh purchase.order '[["origin","like","<orderpoint_name>"]]'` | PO generada en estado draft |
| 4 | PURCHASE_CONFIRM | `execute_method.sh purchase.order button_confirm '[[<PO_id>]]' '{}'` | PO.state -> 'purchase' |
| 5 | — detectar picking — | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` | Picking de recepcion detectado |
| 6 | PICKING_VALIDATE | `execute_method.sh stock.picking button_validate '[[<picking_id>]]' '{}'` | picking.state -> 'done', stock actualizado |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | PICKING_RETURN | Wizard stock.return.picking -> validate return | Devolucion a proveedor para revertir stock |
| 5 | (limpieza) | Verificar que return picking este done | |
| 4 | PURCHASE_CANCEL | `execute_method.sh purchase.order button_cancel '[[<PO_id>]]' '{}'` | Solo si no hay invoices asociadas |
| 3 | PURCHASE_DELETE | `delete_record.sh purchase.order '[<PO_id>]'` | Solo desde state=cancel |
| 2 | (scheduler ya ejecutado) | No hay rollback del scheduler — solo eliminar la PO generada | |
| 1 | REORDERING_RULE_DELETE | `delete_record.sh stock.warehouse.orderpoint '[<orderpoint_id>]'` | Eliminar regla para evitar re-generacion |

### Puntos de No Retorno
- **Paso 6 (picking done)**: No se puede cancelar, solo devolver
- **Nota**: Si el scheduler se ejecuta de nuevo y la regla sigue activa, generara una nueva PO automaticamente

---

