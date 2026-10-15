## 18. FABRICACIÓN — Órdenes de Producción (mrp.production)

### MRP_CREATE — Crear Orden de Fabricación
**Modelo:** mrp.production
**Descripción:** Crea una orden de producción para fabricar un producto a partir de una lista de materiales.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "bom_id": "<int>",
  "product_qty": 1.0,
  "product_uom_id": "<int>"
}
```
**Notas:**
- `product_id`: producto terminado (debe ser storable)
- `bom_id`: lista de materiales (se computa automáticamente desde product_id si existe)
- `product_qty`: cantidad a fabricar (debe ser > 0)
- `product_uom_id`: UdM del producto (se computa desde product_id)
- `picking_type_id`: tipo de operación fabricación (required, se computa desde la compañía)
- `location_src_id`: ubicación de componentes (se computa desde picking_type_id)
- `location_dest_id`: ubicación de producto terminado (se computa desde picking_type_id)
- `date_start`: fecha planificada de inicio (default: ahora)
- `company_id`: obligatorio (default: compañía del usuario)
**Método/acción:** `create_record.sh mrp.production '...'`
**Precondiciones:** BOM válida con componentes, producto storable
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `product_qty == <valor>`
**Estados disponibles en mrp.production (Odoo 18.0):**
- `draft` — Draft
- `confirmed` — Confirmed
- `progress` — In Progress
- `to_close` — To Close (entre progress y done, indica producción completa pero MO pendiente de cierre)
- `done` — Done
- `cancel` — Cancelled
**Estrategia de reversión:** `delete_record.sh mrp.production '[<id>]'` — posible en cualquier estado excepto `done` (ver MRP_DELETE)
**Registros cascada generados:** Ninguno hasta confirmar

### MRP_CONFIRM — Confirmar Orden de Fabricación
**Modelo:** mrp.production
**Descripción:** Confirma la OP, generando los movimientos de stock para componentes y producto terminado.
**Método:** `execute_method.sh mrp.production action_confirm '[[<id>]]' '{}'`
**Precondiciones:** state == 'draft', product_qty > 0, bom_id asignada
**Restricciones:**
- Genera stock.move para cada componente de la BOM
- Genera stock.move para el producto terminado
- Si hay operaciones en la BOM, genera mrp.workorder
**Resultado:** state → `confirmed` (o `progress` si auto-assign)
**Efectos secundarios:**
- Crea stock.move por cada componente (consume) y producto terminado (produce)
- Si hay operaciones: crea mrp.workorder por cada operación
- Intenta reservar componentes automáticamente (action_assign)
**Verificación:** `state in ('confirmed', 'progress')`, `move_raw_ids` no vacío
**Estrategia de reversión:** Cancelar → MRP_CANCEL
**Registros cascada generados:** stock.move (N+1), mrp.workorder (M si hay operaciones)

### MRP_PRODUCE — Producir (Marcar como Hecho)
**Modelo:** mrp.production
**Descripción:** Marca la orden de fabricación como completada, consumiendo componentes y produciendo el producto terminado.
**Método:** `execute_method.sh mrp.production button_mark_done '[[<id>]]' '{}'`
**Precondiciones:** state in ('confirmed', 'progress'), qty_producing > 0
**Restricciones:**
- Si `qty_producing == 0`, se asigna automáticamente = `product_qty`
- Si el producto tiene tracking (lot/serial), se necesita `lot_producing_id`
- Puede devolver wizard `mrp.immediate.production` si no se han establecido cantidades
- Puede devolver wizard de backorder si qty_producing < product_qty
- Para forzar sin wizard de backorder en MRP: el contexto es `mo_ids_to_backorder` (lista de MO IDs que deben generar backorder). Sin este contexto, por defecto ninguna MO genera backorder (`productions_not_to_backorder = self`). **Nota:** `skip_backorder` es para stock.picking, NO para mrp.production.
**Resultado:** state → `done`
**Efectos secundarios:**
- Valida movimientos de consumo de componentes (stock.move → done)
- Valida movimiento de producción del producto terminado
- Actualiza stock (stock.quant)
- Establece `date_finished`
- Automáticamente finaliza workorders (`self.workorder_ids.button_finish()`)
**Verificación:** `state == 'done'`, `date_finished != False`
**⚠️ PUNTO DE NO RETORNO:** Una vez done, NO se puede cancelar. Se requiere mrp.unbuild para revertir.
**Registros cascada generados:** stock.move.line (varios), stock.quant (actualiza)

### MRP_CANCEL — Cancelar Orden de Fabricación
**Modelo:** mrp.production
**Descripción:** Cancela una OP que no ha sido completada.
**Método:** `execute_method.sh mrp.production action_cancel '[[<id>]]' '{}'`
**Precondiciones:** state != 'done'
**Restricciones:**
- NO puede cancelar si state == 'done' (UserError)
- Si tiene workorders en progreso, las cancela también
- **Nota:** Si se ha hecho `_post_inventory()` con cantidad parcial, `action_cancel()` NO cancela — pone la MO en `done` con los moves parciales done y el resto cancelled
**Resultado:** state → `cancel`
**Verificación:** `state == 'cancel'`
**Estrategia de reversión:** No hay vuelta a draft en MRP estándar
**Registros cascada generados:** Cancela stock.move asociados

### MRP_DELETE — Eliminar Orden de Fabricación
**Modelo:** mrp.production
**Método:** `delete_record.sh mrp.production '[<id>]'`
**Restricciones:**
- **⚠️ CORRECCIÓN v2.0:** `unlink()` llama `action_cancel()` automáticamente antes de eliminar. La única restricción real es que NO se puede eliminar en estado `done` (UserError: "You cannot delete a manufacturing order that is already done.").
- Cualquier MO que NO esté en `done` puede ser eliminada — el unlink cancela automáticamente primero.
- Los workorders que no están en `done` también se eliminan automáticamente.
**Resultado:** Registro eliminado

### MRP_SCRAP — Desguazar desde Fabricación
**Modelo:** stock.scrap
**Descripción:** Crea un desguace de componentes o productos terminados asociado a una orden de fabricación.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "scrap_qty": "<float>",
  "production_id": "<int>"
}
```
**Notas:**
- `production_id`: ID de la OP asociada
- `product_id`: puede ser un componente o el producto terminado de la OP
- `location_id`: se computa desde la OP
- `scrap_location_id`: se computa automáticamente
**Método/acción:** `execute_method.sh stock.scrap action_validate '[[<id>]]' '{}'`
**Precondiciones:** OP en estado `confirmed` o `progress`
**Resultado:** state → `done`
**⚠️ PUNTO DE NO RETORNO:** Una vez validado, no se puede deshacer
**Registros cascada generados:** stock.move (1), stock.move.line (1)

---

