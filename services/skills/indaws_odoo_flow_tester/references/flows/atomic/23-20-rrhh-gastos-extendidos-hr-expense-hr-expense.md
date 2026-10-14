## 20. RRHH — Gastos Extendidos (hr.expense / hr.expense.sheet)

### EXPENSE_CREATE — Crear Gasto
**Modelo:** hr.expense
**Campos mínimos:**
```json
{
  "name": "Gasto test",
  "employee_id": "<int>",
  "product_id": "<int>",
  "quantity": 1,
  "unit_amount": "<float>"
}
```
**Notas:**
- `name`: descripción (required)
- `employee_id`: empleado (required)
- `product_id`: categoría de gasto (can_be_expensed=True), no estrictamente required pero sí en la vista
- `quantity`: cantidad (required, default 1)
- `unit_amount`: precio unitario (required, se computa desde product_id)
- `total_amount`: alternativa preferida a `unit_amount` en Odoo 18.0 (campo stored). Ambos funcionan pero `total_amount` es más directo.
- `date`: fecha del gasto (default: hoy)
- `payment_mode`: "own_account" (empleado paga) o "company_account" (empresa paga)
**Método/acción:** `create_record.sh hr.expense '...'`
**Resultado:** Registro creado, sin sheet asociado
**Verificación:** `name == <valor>`, `total_amount > 0`
**Estrategia de reversión:** `delete_record.sh hr.expense '[<id>]'`

### EXPENSE_SUBMIT — Enviar Informe de Gastos
**Modelo:** hr.expense.sheet
**Descripción:** Crea un informe de gastos y lo envía para aprobación.
**Pasos:**
1. Crear sheet: `create_record.sh hr.expense.sheet '{"name": "Informe test", "employee_id": <int>, "expense_line_ids": [[4, <expense_id>]]}'`
2. Submit: `execute_method.sh hr.expense.sheet action_submit_sheet '[[<sheet_id>]]' '{}'`
**Precondiciones:** Al menos un gasto sin sheet asignado, empleado con manager
**Resultado:** approval_state → `submit`, state → `submit`
**Verificación:** `state == 'submit'`
**Estrategia de reversión:** `execute_method.sh hr.expense.sheet action_reset_expense_sheets '[[<sheet_id>]]' '{}'`

### EXPENSE_APPROVE — Aprobar Informe de Gastos
**Modelo:** hr.expense.sheet
**Método:** `execute_method.sh hr.expense.sheet action_approve_expense_sheets '[[<sheet_id>]]' '{}'`
**Precondiciones:** state == 'submit'
**Restricciones:**
- Usuario debe ser HR Officer, Manager o approver designado
- No puede auto-aprobarse (excepto HR Officer)
**Resultado:** approval_state → `approve`, state → `approve`
**⚠️ CAMBIO CRÍTICO en Odoo 18.0:** La aprobación (`_do_approve`) **crea automáticamente los account.move en draft** vía `_do_create_moves()`. Los asientos contables ya existen tras aprobar, no es un paso separado.
**Verificación:** `state == 'approve'`, `account_move_ids` no vacío (moves en draft)
**Estrategia de reversión:** Rechazar → `execute_method.sh hr.expense.sheet action_refuse_expense_sheets '[[<sheet_id>]]' '{}'` (devuelve wizard)

### EXPENSE_POST — Contabilizar Informe de Gastos
**Modelo:** hr.expense.sheet
**Descripción:** Publica los asientos contables del informe aprobado (que ya fueron creados en draft durante la aprobación).
**Método:** `execute_method.sh hr.expense.sheet action_sheet_move_post '[[<sheet_id>]]' '{}'`
**⚠️ CORRECCIÓN v2.0:** El método `action_sheet_move_create` **NO EXISTE** en Odoo 18.0. El método correcto es `action_sheet_move_post()` que **publica** los moves ya creados.
**Precondiciones:** state == 'approve', account_move_ids existentes en draft
**Restricciones:**
- Publica los account.move creados durante la aprobación
- Si payment_mode == 'company_account', puede crear pagos automáticamente
**Resultado:** state → `post`
**Verificación:** `state == 'post'`, `account_move_ids` con state `posted`
**⚠️ PUNTO DE RIESGO:** Una vez contabilizado, revertir requiere cancelar el asiento
**Registros cascada generados:** Ninguno nuevo (los account.move ya se crearon en la aprobación)

**Flujo correcto completo en Odoo 18.0:**
1. `create` → state = `draft`
2. `action_submit_sheet()` → state = `submit`
3. `action_approve_expense_sheets()` → state = `approve` + **crea account.move en draft**
4. `action_sheet_move_post()` → state = `post` (publica los moves)
5. Pago → state = `done`

---

