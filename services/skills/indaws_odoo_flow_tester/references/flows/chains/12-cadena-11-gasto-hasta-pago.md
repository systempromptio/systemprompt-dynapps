## CADENA 11: Gasto hasta Pago
**ID:** CHAIN_EXPENSE_TO_PAYMENT
**Apps:** RRHH + Contabilidad
**Descripcion:** Flujo completo de gasto de empleado desde su creacion hasta el reembolso: creacion del gasto, envio del informe, aprobacion, generacion del asiento contable y pago al empleado.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.expense.sheet.action_sheet_move_post | hr.expense.sheet | account.move | Campo account_move_ids en el sheet (**action_sheet_move_create NO EXISTE en 18.0**) |
| account.payment.action_post | account.payment | account.move | Campo move_id en el payment |

### Paso de Variables Entre Flujos
- `{chain.expense_id}` — ID del hr.expense
- `{chain.sheet_id}` — ID del hr.expense.sheet
- `{chain.employee_id}` — ID del empleado
- `{chain.move_id}` — ID del asiento contable generado
- `{chain.payment_id}` — ID del pago al empleado

### Precondiciones
- Empleado (hr.employee) existente con direccion de trabajo configurada
- Producto de gasto o categoria de gasto configurada
- Diario de gastos configurado
- Diario de banco/caja para el pago
- Usuario con permisos de HR Officer o Manager (para aprobar)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | EXPENSE_CREATE | Crear hr.expense con employee_id, name, unit_amount, quantity | Gasto creado |
| 2 | EXPENSE_SUBMIT | Crear hr.expense.sheet + asociar gasto + action_submit_sheet | sheet.approval_state == 'submit' |
| 3 | EXPENSE_APPROVE | `execute_method.sh hr.expense.sheet action_approve_expense_sheets '[[<sheet_id>]]' '{}'` | sheet.approval_state == 'approve' |
| 4 | EXPENSE_POST | `execute_method.sh hr.expense.sheet action_sheet_move_post '[[<sheet_id>]]' '{}'` | account.move creado, sheet.state == 'post' (**18.0+: usar action_sheet_move_post**) |
| 5 | PAYMENT_CREATE | Crear account.payment outbound para partner del empleado, amount = total del sheet | payment.state == 'draft' |
| 6 | PAYMENT_POST | `execute_method.sh account.payment action_post '[[<payment_id>]]' '{}'` | payment.state -> 'in_process' o 'paid' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | PAYMENT_CANCEL | `execute_method.sh account.payment action_cancel '[[<payment_id>]]' '{}'` | Cancela move asociado |
| 5 | PAYMENT_DELETE | `delete_record.sh account.payment '[<payment_id>]'` | Elimina pago |
| 4 | EXPENSE_MOVE_RESET | Resetear sheet: el asiento debe cancelarse/borrarse primero, luego `execute_method.sh hr.expense.sheet action_reset_expense_sheets '[[<sheet_id>]]' '{}'` | El move generado se cancela al resetear |
| 3 | (sheet vuelve a draft tras reset) | | |
| 2 | EXPENSE_REFUSE | `execute_method.sh hr.expense.sheet action_refuse_expense_sheets '[[<sheet_id>]]' '{}'` | No puede rechazar si moves estan posted |
| 1 | EXPENSE_DELETE | `delete_record.sh hr.expense '[<expense_id>]'` | Sin restricciones criticas |

### Puntos de No Retorno
- **Paso 4 (move creado)**: Si el asiento tiene inalterable_hash, no se puede resetear facilmente
- **Paso 6 (pago reconciliado)**: Debe desreconciliar antes de cancelar

---

