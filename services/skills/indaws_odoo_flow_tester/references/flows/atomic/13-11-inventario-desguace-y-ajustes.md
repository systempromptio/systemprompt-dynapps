## 11. INVENTARIO — Desguace y Ajustes

### SCRAP_CREATE — Crear Desguace
**Modelo:** stock.scrap
**Descripción:** Crea un registro de desguace para sacar productos del stock (defectuosos, obsoletos, etc.).
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "scrap_qty": "<float>",
  "location_id": "<int>",
  "scrap_location_id": "<int>"
}
```
**Notas:**
- `product_id`: debe ser de tipo storable (`type == 'consu'` en Odoo 18)
- `scrap_qty`: cantidad a desguazar (debe ser > 0)
- `location_id`: ubicación origen (se computa automáticamente si no se indica — lot_stock_id del warehouse)
- `scrap_location_id`: ubicación de desguace (se computa automáticamente — ubicación con `scrap_location == True`)
- `lot_id`: obligatorio si el producto tiene tracking serial/lot
- `picking_id`: opcional, si el desguace se origina desde un albarán
- `should_replenish`: si True, genera reabastecimiento automático
**Método/acción de validación:** `execute_method.sh stock.scrap action_validate '[[<id>]]' '{}'`
**Resultado:** state → `done` tras action_validate. Internamente ejecuta `do_scrap()`.
**Restricciones:**
- Si no hay stock suficiente, devuelve wizard de advertencia `stock.warn.insufficient.qty.scrap`
- Para forzar sin stock: `execute_method.sh stock.scrap do_scrap '[[<id>]]' '{}'`
**Verificación:** `state == 'done'`, `date_done != False`
**Estrategia de reversión:** No se puede eliminar en estado `done` (UserError). La reversión requiere un ajuste de inventario manual o una entrada de stock.
**⚠️ PUNTO DE NO RETORNO:** Una vez validado (done), no se puede cancelar ni eliminar.
**Registros cascada generados:** stock.move (1), stock.move.line (1)
**🔧 SQL Fallback para Reversión (solo entornos de prueba):**
Si es necesario revertir un desguace en un entorno de staging/testing, se puede usar SQL directo:
1. Identificar el stock.move asociado: `SELECT id, state FROM stock_move WHERE scrap_id = <scrap_id>;`
2. Revertir quant (sumar cantidad de vuelta): `UPDATE stock_quant SET quantity = quantity + <scrap_qty> WHERE product_id = <product_id> AND location_id = <location_id>;`
3. Reducir quant de scrap location: `UPDATE stock_quant SET quantity = quantity - <scrap_qty> WHERE product_id = <product_id> AND location_id = <scrap_location_id>;`
4. Eliminar move lines: `DELETE FROM stock_move_line WHERE move_id = <move_id>;`
5. Eliminar move: `DELETE FROM stock_move WHERE id = <move_id>;`
6. Resetear scrap: `UPDATE stock_scrap SET state = 'draft', date_done = NULL WHERE id = <scrap_id>;`
7. Eliminar scrap: `DELETE FROM stock_scrap WHERE id = <scrap_id>;`
**⚠️ NUNCA ejecutar SQL en producción. La alternativa ORM es un ajuste de inventario (stock.quant action_apply_inventory).**

### INVENTORY_ADJUSTMENT — Ajuste de Inventario
**Modelo:** stock.quant
**Descripción:** Ajusta la cantidad en stock de un producto en una ubicación específica. En Odoo 18, los ajustes de inventario se hacen directamente sobre los quants.
**Pasos vía API:**
1. Buscar o crear quant: `search_records.sh stock.quant '[["product_id","=",<id>],["location_id","=",<id>]]' '["id","inventory_quantity"]'`
2. Establecer cantidad de inventario: `update_record.sh stock.quant '[<quant_id>]' '{"inventory_quantity": <nueva_cantidad>}'`
3. Aplicar ajuste: `execute_method.sh stock.quant action_apply_inventory '[[<quant_id>]]' '{}'`
**Campos necesarios en el quant:**
```json
{
  "product_id": "<int>",
  "location_id": "<int>",
  "inventory_quantity": "<float>",
  "lot_id": "<int>"
}
```
**Notas:**
- `inventory_quantity`: la cantidad real contada (el sistema calcula la diferencia automáticamente via `inventory_diff_quantity`)
- `lot_id`: obligatorio si el producto tiene tracking
- Si el quant no existe, se puede crear con `create_record.sh stock.quant '...'`
- Si hay quants obsoletos (`is_outdated == True`), puede mostrar wizard de confirmación
- Contexto `{"set_inventory_quantity_auto_apply": True}` aplica sin wizard de confirmación
**Precondiciones:** Producto storable, ubicación interna
**Resultado:** Quant actualizado con `quantity` = `inventory_quantity`, `inventory_quantity` se resetea a 0
**Verificación:** `quantity == <valor_esperado>`, `inventory_diff_quantity == 0`
**Estrategia de reversión:** Repetir el ajuste con la cantidad original
**Registros cascada generados:** stock.move (1 ajuste), stock.move.line (1)

### REORDERING_RULE_CREATE — Crear Regla de Reabastecimiento
**Modelo:** stock.warehouse.orderpoint
**Descripción:** Crea una regla de stock mínimo que genera procurements automáticos cuando el stock virtual cae por debajo del mínimo.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "warehouse_id": "<int>",
  "location_id": "<int>",
  "product_min_qty": "<float>",
  "product_max_qty": "<float>",
  "qty_multiple": 1.0
}
```
**Notas:**
- `product_id`: debe ser storable (`is_storable == True`)
- `warehouse_id`: se computa desde location_id si no se indica
- `location_id`: se computa como `warehouse.lot_stock_id` si no se indica
- `product_min_qty`: cantidad mínima — debajo de este nivel se genera procurement
- `product_max_qty`: cantidad objetivo del procurement (>= min)
- `qty_multiple`: redondeo de la cantidad de procurement (default 1.0)
- `trigger`: "auto" (scheduler automático) o "manual"
- `name`: se genera automáticamente vía secuencia `stock.orderpoint`
**Método/acción:** `create_record.sh stock.warehouse.orderpoint '...'`
**Precondiciones:** Producto storable, warehouse con rutas configuradas
**Resultado:** Regla activa. El scheduler generará procurements cuando stock < min.
**Verificación:** `active == True`, `product_min_qty == <valor>`
**Estrategia de reversión:** `delete_record.sh stock.warehouse.orderpoint '[<id>]'` o archivar: `update_record.sh stock.warehouse.orderpoint '[<id>]' '{"active": false}'`
**Registros cascada generados:** Ninguno directamente. El scheduler genera purchase.order o mrp.production según las rutas.

---

