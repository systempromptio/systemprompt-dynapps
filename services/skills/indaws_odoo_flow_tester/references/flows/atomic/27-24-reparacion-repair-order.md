## 24. REPARACIÓN (repair.order)

### REPAIR_CREATE — Crear Orden de Reparación
**Modelo:** repair.order
**Descripción:** Crea una orden de reparación para un producto.
**Campos mínimos:**
```json
{
  "product_id": "<int>",
  "partner_id": "<int>",
  "picking_type_id": "<int>"
}
```
**Notas:**
- `product_id`: producto a reparar (required, storable)
- `partner_id`: cliente (opcional, se computa desde lot/picking)
- `picking_type_id`: tipo de operación de reparación (required, se computa)
- `lot_id`: lote/serie del producto (si tiene tracking)
- `location_id`: ubicación de origen del producto
- `location_dest_id`: ubicación destino tras reparación
- `schedule_date`: fecha programada (required, default: ahora)
- `user_id`: responsable (default: usuario actual)
- `under_warranty`: si está bajo garantía (precio 0 para componentes)
- `move_ids`: líneas de componentes para la reparación (repair lines)
- state: 'draft' → 'confirmed' → 'under_repair' → 'done' / 'cancel'
**Método/acción:** `create_record.sh repair.order '...'`
**Precondiciones:** Producto storable, tipo de operación de reparación
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `product_id == <id>`
**Estrategia de reversión:** `delete_record.sh repair.order '[<id>]'` — solo en draft

### REPAIR_CONFIRM — Confirmar Orden de Reparación
**Modelo:** repair.order
**Descripción:** Confirma la orden e inicia el proceso de reparación.
**Método:** `execute_method.sh repair.order action_repair_confirm '[[<id>]]' '{}'`
**Precondiciones:** state == 'draft'
**Resultado:** state → `confirmed`
**Verificación:** `state == 'confirmed'`
**Estrategia de reversión:** Cancelar → REPAIR_CANCEL

### REPAIR_DONE — Completar Reparación
**Modelo:** repair.order
**Descripción:** Marca la reparación como completada, validando los movimientos de stock.
**Pasos:**
1. Iniciar reparación: `execute_method.sh repair.order action_repair_start '[[<id>]]' '{}'` → state = `under_repair`
2. Finalizar: `execute_method.sh repair.order action_repair_end '[[<id>]]' '{}'` → state = `done`
**Precondiciones:** state == 'confirmed' (para start), state == 'under_repair' (para end)
**Restricciones:**
- `action_repair_end` valida los movimientos de stock (consume componentes, produce producto reparado)
- Si componentes tienen tracking, se necesita lot_id en las líneas
**Resultado:** state → `done`
**Efectos secundarios:**
- Valida stock.move asociados (done)
- Consume componentes del stock
- Puede generar factura si configurado
**Verificación:** `state == 'done'`
**⚠️ PUNTO DE NO RETORNO:** Una vez done, no se puede cancelar
**Registros cascada generados:** stock.move (N), stock.move.line (N)

### REPAIR_CANCEL — Cancelar Orden de Reparación
**Modelo:** repair.order
**Método:** `execute_method.sh repair.order action_repair_cancel '[[<id>]]' '{}'`
**Precondiciones:** state != 'done'
**Resultado:** state → `cancel`
**Verificación:** `state == 'cancel'`
**Para volver a borrador:** `execute_method.sh repair.order action_repair_cancel_draft '[[<id>]]' '{}'` — state → `draft`

---

