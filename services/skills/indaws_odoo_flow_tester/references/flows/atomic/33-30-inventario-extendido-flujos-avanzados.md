## 30. INVENTARIO EXTENDIDO — Flujos Avanzados

### REORDER_RULE_CREATE — Crear Regla de Reabastecimiento
**Modelo:** stock.warehouse.orderpoint
**Descripción:** (Ver flujo REORDERING_RULE_CREATE en sección 11 para detalles completos)
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "warehouse_id": "<int>",
  "location_id": "<int>",
  "product_min_qty": "<float>",
  "product_max_qty": "<float>"
}
```
**Resultado:** Regla activa, scheduler genera procurements automáticamente
**Verificación:** `active == True`, `product_min_qty == <valor>`

### MULTI_STEP_RECEIPT — Recepción en Múltiples Pasos
**Modelo:** stock.picking
**Descripción:** Procesa una recepción con pasos intermedios (Input → Quality → Stock o Input → Stock).
**Pasos vía API:**
1. Configurar almacén con recepción en 2/3 pasos: `update_record.sh stock.warehouse '[<id>]' '{"reception_steps": "two_steps"}'`
2. Crear picking de recepción (Input): se genera automáticamente al confirmar una OC
3. Validar picking Input: `execute_method.sh stock.picking button_validate '[[<input_picking_id>]]' '{}'`
4. Validar picking interno (Input → Stock): `execute_method.sh stock.picking button_validate '[[<internal_picking_id>]]' '{}'`
**Notas:**
- `reception_steps`: "one_step" (directo), "two_steps" (Input + Internal), "three_steps" (Input + Quality + Internal)
- Los pickings intermedios se generan automáticamente al validar el anterior
**Precondiciones:** Almacén configurado con múltiples pasos de recepción, OC confirmada
**Resultado:** Producto en ubicación de stock final
**Verificación:** `stock.quant` con `quantity > 0` en `lot_stock_id` del almacén

### SERIAL_TRACKING_CREATE — Crear Producto con Trazabilidad por Serie
**Modelo:** product.product
**Descripción:** Configura un producto con trazabilidad por número de serie y registra recepciones con series.
**Pasos vía API:**
1. Crear/actualizar producto: `update_record.sh product.product '[<id>]' '{"tracking": "serial"}'`
2. Crear recepción con series en las move_lines
**Notas:**
- `tracking`: "none" (sin tracking), "lot" (lotes), "serial" (series únicas)
- Con serial: cada unidad requiere un número de serie único
- Los lotes/series se crean automáticamente vía `lot_name` en stock.move.line
**Resultado:** Producto con tracking serial, stock por serie
**Verificación:** `tracking == 'serial'`, `stock.lot` creados

### INVENTORY_ADJUSTMENT — Ajuste de Inventario
**(Ver flujo INVENTORY_ADJUSTMENT en sección 11 para detalles completos)**
**Resumen:** Ajustar stock vía `stock.quant.action_apply_inventory`

---

