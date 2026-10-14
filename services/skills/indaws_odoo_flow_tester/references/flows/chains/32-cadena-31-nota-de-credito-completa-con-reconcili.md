## CADENA 31: Nota de Credito Completa con Reconciliacion
**ID:** CHAIN_REFUND_FLOW
**Apps:** Contabilidad
**Descripcion:** Flujo completo de rectificativa: crear factura de cliente, publicar, crear nota de credito via wizard de reversion, publicar la nota de credito, reconciliar ambos documentos y verificar saldo cero.
**Tiempo estimado:** 2-3 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| account.move.reversal.reverse_moves | account.move.reversal (wizard) | account.move (credit note) | `search_records.sh account.move '[["reversed_entry_id","=",<invoice_id>]]'` |
| Reconciliacion automatica (si refund_method='cancel') | account.move.reversal | account.partial.reconcile | Facturas con payment_state='reversed' |

### Paso de Variables Entre Flujos
- `{chain.invoice_id}` — ID de la factura original
- `{chain.invoice_name}` — Referencia de la factura
- `{chain.credit_note_id}` — ID de la nota de credito generada
- `{chain.credit_note_name}` — Referencia de la nota de credito
- `{chain.partner_id}` — ID del partner
- `{chain.reconcile_ids}` — IDs de las reconciliaciones

### Precondiciones
- Partner cliente con datos fiscales
- Producto facturable
- Diario de ventas configurado
- Cuentas contables de ingresos y clientes configuradas

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | INVOICE_CREATE | Crear account.move tipo 'out_invoice' con partner_id + lineas (product_id, quantity, price_unit) | state=draft, amount_total > 0 |
| 2 | INVOICE_POST | `execute_method.sh account.move action_post '[[<invoice_id>]]' '{}'` | state -> 'posted', payment_state='not_paid' |
| 3 | REVERSAL_WIZARD | Crear account.move.reversal con move_ids=[invoice_id], refund_method='refund', date, reason | Wizard creado |
| 4 | REVERSAL_EXECUTE | `execute_method.sh account.move.reversal reverse_moves '[[<wizard_id>]]' '{}'` | Nota de credito creada, reversed_entry_id == invoice_id |
| 5 | CREDIT_NOTE_POST | `execute_method.sh account.move action_post '[[<credit_note_id>]]' '{}'` | credit_note.state -> 'posted' |
| 6 | RECONCILE | Reconciliar las lineas contables de la factura y la NC: obtener receivable AMLs de ambas y ejecutar `execute_method.sh account.move.line reconcile '[[<aml_ids>]]' '{}'` | Ambos documentos con payment_state='in_payment' o 'paid', amount_residual == 0 |
| 7 | VERIFY_BALANCE | Verificar que el saldo del partner es cero: leer account.move.line receivable del partner | sum(debit-credit) == 0 para el partner |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 7 | (verificacion) | | |
| 6 | UNRECONCILE | `execute_method.sh account.move.line remove_move_reconcile '[[<aml_ids>]]' '{}'` | Deshace la reconciliacion |
| 5 | CREDIT_NOTE_DRAFT | `execute_method.sh account.move button_draft '[[<credit_note_id>]]' '{}'` | Solo si no tiene hash |
| 4 | CREDIT_NOTE_CANCEL_DELETE | `execute_method.sh account.move button_cancel '[[<credit_note_id>]]' '{}'` -> `delete_record.sh account.move '[<credit_note_id>]'` | |
| 3 | (wizard eliminado) | | |
| 2 | INVOICE_DRAFT | `execute_method.sh account.move button_draft '[[<invoice_id>]]' '{}'` | Solo si no tiene hash |
| 1 | INVOICE_CANCEL_DELETE | `execute_method.sh account.move button_cancel '[[<invoice_id>]]' '{}'` -> `delete_record.sh account.move '[<invoice_id>]'` | |

### Puntos de No Retorno
- **Paso 2 (invoice posted)**: Si el diario tiene hash (inalterable_hash) -> no se puede volver a draft
- **Paso 5 (credit note posted)**: Misma restriccion de hash
- **Paso 6 (reconciliacion)**: Reversible con remove_move_reconcile — no es punto de no retorno real
- **Nota**: Si se usa refund_method='cancel' en el wizard, la reconciliacion se hace automaticamente y el payment_state pasa a 'reversed'

---

