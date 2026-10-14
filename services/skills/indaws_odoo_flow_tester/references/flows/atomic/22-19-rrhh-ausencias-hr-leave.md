## 19. RRHH — Ausencias (hr.leave)

### LEAVE_REQUEST_CREATE — Crear Solicitud de Ausencia
**Modelo:** hr.leave
**Descripción:** Crea una solicitud de ausencia para un empleado.
**Campos mínimos:**
```json
{
  "holiday_status_id": "<int>",
  "employee_id": "<int>",
  "date_from": "YYYY-MM-DD HH:MM:SS",
  "date_to": "YYYY-MM-DD HH:MM:SS"
}
```
**Notas:**
- `holiday_status_id`: tipo de ausencia (hr.leave.type, required)
- `employee_id`: empleado (required, default usuario actual)
- `date_from` / `date_to`: período de la ausencia (required)
- `number_of_days`: se calcula automáticamente desde las fechas
- `name`: descripción (opcional)
- Si el tipo de ausencia requiere asignación, el empleado debe tener días disponibles
- **⚠️ CORRECCIÓN v2.0:** hr.leave NO tiene estado `draft`. El estado inicial es `confirm` (To Approve).
- state: `confirm` (To Approve) → `validate1` (Second Approval) → `validate` (Approved)
- Estados adicionales: `refuse` (Refused), `cancel` (Cancelled)
**Método/acción:** `create_record.sh hr.leave '...'`
**Precondiciones:** Tipo de ausencia activo, empleado con asignación si el tipo lo requiere
**Resultado:** state = `confirm` (To Approve)
**Verificación:** `state == 'confirm'`, `number_of_days > 0`
**Estrategia de reversión:** `delete_record.sh hr.leave '[<id>]'` — ver restricciones en LEAVE_DELETE
**Registros cascada generados:** Ninguno

### LEAVE_REQUEST_APPROVE — Aprobar Ausencia (primera aprobación)
**Modelo:** hr.leave
**Descripción:** Aprueba una solicitud de ausencia. Puede requerir una o dos aprobaciones según configuración.
**Método:** `execute_method.sh hr.leave action_approve '[[<id>]]' '{}'`
**Precondiciones:** state == 'confirm'
**Restricciones:**
- El usuario debe ser el responsable (leave_manager_id del empleado) o HR Officer
- Si el tipo de ausencia requiere doble validación (`validation_type == 'both'`) → state pasa a `validate1`, necesita segunda aprobación con `action_validate`
- No se puede auto-aprobar la propia ausencia (salvo excepciones de configuración)
**Resultado:** state → `validate` (aprobación simple) o `validate1` (doble aprobación)
**Efectos secundarios:**
- Descuenta días de la asignación del empleado
- Crea entradas de calendario (resource.calendar.leaves)
- Puede crear entradas de trabajo (hr.work.entry) si módulo instalado
**Verificación:** `state == 'validate'` o `state == 'validate1'`
**Estrategia de reversión:** Rechazar → LEAVE_REQUEST_REFUSE

### LEAVE_REQUEST_VALIDATE — Segunda Aprobación
**Modelo:** hr.leave
**Descripción:** Completa la segunda aprobación para ausencias con doble validación.
**Método:** `execute_method.sh hr.leave action_validate '[[<id>]]' '{}'`
**Precondiciones:** state == 'validate1'
**Resultado:** state → `validate`
**Verificación:** `state == 'validate'`

### LEAVE_REQUEST_REFUSE — Rechazar Ausencia
**Modelo:** hr.leave
**Descripción:** Rechaza una solicitud de ausencia aprobada o pendiente.
**Método:** `execute_method.sh hr.leave action_refuse '[[<id>]]' '{}'`
**Precondiciones:** state in ('confirm', 'validate', 'validate1')
**Restricciones:**
- El usuario debe ser HR Officer o manager
**Resultado:** state → `refuse`
**Efectos secundarios:**
- Restaura días a la asignación del empleado
- Elimina entradas de calendario
**Verificación:** `state == 'refuse'`
**Estrategia de reversión:** Volver a confirm: `execute_method.sh hr.leave action_reset_confirm '[[<id>]]' '{}'`

### LEAVE_REQUEST_RESET — Volver a "To Approve"
**Modelo:** hr.leave
**Descripción:** Devuelve una ausencia rechazada o cancelada al estado "To Approve".
**Método:** `execute_method.sh hr.leave action_reset_confirm '[[<id>]]' '{}'`
**⚠️ CORRECCIÓN v2.0:** El método es `action_reset_confirm()`, NO `action_draft()`. El método `action_draft` NO EXISTE en hr.leave en Odoo 18.0.
**Precondiciones:** state in ('cancel', 'refuse')
**Resultado:** state → `confirm`

### LEAVE_DELETE — Eliminar Ausencia
**Modelo:** hr.leave
**Descripción:** Elimina una solicitud de ausencia.
**Método:** `delete_record.sh hr.leave '[<id>]'`
**Restricciones (dependen del rol del usuario):**
- **Usuarios normales:** pueden eliminar en estado `confirm`, `validate1`, `cancel` — NO pueden eliminar ausencias en el pasado
- **HR Officers:** pueden eliminar en estado `confirm` o `cancel`
- **NUNCA** se puede eliminar en estado `validate` (Approved)

### LEAVE_ALLOCATION_CREATE — Crear Asignación de Ausencia
**Modelo:** hr.leave.allocation
**Descripción:** Asigna días de ausencia a un empleado para un tipo determinado.
**Campos mínimos:**
```json
{
  "holiday_status_id": "<int>",
  "employee_id": "<int>",
  "date_from": "YYYY-MM-DD",
  "number_of_days": "<float>"
}
```
**Notas:**
- `holiday_status_id`: tipo de ausencia (required)
- `employee_id`: empleado (required)
- `number_of_days`: días asignados (required)
- `date_from`: inicio de vigencia (required)
- `date_to`: fin de vigencia (opcional)
- `allocation_type`: "regular" (default) o "accrual"
- state: 'confirm' → 'validate1' → 'validate'
**Método/acción:** `create_record.sh hr.leave.allocation '...'`
**Precondiciones:** Tipo de ausencia que permita asignación
**Resultado:** state = `confirm`
**Verificación:** `state == 'confirm'`, `number_of_days == <valor>`
**Nota:** hr.leave.allocation NO tiene estado `cancel` (solo: confirm, refuse, validate1, validate)
**Estrategia de reversión:** `delete_record.sh hr.leave.allocation '[<id>]'` — solo en estado `confirm` o `refuse`. NO se puede eliminar en `validate1` o `validate`.
**Restricción adicional de unlink:** No se puede eliminar si tiene ausencias validadas (`leaves_taken > 0`).
**Para aprobar:** `execute_method.sh hr.leave.allocation action_validate '[[<id>]]' '{}'`

---

