## 2. COMPRAS (purchase.order)

### PURCHASE_CREATE — Crear Orden de Compra
**Modelo:** purchase.order
**Campos mínimos:**
```json
{
  "partner_id": "<int>",
  "order_line": [[0, 0, {
    "product_id": "<int>",
    "product_qty": "<float>",
    "price_unit": "<float>"
  }]]
}
```
**Notas:**
- `name` en la línea: es un campo `computed, stored, readonly=False`. Se auto-calcula desde `product_id` si se pasa producto. Solo es necesario explícitamente si NO se pasa `product_id`.
- `price_unit`: se computa desde supplierinfo si existe producto con precio de proveedor configurado.
- `order_line` es opcional en `create()` — se puede crear PO sin líneas.
**Resultado:** state = `draft`

### PURCHASE_CONFIRM — Confirmar OC
**Método:** `execute_method.sh purchase.order button_confirm '[[<id>]]' '{}'`
**Precondición:** state == 'draft' o 'sent'
**Restricciones:**
- Si configuración requiere aprobación → state pasa a 'to approve' en lugar de 'purchase'
- Internamente llama `button_approve()` si la aprobación es directa
- **⚠️ Si `company.po_lock == 'lock'`:** `button_approve` pone la PO directamente en estado `done` (en vez de `purchase`). Esto bloquea la PO automáticamente tras confirmar.
**Resultado:** state → `purchase` (o `to approve`, o `done` si `po_lock == 'lock'`)
**Efectos secundarios:**
- Genera stock.picking de recepción (incoming)

### PURCHASE_APPROVE — Aprobar OC (directo)
**Método:** `execute_method.sh purchase.order button_approve '[[<id>]]' '{}'`
**Descripción:** Aprueba la OC directamente, saltando la validación de doble aprobación. Útil cuando la OC está en 'to approve'.
**Precondición:** state == 'to approve' (o 'draft'/'sent' si se bypassa)
**Resultado:** state → `purchase` (o `done` si `po_lock == 'lock'`)

### PURCHASE_DONE — Bloquear OC
**Método:** `execute_method.sh purchase.order button_done '[[<id>]]' '{}'`
**Descripción:** Bloquea la OC. Establece state = `done` y priority = '0'.
**Precondición:** state == 'purchase'
**Resultado:** state → `done`

### PURCHASE_UNLOCK — Desbloquear OC
**Método:** `execute_method.sh purchase.order button_unlock '[[<id>]]' '{}'`
**Descripción:** Desbloquea una OC bloqueada, devolviéndola a estado `purchase`.
**Precondición:** state == 'done'
**Resultado:** state → `purchase`

### PURCHASE_CANCEL — Cancelar OC
**Método:** `execute_method.sh purchase.order button_cancel '[[<id>]]' '{}'`
**Restricciones:**
- NO puede cancelar si existen facturas asociadas en estado != ('cancel', 'draft')
- UserError: "Unable to cancel purchase order(s): <names>. You must first cancel their related vendor bills."
- También resetea `mail_reminder_confirmed = False`
- **Nota:** `button_cancel` NO valida el estado de la PO. Funciona desde cualquier estado (draft, sent, to approve, purchase, done) siempre que no tenga facturas no-draft/no-cancel.
**Resultado:** state → `cancel`

### PURCHASE_DELETE — Eliminar OC
**Método:** `delete_record.sh purchase.order '[<id>]'`
**Restricciones:**
- SOLO se puede eliminar en estado 'cancel'
- UserError: "In order to delete a purchase order, you must cancel it first."

### PURCHASE_RECEIVE — Recibir Mercancía
**Descripción:** Tras confirmar la OC, se genera un picking de recepción. Para registrar la recepción:
**Pasos vía API:**
1. **Obtener picking:** Buscar por origin o acceder via `purchase_order.picking_ids[0]`
2. **Establecer cantidades recibidas en la OC (opcional):** `update_record.sh purchase.order.line '[<line_ids>]' '{"qty_received": <qty>}'`
   - Esto actualiza las cantidades recibidas a nivel de línea de OC.
3. **Validar picking:** Seguir la secuencia completa de SALE_VALIDATE_PICKING (action_confirm → action_assign → set quantity + picked → button_validate)
**Resultado:** picking.state → `done`, líneas de OC actualizan qty_received

### PURCHASE_CREATE_INVOICE — Crear Factura de Proveedor desde OC
**Método:** `execute_method.sh purchase.order action_create_invoice '[[<id>]]' '{}'`
**Precondición:** state == 'purchase', líneas con qty_to_invoice > 0
**Restricciones:**
- Requiere que haya cantidades pendientes de facturar (qty_received - qty_invoiced > 0)
- Si no hay cantidades facturables → UserError "There is no invoiceable line."
**Resultado:** Crea account.move tipo 'in_invoice' en estado 'draft'
**Datos generados:** ref = PO name, invoice_origin = PO name
**Nota:** A diferencia de sale.order (que usa `_create_invoices`), purchase.order tiene el método público `action_create_invoice`.

### PURCHASE_DRAFT — Volver a Borrador
**Método:** `execute_method.sh purchase.order button_draft '[[<id>]]' '{}'`
**Resultado:** state → `draft`

### PURCHASE_DELETE — Eliminar OC (restricciones)
**Nota sobre eliminación:** En purchase.order las restricciones son más estrictas que en sale.order:
- SOLO se puede eliminar en estado `cancel` (UserError: "In order to delete a purchase order, you must cancel it first.")
- No se puede eliminar directamente en estado `draft` (a diferencia de sale.order que permite draft o cancel)
- Primero hay que cancelar (`button_cancel`) y luego eliminar

---

