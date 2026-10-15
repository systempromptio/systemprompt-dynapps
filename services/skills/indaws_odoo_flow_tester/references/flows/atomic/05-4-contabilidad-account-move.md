## 4. CONTABILIDAD (account.move)

### INVOICE_CREATE — Crear Factura
**Modelo:** account.move
**Campos mínimos:**
```json
{
  "move_type": "out_invoice",
  "partner_id": "<int>",
  "journal_id": "<int>",
  "invoice_line_ids": [[0, 0, {
    "product_id": "<int>",
    "quantity": "<float>",
    "price_unit": "<float>",
    "name": "Descripción"
  }]]
}
```
**Notas:**
- `move_type`: "out_invoice" para cliente, "in_invoice" para proveedor
- `journal_id`: opcional, se auto-detecta por move_type
- `name` en la línea: obligatorio si no hay product_id
**Resultado:** state = `draft`

### INVOICE_POST — Publicar Factura
**Método:** `execute_method.sh account.move action_post '[[<id>]]' '{}'`
**Precondición:** state == 'draft'
**Restricciones:**
- Genera secuencia numérica automática (name)
- Si journal tiene restrict_mode_hash_table=True → calcula inalterable_hash
- **`partner_id` es obligatorio** para invoices (sale/purchase documents) al publicar. No es requerido en `create()` pero `_post()` lo valida.
- **`amount_total` no puede ser negativo** para invoices — si es negativo, se debe crear una nota de crédito en su lugar.
- Puede mostrar wizard de confirmación si detecta montos anómalos, pero esta detección está **deshabilitada por defecto** vía API (`disable_abnormal_invoice_detection` default True). Solo se activa si explícitamente se pasa `disable_abnormal_invoice_detection=False` en el contexto.
**Resultado:** state → `posted`
**⚠️ PUNTO DE RIESGO: Si se genera inalterable_hash, NO se puede volver a draft**

### INVOICE_DRAFT — Volver a Borrador
**Método:** `execute_method.sh account.move button_draft '[[<id>]]' '{}'`
**Precondición:** state == 'posted' o 'cancel'
**Restricciones CRÍTICAS — NO se puede si (via `_check_draftable`):**
- `inalterable_hash` está asignado → UserError "You cannot reset to draft a locked journal entry."
- Es un asiento de diferencia de cambio (exchange_move_id en full/partial reconcile)
- Es un asiento de cash basis (tax_cash_basis_rec_id o tax_cash_basis_origin_move_id)
- `need_cancel_request == True` (governance) → UserError "You need to request a cancellation instead."
**Efectos:**
- Elimina líneas analíticas (analytic_line_ids.unlink)
- Remueve reconciliaciones (line_ids.remove_move_reconcile)
- Detacha archivos asociados (invoice_pdf_report_file)
- Resetea sending_data a False
**Resultado:** state → `draft`

### INVOICE_CANCEL — Cancelar Factura
**Método:** `execute_method.sh account.move button_cancel '[[<id>]]' '{}'`
**Precondición:** state == 'draft' o 'posted' (si posted, ejecuta button_draft internamente primero)
**Restricciones:** Hereda restricciones de button_draft si el move está posted
**Efectos:**
- Si posted → primero ejecuta button_draft (con todas sus restricciones)
- Cancela pagos asociados (payment_ids.state = "canceled")
- Establece auto_post = 'no'
**Resultado:** state → `cancel`

### INVOICE_DELETE — Eliminar Factura
**Método:** `delete_record.sh account.move '[<id>]'`
**Restricciones CRÍTICAS:**
- Si `posted_before=True` y `check_account_audit_trail=True` → NO borrar (auditoría)
- Si `sequence_number != 0` y no es el último de la cadena → NO borrar (secuencias)
  - **Bypass:** Si la compañía tiene `quick_edit_mode` activo, permite eliminar moves que no sean últimos de cadena
- **⚠️ CORRECCIÓN v2.0:** `inalterable_hash` **NO bloquea `unlink()`**. El hash fiscal bloquea `button_draft` (vía `_check_draftable`) pero NO impide la eliminación directa. Un move con hash SE PUEDE eliminar si es el último de la cadena y no tiene audit trail activo.

**Bypass con `force_delete` (solo para testing):**
- El flag de contexto `force_delete=True` bypassa las verificaciones de audit trail en el método `unlink()` de `account.move`
- Solo funciona para el modelo `account.move`, NO para otros modelos
- Referencia en código: `account_move.py` líneas 3466, 3500, 3510
- Caso de uso: cuando necesitas eliminar un asiento que tiene `posted_before=True` pero la compañía tiene `check_account_audit_trail=True`
- Ejemplo: `execute_method.sh account.move unlink '[[<id>]]' '{}' '{"force_delete": true}'`

**Nota sobre `check_account_audit_trail`:**
- Es un campo en `res.company` (configuración a nivel de compañía)
- Viene habilitado por defecto en las localizaciones alemana (`l10n_de`) y danesa (`l10n_dk`)
- Cuando está habilitado, los asientos con `posted_before=True` no pueden eliminarse (solo cancelarse o revertirse)
- Detección via API: `search_records.sh res.company '[["id","=",<company_id>]]' '["check_account_audit_trail"]'`

### INVOICE_REVERSE — Crear Nota de Crédito (via wizard)
**Pasos via API:**
1. Crear wizard: `create_record.sh account.move.reversal '{"move_ids": [[6, 0, [<id>]]], "reason": "Reversión test", "journal_id": <journal_id>, "company_id": <company_id>}'`
2. Ejecutar: `execute_method.sh account.move.reversal refund_moves '[[<wizard_id>]]' '{}'`
**Restricciones:**
- Solo reversa moves con state == 'posted'
- Todos los moves deben ser de la misma compañía
- **`company_id` es `required=True`** en el wizard — debe incluirse en el create. Se puede obtener del move original (`move.company_id`).
- `journal_id` es computed (`required=True` pero se auto-calcula desde move_ids)
**Resultado:** Nuevo account.move inverso (out_refund / in_refund)

### INVOICE_REVERSE_DIRECT — Reversar Asientos (método directo)
**Método:** `execute_method.sh account.move _reverse_moves '[[<ids>]]' '{"cancel": true}'`
**Descripción:** Método directo para reversar asientos sin pasar por el wizard. Útil para asientos con `inalterable_hash` o bloqueo fiscal donde `button_draft` / `button_cancel` no funcionan.
**Parámetros:**
- `cancel`: si `True`, publica automáticamente el reverso y reconcilia con el original. Si `False`, crea el reverso en draft.
- `default_values_list`: lista de dicts con valores por defecto para cada move a reversar (ej: `[{"date": "2026-03-09", "ref": "Reversión"}]`)
**Precondición:** state == 'posted'
**Resultado:**
- Si `cancel=True`: Crea reverso en `posted`, reconcilia automáticamente original ↔ reverso → `payment_state` del original cambia a `reversed`
- Si `cancel=False`: Crea reverso en `draft`
**Nota:** Este es el método interno que usa el wizard `account.move.reversal`. Es la ÚNICA forma de "deshacer" un asiento con hash de integridad.

### INVOICE_RECONCILE — Reconciliar Líneas de Factura
**Método:** `execute_method.sh account.move.line reconcile '[[<line_id_1>, <line_id_2>]]' '{}'`
**Descripción:** Reconcilia apuntes de factura con pago u otros apuntes. Ver ACCOUNT_RECONCILE (sección 31) para detalles completos.
**Resultado:** Crea `account.full.reconcile` o `account.partial.reconcile`

### INVOICE_UNRECONCILE — Deshacer Reconciliación
**Método:** `execute_method.sh account.move.line remove_move_reconcile '[[<line_ids>]]' '{}'`
**Descripción:** Elimina la reconciliación entre apuntes. Ver ACCOUNT_UNRECONCILE (sección 31) para detalles completos.
**Resultado:** Restaura `amount_residual`, puede generar asiento de reversión si hay diferencias de cambio.

---

