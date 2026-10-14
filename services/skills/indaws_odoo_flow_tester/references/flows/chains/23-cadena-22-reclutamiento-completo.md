## CADENA 22: Reclutamiento Completo
**ID:** CHAIN_RECRUITMENT
**Apps:** Reclutamiento + RRHH
**Descripcion:** Ciclo completo de reclutamiento: publicacion de puesto, creacion de candidato, avance por etapas de seleccion, contratacion y creacion automatica del empleado.
**Tiempo estimado:** 2-3 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| hr.applicant.create_employee_from_applicant | hr.applicant | hr.employee | Campo employee_id en el applicant |
| hr.applicant -> hr.candidate | hr.applicant | hr.candidate | Campo candidate_id en el applicant |

### Paso de Variables Entre Flujos
- `{chain.job_id}` — ID del puesto de trabajo (hr.job)
- `{chain.department_id}` — ID del departamento
- `{chain.candidate_id}` — ID del candidato (hr.candidate)
- `{chain.applicant_id}` — ID de la solicitud (hr.applicant)
- `{chain.stage_ids}` — IDs de las etapas de reclutamiento
- `{chain.employee_id}` — ID del empleado creado

### Precondiciones
- Puesto de trabajo (hr.job) existente con departamento asignado
- Etapas de reclutamiento (hr.recruitment.stage) configuradas
- Departamento (hr.department) existente
- Permisos de HR Officer o Manager

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | JOB_VERIFY | Verificar hr.job existente o crear uno nuevo | job_id disponible |
| 2 | APPLICANT_CREATE | Crear hr.applicant con partner_name (nombre candidato), job_id, department_id, stage_id (primera etapa) | Solicitud creada en etapa inicial |
| 3 | APPLICANT_ADVANCE_1 | `update_record.sh hr.applicant '[<applicant_id>]' '{"stage_id": <stage_2_id>}'` | stage_id actualizado (ej: "Entrevista inicial") |
| 4 | APPLICANT_ADVANCE_2 | `update_record.sh hr.applicant '[<applicant_id>]' '{"stage_id": <stage_3_id>}'` | stage_id actualizado (ej: "Segunda entrevista") |
| 5 | APPLICANT_ADVANCE_FINAL | `update_record.sh hr.applicant '[<applicant_id>]' '{"stage_id": <stage_final_id>}'` | stage_id en etapa de contratacion |
| 6 | APPLICANT_HIRE | `execute_method.sh hr.applicant create_employee_from_applicant '[[<applicant_id>]]' '{}'` | employee_id creado y asociado al applicant |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | EMPLOYEE_ARCHIVE + DELETE | `update_record.sh hr.employee '[<employee_id>]' '{"active": false}'` -> `delete_record.sh hr.employee '[<employee_id>]'` | Verificar que no tenga contratos, nominas u otros registros |
| 5-3 | (stages son solo updates) | No requieren rollback explicito | |
| 2 | APPLICANT_DELETE | `delete_record.sh hr.applicant '[<applicant_id>]'` | Eliminar candidatura |
| 1 | (job existente) | No eliminar si ya existia antes del test | Solo eliminar si fue creado durante el test |

### Puntos de No Retorno
- **Paso 6 (employee creado)**: Si el empleado ya tiene contrato, nominas o asistencias asociadas, no se puede eliminar facilmente
- **Nota**: La creacion del empleado genera registros en hr.employee, res.partner, res.users (si se crea usuario)

---

