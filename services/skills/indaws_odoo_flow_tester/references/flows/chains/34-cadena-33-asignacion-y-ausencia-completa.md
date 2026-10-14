## CADENA 33: Asignacion y Ausencia Completa
**ID:** CHAIN_LEAVE_ALLOCATION_FULL
**Apps:** Ausencias (RRHH)
**Descripcion:** Ciclo completo de gestion de ausencias: crear asignacion de dias, aprobar la asignacion, crear solicitud de ausencia, aprobar la ausencia, luego rechazar la ausencia y eliminar registros.
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
- Empleado (hr.employee) con calendario de trabajo asignado y manager configurado
- Tipo de ausencia (hr.leave.type) configurado con allocation_type='fixed_allocation'
- Manager (leave_manager_id) configurado para aprobacion
- Calendario de trabajo (resource.calendar) con dias laborables definidos

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | ALLOCATION_CREATE | Crear hr.leave.allocation con employee_id, holiday_status_id, number_of_days | state='draft' |
| 2 | ALLOCATION_CONFIRM | `execute_method.sh hr.leave.allocation action_confirm '[[<allocation_id>]]' '{}'` | state -> 'confirm' |
| 3 | ALLOCATION_VALIDATE | `execute_method.sh hr.leave.allocation action_validate '[[<allocation_id>]]' '{}'` | state -> 'validate', dias asignados al empleado |
| 4 | LEAVE_CREATE | Crear hr.leave con employee_id, holiday_status_id, date_from, date_to (dentro de dias laborables) | state='draft' o 'confirm' |
| 5 | LEAVE_APPROVE | `execute_method.sh hr.leave action_approve '[[<leave_id>]]' '{}'` | state -> 'validate', remaining_leaves decrementado |
| 6 | LEAVE_REFUSE | `execute_method.sh hr.leave action_refuse '[[<leave_id>]]' '{}'` | state -> 'refuse', dias restaurados |
| 7 | LEAVE_DELETE | `execute_method.sh hr.leave action_draft '[[<leave_id>]]' '{}'` -> `delete_record.sh hr.leave '[<leave_id>]'` | Ausencia eliminada |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 7 | (ya eliminada) | | |
| 6 | (refuse es parte del test) | | |
| 5 | (refuse en paso 6 ya revirtio) | | |
| 4 | (eliminada en paso 7) | | |
| 3 | ALLOCATION_REFUSE | `execute_method.sh hr.leave.allocation action_refuse '[[<allocation_id>]]' '{}'` | state -> 'refuse' |
| 2 | ALLOCATION_DRAFT | `execute_method.sh hr.leave.allocation action_draft '[[<allocation_id>]]' '{}'` | state -> 'draft' |
| 1 | ALLOCATION_DELETE | `delete_record.sh hr.leave.allocation '[<allocation_id>]'` | Solo desde state='draft' o 'refuse' |

### Puntos de No Retorno
- **Paso 5 (leave validated)**: Reversible con action_refuse — no es punto de no retorno real
- **Nota**: Si hr_payroll esta instalado y la ausencia ya impacto un payslip procesado, el impacto en nomina NO se revierte automaticamente al rechazar la ausencia
- **Nota**: action_approve llama internamente a action_validate que valida las fechas contra el calendario de trabajo

---

