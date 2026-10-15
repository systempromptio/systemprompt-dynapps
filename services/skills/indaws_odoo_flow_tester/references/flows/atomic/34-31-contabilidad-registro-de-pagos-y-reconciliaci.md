## 31. CONTABILIDAD — Registro de Pagos y Reconciliación

### ACCOUNT_PAYMENT_REGISTER — Registrar Pago desde Factura
**Modelo:** account.payment.register (wizard transitorio)
**Descripción:** Registra un pago directamente desde una o varias facturas, creando el pago y reconciliándolo automáticamente con la factura.
**Pasos vía API:**
1. Crear wizard con contexto de facturas activas:
   `create_record.sh account.payment.register '{"payment_date": "YYYY-MM-DD", "journal_id": <int>}' --context '{"active_ids": [<invoice_id>], "active_model": "account.move"}'`
2. Ejecutar: `execute_method.sh account.payment.register action_create_payments '[[<wizard_id>]]' '{}'`
**Campos del wizard (auto-computed):**
- `payment_date`: fecha de pago (default: hoy)
- `amount`: monto (se computa desde las facturas seleccionadas)
- `currency_id`: moneda (se computa)
- `journal_id`: diario de banco/caja (se computa)
- `communication`: referencia/memo (se computa desde la factura)
- `group_payment`: True para agrupar pagos por partner (default: según configuración)
- `payment_difference_handling`: "open" (dejar abierto) o "reconcile" (write-off)
**Precondiciones:** Facturas en estado `posted` con amount_residual > 0
**Restricciones:**
- Si `require_partner_bank_account == True` y cuenta bancaria del partner no validada → UserError
- Crea un pago por factura, o uno agrupado por partner si `group_payment == True`
**Resultado:** Crea account.payment en estado `in_process` (o `paid`), reconcilia automáticamente con la factura
**Verificación:** `payment_state` de la factura cambia a `paid` o `in_payment`
**Registros cascada generados:** account.payment (1 o N), account.move (1 o N), reconciliaciones automáticas

### ACCOUNT_RECONCILE — Reconciliar Apuntes Contables
**Modelo:** account.move.line
**Descripción:** Reconcilia dos o más apuntes contables entre sí (ej: factura con pago).
**Método:** `execute_method.sh account.move.line reconcile '[[<line_id_1>, <line_id_2>]]' '{}'`
**Precondiciones:**
- Las líneas deben pertenecer a cuentas reconciliables (account.account con `reconcile == True`)
- Las líneas deben ser del mismo partner y misma cuenta
- Las líneas deben estar en moves con state == 'posted'
**Restricciones:**
- Internamente llama `_reconcile_plan([self])`
- Si la suma no cuadra exactamente, puede generar un apunte de diferencia de cambio
**Resultado:** `matching_number` asignado, `amount_residual` → 0 (si full reconcile)
**Verificación:** `reconciled == True` (en las líneas), `amount_residual == 0`
**Estrategia de reversión:** → ACCOUNT_UNRECONCILE
**Registros cascada generados:** account.partial.reconcile o account.full.reconcile, posible account.move (diferencia de cambio)

### ACCOUNT_UNRECONCILE — Deshacer Reconciliación
**Modelo:** account.move.line
**Descripción:** Deshace la reconciliación entre apuntes contables.
**Método:** `execute_method.sh account.move.line remove_move_reconcile '[[<line_ids>]]' '{}'`
**Precondiciones:** Las líneas deben tener `matched_debit_ids` o `matched_credit_ids`
**Efectos:**
- Elimina (unlink) los registros `account.partial.reconcile` y `account.full.reconcile` asociados
- Restaura `amount_residual` original de los apuntes
**Resultado:** `reconciled` → False, `amount_residual` restaurado
**Verificación:** `reconciled == False`
**Registros eliminados:** account.partial.reconcile, account.full.reconcile

---

