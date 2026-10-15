## 21. RECLUTAMIENTO (hr.applicant)

### APPLICANT_CREATE — Crear Candidatura
**Modelo:** hr.applicant
**Descripción:** Crea una candidatura vinculada a un puesto de trabajo.
**Campos mínimos:**
```json
{
  "partner_name": "Nombre del candidato",
  "job_id": "<int>",
  "stage_id": "<int>"
}
```
**Notas:**
- `partner_name`: nombre del candidato (se computa desde candidate_id si existe)
- `job_id`: puesto de trabajo (hr.job, opcional pero recomendado)
- `stage_id`: etapa del proceso (se computa automáticamente si no se indica)
- `candidate_id`: se crea automáticamente un hr.candidate si no se indica
- `department_id`: se computa desde job_id
- `user_id`: responsable (default: usuario actual)
- `priority`: '0' (Normal), '1' (Good), '2' (Very Good), '3' (Excellent)
- `kanban_state`: 'normal', 'blocked', 'done'
**Método/acción:** `create_record.sh hr.applicant '...'`
**Precondiciones:** Ninguna estricta. Recomendado: job_id existente
**Resultado:** Candidatura creada en primera etapa
**Verificación:** `partner_name == <valor>`, `stage_id != False`
**Estrategia de reversión:** `delete_record.sh hr.applicant '[<id>]'`
**Registros cascada generados:** hr.candidate (1, si no se indicó candidate_id)

### APPLICANT_STAGE_CHANGE — Cambiar Etapa de Candidatura
**Modelo:** hr.applicant
**Descripción:** Mueve la candidatura a otra etapa del proceso de selección.
**Método:** `update_record.sh hr.applicant '[<id>]' '{"stage_id": <stage_id>}'`
**Notas:**
- Las etapas son globales o por departamento (`hr.recruitment.stage`)
- `last_stage_id`: se actualiza automáticamente con la etapa anterior
- Se puede usar kanban_state para indicar progreso dentro de la etapa
**Precondiciones:** stage_id debe existir
**Resultado:** `stage_id` actualizado
**Verificación:** `stage_id == <nuevo_stage_id>`
**Estrategia de reversión:** Volver a la etapa anterior: `update_record.sh hr.applicant '[<id>]' '{"stage_id": <last_stage_id>}'`

### APPLICANT_HIRE — Contratar Candidato
**Modelo:** hr.applicant
**Descripción:** Crea un empleado a partir de la candidatura.
**Método:** `execute_method.sh hr.applicant create_employee_from_applicant '[[<id>]]' '{}'`
**Precondiciones:** Candidatura activa, preferiblemente en etapa final
**Restricciones:**
- Devuelve acción de formulario para completar datos del nuevo empleado
- Si candidate_id ya tiene employee_id → puede vincular al existente
**Resultado:** Crea hr.employee vinculado a la candidatura
**Efectos secundarios:**
- Crea hr.employee con datos del candidato
- Vincula employee_id en hr.candidate
- Cierra la candidatura (archive_applicant)
**Verificación:** `employee_id != False` (en hr.candidate)
**⚠️ PUNTO DE RIESGO:** Una vez creado el empleado, la candidatura se archiva
**Registros cascada generados:** hr.employee (1)

---

