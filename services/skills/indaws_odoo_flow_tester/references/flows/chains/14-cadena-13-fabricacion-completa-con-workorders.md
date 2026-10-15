## CADENA 13: Fabricacion Completa con Workorders
**ID:** CHAIN_MRP_FULL
**Apps:** MRP + Inventario
**Descripcion:** Fabricacion completa incluyendo ordenes de trabajo (workorders) en centros de trabajo, seguido de la entrega del producto terminado.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| mrp.production.action_confirm | mrp.production | mrp.workorder | `search_records.sh mrp.workorder '[["production_id","=",<MO_id>]]'` |
| mrp.production.action_confirm | mrp.production | stock.move | `search_records.sh stock.move '[["production_id","=",<MO_id>]]'` |
| mrp.production.button_mark_done | mrp.production | stock.picking (delivery) | Deteccion via SO asociado si existe |

### Paso de Variables Entre Flujos
- `{chain.bom_id}` — ID de la lista de materiales
- `{chain.product_id}` — ID del producto a fabricar
- `{chain.MO_id}` — ID de la mrp.production
- `{chain.MO_name}` — Referencia de la orden de fabricacion
- `{chain.workorder_ids}` — IDs de las ordenes de trabajo generadas
- `{chain.workcenter_id}` — ID del centro de trabajo
- `{chain.picking_id}` — ID del picking de entrega (si hay SO asociado)

### Precondiciones
- Lista de materiales (mrp.bom) con operaciones (mrp.routing.workcenter) definidas
- Centros de trabajo (mrp.workcenter) configurados
- Producto fabricable (type='product')
- Componentes con stock disponible (o reserva manual)
- Modulo `mrp_workorder` instalado (Enterprise) para workorders

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | BOM_CREATE | Crear mrp.bom con product_tmpl_id + bom_line_ids (componentes) + operation_ids (operaciones con workcenter_id) | BOM creada con lineas y operaciones |
| 2 | MRP_CREATE | Crear mrp.production con product_id, bom_id, product_qty | state=draft |
| 3 | MRP_CONFIRM | `execute_method.sh mrp.production action_confirm '[[<MO_id>]]' '{}'` | state -> 'confirmed', workorders generados |
| 4 | — detectar workorders — | `search_records.sh mrp.workorder '[["production_id","=",<MO_id>]]' '["id","name","state","workcenter_id"]'` | Workorders en estado 'ready' o 'pending' |
| 5 | WORKORDER_START | `execute_method.sh mrp.workorder button_start '[[<wo_id>]]' '{}'` (por cada workorder en orden) | workorder.state -> 'progress' |
| 6 | WORKORDER_DONE | `execute_method.sh mrp.workorder button_finish '[[<wo_id>]]' '{}'` (por cada workorder en orden) | workorder.state -> 'done' |
| 7 | MRP_DONE | `execute_method.sh mrp.production button_mark_done '[[<MO_id>]]' '{}'` | state -> 'done', qty_produced == product_qty |
| 8 | PICKING_VALIDATE | Validar picking de entrega (si hay SO asociado) | picking.state -> 'done' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 8 | PICKING_RETURN | Wizard stock.return.picking -> validate return | Solo si hay picking de entrega |
| 7 | (MO done) | **PUNTO DE NO RETORNO**: MO done no se puede cancelar | Requiere ajuste inventario o scrap |
| 6 | (workorders done) | Workorders completadas no se pueden revertir | Parte del punto de no retorno de la MO |
| 5 | (workorders started) | Si state='progress': `button_unwork` puede deshacer tiempo registrado | Solo parcialmente reversible |
| 4 | (detectados) | | |
| 3 | MRP_CANCEL | `execute_method.sh mrp.production action_cancel '[[<MO_id>]]' '{}'` | Solo si state != 'done' |
| 2 | MRP_DELETE | `delete_record.sh mrp.production '[<MO_id>]'` | Solo desde state='cancel' o 'draft' |
| 1 | BOM_DELETE | `delete_record.sh mrp.bom '[<bom_id>]'` | Solo si no hay MOs activas referenciando esta BOM |

### Puntos de No Retorno
- **Paso 6 (workorders done)**: Las horas y consumos registrados en workorders no se pueden deshacer
- **Paso 7 (MO done)**: Fabricacion completada no se puede cancelar — solo scrap o ajuste de inventario
- **Paso 8 (picking done)**: Solo devolucion, no cancelacion

---

