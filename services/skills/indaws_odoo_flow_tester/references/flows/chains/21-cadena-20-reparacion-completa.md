## CADENA 20: Reparacion Completa
**ID:** CHAIN_REPAIR_COMPLETE
**Apps:** Reparaciones + Inventario + Contabilidad
**Descripcion:** Flujo completo de reparacion: crear orden de reparacion, confirmar, anadir piezas de repuesto, ejecutar la reparacion, completar y facturar.
**Tiempo estimado:** 2-4 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| repair.order.action_validate | repair.order | stock.move | `search_records.sh stock.move '[["repair_id","=",<repair_id>]]'` |
| repair.order.action_repair_end | repair.order | stock.move (done) | Movimientos de stock completados |
| Facturacion reparacion | repair.order | account.move | `search_records.sh account.move '[["repair_ids","in",[<repair_id>]]]'` |

### Paso de Variables Entre Flujos
- `{chain.repair_id}` — ID de la repair.order
- `{chain.repair_name}` — Referencia de la reparacion
- `{chain.product_id}` — ID del producto a reparar
- `{chain.lot_id}` — ID del lote/serie (si aplica)
- `{chain.partner_id}` — ID del cliente
- `{chain.move_ids}` — IDs de los stock.move generados
- `{chain.invoice_id}` — ID de la factura generada

### Precondiciones
- Modulo `repair` instalado
- Producto reparable existente (type='product')
- Piezas de repuesto disponibles en stock
- Partner (cliente) existente
- Diario de ventas para facturacion

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | REPAIR_CREATE | Crear repair.order con product_id, partner_id, move_ids (lineas de piezas) | state=draft |
| 2 | REPAIR_CONFIRM | `execute_method.sh repair.order action_validate '[[<repair_id>]]' '{}'` | state -> 'confirmed' |
| 3 | REPAIR_ADD_PARTS | Anadir/actualizar lineas de operacion (piezas + mano de obra) via update_record | Lineas con product_id, product_uom_qty, price_unit |
| 4 | REPAIR_START | `execute_method.sh repair.order action_repair_start '[[<repair_id>]]' '{}'` | state -> 'under_repair' |
| 5 | REPAIR_END | `execute_method.sh repair.order action_repair_end '[[<repair_id>]]' '{}'` | state -> 'done', movimientos de stock realizados |
| 6 | REPAIR_INVOICE | `execute_method.sh repair.order action_create_invoice '[[<repair_id>]]' '{}'` | Factura creada en draft |
| 7 | INVOICE_POST | `execute_method.sh account.move action_post '[[<invoice_id>]]' '{}'` | invoice.state -> 'posted' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 7 | INVOICE_DRAFT -> CANCEL -> DELETE | button_draft -> button_cancel -> delete_record | Verificar restricciones hash |
| 6 | (factura eliminada en paso 7) | | |
| 5 | **PUNTO DE NO RETORNO PARCIAL** | Reparacion done — movimientos de stock ya realizados | Requiere ajuste de inventario si se quiere revertir stock |
| 4 | (incluido en done) | | |
| 3 | (lineas eliminadas con la reparacion) | | |
| 2 | REPAIR_CANCEL | `execute_method.sh repair.order action_repair_cancel '[[<repair_id>]]' '{}'` | Solo si state != 'done' |
| 1 | REPAIR_DELETE | `delete_record.sh repair.order '[<repair_id>]'` | Solo desde state='cancel' o 'draft' |

### Puntos de No Retorno
- **Paso 5 (repair done)**: Los movimientos de stock de piezas se han consumido. Revertir requiere ajuste de inventario
- **Paso 7 (invoice posted)**: Si journal tiene hash -> no se puede volver a draft

---

