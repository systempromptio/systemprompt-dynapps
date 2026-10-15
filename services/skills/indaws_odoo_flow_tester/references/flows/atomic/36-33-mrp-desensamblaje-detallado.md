## 33. MRP — Desensamblaje Detallado

### MRP_UNBUILD_CREATE — Crear Orden de Desensamblaje
**Modelo:** mrp.unbuild
**Descripción:** Crea una orden de desensamblaje para deshacer una fabricación.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "product_qty": "<float>",
  "location_id": "<int>",
  "location_dest_id": "<int>"
}
```
**Notas:**
- `product_id`: producto terminado a desensamblar (required, `type == 'consu'`)
- `product_qty`: cantidad a desensamblar (required, debe ser > 0 — CHECK constraint)
- `bom_id`: se computa automáticamente desde product_id (busca BOM tipo 'normal')
- `product_uom_id`: se computa desde product_id
- `mo_id`: opcional — si se indica, toma producto, BOM, lote y cantidad de la OP original
  - Si `mo_id` y tracking == 'serial' → product_qty = 1 automáticamente
  - Si `mo_id` → product_qty = mo_id.qty_produced
- `lot_id`: obligatorio si producto tiene tracking != 'none'
- `location_id` / `location_dest_id`: se computan desde warehouse de la compañía
- `name`: se genera automáticamente via secuencia `mrp.unbuild`
- States: `draft` → `done` (solo 2 estados)
**Método/acción:** `create_record.sh mrp.unbuild '...'`
**Precondiciones:** Producto storable con BOM, stock disponible en location_id
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `product_qty == <valor>`, `bom_id != False`
**Estrategia de reversión:** `delete_record.sh mrp.unbuild '[<id>]'` — solo en estado `draft`
**Registros cascada generados:** Ninguno hasta validar

### MRP_UNBUILD_VALIDATE — Validar Orden de Desensamblaje
**Modelo:** mrp.unbuild
**Descripción:** Ejecuta el desensamblaje, consumiendo el producto terminado y devolviendo los componentes al stock.
**Método:** `execute_method.sh mrp.unbuild action_validate '[[<id>]]' '{}'`
**Precondiciones:**
- state == 'draft'
- Si tracking != 'none' → lot_id obligatorio
- Si mo_id → mo_id.state == 'done'
- Stock suficiente del producto en location_id
**Restricciones:**
- Si stock insuficiente → devuelve wizard `stock.warn.insufficient.qty.unbuild`
- Para forzar sin stock: `execute_method.sh mrp.unbuild action_unbuild '[[<id>]]' '{}'`
- Si componentes tienen tracking y no se indica mo_id → UserError
- Si byproducts tienen tracking y no se indica mo_id → UserError
**Resultado:** state → `done`
**Efectos secundarios:**
- Genera consume_moves (producto terminado → producción) y produce_moves (producción → componentes)
- Los moves se validan automáticamente (_action_done)
- Si mo_id: publica mensaje en la OP con cantidad desensamblada
**Verificación:** `state == 'done'`
**⚠️ PUNTO DE NO RETORNO:** Una vez done, no se puede cancelar ni eliminar (UserError)
**Registros cascada generados:** stock.move (N consume + M produce), stock.move.line (varios)

---

