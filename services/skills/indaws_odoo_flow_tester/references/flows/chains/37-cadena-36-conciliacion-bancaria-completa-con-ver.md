## CADENA 36: Conciliacion Bancaria Completa con Verificacion de Saldos
**ID:** CHAIN_BANK_RECONCILIATION_COMPLETE
**Apps:** Contabilidad
**Marcado:** [MIG]
**Descripcion:** Flujo extendido de conciliacion bancaria: crear extracto, anadir multiples lineas, conciliar con facturas existentes, verificar saldos del extracto, desconciliar y limpiar. Incluye verificacion de is_complete y balance_end.
**Tiempo estimado:** 2-3 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| Conciliacion (reconcile) | account.bank.statement.line | account.partial.reconcile | `search_records.sh account.partial.reconcile '[["credit_move_id","=",<move_line_id>]]'` |
| Conciliacion con factura | account.bank.statement.line | account.payment (auto) | `search_records.sh account.payment '[["partner_id","=",<partner_id>]]'` |

### Paso de Variables Entre Flujos
- `{chain.statement_id}` — ID del account.bank.statement
- `{chain.journal_id}` — ID del diario bancario
- `{chain.st_line_ids}` — IDs de las lineas del extracto creadas
- `{chain.aml_ids}` — IDs de account.move.line de facturas para conciliar
- `{chain.invoice_id}` — ID de la factura publicada a conciliar

### Precondiciones
- Diario bancario (account.journal type='bank') con cuenta bancaria configurada
- Al menos una factura publicada (account.move state='posted') con saldo pendiente (payment_state != 'paid')
- Partner con customer_rank > 0

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | BANK_STATEMENT_CREATE | Crear account.bank.statement con journal_id, name, balance_start, balance_end_real | Extracto creado |
| 2 | BANK_STATEMENT_LINE_CREATE | Crear account.bank.statement.line con statement_id, journal_id, date, payment_ref, amount, partner_id | is_reconciled=False, amount != 0 |
| 3 | — detectar AML — | Buscar account.move.line de factura: `[["move_id","=",<invoice_id>],["account_type","=","asset_receivable"],["reconciled","=",false]]` | AML encontrado con amount_residual > 0 |
| 4 | BANK_RECONCILE | `execute_method.sh account.bank.statement.line reconcile '[[<st_line_id>]]' '{"counterpart_vals_list": [{"move_line_id": <aml_id>}]}'` | is_reconciled -> True |
| 5 | — verificar saldos — | Leer statement: `search_records.sh account.bank.statement '[["id","=",<statement_id>]]' '["balance_start","balance_end","balance_end_real","is_complete"]'` | balance_end calculado correctamente |
| 6 | BANK_UNRECONCILE | `execute_method.sh account.bank.statement.line action_undo_reconciliation '[[<st_line_id>]]' '{}'` | is_reconciled -> False |
| 7 | BANK_STATEMENT_LINE_DELETE | `delete_record.sh account.bank.statement.line '[<st_line_ids>]'` | Lineas eliminadas |
| 8 | BANK_STATEMENT_DELETE | `delete_record.sh account.bank.statement '[<statement_id>]'` | Extracto eliminado |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 4 | BANK_UNRECONCILE | `execute_method.sh account.bank.statement.line action_undo_reconciliation '[[<st_line_id>]]' '{}'` | Solo si paso 4 se ejecuto |
| 2 | BANK_STATEMENT_LINE_DELETE | `delete_record.sh account.bank.statement.line '[<st_line_ids>]'` | Solo si no estan conciliadas |
| 1 | BANK_STATEMENT_DELETE | `delete_record.sh account.bank.statement '[<statement_id>]'` | Solo si no tiene lineas |

### Puntos de No Retorno
- **Ninguno real**: Todo el flujo bancario es reversible con action_undo_reconciliation
- **Nota**: Si audit trail esta habilitado, los moves de las lineas no se eliminan sino que se cancelan

---

