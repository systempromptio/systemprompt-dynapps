## CADENA 32: Ciclo de Vida de Contrato
**ID:** CHAIN_HR_CONTRACT_LIFECYCLE
**Apps:** RRHH (Contratos)
**Descripcion:** Ciclo de vida completo de contrato de empleado: crear empleado, crear contrato, activarlo, cerrarlo al expirar y crear un nuevo contrato de renovacion.
**Tiempo estimado:** 1-2 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.contract state='open' | hr.contract | — | Empleado con contract_id actualizado |
| hr.contract state='close' | hr.contract | — | Contrato expirado, empleado puede quedar sin contrato activo |

### Paso de Variables Entre Flujos
- `{chain.employee_id}` — ID del hr.employee
- `{chain.department_id}` — ID del departamento
- `{chain.job_id}` — ID del puesto de trabajo
- `{chain.contract_1_id}` — ID del primer contrato
- `{chain.contract_2_id}` — ID del segundo contrato (renovacion)

### Precondiciones
- Departamento (hr.department) existente
- Puesto de trabajo (hr.job) existente
- Tipo de estructura salarial (hr.payroll.structure.type) si hr_payroll instalado
- Company con divisa configurada

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | EMPLOYEE_CREATE | Crear hr.employee con name, department_id, job_id | Empleado creado, active=True |
| 2 | CONTRACT_CREATE | Crear hr.contract con employee_id, name, wage, date_start, state='draft' | contract.state == 'draft' |
| 3 | CONTRACT_ACTIVATE | `update_record.sh hr.contract '[<contract_1_id>]' '{"state": "open"}'` | state -> 'open', empleado.contract_id actualizado |
| 4 | CONTRACT_CLOSE | `update_record.sh hr.contract '[<contract_1_id>]' '{"state": "close", "date_end": "<fecha_cierre>"}'` | state -> 'close', contrato expirado |
| 5 | CONTRACT_RENEW | Crear hr.contract con employee_id, name (renovacion), wage (nuevo salario), date_start (dia siguiente a date_end), state='draft' | Nuevo contrato creado |
| 6 | CONTRACT_ACTIVATE_NEW | `update_record.sh hr.contract '[<contract_2_id>]' '{"state": "open"}'` | state -> 'open', empleado.contract_id actualizado al nuevo contrato |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | CONTRACT_2_CANCEL | `update_record.sh hr.contract '[<contract_2_id>]' '{"state": "cancel"}'` | Cancelar nuevo contrato |
| 5 | CONTRACT_2_DELETE | `delete_record.sh hr.contract '[<contract_2_id>]'` | Solo desde cancel/draft |
| 4 | CONTRACT_1_REOPEN | `update_record.sh hr.contract '[<contract_1_id>]' '{"state": "open", "date_end": false}'` | Reabrir contrato original |
| 3 | CONTRACT_1_DRAFT | `update_record.sh hr.contract '[<contract_1_id>]' '{"state": "draft"}'` | Volver a borrador |
| 2 | CONTRACT_1_DELETE | `delete_record.sh hr.contract '[<contract_1_id>]'` | Solo si no hay nominas asociadas |
| 1 | EMPLOYEE_ARCHIVE_DELETE | `update_record.sh hr.employee '[<employee_id>]' '{"active": false}'` -> `delete_record.sh hr.employee '[<employee_id>]'` | Solo si no tiene registros dependientes (nominas, asistencias, gastos) |

### Puntos de No Retorno
- **Paso 3 (contrato activo)**: Si se han generado work entries o nominas con este contrato, no se puede revertir facilmente
- **Paso 4 (contrato cerrado)**: Reversible si no hay dependencias (nominas, work entries del periodo)
- **Nota**: En Odoo 18, el estado del contrato se gestiona via write directo (no hay metodos de transicion de estado dedicados). El campo `kanban_state='done'` + cron puede cambiar automaticamente state de draft a open

---

