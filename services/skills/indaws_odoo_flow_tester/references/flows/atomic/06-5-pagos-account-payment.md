## 5. PAGOS (account.payment)

### PAYMENT_CREATE — Crear Pago
**Modelo:** account.payment
**Campos mínimos:**
```json
{
  "payment_type": "inbound",
  "partner_type": "customer",
  "partner_id": "<int>",
  "amount": "<float>",
  "journal_id": "<int>",
  "date": "YYYY-MM-DD"
}
```
**Notas:**
- `payment_type`: "inbound" para cobro, "outbound" para pago
- `partner_type`: "customer" o "supplier"
- `journal_id`: diario de banco/caja
**Restricciones:**
- amount NO puede ser negativo (CHECK constraint)
**Resultado:** state = `draft`

### PAYMENT_POST — Confirmar Pago
**Método:** `execute_method.sh account.payment action_post '[[<id>]]' '{}'`
**Restricciones:**
- Si outbound y require_partner_bank_account → partner_bank_id debe tener allow_out_payment=True
**Resultado:** state → `in_process` (o `paid` si outstanding_account_id.account_type == 'asset_cash')
**Efectos:** El account.move asociado ya existe (se crea al crear el payment)

### PAYMENT_CANCEL — Cancelar Pago
**Método:** `execute_method.sh account.payment action_cancel '[[<id>]]' '{}'`
**Efectos:**
- Establece state = `canceled`
- Elimina (unlink) moves en draft
- Cancela moves posted (via button_cancel en los restantes)
**Resultado:** state → `canceled`

### PAYMENT_DRAFT — Volver a Borrador
**Método:** `execute_method.sh account.payment action_draft '[[<id>]]' '{}'`
**Efectos:** Resetea move asociado via button_draft (con todas sus restricciones)
**Resultado:** state → `draft`

### PAYMENT_DELETE — Eliminar Pago
**Método:** `delete_record.sh account.payment '[<id>]'`
**Efectos:** Resetea y borra moves asociados
**Restricciones:** Hereda restricciones de account.move.unlink()

---

