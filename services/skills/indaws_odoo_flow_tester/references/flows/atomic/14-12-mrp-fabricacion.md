## 12. MRP — Fabricación

### MRP_UNBUILD — Desensamblar Orden de Fabricación
**Modelo:** mrp.unbuild
**Descripción:** Deshace una fabricación, devolviendo los componentes al stock y retirando el producto terminado.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "bom_id": "<int>",
  "product_qty": "<float>",
  "location_id": "<int>",
  "location_dest_id": "<int>"
}
```
**Notas:**
- `mo_id`: opcional — si se indica, toma producto, BOM y lote de la OP original
- `bom_id`: se computa automáticamente si se indica `product_id` (busca BOM por producto)
- `product_qty`: cantidad a desensamblar (debe ser > 0, CHECK constraint)
- `location_id`: ubicación donde está el producto terminado
- `location_dest_id`: ubicación destino de los componentes recuperados
- `lot_id`: obligatorio si el producto tiene tracking
- Si `mo_id` se indica: el MO debe estar en estado `done`
**Método/acción:** `execute_method.sh mrp.unbuild action_validate '[[<id>]]' '{}'`
**Precondiciones:**
- Producto terminado disponible en `location_id`
- Si tiene tracking, `lot_id` obligatorio
- Si `mo_id`: MO.state == 'done'
**Restricciones:**
- Si stock insuficiente, devuelve wizard `stock.warn.insufficient.qty.unbuild`
- Para forzar: `execute_method.sh mrp.unbuild action_unbuild '[[<id>]]' '{}'`
- Si componentes tienen tracking y no se indica `mo_id`, UserError
**Resultado:** state → `done`
**Verificación:** `state == 'done'`
**Estrategia de reversión:** No se puede eliminar en estado `done` (UserError). Requiere crear una nueva orden de fabricación.
**⚠️ PUNTO DE NO RETORNO:** Una vez hecho, genera movimientos de stock irreversibles.
**Registros cascada generados:** stock.move (N consume + M produce), stock.move.line (varios)

### BOM_CREATE — Crear Lista de Materiales
**Modelo:** mrp.bom
**Descripción:** Crea una lista de materiales (BOM) que define los componentes necesarios para fabricar un producto.
**Campos mínimos:**
```json
{
  "product_tmpl_id": "<int>",
  "product_qty": 1.0,
  "type": "normal",
  "bom_line_ids": [[0, 0, {
    "product_id": "<int>",
    "product_qty": "<float>"
  }]]
}
```
**Notas:**
- `product_tmpl_id`: template del producto a fabricar (debe ser storable, `type == 'consu'`)
- `product_id`: variante específica (opcional — si no se indica, aplica a todas las variantes)
- `type`: "normal" (fabricar) o "phantom" (kit — se descompone en componentes)
- `product_qty`: cantidad del producto que genera esta BOM (default 1.0)
- `bom_line_ids`: componentes con product_id y product_qty
- `operation_ids`: operaciones de routing (opcional, para work orders)
- `consumption`: "flexible", "warning" o "strict"
- `code`: referencia interna (opcional)
**Método/acción:** `create_record.sh mrp.bom '...'`
**Precondiciones:** Producto template storable, productos componente existentes
**Resultado:** BOM activa, lista para uso en órdenes de fabricación
**Verificación:** `active == True`, `bom_line_ids` no vacío
**Estrategia de reversión:** `delete_record.sh mrp.bom '[<id>]'` o archivar: `update_record.sh mrp.bom '[<id>]' '{"active": false}'`
**Registros cascada generados:** mrp.bom.line (N componentes)

### WORKORDER_DONE — Completar Orden de Trabajo
**Modelo:** mrp.workorder
**Descripción:** Marca una orden de trabajo como completada dentro de una orden de fabricación.
**Método:** `execute_method.sh mrp.workorder button_finish '[[<id>]]' '{}'`
**Precondiciones:** state not in ('done', 'cancel'), production_id existente
**Restricciones:**
- Automáticamente marca las materias primas asociadas a la operación como `picked = True`
- Establece las cantidades de componentes según `qty_producing` de la OP
- Si `qty_producing == 0`, usa `product_qty` como cantidad disponible
**Resultado:** state → `done`, date_finished se establece
**Efectos secundarios:**
- Crea registros de tiempo de trabajo (mrp.workcenter.productivity)
- Actualiza la OP padre si es la última workorder
**Verificación:** `state == 'done'`, `date_finished != False`
**Estrategia de reversión:** No hay reversión directa. Se requiere crear nueva workorder o desensamblar.
**Registros cascada generados:** mrp.workcenter.productivity (tiempos de trabajo)

---

