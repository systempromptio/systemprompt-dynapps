## 8. RRHH — Gastos (hr.expense)

### EXPENSE_CREATE — Crear Gasto
**Modelo:** hr.expense
**Campos mínimos:**
```json
{
  "name": "Gasto test",
  "employee_id": "<int>",
  "quantity": 1,
  "unit_amount": "<float>"
}
```
**Notas:** `unit_amount` es el precio unitario.

### EXPENSE_SUBMIT — Enviar Informe
**Pasos:**
1. Crear sheet: `create_record.sh hr.expense.sheet '{"name": "Informe test", "employee_id": <int>, "expense_line_ids": [[4, <expense_id>]]}'`
2. Submit: `execute_method.sh hr.expense.sheet action_submit_sheet '[[<sheet_id>]]' '{}'`

### EXPENSE_APPROVE — Aprobar
**Método:** `execute_method.sh hr.expense.sheet action_approve_expense_sheets '[[<sheet_id>]]' '{}'`
**Restricciones:** Usuario debe ser HR Officer, Manager o approver

### EXPENSE_REFUSE — Rechazar
**Método:** `execute_method.sh hr.expense.sheet action_refuse_expense_sheets '[[<sheet_id>]]' '{}'`
**Nota:** Este método devuelve un **wizard** (action dict), no rechaza directamente. El rechazo real se ejecuta vía `_do_refuse(reason)` tras completar el wizard.
**Restricciones:** NO puede rechazar si moves están posted (solo si todos los moves son draft o no existen)

### EXPENSE_RESET — Volver a Borrador
**Método:** `execute_method.sh hr.expense.sheet action_reset_expense_sheets '[[<sheet_id>]]' '{}'`
**Resultado:** state → `draft`
**Nota:** El método correcto es `action_reset_expense_sheets`, NO `action_reset_draft`.

### EXPENSE_DELETE — Eliminar
**Método:** `delete_record.sh hr.expense '[<id>]'`
**Restricciones:** Sin restricciones críticas por estado en `unlink()`

---

