## CADENA 16: Nomina Completa
**ID:** CHAIN_PAYROLL
**Apps:** RRHH (Nomina + Asistencias)
**Descripcion:** Ciclo completo de nomina: contrato de empleado, registro de asistencia, generacion de nomina y confirmacion.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.payslip.action_payslip_done | hr.payslip | account.move | Campo move_id en el payslip |

### Paso de Variables Entre Flujos
- `{chain.employee_id}` — ID del empleado
- `{chain.contract_id}` — ID del contrato hr.contract
- `{chain.attendance_id}` — ID del registro de asistencia
- `{chain.payslip_id}` — ID de la nomina
- `{chain.payslip_name}` — Referencia de la nomina
- `{chain.move_id}` — ID del asiento contable generado

### Precondiciones
- Empleado (hr.employee) con direccion de trabajo configurada
- Estructura salarial (hr.payroll.structure) configurada
- Tipo de estructura salarial (hr.payroll.structure.type) asignada
- Reglas salariales configuradas para la estructura
- Diario de nomina configurado
- Modulo `hr_payroll` instalado (Enterprise)
- Modulo `hr_attendance` instalado (para asistencias)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | CONTRACT_CREATE | Crear hr.contract con employee_id, wage, date_start, structure_type_id, state='open' | Contrato creado y activo |
| 2 | ATTENDANCE_CHECK_IN | `execute_method.sh hr.employee attendance_manual '[[<employee_id>]]' '{"next_action": "check_in"}'` | Asistencia creada con check_in |
| 3 | ATTENDANCE_CHECK_OUT | `execute_method.sh hr.employee attendance_manual '[[<employee_id>]]' '{"next_action": "check_out"}'` | Asistencia con check_out, worked_hours > 0 |
| 4 | PAYSLIP_CREATE | Crear hr.payslip con employee_id, contract_id, date_from, date_to + `execute_method.sh hr.payslip compute_sheet '[[<payslip_id>]]' '{}'` | Nomina en draft con lineas calculadas |
| 5 | PAYSLIP_CONFIRM | `execute_method.sh hr.payslip action_payslip_done '[[<payslip_id>]]' '{}'` | payslip.state -> 'done', asiento contable generado |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 5 | PAYSLIP_REFUND | `execute_method.sh hr.payslip refund_sheet '[[<payslip_id>]]' '{}'` | Crea nomina inversa (credit note de nomina); la nomina original NO se puede cancelar directamente si state=done |
| 4 | PAYSLIP_DRAFT -> DELETE | Si state='draft': `execute_method.sh hr.payslip action_payslip_cancel '[[<payslip_id>]]' '{}'` -> `delete_record.sh hr.payslip '[<payslip_id>]'` | Solo desde draft; si done, usar refund |
| 3 | ATTENDANCE_DELETE | `delete_record.sh hr.attendance '[<attendance_id>]'` | Eliminar registro de asistencia |
| 2 | (eliminado en paso 3) | | |
| 1 | CONTRACT_CANCEL -> DELETE | `update_record.sh hr.contract '[<contract_id>]' '{"state": "cancel"}'` -> `delete_record.sh hr.contract '[<contract_id>]'` | Solo si no hay nominas asociadas activas |

### Puntos de No Retorno
- **Paso 5 (payslip done)**: Nomina confirmada genera asiento contable; no se puede cancelar directamente, solo se puede crear refund (nomina inversa)
- **Nota**: El asiento contable generado hereda las restricciones de account.move (hash, audit trail)

---

