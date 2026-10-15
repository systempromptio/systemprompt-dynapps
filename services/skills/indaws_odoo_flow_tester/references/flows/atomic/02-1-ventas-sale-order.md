## 1. VENTAS (sale.order)

### SALE_CREATE — Crear Pedido de Venta
**Modelo:** sale.order
**Campos mínimos:**
```json
{
  "partner_id": "<int>",
  "order_line": [[0, 0, {
    "product_id": "<int>",
    "product_uom_qty": "<float>",
    "price_unit": "<float>"
  }]]
}
```
**Notas:**
- `price_unit` es opcional — usa `list_price` del producto si no se indica.
- `order_line` es opcional en `create()` — se puede crear un SO vacío y añadir líneas después. Sin embargo, es **obligatorio tener al menos una línea con product_id para confirmar** (`action_confirm`).
**Resultado:** Registro en estado `draft`
**Verificación:** `state == 'draft'`, `amount_total > 0` (si tiene líneas)

### SALE_CONFIRM — Confirmar Pedido
**Método:** `execute_method.sh sale.order action_confirm '[[<id>]]' '{}'`
**Precondición:** state == 'draft' o 'sent'
**Restricciones:**
- TODAS las líneas regulares (no display_type, no is_downpayment) deben tener product_id
- No puede confirmar si state != 'draft'/'sent' → UserError
**Resultado:** state → `sale`
**Efectos secundarios (sale_stock):**
- Genera procurement group
- Crea stock.picking de salida (delivery order)
- Pickings se confirman automáticamente
**Verificación:** `state == 'sale'`, picking_ids no vacío

### SALE_CANCEL — Cancelar Pedido
**Método:** `execute_method.sh sale.order action_cancel '[[<id>]]' '{}'`
**Restricciones:**
- NO puede cancelar si `locked == True` → UserError "You cannot cancel a locked order. Please unlock it first."
- Si state != 'draft', Odoo muestra wizard sale.order.cancel
  - Para API: usar execute_method con contexto `{'disable_cancel_warning': True}`
- Cancela facturas en draft asociadas (comportamiento del módulo `sale` base)
- **⚠️ Módulo `sale_stock`:** La cancelación de pickings NO completados (state != 'done') es una extensión del módulo `sale_stock`, NO del módulo base `sale`. Si `sale_stock` no está instalado, `_action_cancel` solo cancela facturas draft.
- Pickings 'done' NO se cancelan (requieren devolución)
- **⚠️ NOTA RPC:** `_action_cancel` es un método PRIVADO (prefijo `_`) y NO debe usarse via JSON-RPC. Usar `action_cancel` (sin prefijo `_`) con contexto `{'disable_cancel_warning': True}` para bypass del wizard de confirmación.
**Resultado:** state → `cancel`

### SALE_DELETE — Eliminar Pedido
**Método:** `delete_record.sh sale.order '[<id>]'`
**Restricciones:**
- SOLO se puede eliminar en estado 'draft' o 'cancel'
- Si state not in ('draft', 'cancel') → UserError "You can not delete a sent quotation or a confirmed sales order."
**Precondición:** state == 'draft' o 'cancel'

### SALE_DRAFT — Volver a Borrador
**Método:** `execute_method.sh sale.order action_draft '[[<id>]]' '{}'`
**Precondición:** state == 'cancel' o 'sent'
**Resultado:** state → `draft`, limpia firma (signature, signed_by, signed_on)
**Nota:** `action_draft` usa `filtered()` internamente — si se llama en un SO con otro estado (ej: 'sale'), simplemente no hace nada (no lanza error).

### SALE_VALIDATE_PICKING — Validar Albarán de Venta (Secuencia Completa)
**Descripción:** Tras confirmar el pedido, se genera automáticamente un picking de salida. Para validarlo correctamente se debe seguir esta secuencia:
**Pasos vía API:**
1. **Obtener picking:** El picking se crea automáticamente al confirmar el SO. Acceder via `sale_order.picking_ids[0]` o buscar por origin.
2. **Confirmar picking:** `execute_method.sh stock.picking action_confirm '[[<picking_id>]]' '{}'`
   - En la mayoría de casos el picking ya está confirmado automáticamente tras confirmar el SO.
3. **Reservar stock:** `execute_method.sh stock.picking action_assign '[[<picking_id>]]' '{}'`
   - Reserva stock disponible. State pasa a `assigned` si hay stock suficiente.
   - Si no hay stock suficiente, state queda en `confirmed` o `waiting`.
4. **Establecer cantidades procesadas:**
   - En move_line_ids: `update_record.sh stock.move.line '[<line_ids>]' '{"quantity": <qty>}'`
   - En move_ids: `update_record.sh stock.move '[<move_ids>]' '{"picked": true}'`
   - **⚠️ CRÍTICO:** En Odoo 18, el campo es `quantity` en stock.move.line (NO `qty_done` que era Odoo < 17). El campo `picked` en stock.move indica que el operario ha procesado el movimiento.
5. **Validar picking:** `execute_method.sh stock.picking button_validate '[[<picking_id>]]' '{}'`
   - Para evitar wizard de backorder: contexto `{"skip_backorder": true, "picking_ids_not_to_backorder": [<picking_id>]}`
**Resultado:** picking.state → `done`
**Verificación:** `state == 'done'`, `date_done != False`
**⚠️ PUNTO DE NO RETORNO:** Una vez `done`, solo se puede revertir con devolución (stock.return.picking)

### SALE_CREATE_INVOICE — Crear Factura desde SO
**Método directo:** `execute_method.sh sale.order _create_invoices '[[<id>]]' '{}'`
**Método via wizard (recomendado):**
1. Crear wizard: `create_record.sh sale.advance.payment.inv '{}' --context '{"active_ids": [<so_id>], "active_model": "sale.order"}'`
2. Ejecutar: `execute_method.sh sale.advance.payment.inv create_invoices '[[<wizard_id>]]' '{}'`
**Precondición:** state == 'sale', líneas con qty_to_invoice > 0
**Resultado:** Crea account.move tipo 'out_invoice' en estado 'draft'
**Datos generados:** invoice_origin = SO name
**⚠️ NOTA RPC:** El método `_create_invoices` es PRIVADO (prefijo `_`) y NO debe usarse via JSON-RPC en producción. Aunque técnicamente funciona via XML-RPC, la vía correcta es el wizard `sale.advance.payment.inv` → `create_invoices` con contexto `{"active_ids": [<so_id>], "active_model": "sale.order"}`. El wizard es la vía oficial que soporta anticipos, porcentajes y facturas finales.

---

