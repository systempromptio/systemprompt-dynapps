## CADENA 29: Extracto Bancario Completo
**ID:** CHAIN_BANK_STATEMENT_FULL
**Apps:** Contabilidad
**Descripcion:** Ciclo completo de extracto bancario: importar/crear extracto, crear lineas, conciliar con facturas existentes, desconciliar y eliminar el extracto.
**Tiempo estimado:** 2-3 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| Conciliacion (reconcile) | account.bank.statement.line | account.partial.reconcile | `search_records.sh account.partial.reconcile '[["credit_move_id","=",<move_line_id>]]'` |
| Linea de extracto | account.bank.statement.line | account.move | Campo move_id en la linea (inherits account.move) |

### Paso de Variables Entre Flujos
- `{chain.statement_id}` — ID del account.bank.statement
- `{chain.statement_name}` — Nombre/referencia del extracto
- `{chain.journal_id}` — ID del diario bancario
- `{chain.line_ids}` — IDs de las lineas del extracto
- `{chain.st_line_id}` — ID de la linea de extracto a conciliar
- `{chain.aml_id}` — ID del account.move.line de la factura para conciliar
- `{chain.invoice_id}` — ID de la factura existente para conciliar

### Precondiciones
- Diario bancario (account.journal type='bank') con cuenta bancaria configurada
- Factura de cliente publicada y pendiente de pago (para conciliar)
- Partner asociado a la factura

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | BANK_STATEMENT_CREATE | Crear account.bank.statement con journal_id, date, name | Extracto creado |
| 2 | BANK_STATEMENT_LINE_CREATE | Crear account.bank.statement.line con statement_id, date, payment_ref, amount, partner_id | Lineas creadas con amount != 0 |
| 3 | BANK_RECONCILE | Conciliar linea del extracto con factura existente: match por partner_id + amount | is_reconciled -> True |
| 4 | BANK_UNRECONCILE | `execute_method.sh account.bank.statement.line action_undo_reconciliation '[[<st_line_id>]]' '{}'` | is_reconciled -> False, partial.reconcile eliminado |
| 5 | BANK_STATEMENT_LINE_DELETE | `delete_record.sh account.bank.statement.line '[<line_ids>]'` | Lineas eliminadas |
| 6 | BANK_STATEMENT_DELETE | `delete_record.sh account.bank.statement '[<statement_id>]'` | Extracto eliminado |

### Plan de Rollback (orden inverso)
La cadena incluye su propio rollback (pasos 4-6), por lo que el rollback completo es:

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 3 | BANK_UNRECONCILE | action_undo_reconciliation | Deshace conciliacion, elimina partial.reconcile y pagos generados |
| 2 | BANK_STATEMENT_LINE_DELETE | delete_record | Solo si no estan conciliadas |
| 1 | BANK_STATEMENT_DELETE | delete_record | Solo si no tiene lineas o todas borradas |

### Puntos de No Retorno
- **Paso 3 (conciliacion)**: Reversible con action_undo_reconciliation — no es punto de no retorno real
- **Nota**: action_undo_reconciliation tambien elimina los pagos generados automaticamente (self.payment_ids.unlink()) y desreconcilia las lineas contables (self.line_ids.remove_move_reconcile())

---

