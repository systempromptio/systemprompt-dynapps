## CADENA 19: Fabricacion Completa con Entrega (MRP Complete)
**ID:** CHAIN_MRP_COMPLETE
**Apps:** MRP + Inventario + Ventas (opcional)
**Marcado:** [MIG]
**Descripcion:** Flujo completo desde la creacion de la lista de materiales hasta la entrega del producto terminado al cliente, incluyendo reserva de componentes y produccion.
**Tiempo estimado:** 3-5 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| mrp.production.action_confirm | mrp.production | stock.move (componentes) | `search_records.sh stock.move '[["raw_material_production_id","=",<MO_id>]]'` |
| mrp.production.action_confirm | mrp.production | stock.move (producto terminado) | `search_records.sh stock.move '[["production_id","=",<MO_id>]]'` |
| mrp.production.button_mark_done | mrp.production | stock.move.line | Incluidos en los stock.move |
| SO asociado -> MO done | mrp.production | stock.picking (delivery) | `search_records.sh stock.picking '[["origin","=","<SO_name>"],["picking_type_code","=","outgoing"]]'` |

### Paso de Variables Entre Flujos
- `{chain.bom_id}` — ID de la lista de materiales
- `{chain.product_id}` — ID del producto terminado
- `{chain.component_ids}` — IDs de los componentes
- `{chain.MO_id}` — ID de la mrp.production
- `{chain.MO_name}` — Referencia de la MO
- `{chain.picking_out_id}` — ID del picking de entrega

### Precondiciones
- Producto fabricable (type='product') con BoM definida
- Componentes con stock disponible en ubicacion de produccion
- Centro de trabajo configurado (si usa workorders)
- Ubicacion de produccion y producto terminado configuradas en el almacen

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | BOM_VERIFY | Verificar mrp.bom existente para el producto, o crear una nueva con bom_line_ids | BOM con lineas de componentes |
| 2 | MRP_CREATE | Crear mrp.production con product_id, bom_id, product_qty | state=draft |
| 3 | MRP_CONFIRM | `execute_method.sh mrp.production action_confirm '[[<MO_id>]]' '{}'` | state -> 'confirmed' |
| 4 | MRP_ASSIGN | `execute_method.sh mrp.production action_assign '[[<MO_id>]]' '{}'` | reservation_state -> 'assigned' (componentes reservados) |
| 5 | MRP_PRODUCE | `update_record.sh mrp.production '[<MO_id>]' '{"qty_producing": <qty>}'` | qty_producing == product_qty |
| 6 | MRP_DONE | `execute_method.sh mrp.production button_mark_done '[[<MO_id>]]' '{}'` | state -> 'done', producto terminado en stock |
| 7 | — detectar picking (si SO) — | Buscar picking de salida si hay venta asociada | picking encontrado |
| 8 | PICKING_VALIDATE | `execute_method.sh stock.picking button_validate '[[<picking_out_id>]]' '{}'` | picking.state -> 'done' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 8 | PICKING_RETURN | Wizard stock.return.picking -> validate return | Si habia entrega |
| 7 | (deteccion) | | |
| 6 | **PUNTO DE NO RETORNO** | MO done no se puede cancelar | Opcion: crear scrap del producto terminado |
| 5 | (incluido en MO) | | |
| 4 | (reserva se libera al cancelar) | | |
| 3 | MRP_CANCEL | `execute_method.sh mrp.production action_cancel '[[<MO_id>]]' '{}'` | Solo si state != 'done' |
| 2 | MRP_DELETE | `delete_record.sh mrp.production '[<MO_id>]'` | Solo desde state='cancel' o 'draft' |
| 1 | BOM_DELETE | `delete_record.sh mrp.bom '[<bom_id>]'` | Solo si fue creada durante el test |

### Puntos de No Retorno
- **Paso 6 (MO done)**: Fabricacion completada — stock de componentes consumido y producto terminado creado. Solo reversible via scrap o ajuste de inventario
- **Paso 8 (picking done)**: Solo devolucion, no cancelacion

---

