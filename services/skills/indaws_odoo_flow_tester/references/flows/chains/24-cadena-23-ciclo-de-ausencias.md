## CADENA 23: Ciclo de Ausencias
**ID:** CHAIN_LEAVE_CYCLE
**Apps:** Ausencias (RRHH)
**Descripcion:** Ciclo completo de gestion de ausencias: asignacion de dias disponibles, solicitud de ausencia por el empleado, aprobacion del manager, y verificacion del impacto en nomina.
**Tiempo estimado:** 1-2 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.leave.allocation.action_validate | hr.leave.allocation | — | Actualiza remaining_leaves del empleado |
| hr.leave.action_approve | hr.leave | resource.calendar.leaves | Crea entradas en el calendario del recurso |

### Paso de Variables Entre Flujos
- `{chain.leave_type_id}` — ID del tipo de ausencia (hr.leave.type)
- `{chain.allocation_id}` — ID de la asignacion (hr.leave.allocation)
- `{chain.leave_id}` — ID de la solicitud de ausencia (hr.leave)
- `{chain.employee_id}` — ID del empleado

### Precondiciones
- Empleado (hr.employee) con calendario de trabajo asignado
- Tipo de ausencia (hr.leave.type) configurado (ej: vacaciones, enfermedad)
- Tipo de ausencia con allocation_type='fixed_allocation' (para que requiera asignacion previa)
- Manager configurado para aprobacion

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | LEAVE_TYPE_VERIFY | Verificar hr.leave.type existente | leave_type_id disponible |
| 2 | ALLOCATION_CREATE | Crear hr.leave.allocation con employee_id, holiday_status_id, number_of_days | state='draft' |
| 3 | ALLOCATION_CONFIRM | `execute_method.sh hr.leave.allocation action_confirm '[[<allocation_id>]]' '{}'` | state -> 'confirm' |
| 4 | ALLOCATION_VALIDATE | `execute_method.sh hr.leave.allocation action_validate '[[<allocation_id>]]' '{}'` | state -> 'validate', dias asignados al empleado |
| 5 | LEAVE_REQUEST | Crear hr.leave con employee_id, holiday_status_id, date_from, date_to | state='draft' o 'confirm' |
| 6 | LEAVE_APPROVE | `execute_method.sh hr.leave action_approve '[[<leave_id>]]' '{}'` | state -> 'validate', remaining_leaves decrementado |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | LEAVE_REFUSE | `execute_method.sh hr.leave action_refuse '[[<leave_id>]]' '{}'` | state -> 'refuse', dias restaurados |
| 5 | LEAVE_RESET -> DELETE | `execute_method.sh hr.leave action_draft '[[<leave_id>]]' '{}'` -> `delete_record.sh hr.leave '[<leave_id>]'` | Resetear a draft antes de eliminar |
| 4 | ALLOCATION_REFUSE | `execute_method.sh hr.leave.allocation action_refuse '[[<allocation_id>]]' '{}'` | state -> 'refuse' |
| 3 | ALLOCATION_RESET | `execute_method.sh hr.leave.allocation action_draft '[[<allocation_id>]]' '{}'` | state -> 'draft' |
| 2 | ALLOCATION_DELETE | `delete_record.sh hr.leave.allocation '[<allocation_id>]'` | Solo desde state='draft' o 'refuse' |
| 1 | (tipo existente) | No eliminar | |

### Puntos de No Retorno
- **Paso 6 (leave validated)**: Reversible con action_refuse, pero si la ausencia ya esta en un payslip procesado, el impacto en nomina NO se revierte automaticamente
- **Nota**: Si hr_payroll esta instalado, la ausencia validada puede haber impactado el calculo de nomina del periodo correspondiente

---

