## 13. RRHH — Nóminas y Contratos [Enterprise: hr_payroll]

### PAYSLIP_CREATE — Crear Nómina
**Modelo:** hr.payslip
**Descripción:** Crea una nómina para un empleado en un período específico. Módulo Enterprise `hr_payroll`.
**Campos mínimos:**
```json
{
  "employee_id": "<int>",
  "date_from": "YYYY-MM-DD",
  "date_to": "YYYY-MM-DD",
  "contract_id": "<int>",
  "struct_id": "<int>"
}
```
**Notas:**
- `employee_id`: empleado con contrato activo
- `date_from` / `date_to`: se computan automáticamente (mes actual) si no se indican
- `contract_id`: se computa desde el empleado si no se indica
- `struct_id`: estructura salarial, se computa desde el contrato
- `name`: se computa automáticamente ("Salary Slip - Employee - Period")
**Método/acción:** `create_record.sh hr.payslip '...'`
**Precondiciones:** Empleado con contrato activo en el período, estructura salarial configurada
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `employee_id == <id>`
**Estrategia de reversión:** `delete_record.sh hr.payslip '[<id>]'` — solo en estado draft
**Registros cascada generados:** Ninguno hasta compute_sheet

### PAYSLIP_CONFIRM — Confirmar Nómina
**Modelo:** hr.payslip
**Descripción:** Calcula las líneas de la nómina y la confirma.
**Pasos:**
1. Calcular: `execute_method.sh hr.payslip compute_sheet '[[<id>]]' '{}'`
2. Confirmar: `execute_method.sh hr.payslip action_payslip_done '[[<id>]]' '{}'`
**Precondiciones:**
- state == 'draft' o 'verify'
- El contrato no puede estar cancelado
- El empleado debe tener contrato válido en el período de la nómina
**Restricciones:**
- No puede confirmar nómina cancelada (ValidationError)
- Si la nómina pertenece a un batch, la compañía debe coincidir
**Resultado:** state → `done` (tras action_payslip_done)
**Efectos secundarios:**
- `compute_sheet` genera `line_ids` (hr.payslip.line), `worked_days_line_ids` (hr.payslip.worked_days)
- `action_payslip_done` genera asiento contable (account.move)
**Verificación:** `state == 'done'`, `line_ids` no vacío, `number` asignado
**Estrategia de reversión:**
- Cancelar: `execute_method.sh hr.payslip action_payslip_cancel '[[<id>]]' '{}'` (solo admin/system para nóminas en 'done')
- Volver a draft: `execute_method.sh hr.payslip action_payslip_draft '[[<id>]]' '{}'`
**Registros cascada generados:** hr.payslip.line (N), hr.payslip.worked_days (M), account.move (1 al confirmar)

### CONTRACT_CREATE — Crear Contrato de Empleado
**Modelo:** hr.contract
**Descripción:** Crea un contrato laboral para un empleado.
**Campos mínimos:**
```json
{
  "name": "Contrato test",
  "employee_id": "<int>",
  "wage": "<float>",
  "date_start": "YYYY-MM-DD"
}
```
**Notas:**
- `employee_id`: empleado existente
- `wage`: salario bruto mensual
- `date_start`: fecha de inicio (default: hoy)
- `date_end`: fecha fin (opcional — contrato indefinido si no se indica)
- `structure_type_id`: tipo de estructura salarial (se computa desde la compañía si existe hr_payroll)
- `resource_calendar_id`: horario laboral (se computa desde el empleado)
- `contract_type_id`: tipo de contrato (opcional)
- `state`: draft → open → close → cancel
- `job_id`, `department_id`: se computan desde el empleado
**Método/acción:** `create_record.sh hr.contract '...'`
**Precondiciones:** Empleado existente
**Resultado:** state = `draft` (New)
**Verificación:** `state == 'draft'`, `wage == <valor>`
**Estrategia de reversión:** `delete_record.sh hr.contract '[<id>]'` — si está en draft, sin restricciones. **⚠️ Si se creó con state='open', auto-genera hr.work.entry que pueden bloquear la eliminación si están validados.** Secuencia: cancel contrato → eliminar work entries draft → eliminar contrato. Si work entries validados: archivar.
**Registros cascada generados:** hr.work.entry (N, si state='open' y módulo hr_work_entry instalado)

---

