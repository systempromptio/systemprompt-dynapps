## CADENA 21: Gasto hasta Pago Completo
**ID:** CHAIN_EXPENSE_TO_PAY
**Apps:** RRHH + Contabilidad
**Descripcion:** Flujo extendido de gastos: creacion del gasto, agrupacion en informe, aprobacion del manager, contabilizacion que genera asiento contable, y pago al empleado con reconciliacion.
**Tiempo estimado:** 2-3 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.expense.sheet.action_sheet_move_post | hr.expense.sheet | account.move | Campo account_move_ids en el sheet (**action_sheet_move_create NO EXISTE en 18.0**) |
| account.payment.action_post | account.payment | account.move | Campo move_id en el payment |

### Paso de Variables Entre Flujos
- `{chain.expense_id}` — ID del hr.expense
- `{chain.expense_ids}` — IDs de multiples gastos (si aplica)
- `{chain.sheet_id}` — ID del hr.expense.sheet
- `{chain.employee_id}` — ID del empleado
- `{chain.move_id}` — ID del asiento contable generado
- `{chain.payment_id}` — ID del pago de reembolso

### Precondiciones
- Empleado con address_home_id (direccion privada) configurada
- Producto de tipo gasto (can_be_expensed=True)
- Diario de gastos configurado
- Diario de banco/caja para el pago
- Manager configurado para aprobacion (parent_id en employee o department_id)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | EXPENSE_CREATE | Crear hr.expense con employee_id, product_id, name, total_amount_currency, quantity | Gasto creado, state='draft' |
| 2 | SHEET_CREATE | Crear hr.expense.sheet con employee_id, name, expense_line_ids | Sheet creado |
| 3 | SHEET_SUBMIT | `execute_method.sh hr.expense.sheet action_submit_sheet '[[<sheet_id>]]' '{}'` | approval_state -> 'submit' |
| 4 | SHEET_APPROVE | `execute_method.sh hr.expense.sheet approve_expense_sheets '[[<sheet_id>]]' '{}'` | approval_state -> 'approve' |
| 5 | SHEET_POST | `execute_method.sh hr.expense.sheet action_sheet_move_post '[[<sheet_id>]]' '{}'` | state -> 'post', account.move creado (**18.0+: usar action_sheet_move_post**) |
| 6 | PAYMENT_CREATE | Crear account.payment outbound, partner=employee.address_home_id, amount=sheet.total_amount | payment.state == 'draft' |
| 7 | PAYMENT_POST | `execute_method.sh account.payment action_post '[[<payment_id>]]' '{}'` | payment.state -> 'in_process' o 'paid' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 7 | PAYMENT_CANCEL | action_cancel en payment | Cancela move asociado |
| 6 | PAYMENT_DELETE | delete_record account.payment | |
| 5 | SHEET_RESET | action_reset_expense_sheets | Resetea sheet a draft, cancela move |
| 4 | (sheet vuelve a draft tras reset) | | |
| 3 | (sheet vuelve a draft tras reset) | | |
| 2 | SHEET_DELETE | delete_record hr.expense.sheet | Solo si no tiene moves activos |
| 1 | EXPENSE_DELETE | delete_record hr.expense | Sin restricciones criticas |

### Puntos de No Retorno
- **Paso 5 (move creado)**: Si el asiento tiene inalterable_hash, el reset del sheet falla
- **Paso 7 (pago reconciliado)**: Debe desreconciliar antes de cancelar

---

