## CADENA 14: Conciliacion Bancaria Completa
**ID:** CHAIN_BANK_RECONCILIATION
**Apps:** Contabilidad
**Descripcion:** Flujo de conciliacion bancaria: creacion de extracto bancario, adicion de lineas y conciliacion con movimientos contables existentes.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| Conciliacion (reconcile) | account.bank.statement.line | account.partial.reconcile | `search_records.sh account.partial.reconcile '[["credit_move_id","=",<move_line_id>]]'` |

### Paso de Variables Entre Flujos
- `{chain.statement_id}` — ID del account.bank.statement
- `{chain.statement_name}` — Nombre/referencia del extracto
- `{chain.journal_id}` — ID del diario bancario
- `{chain.line_ids}` — IDs de las lineas del extracto
- `{chain.st_line_id}` — ID de la linea de extracto a conciliar
- `{chain.reconcile_ids}` — IDs de las reconciliaciones creadas

### Precondiciones
- Diario bancario (account.journal type='bank') configurado
- Movimientos contables existentes para conciliar (facturas posted, pagos)
- Cuenta bancaria configurada en el diario

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | BANK_STATEMENT_CREATE | Crear account.bank.statement con journal_id, date, name | Extracto creado |
| 2 | BANK_STATEMENT_LINE_CREATE | Crear account.bank.statement.line con statement_id, date, payment_ref, amount, partner_id | Lineas creadas con amount != 0 |
| 3 | BANK_RECONCILE | `execute_method.sh account.bank.statement.line reconcile '[[<st_line_id>]]' '{"counterpart_vals_list": [{"move_line_id": <aml_id>}]}'` | is_reconciled -> True |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 3 | BANK_UNRECONCILE | `execute_method.sh account.bank.statement.line action_undo_reconciliation '[[<st_line_id>]]' '{}'` | Deshace la conciliacion, elimina partial.reconcile |
| 2 | BANK_STATEMENT_LINE_DELETE | `delete_record.sh account.bank.statement.line '[<line_ids>]'` | Solo si no estan conciliadas |
| 1 | BANK_STATEMENT_DELETE | `delete_record.sh account.bank.statement '[<statement_id>]'` | Solo si no tiene lineas o todas borradas |

### Puntos de No Retorno
- **Paso 3 (reconciliacion)**: Reversible con action_undo_reconciliation — no es punto de no retorno real
- **Nota**: Si la linea del extracto genero un pago automatico y ese pago se reconcilio con facturas, deshacer la conciliacion bancaria NO deshace la reconciliacion del pago con la factura

---

