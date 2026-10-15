## 32. INVENTARIO — Lotes y Ajustes Avanzados

### STOCK_LOT_CREATE — Crear Lote/Número de Serie
**Modelo:** stock.lot
**Descripción:** Crea un lote o número de serie para un producto con trazabilidad.
**Campos mínimos:**
```json
{
  "name": "LOT-001",
  "product_id": "<int>",
  "company_id": "<int>"
}
```
**Notas:**
- `name`: número de lote/serie (required, se genera automáticamente via secuencia `stock.lot.serial` si no se indica)
- `product_id`: producto con `tracking != 'none'` y `is_storable == True` (required)
- `company_id`: se computa automáticamente
- `ref`: referencia interna (opcional)
- `note`: descripción HTML (opcional)
- Constraint: nombre único por producto + compañía
**Método/acción:** `create_record.sh stock.lot '...'`
**Precondiciones:** Producto con tracking 'lot' o 'serial'
**Resultado:** Lote/serie creado
**Verificación:** `name == <valor>`, `product_id == <id>`
**Estrategia de reversión:** `delete_record.sh stock.lot '[<id>]'` — solo si no tiene quants asociados
**Registros cascada generados:** Ninguno

### STOCK_QUANT_UPDATE — Ajuste de Inventario (Quant)
**(Alias de INVENTORY_ADJUSTMENT en sección 11)**
**Modelo:** stock.quant
**Descripción:** Ajusta la cantidad real de un producto en una ubicación específica.
**Pasos vía API:**
1. Buscar quant: `search_records.sh stock.quant '[["product_id","=",<id>],["location_id","=",<id>]]' '["id","quantity","inventory_quantity"]'`
2. Establecer cantidad: `update_record.sh stock.quant '[<quant_id>]' '{"inventory_quantity": <nueva_cantidad>}'`
3. Aplicar: `execute_method.sh stock.quant action_apply_inventory '[[<quant_id>]]' '{}'`
**Alternativa rápida:** Usar contexto `{"set_inventory_quantity_auto_apply": True}` al hacer write para aplicar sin paso 3
**Resultado:** `quantity` actualizado, `inventory_quantity` resetea a 0
**Verificación:** `quantity == <valor_esperado>`, `inventory_diff_quantity == 0`

### STOCK_RETURN_CREATE — Crear Devolución de Picking
**(Alias del paso 1-2 de PICKING_RETURN en sección 3)**
**Modelo:** stock.return.picking (wizard)
**Descripción:** Crea un picking de devolución a partir de un picking validado.
**Pasos vía API:**
1. Crear wizard: `create_record.sh stock.return.picking '{"picking_id": <id>}' --context '{"active_id": <id>, "active_model": "stock.picking"}'`
2. Ejecutar: `execute_method.sh stock.return.picking action_create_returns_all '[[<wizard_id>]]' '{}'` (completa) o `action_create_returns` (parcial, tras ajustar cantidades)
**Precondiciones:** picking.state == 'done'
**Resultado:** Nuevo picking de devolución en estado `confirmed`
**Verificación:** Nuevo picking con `return_id == <original_picking_id>`, `origin == "Return of <picking_name>"`

### STOCK_RETURN_VALIDATE — Validar Devolución de Picking
**(Alias de PICKING_VALIDATE para el picking de devolución)**
**Modelo:** stock.picking
**Método:** `execute_method.sh stock.picking button_validate '[[<return_picking_id>]]' '{}'`
**Precondiciones:** Picking de devolución en estado 'confirmed' o 'assigned'
**Resultado:** state → `done`, stock devuelto a la ubicación original
**Verificación:** `state == 'done'`

---

