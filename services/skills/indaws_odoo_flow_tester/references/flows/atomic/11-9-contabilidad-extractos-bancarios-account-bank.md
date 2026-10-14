## 9. CONTABILIDAD — Extractos Bancarios (account.bank.statement)

### BANK_STATEMENT_CREATE — Crear Extracto Bancario
**Modelo:** account.bank.statement
**Descripción:** Crea un extracto bancario que agrupa líneas de transacciones bancarias para un diario y período específico.
**Campos mínimos:**
```json
{
  "name": "Extracto test",
  "journal_id": "<int>",
  "balance_start": "<float>",
  "balance_end_real": "<float>",
  "line_ids": [[6, 0, [<st_line_ids>]]]
}
```
**Notas:**
- `journal_id` se computa automáticamente desde las líneas si no se indica
- `balance_start` se computa desde el extracto anterior del mismo diario
- `balance_end_real` es el saldo final declarado (el que debe cuadrar)
- `line_ids` son IDs de `account.bank.statement.line` existentes a agrupar
- `date` se computa automáticamente desde las líneas
**Método/acción:** Crear vía `create_record.sh account.bank.statement '...'`
**Precondiciones:** Debe existir un diario de tipo banco/caja. Las líneas referenciadas deben pertenecer al mismo diario.
**Resultado:** Registro creado. `is_complete` = True si balance_end == balance_end_real
**Verificación:** `is_complete == True`, `is_valid == True`
**Estrategia de reversión:** `delete_record.sh account.bank.statement '[<id>]'` — solo desasocia líneas, no las elimina
**Registros cascada generados:** Ninguno (agrupa líneas existentes)

### BANK_STATEMENT_LINE_CREATE — Crear Línea de Extracto Bancario
**Modelo:** account.bank.statement.line
**Descripción:** Crea una transacción bancaria (línea de extracto). Hereda de account.move (_inherits).
**Campos mínimos:**
```json
{
  "journal_id": "<int>",
  "payment_ref": "Referencia pago",
  "amount": "<float>",
  "date": "YYYY-MM-DD",
  "partner_id": "<int>"
}
```
**Notas:**
- `payment_ref` es la etiqueta/referencia de la transacción
- `amount` positivo = entrada, negativo = salida
- `partner_id` es opcional
- `statement_id` es opcional — puede asignarse después
- Al crear, genera automáticamente un `account.move` (journal entry) con estado `posted`
- Crea líneas de liquidez + contrapartida (suspense account) automáticamente
**Método/acción:** `create_record.sh account.bank.statement.line '...'`
**Precondiciones:** Diario de tipo banco/caja con suspense account configurada
**Resultado:** Línea creada, move_id asociado en estado `posted`, `is_reconciled` = False
**Verificación:** `is_reconciled == False`, `amount == <valor_esperado>`
**Estrategia de reversión:** `delete_record.sh account.bank.statement.line '[<id>]'` — elimina el account.move asociado en cascada. Si la empresa tiene audit trail activo, cancela el move en lugar de eliminarlo.
**Registros cascada generados:** account.move (1), account.move.line (2 — liquidez + suspense)

### BANK_RECONCILE — Reconciliar Línea Bancaria
**Modelo:** account.bank.statement.line
**Descripción:** Reconcilia una línea de extracto bancario con apuntes contables existentes (facturas, pagos, etc.).
**Método/acción:** La reconciliación en Odoo 18 se realiza mediante el widget de reconciliación del frontend. Vía API, se manipulan directamente los apuntes contables:
1. Identificar la línea de suspense: buscar en `move_id.line_ids` la línea con `account_id == journal.suspense_account_id`
2. Reemplazar la línea de suspense con contrapartidas reales usando write en el move:
```
execute_method.sh account.bank.statement.line write '[[<id>]]' '{"line_ids": [<commands>]}'
```
**Precondiciones:** `is_reconciled == False`, línea debe estar en estado `posted`
**Restricciones:**
- La suma de las contrapartidas debe cuadrar con el monto de la línea de liquidez
- Si se reconcilia con facturas, los apuntes deben ser de cuentas reconciliables
**Resultado:** `is_reconciled` → True, `amount_residual` → 0
**Verificación:** `is_reconciled == True`
**Estrategia de reversión:** `execute_method.sh account.bank.statement.line action_undo_reconciliation '[[<id>]]' '{}'` — deshace la reconciliación y restaura la línea de suspense original
**Registros cascada generados:** Puede generar account.payment si se reconcilia con factura

---

